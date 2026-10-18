//! Frozen milestone probe lists (decision 0043): `eval/milestones/<name>.txt` names the cases a
//! milestone's probe criterion counts, one case id per line in the form of the baseline files
//! (`probes/<path>.case.json`); `#` starts a comment line. The report scores each list the way the
//! baseline does, so its known failures equal the list's F lines in `eval/baseline/` (decision
//! 0044).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::baseline::Baseline;
use crate::runner::CaseResult;
use crate::suite::BaselineMode;

/// Directory of the lists, relative to the repository root.
pub const LISTS_DIR: &str = "eval/milestones";

/// One list line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedCase {
    /// 1-based line number in the list file.
    pub line: usize,
    /// Case id, `<suite>/<path relative to the suite root>`.
    pub id: String,
}

/// One `eval/milestones/<name>.txt` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilestoneList {
    /// File stem (`m1-probes`).
    pub name: String,
    /// The listed cases, in file order.
    pub cases: Vec<ListedCase>,
}

/// Why a list file or one of its lines is unusable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ListError {
    /// The directory or a list file cannot be read.
    #[error("{path}: {message}")]
    Unreadable { path: String, message: String },
    /// A line is not a case id.
    #[error("eval/milestones/{list}.txt line {line}: {why}")]
    Line {
        list: String,
        line: usize,
        why: &'static str,
    },
    /// A case is listed twice.
    #[error("eval/milestones/{list}.txt line {line}: {case} is already listed on line {first}")]
    Duplicate {
        list: String,
        line: usize,
        case: String,
        first: usize,
    },
    /// A line names a case that no suite discovers.
    #[error("eval/milestones/{list}.txt line {line}: no case {case}")]
    UnknownCase {
        list: String,
        line: usize,
        case: String,
    },
}

/// Parses the text of the list called `list`.
///
/// # Errors
/// The first line that is not a case id, or that repeats an earlier one.
pub fn parse(list: &str, text: &str) -> Result<Vec<ListedCase>, ListError> {
    let mut first_line: BTreeMap<&str, usize> = BTreeMap::new();
    let mut out = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = n + 1;
        let id = raw.trim();
        if id.is_empty() || id.starts_with('#') {
            continue;
        }
        if id.contains(char::is_whitespace) || !id.contains('/') {
            return Err(ListError::Line {
                list: list.to_string(),
                line,
                why: "expected one case id, `<suite>/<path>`",
            });
        }
        if let Some(&first) = first_line.get(id) {
            return Err(ListError::Duplicate {
                list: list.to_string(),
                line,
                case: id.to_string(),
                first,
            });
        }
        first_line.insert(id, line);
        out.push(ListedCase {
            line,
            id: id.to_string(),
        });
    }
    Ok(out)
}

fn unreadable(path: &Path, e: &std::io::Error) -> ListError {
    ListError::Unreadable {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}

/// Every list under `<repo_root>/eval/milestones`, by name; none when the directory is absent.
///
/// # Errors
/// When a list is unreadable or malformed.
pub fn load_all(repo_root: &Path) -> Result<Vec<MilestoneList>, ListError> {
    let dir = repo_root.join(LISTS_DIR);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut lists = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| unreadable(&dir, &e))? {
        let path = entry.map_err(|e| unreadable(&dir, &e))?.path();
        if path.extension().is_none_or(|ext| ext != "txt") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let text = std::fs::read_to_string(&path).map_err(|e| unreadable(&path, &e))?;
        lists.push(MilestoneList {
            name: name.to_string(),
            cases: parse(name, &text)?,
        });
    }
    lists.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(lists)
}

/// Checks that every listed case is one the suites discover (`known` holds their case ids).
///
/// # Errors
/// The first line naming an unknown case.
pub fn check_known(lists: &[MilestoneList], known: &BTreeSet<String>) -> Result<(), ListError> {
    for list in lists {
        if let Some(c) = list.cases.iter().find(|c| !known.contains(&c.id)) {
            return Err(ListError::UnknownCase {
                list: list.name.clone(),
                line: c.line,
                case: c.id.clone(),
            });
        }
    }
    Ok(())
}

/// How one list scored in a run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListSummary {
    /// The list's name (`m1-probes`).
    pub name: String,
    /// Listed cases.
    pub cases: usize,
    /// Listed cases the run executed (fewer under `--suite` or `--filter`).
    pub cases_run: usize,
    /// Scored cells of the listed cases.
    pub cells: usize,
    /// Cells that match Excel bit-exact.
    pub bit_exact: usize,
    /// Cells that gave their pinned by-design value (decision 0036).
    pub deviations: usize,
    /// Known-failure items, counted as the baseline counts its F lines.
    pub known_failures: usize,
    /// Listed cases with at least one known failure.
    pub failing_cases: Vec<String>,
}

/// Scores `list` from the run's `results`; `modes` maps a suite name to its baseline mode.
#[must_use]
pub fn summarize(
    list: &MilestoneList,
    results: &[CaseResult],
    modes: &BTreeMap<String, BaselineMode>,
) -> ListSummary {
    let by_id: BTreeMap<&str, &CaseResult> = results.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut s = ListSummary {
        name: list.name.clone(),
        cases: list.cases.len(),
        ..ListSummary::default()
    };
    for r in list.cases.iter().filter_map(|c| by_id.get(c.id.as_str())) {
        s.cases_run += 1;
        s.cells += r.checked;
        s.bit_exact += r.passed;
        s.deviations += r.deviations;
        let mode = modes.get(&r.suite).copied().unwrap_or(BaselineMode::Cells);
        let failing = Baseline::from_results(mode, &[*r]).failing.len();
        if failing > 0 {
            s.known_failures += failing;
            s.failing_cases.push(r.id.clone());
        }
    }
    s
}

/// The report.md section for `lists`; nothing when there are none.
pub fn write_markdown(lists: &[ListSummary], md: &mut String) {
    if lists.is_empty() {
        return;
    }
    let _ = writeln!(
        md,
        "\n## Milestone probe lists\n\nFrozen lists under `{LISTS_DIR}/` (decisions 0043, 0044). \
         Known failures count items as the baseline counts F lines.\n"
    );
    let _ = writeln!(
        md,
        "| list | cases | scored cells | bit-exact | deviations | known failures (cells) | failing cases |\n\
         |---|---|---|---|---|---|---|"
    );
    for s in lists {
        let cases = if s.cases_run == s.cases {
            s.cases.to_string()
        } else {
            format!("{} ({} not run)", s.cases, s.cases - s.cases_run)
        };
        let failing: Vec<String> = s.failing_cases.iter().map(|c| format!("`{c}`")).collect();
        let _ = writeln!(
            md,
            "| {} | {cases} | {} | {} | {} | {} | {} |",
            s.name,
            s.cells,
            s.bit_exact,
            s.deviations,
            s.known_failures,
            failing.join(", ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{CaseStatus, CellResult};
    use crate::value::{Value, Verdict};

    #[test]
    fn skips_comments_and_blank_lines() {
        let parsed = parse(
            "m1",
            "# frozen\n\nprobes/a.case.json\n  # note\nprobes/b.case.json\n",
        )
        .expect("parse");
        assert_eq!(
            parsed,
            [
                ListedCase {
                    line: 3,
                    id: "probes/a.case.json".into()
                },
                ListedCase {
                    line: 5,
                    id: "probes/b.case.json".into()
                },
            ]
        );
    }

    #[test]
    fn rejects_duplicates_and_malformed_lines() {
        assert_eq!(
            parse(
                "m1",
                "probes/a.case.json\nprobes/b.case.json\nprobes/a.case.json\n"
            ),
            Err(ListError::Duplicate {
                list: "m1".into(),
                line: 3,
                case: "probes/a.case.json".into(),
                first: 1,
            })
        );
        for bad in ["a.case.json", "probes/a.case.json probes/b.case.json"] {
            assert!(
                matches!(parse("m1", bad), Err(ListError::Line { line: 1, .. })),
                "{bad}"
            );
        }
    }

    #[test]
    fn an_unknown_case_names_its_line() {
        let list = MilestoneList {
            name: "m1-probes".into(),
            cases: parse(
                "m1-probes",
                "# c\nprobes/a.case.json\nprobes/gone.case.json\n",
            )
            .expect("parse"),
        };
        let known: BTreeSet<String> = ["probes/a.case.json".to_string()].into();
        let err = check_known(std::slice::from_ref(&list), &known).expect_err("unknown");
        assert_eq!(
            err.to_string(),
            "eval/milestones/m1-probes.txt line 3: no case probes/gone.case.json"
        );
        let known: BTreeSet<String> = list.cases.iter().map(|c| c.id.clone()).collect();
        assert_eq!(check_known(&[list], &known), Ok(()));
    }

    #[test]
    fn loads_every_txt_list_by_name() {
        let root = std::env::temp_dir().join(format!("milestone-lists-{}", std::process::id()));
        let dir = root.join(LISTS_DIR);
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(dir.join("m2-probes.txt"), "probes/b.case.json\n").expect("write");
        std::fs::write(dir.join("m1-probes.txt"), "probes/a.case.json\n").expect("write");
        std::fs::write(dir.join("README.md"), "not a list\n").expect("write");
        let lists = load_all(&root);
        let _ = std::fs::remove_dir_all(&root);
        let names: Vec<String> = lists.expect("load").into_iter().map(|l| l.name).collect();
        assert_eq!(names, ["m1-probes", "m2-probes"]);
    }

    fn result(id: &str, status: CaseStatus, verdicts: &[Verdict]) -> CaseResult {
        let cells: Vec<CellResult> = verdicts
            .iter()
            .enumerate()
            .map(|(i, &verdict)| CellResult {
                sheet: "S".into(),
                cell: format!("A{}", i + 1),
                formula: None,
                anchor_formula: None,
                expected: Value::Number(1.0),
                actual: Value::Number(1.0),
                verdict,
                from_excel: true,
                listed_deviation: verdict == Verdict::Deviation,
                graph_subnormal: false,
            })
            .collect();
        CaseResult {
            id: id.into(),
            suite: "probes".into(),
            status,
            message: None,
            expected_source: String::new(),
            checked: cells.len(),
            passed: verdicts.iter().filter(|v| **v == Verdict::Pass).count(),
            deviations: verdicts
                .iter()
                .filter(|v| **v == Verdict::Deviation)
                .count(),
            skipped_volatile: 0,
            millis: 0,
            tags: Vec::new(),
            cells,
            text: Vec::new(),
        }
    }

    #[test]
    fn scores_a_list_as_the_baseline_counts_it() {
        let list = MilestoneList {
            name: "m1-probes".into(),
            cases: parse(
                "m1-probes",
                "probes/a.case.json\nprobes/b.case.json\nprobes/c.case.json\nprobes/d.case.json\n",
            )
            .expect("parse"),
        };
        let results = [
            result(
                "probes/a.case.json",
                CaseStatus::Fail,
                &[Verdict::Pass, Verdict::NumericMismatch, Verdict::Deviation],
            ),
            result("probes/b.case.json", CaseStatus::Pass, &[Verdict::Pass]),
            result("probes/c.case.json", CaseStatus::Panic, &[]),
            // not listed
            result(
                "probes/z.case.json",
                CaseStatus::Fail,
                &[Verdict::NumericMismatch],
            ),
        ];
        let modes: BTreeMap<String, BaselineMode> =
            [("probes".to_string(), BaselineMode::Cells)].into();
        let s = summarize(&list, &results, &modes);
        assert_eq!(
            s,
            ListSummary {
                name: "m1-probes".into(),
                cases: 4,
                cases_run: 3,
                cells: 4,
                bit_exact: 2,
                deviations: 1,
                known_failures: 2,
                failing_cases: vec!["probes/a.case.json".into(), "probes/c.case.json".into()],
            }
        );
        let mut md = String::new();
        write_markdown(&[s], &mut md);
        assert!(md.contains("## Milestone probe lists"), "{md}");
        assert!(
            md.contains(
                "| m1-probes | 4 (1 not run) | 4 | 2 | 1 | 2 | `probes/a.case.json`, `probes/c.case.json` |"
            ),
            "{md}"
        );
    }
}
