//! Glue-aware per-function attribution and worst-cell parity tiers (decision 0118, SPEC-0022,
//! T-0068), and per-suite workbook closeness.
//!
//! [`FunctionStat::closeness`] counts a cell toward every function its formula calls, so one off
//! sub-expression counts against every function of a long formula. Decision 0118 instead counts
//! an Excel cell toward its formula's sole non-glue call ([`crate::closeness::sole_call`]). The report
//! carries both counts next to the existing ones, and a tier per function placed by its worst
//! attributed cell. It computes no verdict and gates nothing: T-0080 points `v15_checkpoint` at
//! these counts, and the progress dashboard (T-0081) renders them.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::closeness::{Closeness, Visible};
use crate::evidence_floor::EVIDENCE_FLOOR;
use crate::formula_calls::{calls_only_in_scope, functions_in};
use crate::milestone_exit::{meets_rate, RATE99_NUM, RATE_NUM};
use crate::report::{FunctionStat, RunReport};
use crate::runner::{CaseResult, CellResult};
use crate::value::Verdict;

/// A visibly-off Excel cell that decision 0118 attributes to no function (item 6): its in-scope
/// formula calls two or more distinct non-glue functions, or only glue with two or more distinct
/// calls. It still needs an owning task or a `DEVIATION` before Phase 1 exits; the report lists
/// them all and `scripts/exit_owners.py --unattributed` drops those an open task owns (T-0082).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnattributedCell {
    /// The case id, `suite/case`.
    pub case: String,
    /// `Sheet!A1`.
    pub cell: String,
    /// The cell's formula as its case records it.
    pub formula: String,
}

/// A visibly-off Excel cell attributed to one function by decision 0118 ([`crate::closeness::sole_call`]),
/// with what `scripts/exit_owners.py --all-off` needs to list it (T-0086).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributedCell {
    /// The case id, `suite/case`.
    pub case: String,
    /// `Sheet!A1`.
    pub cell: String,
    /// The function the cell is attributed to.
    pub function: String,
    /// The cell's formula as its case records it.
    pub formula: String,
    /// Excel's value, as the report prints it.
    pub expected: String,
    /// The engine's value, as the report prints it.
    pub actual: String,
}

/// The in-scope, visibly-off, non-deviation Excel cells of `results` with the distinct functions
/// their formulas call (at least one), in case order. The one rule behind both lists below.
fn off_excel_cells<'a>(
    catalog: &'a Catalog,
    results: &'a [CaseResult],
) -> impl Iterator<Item = (&'a CaseResult, &'a CellResult, BTreeSet<String>)> + 'a {
    results.iter().flat_map(move |r| {
        r.cells.iter().filter_map(move |c| {
            if !c.from_excel || c.verdict == Verdict::Deviation || !Visible::is_off(c) {
                return None;
            }
            let called = c.formula.as_deref().map(functions_in).unwrap_or_default();
            (!called.is_empty() && calls_only_in_scope(catalog, &called)).then_some((r, c, called))
        })
    })
}

/// The [`UnattributedCell`]s of `results`, in case order: the visibly-off, non-deviation Excel
/// cells whose formula calls only in-scope functions, at least one, and no sole one.
#[must_use]
pub fn unattributed_off(catalog: &Catalog, results: &[CaseResult]) -> Vec<UnattributedCell> {
    off_excel_cells(catalog, results)
        .filter(|(_, _, called)| crate::closeness::sole_call(called).is_none())
        .map(|(r, c, _)| UnattributedCell {
            case: r.id.clone(),
            cell: c.key(),
            formula: c.formula.clone().unwrap_or_default(),
        })
        .collect()
}

/// The [`AttributedCell`]s of `results`, in case order: the cells [`unattributed_off`] leaves out.
#[must_use]
pub fn attributed_off(catalog: &Catalog, results: &[CaseResult]) -> Vec<AttributedCell> {
    off_excel_cells(catalog, results)
        .filter_map(|(r, c, called)| {
            let function = crate::closeness::sole_call(&called)?.clone();
            Some(AttributedCell {
                case: r.id.clone(),
                cell: c.key(),
                function,
                formula: c.formula.clone().unwrap_or_default(),
                expected: c.expected.to_string(),
                actual: c.actual.to_string(),
            })
        })
        .collect()
}

/// A function's parity tier: the worst of its counted cells (decision 0083's order of parity),
/// then the share of them within 15 significant digits. Best first.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// Every cell bit-exact.
    Exact,
    /// Every cell within 4 ulp.
    Ulp4,
    /// Every cell within 15 significant digits.
    Digits15,
    /// At least 99.9% of the cells within 15 digits: [`RATE_NUM`] over
    /// [`RATE_DEN`](crate::milestone_exit::RATE_DEN).
    Rate999,
    /// At least 99% of the cells within 15 digits: [`RATE99_NUM`] over
    /// [`RATE_DEN`](crate::milestone_exit::RATE_DEN).
    Rate99,
    /// Fewer than 99% of the cells within 15 digits.
    Below99,
    /// No scored cell to place the function by.
    #[default]
    Unscored,
}

impl Tier {
    /// Every tier, best first.
    pub const ALL: [Tier; 7] = [
        Tier::Exact,
        Tier::Ulp4,
        Tier::Digits15,
        Tier::Rate999,
        Tier::Rate99,
        Tier::Below99,
        Tier::Unscored,
    ];

    /// The tier of the cells counted in `cells`.
    #[must_use]
    pub fn of(cells: &Closeness) -> Tier {
        let scored = cells.scored();
        let visible = cells.visible();
        let share_at_least = |num: usize| meets_rate(scored - visible, scored, num);
        if scored == 0 {
            Tier::Unscored
        } else if cells.pass == scored {
            Tier::Exact
        } else if visible == 0 && cells.digits15 == 0 {
            Tier::Ulp4
        } else if visible == 0 {
            Tier::Digits15
        } else if share_at_least(RATE_NUM) {
            Tier::Rate999
        } else if share_at_least(RATE99_NUM) {
            Tier::Rate99
        } else {
            Tier::Below99
        }
    }

    /// The name report.json uses, with what it means, for report.md.
    fn label(self) -> &'static str {
        match self {
            Tier::Exact => "`exact`: every cell bit-exact",
            Tier::Ulp4 => "`ulp4`: every cell within 4 ulp",
            Tier::Digits15 => "`digits15`: every cell within 15 digits",
            Tier::Rate999 => "`rate999`: ≥ 99.9% within 15 digits",
            Tier::Rate99 => "`rate99`: ≥ 99% within 15 digits",
            Tier::Below99 => "`below99`: < 99% within 15 digits",
            Tier::Unscored => "`unscored`: no scored Excel cell",
        }
    }
}

/// Which cells placed a function's [`Tier`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attribution {
    /// Its attributed cells ([`FunctionStat::attributed`]).
    Sole,
    /// Every Excel cell that calls it ([`FunctionStat::excel`]): the fallback below
    /// [`EVIDENCE_FLOOR`] attributed cells (decision 0118 item 2).
    #[default]
    All,
}

impl FunctionStat {
    /// Counts a non-deviation cell that calls this function toward [`FunctionStat::excel`] when
    /// its expectation comes from Excel, and toward [`FunctionStat::attributed`] too when `sole`
    /// (this function is the cell's [`crate::closeness::sole_call`]). Other cells count in neither.
    pub(crate) fn add_attributed(&mut self, cell: &CellResult, sole: bool) {
        if !cell.from_excel {
            return;
        }
        self.excel.add(cell);
        if sole {
            self.attributed.add(cell);
        }
    }

    /// The cells decision 0118 measures this function by: its attributed cells, or every Excel
    /// cell that calls it below [`EVIDENCE_FLOOR`] attributed cells. Its [`Tier`] and its Phase 1
    /// rate (`v15_checkpoint`) both read them.
    #[must_use]
    pub fn counted(&self) -> (Attribution, &Closeness) {
        if self.attributed.scored() < EVIDENCE_FLOOR {
            (Attribution::All, &self.excel)
        } else {
            (Attribution::Sole, &self.attributed)
        }
    }

    /// Places [`FunctionStat::tier`] by [`FunctionStat::counted`].
    pub(crate) fn place_tier(&mut self) {
        let (attribution, cells) = self.counted();
        (self.attribution, self.tier) = (attribution, Tier::of(cells));
    }
}

/// How many in-scope functions one [`Tier`] holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierCount {
    /// The tier.
    pub tier: Tier,
    /// In-scope functions placed in it.
    pub functions: usize,
    /// Of those, the ones placed by every Excel cell that calls them ([`Attribution::All`]).
    pub fallback: usize,
}

/// The in-scope catalog functions per tier, best first ([`Tier::ALL`]). They are the functions of
/// the fifteen-digit checkpoint: names the catalog does not know (a user-defined LAMBDA name) stay
/// out, and a catalog function no cell calls is `unscored` and counts as a fallback. With an
/// empty catalog (a run without one) every function of `functions` counts.
#[must_use]
pub fn tier_counts(catalog: &Catalog, functions: &[FunctionStat]) -> Vec<TierCount> {
    let placed: Vec<(Tier, Attribution)> = if catalog.functions().next().is_none() {
        functions.iter().map(|f| (f.tier, f.attribution)).collect()
    } else {
        catalog
            .in_scope_functions()
            .map(|c| {
                functions
                    .iter()
                    .find(|f| f.name == c.name)
                    .map_or((Tier::Unscored, Attribution::All), |f| {
                        (f.tier, f.attribution)
                    })
            })
            .collect()
    };
    Tier::ALL
        .iter()
        .map(|&tier| {
            let of_tier = placed.iter().filter(|(t, _)| *t == tier);
            TierCount {
                tier,
                functions: of_tier.clone().count(),
                fallback: of_tier.filter(|(_, a)| *a == Attribution::All).count(),
            }
        })
        .collect()
}

/// A suite's workbooks (cases) by how close their scored cells come to Excel. A case counts once
/// it has a scored cell; its cells are the ones the suite's `closeness` counts (in scope,
/// deviations passing), so `any_visible` is 0 exactly when the suite has no visibly-off cell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workbooks {
    /// Cases with at least one scored cell.
    pub scored: usize,
    /// Of those, cases whose every cell is bit-exact.
    pub all_exact: usize,
    /// Cases whose every cell is within 15 significant digits (the bit-exact ones included).
    pub all_within15: usize,
    /// Cases with a visibly-off cell.
    pub any_visible: usize,
}

impl Workbooks {
    /// Counts one case by its cells' closeness; a case with no scored cell counts nowhere.
    pub fn add(&mut self, case: &Closeness) {
        let scored = case.scored();
        if scored == 0 {
            return;
        }
        self.scored += 1;
        self.all_exact += usize::from(case.pass == scored);
        self.all_within15 += usize::from(case.visible() == 0);
        self.any_visible += usize::from(case.visible() > 0);
    }
}

/// The report section: [`RunReport::tiers`], then their total.
pub fn write_markdown(report: &RunReport, md: &mut String) {
    let _ = writeln!(md, "\n## Parity tiers (glue-aware)\n");
    let _ = writeln!(
        md,
        "Each in-scope catalog function is placed by the worst of its attributed Excel cells: those \
         whose formula calls it as the only function outside the glue list (decision 0118). Below \
         {EVIDENCE_FLOOR} attributed cells it falls back to every Excel cell that calls it. \
         Information only: nothing gates on it.\n"
    );
    let _ = writeln!(
        md,
        "| tier | functions | of which fallback |\n|---|---|---|"
    );
    for t in &report.tiers {
        let _ = writeln!(
            md,
            "| {} | {} | {} |",
            t.tier.label(),
            t.functions,
            t.fallback
        );
    }
    let (n, fb) = report
        .tiers
        .iter()
        .fold((0, 0), |(n, fb), t| (n + t.functions, fb + t.fallback));
    let _ = writeln!(md, "| total | {n} | {fb} |");
    let _ = writeln!(
        md,
        "\nVisibly-off Excel cells attributed to no function (decision 0118 item 6): {}; \
         `make unowned-off` lists those with no owning task.",
        report.unattributed_off.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogFunction, Scope};
    use crate::report::tests::{case, cell};
    use crate::value::{Value, Verdict};

    fn buckets(pass: usize, digits15: usize, visible: usize) -> Closeness {
        Closeness {
            pass,
            digits15,
            visible_numeric: visible,
            ..Closeness::default()
        }
    }

    #[test]
    fn the_worst_cell_and_the_share_within_15_digits_place_the_tier() {
        assert_eq!(Tier::of(&Closeness::default()), Tier::Unscored);
        assert_eq!(Tier::of(&buckets(5, 0, 0)), Tier::Exact);
        let ulp = Closeness {
            ulp4: 1,
            ..buckets(5, 0, 0)
        };
        assert_eq!(Tier::of(&ulp), Tier::Ulp4);
        // One cell more than 4 ulp off but within 15 digits is the worst cell.
        assert_eq!(
            Tier::of(&Closeness {
                ulp1: 3,
                ..buckets(5, 1, 0)
            }),
            Tier::Digits15
        );
        // Exactly 99.9% within 15 digits meets the rate; one cell more does not.
        assert_eq!(Tier::of(&buckets(999, 0, 1)), Tier::Rate999);
        assert_eq!(Tier::of(&buckets(998, 0, 2)), Tier::Rate99);
        // Exactly 99% is still `rate99`.
        assert_eq!(Tier::of(&buckets(990, 0, 10)), Tier::Rate99);
        assert_eq!(Tier::of(&buckets(989, 0, 11)), Tier::Below99);
        let other = Closeness {
            visible_other: 1,
            ..buckets(1, 0, 0)
        };
        assert_eq!(Tier::of(&other), Tier::Below99);
    }

    #[test]
    fn fewer_than_the_floor_of_attributed_cells_falls_back_to_every_excel_cell() {
        let mut f = FunctionStat {
            attributed: buckets(19, 0, 0),
            excel: buckets(19, 0, 1),
            ..FunctionStat::default()
        };
        f.place_tier();
        assert_eq!((f.attribution, f.tier), (Attribution::All, Tier::Below99));
        f.attributed = buckets(20, 0, 0);
        f.place_tier();
        assert_eq!((f.attribution, f.tier), (Attribution::Sole, Tier::Exact));
    }

    fn excel(formula: &str, verdict: Verdict) -> CellResult {
        CellResult {
            from_excel: true,
            ..cell(formula, verdict)
        }
    }

    fn off(formula: &str) -> CellResult {
        CellResult {
            actual: Value::Number(2.0),
            ..excel(formula, Verdict::NumericMismatch)
        }
    }

    #[test]
    fn excel_cells_count_toward_their_functions_and_their_sole_call() {
        let mut r = case("probes/a", 1);
        r.cells = vec![
            off("IF(A1,GAMMA(1))"),
            excel("GAMMA(1)+GAMMA(2)", Verdict::Pass),
            off("GAMMA(BETA.DIST(0.5,2,3,TRUE))"),
            off("IF(AND(A1,B1),1,2)"),
            excel("IF(A1,1,2)", Verdict::Pass),
            // A by-design value and a non-Excel (corpus-bench style) cell count in neither.
            excel("GAMMA(3)", Verdict::Deviation),
            cell("GAMMA(4)", Verdict::NumericMismatch),
        ];
        let report = RunReport::build("e", &[r.clone()], &[], &Catalog::default());
        let stat = |name: &str| {
            let f = report.functions.iter().find(|f| f.name == name);
            f.map(|f| {
                (
                    f.excel.scored(),
                    f.excel.visible(),
                    f.attributed.scored(),
                    f.attributed.visible(),
                )
            })
        };
        assert_eq!(stat("GAMMA"), Some((3, 2, 2, 1)));
        assert_eq!(stat("BETA.DIST"), Some((1, 1, 0, 0)));
        assert_eq!(stat("IF"), Some((3, 2, 1, 0)));
        assert_eq!(stat("AND"), Some((1, 1, 0, 0)));
        let gamma = report.functions.iter().find(|f| f.name == "GAMMA");
        // The existing count still takes every cell that calls GAMMA.
        assert_eq!(gamma.map(|f| f.closeness.scored()), Some(4));
        assert_eq!(
            gamma.map(|f| (f.attribution, f.tier)),
            Some((Attribution::All, Tier::Below99))
        );
        let md = report.to_markdown(1);
        assert!(md.contains("## Parity tiers (glue-aware)"), "{md}");
        assert!(
            md.contains("| `below99`: < 99% within 15 digits | 4 | 4 |"),
            "{md}"
        );
        assert!(md.contains("| total | 4 | 4 |"), "{md}");
        // With a catalog, only its in-scope functions count, and an uncalled one is unscored.
        let entry = |name: &str, scope: Scope| CatalogFunction {
            name: name.into(),
            category: "math".into(),
            milestone: "M5".into(),
            scope,
            reason: None,
            oracle: None,
            v15: None,
            v15_reason: None,
            params: Vec::new(),
        };
        let catalog = Catalog::from_functions([
            entry("GAMMA", Scope::In),
            entry("COS", Scope::In),
            entry("IF", Scope::Out),
        ]);
        let report = RunReport::build("e", &[r], &[], &catalog);
        let md = report.to_markdown(1);
        assert!(
            md.contains("| `below99`: < 99% within 15 digits | 1 | 1 |"),
            "{md}"
        );
        assert!(
            md.contains("| `unscored`: no scored Excel cell | 1 | 1 |"),
            "{md}"
        );
        assert!(md.contains("| total | 2 | 2 |"), "{md}");
    }

    #[test]
    fn only_visibly_off_excel_cells_with_no_sole_call_are_unattributed() {
        let mut r = case("probes/a", 1);
        r.cells = vec![
            // Two specialised calls, and glue only with two distinct calls: listed.
            off("GAMMA(BETA.DIST(0.5,2,3,TRUE))"),
            off("IF(AND(A1,B1),1,2)"),
            // Attributed (one specialised call, or one glue function), passing, a by-design
            // value, a non-Excel cell: never listed.
            off("IF(A1,GAMMA(1))"),
            off("IF(A1,1,2)"),
            excel("GAMMA(1)+BETA.DIST(0.5,2,3,TRUE)", Verdict::Pass),
            CellResult {
                actual: Value::Number(2.0),
                ..excel("GAMMA(2)+BETA.DIST(0.5,2,3,TRUE)", Verdict::Deviation)
            },
            CellResult {
                actual: Value::Number(2.0),
                ..cell("GAMMA(3)+BETA.DIST(0.5,2,3,TRUE)", Verdict::NumericMismatch)
            },
        ];
        let report = RunReport::build("e", &[r], &[], &Catalog::default());
        let listed: Vec<(&str, &str)> = report
            .unattributed_off
            .iter()
            .map(|u| (u.case.as_str(), u.formula.as_str()))
            .collect();
        assert_eq!(
            listed,
            [
                ("probes/a", "GAMMA(BETA.DIST(0.5,2,3,TRUE))"),
                ("probes/a", "IF(AND(A1,B1),1,2)")
            ]
        );
        let attributed: Vec<(&str, &str)> = report
            .attributed_off
            .iter()
            .map(|a| (a.function.as_str(), a.formula.as_str()))
            .collect();
        assert_eq!(
            attributed,
            [("GAMMA", "IF(A1,GAMMA(1))"), ("IF", "IF(A1,1,2)")]
        );
        let md = report.to_markdown(1);
        assert!(
            md.contains("attributed to no function (decision 0118 item 6): 2;"),
            "{md}"
        );
    }

    #[test]
    fn suites_count_workbooks_by_their_worst_in_scope_cell() {
        let ulp = || CellResult {
            actual: Value::Number(f64::from_bits(1.0_f64.to_bits() + 1)),
            ..cell("ABS(1)", Verdict::NumericUlp)
        };
        let with = |id: &str, cells: Vec<CellResult>| {
            let mut r = case(id, 1);
            r.cells = cells;
            r
        };
        let results = [
            with(
                "s/exact",
                vec![cell("ABS(1)", Verdict::Pass), cell("1", Verdict::Deviation)],
            ),
            with("s/ulp", vec![cell("ABS(1)", Verdict::Pass), ulp()]),
            with("s/off", vec![ulp(), off("ABS(2)")]),
            with("s/unscored", vec![cell("ABS(1)", Verdict::Unsupported)]),
            with("s/empty", Vec::new()),
            with("t/off", vec![off("ABS(2)")]),
        ];
        let report = RunReport::build("e", &results, &[], &Catalog::default());
        let books: Vec<(&str, Workbooks)> = report
            .suites
            .iter()
            .map(|s| (s.name.as_str(), s.workbooks))
            .collect();
        let counts = |scored, all_exact, all_within15, any_visible| Workbooks {
            scored,
            all_exact,
            all_within15,
            any_visible,
        };
        assert_eq!(
            books,
            [("s", counts(3, 1, 2, 1)), ("t", counts(1, 0, 0, 1))]
        );
    }
}
