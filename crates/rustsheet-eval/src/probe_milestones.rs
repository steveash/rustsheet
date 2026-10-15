//! Probe milestones (decision 0051): every `probes` case, and every checked cell of
//! `probes/catalog/*`, counts toward exactly one milestone's exit criterion. The first rule that
//! applies decides:
//!
//! 1. a case on the frozen list `eval/milestones/m1-probes.txt` is M1;
//! 2. a cell line of [`MAPPING_PATH`] (`case`, `Sheet!A1`, milestone, subject) sets that cell's
//!    milestone, and a case line (`case`, milestone, subject) sets the case's;
//! 3. a `probes/catalog/*` cell takes the highest catalog milestone of the functions its formula
//!    calls, apart from the observing wrappers OR, IFERROR, ERROR.TYPE and INDEX (M2 when none);
//! 4. any other case takes its case.json milestone tag (the highest one when it has several);
//! 5. a case with no milestone tag is M2.
//!
//! The report scores each milestone the way the baseline does, so its known failures equal the
//! F lines of `eval/baseline/probes.txt` that the rules assign to it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::baseline::Baseline;
use crate::case::split_qualified;
use crate::catalog::Catalog;
use crate::cellref::Addr;
use crate::milestone_lists::MilestoneList;
use crate::report::functions_in;
use crate::runner::{CaseResult, CellResult};
use crate::suite::BaselineMode;
use crate::value::Verdict;

/// The mapping file, relative to the repository root.
pub const MAPPING_PATH: &str = "eval/milestones/probe-milestones.tsv";
/// The suite whose cases the rules assign.
pub const SUITE: &str = "probes";
/// The frozen list whose cases are M1 (rule 1).
pub const FROZEN_M1_LIST: &str = "m1-probes";
/// Cases scored per cell by rule 3.
const CATALOG_PREFIX: &str = "probes/catalog/";
/// Functions that only observe a catalog cell's result, so they do not set its milestone.
const OBSERVING_WRAPPERS: [&str; 4] = ["OR", "IFERROR", "ERROR.TYPE", "INDEX"];
/// Rule 5's milestone, and rule 3's for a catalog cell that calls no catalog function.
const DEFAULT_MILESTONE: u32 = 2;
/// The milestones that always get a report row; M0, M1 and M2 share the first one. Any other
/// milestone a case is assigned gets a row of its own after these, so nothing drops out.
const REPORT_ROWS: [u32; 8] = [2, 5, 6, 7, 8, 9, 10, 11];

/// Parses `M<digits>`. Numeric order is the phase order of docs/milestones.md for every
/// milestone a probe can take (M0–M2 and M5–M11).
pub(crate) fn parse_milestone(s: &str) -> Option<u32> {
    let digits = s.strip_prefix('M')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// The report row a milestone counts in: M0 and M1 count with M2 (decision 0051, item 3).
fn row_of(milestone: u32) -> u32 {
    milestone.max(DEFAULT_MILESTONE)
}

fn row_label(row: u32) -> String {
    if row == DEFAULT_MILESTONE {
        "M0-M2".to_string()
    } else {
        format!("M{row}")
    }
}

/// One line of the mapping file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assigned {
    /// 1-based line number.
    pub line: usize,
    /// The milestone number (`M9` → 9).
    pub milestone: u32,
}

/// The parsed mapping file: case lines and cell lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mapping {
    /// Case id → the case's milestone.
    pub cases: BTreeMap<String, Assigned>,
    /// (case id, `Sheet!A1`) → that cell's milestone.
    pub cells: BTreeMap<(String, String), Assigned>,
}

/// Why the mapping file or one of its lines is unusable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MappingError {
    /// The file cannot be read.
    #[error("{MAPPING_PATH}: {message}")]
    Unreadable { message: String },
    /// A line is malformed.
    #[error("{MAPPING_PATH} line {line}: {why}")]
    Line { line: usize, why: &'static str },
    /// A case, or a cell of a case, is listed twice.
    #[error("{MAPPING_PATH} line {line}: {item} is already listed on line {first}")]
    Duplicate {
        line: usize,
        item: String,
        first: usize,
    },
    /// A line names a case that the `probes` suite does not discover.
    #[error("{MAPPING_PATH} line {line}: no case {case}")]
    UnknownCase { line: usize, case: String },
    /// A cell line names a cell that its case does not check.
    #[error("{MAPPING_PATH} line {line}: {case} has no checked cell {cell}")]
    UnknownCell {
        line: usize,
        case: String,
        cell: String,
    },
}

impl Mapping {
    /// Parses the mapping file's text. Lines are tab-separated: `case, milestone, subject`, or
    /// `case, Sheet!A1, milestone, subject` for one cell; `#` starts a comment line.
    ///
    /// # Errors
    /// The first malformed line, or the first that repeats a case or a cell.
    pub fn parse(text: &str) -> Result<Mapping, MappingError> {
        let mut mapping = Mapping::default();
        for (n, raw) in text.lines().enumerate() {
            let line = n + 1;
            if raw.trim().is_empty() || raw.starts_with('#') {
                continue;
            }
            let bad = |why| MappingError::Line { line, why };
            let fields: Vec<&str> = raw.split('\t').collect();
            let (case, cell, milestone, subject) = match fields.as_slice() {
                [case, milestone, subject] => (*case, None, *milestone, *subject),
                [case, cell, milestone, subject] => (*case, Some(*cell), *milestone, *subject),
                _ => {
                    return Err(bad(
                        "expected `case<TAB>milestone<TAB>subject` or `case<TAB>Sheet!A1<TAB>milestone<TAB>subject`",
                    ))
                }
            };
            if !case.starts_with("probes/") || case.contains(char::is_whitespace) {
                return Err(bad("the case id must be `probes/<path>`"));
            }
            if subject.trim().is_empty() {
                return Err(bad("the subject is empty"));
            }
            let milestone =
                parse_milestone(milestone).ok_or(bad("the milestone must be `M<digits>`"))?;
            let assigned = Assigned { line, milestone };
            let first = match cell {
                None => mapping.cases.insert(case.to_string(), assigned),
                Some(cell) => {
                    let key = split_qualified(cell)
                        .and_then(|(sheet, a1)| Some(format!("{sheet}!{}", Addr::parse(&a1)?)))
                        .ok_or(bad("the cell must be `Sheet!A1`"))?;
                    mapping.cells.insert((case.to_string(), key), assigned)
                }
            };
            if let Some(first) = first {
                return Err(MappingError::Duplicate {
                    line,
                    item: cell.map_or_else(|| case.to_string(), |c| format!("{case} {c}")),
                    first: first.line,
                });
            }
        }
        Ok(mapping)
    }

    /// Reads [`MAPPING_PATH`] under `repo_root`; an absent file is an empty mapping.
    ///
    /// # Errors
    /// When the file is unreadable or malformed.
    pub fn load(repo_root: &Path) -> Result<Mapping, MappingError> {
        let path = repo_root.join(MAPPING_PATH);
        if !path.exists() {
            return Ok(Mapping::default());
        }
        let text = std::fs::read_to_string(&path).map_err(|e| MappingError::Unreadable {
            message: e.to_string(),
        })?;
        Mapping::parse(&text)
    }

    /// Checks that every line names a case the suites discover (`known` holds their case ids).
    ///
    /// # Errors
    /// The first line, in file order, that names an unknown case.
    pub fn check_known(&self, known: &BTreeSet<String>) -> Result<(), MappingError> {
        let cases = self.cases.iter().map(|(c, a)| (c, a.line));
        let cells = self.cells.iter().map(|((c, _), a)| (c, a.line));
        match cases
            .chain(cells)
            .filter(|(c, _)| !known.contains(*c))
            .min_by_key(|(_, line)| *line)
        {
            Some((case, line)) => Err(MappingError::UnknownCase {
                line,
                case: case.clone(),
            }),
            None => Ok(()),
        }
    }

    /// Checks that every cell line names a checked cell of its case, so a typo cannot leave a cell
    /// counted at its case's milestone unnoticed. `expectations` maps a case id to the cells the
    /// case holds an expectation for (the keys of its `expected` and of its oracle sidecar, as
    /// `Sheet!A1`; `case::expectation_cells`). A case it does not list is not checked here:
    /// [`Mapping::check_known`] rejects unknown cases.
    ///
    /// # Errors
    /// The first line, in file order, whose cell its case holds no expectation for.
    pub fn check_cells(
        &self,
        expectations: &BTreeMap<String, BTreeSet<String>>,
    ) -> Result<(), MappingError> {
        match self
            .cells
            .iter()
            .filter(|((case, cell), _)| {
                expectations
                    .get(case)
                    .is_some_and(|cells| !cells.contains(cell))
            })
            .min_by_key(|(_, a)| a.line)
        {
            Some(((case, cell), a)) => Err(MappingError::UnknownCell {
                line: a.line,
                case: case.clone(),
                cell: cell.clone(),
            }),
            None => Ok(()),
        }
    }
}

/// Applies rules 1–5.
#[derive(Debug, Clone, Default)]
pub struct Rules {
    frozen_m1: BTreeSet<String>,
    mapping: Mapping,
    /// Catalog function name → milestone number.
    catalog: BTreeMap<String, u32>,
}

impl Rules {
    /// The rules from the frozen lists (only [`FROZEN_M1_LIST`] matters), the mapping file and
    /// the catalog.
    #[must_use]
    pub fn new(lists: &[MilestoneList], mapping: Mapping, catalog: &Catalog) -> Rules {
        Rules {
            frozen_m1: lists
                .iter()
                .filter(|l| l.name == FROZEN_M1_LIST)
                .flat_map(|l| l.cases.iter().map(|c| c.id.clone()))
                .collect(),
            mapping,
            catalog: catalog
                .functions()
                .filter_map(|f| Some((f.name.clone(), parse_milestone(&f.milestone)?)))
                .collect(),
        }
    }

    /// The milestone of the whole case: rules 1, 2 (case lines), 4 and 5. A catalog case's own
    /// milestone (for items that are not cells) is M2.
    fn case_milestone(&self, r: &CaseResult) -> u32 {
        if self.frozen_m1.contains(&r.id) {
            return 1;
        }
        if let Some(a) = self.mapping.cases.get(&r.id) {
            return a.milestone;
        }
        if r.id.starts_with(CATALOG_PREFIX) {
            return DEFAULT_MILESTONE;
        }
        r.tags
            .iter()
            .filter_map(|t| parse_milestone(t))
            .max()
            .unwrap_or(DEFAULT_MILESTONE)
    }

    /// The milestone of one scored cell of `r`.
    fn cell_milestone(&self, r: &CaseResult, cell: &CellResult) -> u32 {
        if self.frozen_m1.contains(&r.id) {
            return 1;
        }
        if let Some(a) = self.mapping.cells.get(&(r.id.clone(), cell.key())) {
            return a.milestone;
        }
        if r.id.starts_with(CATALOG_PREFIX) && !self.mapping.cases.contains_key(&r.id) {
            return cell
                .formula
                .as_deref()
                .map(functions_in)
                .unwrap_or_default()
                .iter()
                .filter(|f| !OBSERVING_WRAPPERS.contains(&f.as_str()))
                .filter_map(|f| self.catalog.get(f).copied())
                .max()
                .unwrap_or(DEFAULT_MILESTONE);
        }
        self.case_milestone(r)
    }
}

/// How the probe cases assigned to one milestone scored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneRow {
    /// `M0-M2`, `M5`, … `M11`.
    pub milestone: String,
    /// Cases with at least one item in this milestone (a catalog case can count in several).
    pub cases: usize,
    /// Scored cells.
    pub cells: usize,
    /// Cells that match Excel bit-exact.
    pub bit_exact: usize,
    /// Cells that gave their pinned by-design value (decision 0036).
    pub deviations: usize,
    /// Known-failure items, counted as the baseline counts its F lines.
    pub known_failures: usize,
    /// The cases behind `known_failures`: case id and its known-failure items in this row,
    /// most first, then by id.
    pub failing_cases: Vec<(String, usize)>,
}

/// Scores the `probes` cases of `results` per milestone; `mode` is the suite's baseline mode.
/// Empty when the run executed no `probes` case.
#[must_use]
pub fn summarize(rules: &Rules, results: &[CaseResult], mode: BaselineMode) -> Vec<MilestoneRow> {
    let probes: Vec<&CaseResult> = results.iter().filter(|r| r.suite == SUITE).collect();
    if probes.is_empty() {
        return Vec::new();
    }
    let mut rows: BTreeMap<u32, MilestoneRow> = REPORT_ROWS
        .iter()
        .map(|&m| (m, MilestoneRow::default()))
        .collect();
    for r in probes {
        let mut rows_of_case: BTreeSet<u32> = BTreeSet::new();
        let cell_rows: BTreeMap<String, u32> = r
            .cells
            .iter()
            .map(|c| (c.key(), row_of(rules.cell_milestone(r, c))))
            .collect();
        for c in &r.cells {
            let row = cell_rows
                .get(&c.key())
                .copied()
                .unwrap_or(DEFAULT_MILESTONE);
            let s = rows.entry(row).or_default();
            s.cells += 1;
            if c.verdict.is_pass() {
                s.bit_exact += 1;
            } else if c.verdict == Verdict::Deviation {
                s.deviations += 1;
            }
            rows_of_case.insert(row);
        }
        let case_row = row_of(rules.case_milestone(r));
        if r.cells.is_empty() {
            rows_of_case.insert(case_row);
        }
        // Items that are not cells (`*`, `!Panic`, …) count at the case's own milestone.
        for (_, item) in Baseline::from_results(mode, &[r]).failing {
            let row = cell_rows.get(&item).copied().unwrap_or(case_row);
            let s = rows.entry(row).or_default();
            s.known_failures += 1;
            // Only this case appends during this iteration, so the row's last entry is this case's if present.
            match s.failing_cases.last_mut() {
                Some((id, n)) if *id == r.id => *n += 1,
                _ => s.failing_cases.push((r.id.clone(), 1)),
            }
            rows_of_case.insert(row);
        }
        for row in rows_of_case {
            rows.entry(row).or_default().cases += 1;
        }
    }
    rows.into_iter()
        .map(|(row, mut s)| {
            s.failing_cases
                .sort_by(|(a, na), (b, nb)| nb.cmp(na).then_with(|| a.cmp(b)));
            MilestoneRow {
                milestone: row_label(row),
                ..s
            }
        })
        .collect()
}

/// The report.md section for `rows`; nothing when there are none.
pub fn write_markdown(rows: &[MilestoneRow], md: &mut String) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(
        md,
        "\n## Probe milestones (decision 0051)\n\n`probes` cases (and `probes/catalog/*` cells) by \
         the milestone whose exit they count toward: the frozen M1 list, then `{MAPPING_PATH}`, \
         then each catalog cell's functions, then the case's milestone tag, else M2. Known \
         failures count items as the baseline counts F lines.\n"
    );
    let _ = writeln!(
        md,
        "| milestone | cases | scored cells | bit-exact | deviations | known failures (cells) |\n\
         |---|---|---|---|---|---|"
    );
    for s in rows {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} |",
            s.milestone, s.cases, s.cells, s.bit_exact, s.deviations, s.known_failures
        );
    }
    for s in rows.iter().filter(|s| !s.failing_cases.is_empty()) {
        let _ = writeln!(md, "\nFailing cases in {}:\n", s.milestone);
        for (id, n) in &s.failing_cases {
            let _ = writeln!(md, "- `{id}`: {n}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogFunction, Scope};
    use crate::milestone_lists::ListedCase;
    use crate::runner::CaseStatus;
    use crate::value::Value;

    #[test]
    fn parses_case_and_cell_lines_and_skips_comments() {
        let mapping = Mapping::parse(
            "# header\n\nprobes/a.case.json\tM9\tGEOMEAN\nprobes/b.case.json\tSheet1!$H$28\tM9\tGEOMEAN\n",
        )
        .expect("parse");
        assert_eq!(
            mapping.cases.get("probes/a.case.json"),
            Some(&Assigned {
                line: 3,
                milestone: 9
            })
        );
        assert_eq!(
            mapping
                .cells
                .get(&("probes/b.case.json".to_string(), "Sheet1!H28".to_string())),
            Some(&Assigned {
                line: 4,
                milestone: 9
            })
        );
    }

    #[test]
    fn rejects_duplicates_and_bad_milestones() {
        assert_eq!(
            Mapping::parse("probes/a.case.json\tM9\tx\nprobes/a.case.json\tM6\ty\n"),
            Err(MappingError::Duplicate {
                line: 2,
                item: "probes/a.case.json".into(),
                first: 1
            })
        );
        let dup_cell = "probes/a.case.json\tS!A1\tM9\tx\nprobes/a.case.json\tS!$A$1\tM6\ty\n";
        assert!(matches!(
            Mapping::parse(dup_cell),
            Err(MappingError::Duplicate {
                line: 2,
                first: 1,
                ..
            })
        ));
        for bad in ["M", "9", "m9", "M9a", "M-1", "Mx"] {
            let err = Mapping::parse(&format!("probes/a.case.json\t{bad}\tx\n")).expect_err(bad);
            assert_eq!(
                err.to_string(),
                format!("{MAPPING_PATH} line 1: the milestone must be `M<digits>`"),
                "{bad}"
            );
        }
        for bad in [
            "probes/a.case.json\tM9\n",
            "a.case.json\tM9\tx\n",
            "probes/a.case.json\tA1\tM9\tx\n",
            "probes/a.case.json\tM9\t \n",
        ] {
            assert!(
                matches!(Mapping::parse(bad), Err(MappingError::Line { line: 1, .. })),
                "{bad}"
            );
        }
    }

    #[test]
    fn an_unknown_case_names_its_line() {
        let mapping =
            Mapping::parse("# c\nprobes/a.case.json\tM9\tx\nprobes/gone.case.json\tS!A1\tM9\tx\n")
                .expect("parse");
        let known: BTreeSet<String> = ["probes/a.case.json".to_string()].into();
        assert_eq!(
            mapping
                .check_known(&known)
                .expect_err("unknown")
                .to_string(),
            format!("{MAPPING_PATH} line 3: no case probes/gone.case.json")
        );
        let known: BTreeSet<String> = ["probes/a.case.json", "probes/gone.case.json"]
            .map(String::from)
            .into();
        assert_eq!(mapping.check_known(&known), Ok(()));
    }

    #[test]
    fn an_unchecked_cell_names_its_line() {
        let mapping = Mapping::parse(
            "# c\nprobes/a.case.json\tS!A1\tM9\tx\nprobes/a.case.json\tS!B2\tM9\tx\nprobes/b.case.json\tS!Z9\tM9\tx\n",
        )
        .expect("parse");
        let cells = |keys: &[&str]| keys.iter().map(ToString::to_string).collect();
        let mut expectations =
            BTreeMap::from([("probes/a.case.json".to_string(), cells(&["S!A1"]))]);
        assert_eq!(
            mapping
                .check_cells(&expectations)
                .expect_err("unchecked")
                .to_string(),
            format!("{MAPPING_PATH} line 3: probes/a.case.json has no checked cell S!B2")
        );
        // A case the expectations do not list (probes/b) is left to `check_known`.
        expectations.insert("probes/a.case.json".to_string(), cells(&["S!A1", "S!B2"]));
        assert_eq!(mapping.check_cells(&expectations), Ok(()));
    }

    #[test]
    fn the_committed_mapping_parses() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mapping = Mapping::load(&root).unwrap_or_else(|e| panic!("{e}"));
        assert!(mapping.cases.len() >= 100);
        assert_eq!(
            mapping
                .cells
                .get(&(
                    "probes/subtotal-reference-operands-d.case.json".to_string(),
                    "Sheet1!H28".to_string()
                ))
                .map(|a| a.milestone),
            Some(9)
        );
    }

    fn cell(key: &str, formula: &str, verdict: Verdict) -> CellResult {
        let (sheet, a1) = key.split_once('!').unwrap_or(("S", key));
        CellResult {
            sheet: sheet.into(),
            cell: a1.into(),
            formula: Some(formula.into()),
            expected: Value::Number(1.0),
            actual: Value::Number(1.0),
            verdict,
            from_excel: true,
            listed_deviation: verdict == Verdict::Deviation,
            graph_subnormal: false,
        }
    }

    fn case(id: &str, tags: &[&str], status: CaseStatus, cells: Vec<CellResult>) -> CaseResult {
        CaseResult {
            id: id.into(),
            suite: SUITE.into(),
            status,
            message: None,
            expected_source: String::new(),
            checked: cells.len(),
            passed: cells.iter().filter(|c| c.verdict.is_pass()).count(),
            deviations: cells
                .iter()
                .filter(|c| c.verdict == Verdict::Deviation)
                .count(),
            skipped_volatile: 0,
            millis: 0,
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
            cells,
        }
    }

    fn rules(mapping: &str) -> Rules {
        let frozen = MilestoneList {
            name: FROZEN_M1_LIST.into(),
            cases: vec![ListedCase {
                line: 1,
                id: "probes/frozen.case.json".into(),
            }],
        };
        let catalog = Catalog::from_functions(
            [
                ("GEOMEAN", "M9"),
                ("XLOOKUP", "M6"),
                ("SUM", "M2"),
                ("OR", "M2"),
            ]
            .map(|(name, milestone)| CatalogFunction {
                name: name.into(),
                category: "math".into(),
                milestone: milestone.into(),
                scope: Scope::In,
                reason: None,
                oracle: None,
            }),
        );
        Rules::new(
            &[frozen],
            Mapping::parse(mapping).expect("mapping"),
            &catalog,
        )
    }

    fn pass(key: &str, formula: &str) -> CellResult {
        cell(key, formula, Verdict::Pass)
    }

    #[test]
    fn rule_1_a_frozen_case_stays_m1_even_when_listed_or_tagged_later() {
        let rules = rules("probes/frozen.case.json\tM9\tx\n");
        let r = case(
            "probes/frozen.case.json",
            &["M7"],
            CaseStatus::Pass,
            vec![pass("S!A1", "GEOMEAN(1)")],
        );
        assert_eq!(rules.case_milestone(&r), 1);
        assert_eq!(rules.cell_milestone(&r, &r.cells[0]), 1);
    }

    #[test]
    fn rule_2_a_line_beats_the_tag_and_a_cell_line_beats_the_case() {
        let rules = rules("probes/a.case.json\tM9\tx\nprobes/b.case.json\tS!A2\tM9\ty\n");
        let a = case("probes/a.case.json", &["M1"], CaseStatus::Pass, vec![]);
        assert_eq!(rules.case_milestone(&a), 9);
        let b = case(
            "probes/b.case.json",
            &["M2"],
            CaseStatus::Pass,
            vec![pass("S!A1", "SUM(1)"), pass("S!A2", "GEOMEAN(5,7)")],
        );
        assert_eq!(rules.cell_milestone(&b, &b.cells[0]), 2);
        assert_eq!(rules.cell_milestone(&b, &b.cells[1]), 9);
    }

    #[test]
    fn rule_3_a_catalog_cell_counts_at_its_functions_milestone() {
        let rules = rules("");
        let r = case(
            "probes/catalog/availability-01.case.json",
            &["M0"],
            CaseStatus::Pass,
            vec![
                pass("S!A1", "OR(ISERROR(_xlfn.XLOOKUP(1,A:A,B:B)))"),
                pass("S!A2", "IFERROR(GEOMEAN(1),\"SUM(\")"),
                pass("S!A3", "ERROR.TYPE(#N/A)"),
                pass("S!A4", "1+1"),
                pass("S!A5", "OR(SUM(1),GEOMEAN(2))"),
            ],
        );
        let got: Vec<u32> = r
            .cells
            .iter()
            .map(|c| rules.cell_milestone(&r, c))
            .collect();
        assert_eq!(got, [6, 9, 2, 2, 9]);
    }

    #[test]
    fn rules_4_and_5_the_highest_tag_else_m2() {
        let rules = rules("");
        let tagged = case(
            "probes/t.case.json",
            &["lookup", "M6", "M9"],
            CaseStatus::Pass,
            vec![],
        );
        assert_eq!(rules.case_milestone(&tagged), 9);
        let untagged = case("probes/u.case.json", &["lookup"], CaseStatus::Pass, vec![]);
        assert_eq!(rules.case_milestone(&untagged), 2);
    }

    #[test]
    fn scores_rows_as_the_baseline_counts_them() {
        let rules = rules("probes/d.case.json\tS!A2\tM9\tGEOMEAN\n");
        let results = [
            // M2 cells and one M9 cell override; both failures count where their cell does.
            case(
                "probes/d.case.json",
                &["M2"],
                CaseStatus::Fail,
                vec![
                    cell("S!A1", "SUM(1)", Verdict::NumericMismatch),
                    cell("S!A2", "GEOMEAN(5,7)", Verdict::NumericMismatch),
                    cell("S!A3", "SUM(2)", Verdict::Deviation),
                ],
            ),
            // A whole-case failure counts at the case's milestone.
            case("probes/p.case.json", &["M7"], CaseStatus::Panic, vec![]),
            case(
                "probes/catalog/c.case.json",
                &["M0"],
                CaseStatus::Fail,
                vec![
                    pass("S!A1", "XLOOKUP(1,A1,B1)"),
                    cell("S!A2", "GEOMEAN(1)", Verdict::TypeMismatch),
                ],
            ),
            case(
                "probes/m1.case.json",
                &["M1"],
                CaseStatus::Pass,
                vec![pass("S!A1", "1")],
            ),
            case(
                "probes/m12.case.json",
                &["M12"],
                CaseStatus::Pending,
                vec![],
            ),
            // Another suite is ignored.
            CaseResult {
                suite: "other".into(),
                ..case("other/x.case.json", &["M5"], CaseStatus::Panic, vec![])
            },
        ];
        let rows = summarize(&rules, &results, BaselineMode::Cells);
        let row = |m: &str, cases, cells, bit_exact, deviations, failing: &[(&str, usize)]| {
            MilestoneRow {
                milestone: m.into(),
                cases,
                cells,
                bit_exact,
                deviations,
                known_failures: failing.iter().map(|f| f.1).sum(),
                failing_cases: failing.iter().map(|f| (f.0.to_owned(), f.1)).collect(),
            }
        };
        assert_eq!(
            rows,
            [
                row("M0-M2", 2, 3, 1, 1, &[("probes/d.case.json", 1)]),
                row("M5", 0, 0, 0, 0, &[]),
                row("M6", 1, 1, 1, 0, &[]),
                row("M7", 1, 0, 0, 0, &[("probes/p.case.json", 1)]),
                row("M8", 0, 0, 0, 0, &[]),
                row(
                    "M9",
                    2,
                    2,
                    0,
                    0,
                    &[("probes/catalog/c.case.json", 1), ("probes/d.case.json", 1)]
                ),
                row("M10", 0, 0, 0, 0, &[]),
                row("M11", 0, 0, 0, 0, &[]),
                row("M12", 1, 0, 0, 0, &[]),
            ]
        );
        let mut md = String::new();
        write_markdown(&rows, &mut md);
        assert!(md.contains("## Probe milestones (decision 0051)"), "{md}");
        assert!(md.contains("| M0-M2 | 2 | 3 | 1 | 1 | 1 |"), "{md}");
        assert!(md.contains("| M9 | 2 | 2 | 0 | 0 | 2 |"), "{md}");
        assert!(
            md.contains("Failing cases in M9:\n\n- `probes/catalog/c.case.json`: 1\n- `probes/d.case.json`: 1\n"),
            "{md}"
        );
        assert!(summarize(&rules, &results[5..], BaselineMode::Cells).is_empty());
    }
}
