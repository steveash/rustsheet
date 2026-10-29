use super::*;
use crate::catalog::Catalog;
use crate::catalog_coverage::MilestoneCoverage;
use crate::closeness::Closeness;
use crate::function_scope::{FunctionScope, MilestoneScope};
use crate::probe_milestones::MilestoneRow;
use crate::report::SuiteSummary;
use crate::runner::CellResult;

fn report(gen_cells: Option<(usize, usize)>) -> RunReport {
    report15(gen_cells.map(|(cells, passed)| (cells, passed, cells - passed)))
}

/// `(cells, bit-exact, visibly off)` of the generated suite.
fn report15(gen_cells: Option<(usize, usize, usize)>) -> RunReport {
    let mut r = RunReport::build("t", &[], &[], &Catalog::from_functions([]));
    r.catalog_coverage = vec![MilestoneCoverage {
        milestone: "M5".into(),
        functions: 10,
        covered: 10,
        ..MilestoneCoverage::default()
    }];
    r.suites = gen_cells
        .map(|(cells, passed, visible)| SuiteSummary {
            name: "gen-m5".into(),
            cells,
            cells_passed: passed,
            closeness: Closeness {
                visible_numeric: visible,
                ..Closeness::default()
            },
            ..SuiteSummary::default()
        })
        .into_iter()
        .collect();
    r.function_scope = Some(FunctionScope {
        rows: vec![MilestoneScope {
            milestone: "M5".into(),
            failing: vec!["a!S!A1".into()],
            excepted: vec!["a!S!A1".into()],
            visibly_off: vec!["a!S!A1".into()],
            ..MilestoneScope::default()
        }],
        ..FunctionScope::default()
    });
    r.probe_milestones = vec![MilestoneRow {
        milestone: "M5".into(),
        ..MilestoneRow::default()
    }];
    r
}

fn row(r: &RunReport, m: &str) -> ExitRow {
    summarize(r)
        .into_iter()
        .find(|row| row.milestone == m)
        .unwrap_or_default()
}

fn m5(r: &RunReport) -> ExitRow {
    row(r, "M5")
}

#[test]
fn the_rate_is_a_share_of_at_least_num_over_den_and_never_of_no_cells() {
    assert!(meets_rate(999, 1000, RATE_NUM) && !meets_rate(998, 1000, RATE_NUM));
    assert!(meets_rate(990, 1000, RATE99_NUM) && !meets_rate(989, 1000, RATE99_NUM));
    assert!(!meets_rate(0, 0, RATE_NUM));
}

#[test]
fn all_met() {
    let r = report(Some((22_400, 22_380)));
    let rows = summarize(&r);
    let row = rows.iter().find(|row| row.milestone == "M5").unwrap();
    assert_eq!(row.gen_allowance, Some(22));
    assert_eq!(row.gen_known_failures, Some(20));
    assert_eq!(row.calc_failing, Some(0));
    assert!(row.exit, "{row:?}");
    let mut md = String::new();
    write_markdown(&rows, &mut md);
    assert!(
        md.contains(
            "| M5 | 10/10 met | 22400, 0.99911, 20/22 [20] met | 0 [0] met | 0 [0] met | n/a n/a | n/a | met |"
        ),
        "{md}"
    );
    assert!(
        md.contains(
            "| M6 | n/a/n/a n/a | n/a | n/a [n/a] n/a | n/a [n/a] n/a | n/a n/a | n/a | not met |"
        ),
        "{md}"
    );
}

#[test]
fn coverage_short_fails_alone() {
    let mut r = report(Some((22_400, 22_400)));
    r.catalog_coverage[0].covered = 9;
    let row = m5(&r);
    assert_eq!(row.coverage_met, Some(false));
    assert!(!row.exit);
}

#[test]
fn too_few_cells_fails_alone() {
    let row = m5(&report(Some((19_999, 19_999))));
    assert_eq!(row.gen_met, Some(false));
    assert_eq!(row.coverage_met, Some(true));
    assert!(!row.exit);
}

#[test]
fn low_rate_fails_alone() {
    // 0.998 of 22,400 scored cells: 45 failures against an allowance of 22.
    let row = m5(&report(Some((22_400, 22_355))));
    assert_eq!(row.gen_met, Some(false));
    assert!(!row.exit);
}

#[test]
fn calc_failure_fails_alone() {
    let mut r = report(Some((22_400, 22_400)));
    r.function_scope.as_mut().unwrap().rows[0].excepted.clear();
    let row = m5(&r);
    assert_eq!(row.calc_met, Some(false));
    assert!(!row.exit);
}

#[test]
fn probe_failure_fails_alone() {
    let mut r = report(Some((22_400, 22_400)));
    r.probe_milestones[0].known_failures = 1;
    assert!(
        m5(&r).exit,
        "a bit-level probe failure is Phase 1b's (decision 0114)"
    );
    r.probe_milestones[0].visibly_off = 1;
    let row = m5(&r);
    assert_eq!(row.probes_met, Some(false));
    assert!(!row.exit);
}

#[test]
fn bit_failures_within_fifteen_digits_do_not_block_the_exit() {
    // Decision 0114: 400 bit failures, 20 of them visibly off, is met; 23 visibly off is not.
    let row = m5(&report15(Some((22_400, 22_000, 20))));
    assert_eq!(row.gen_known_failures, Some(400));
    assert_eq!(row.gen_visibly_off, Some(20));
    assert!(row.exit, "{row:?}");
    assert!(!m5(&report15(Some((22_400, 22_000, 23)))).exit);
    let mut r = report(Some((22_400, 22_400)));
    let scope = r.function_scope.as_mut().unwrap();
    scope.rows[0].failing.push("b!S!A1".into());
    assert!(
        m5(&r).exit,
        "a bit-only ironcalc-calc failure is Phase 1b's"
    );
    r.function_scope.as_mut().unwrap().rows[0]
        .visibly_off
        .push("b!S!A1".into());
    assert_eq!(m5(&r).calc_met, Some(false));
}

#[test]
fn missing_gen_suite_is_not_applicable() {
    let row = m5(&report(None));
    assert_eq!(row.gen_met, None);
    assert!(!row.exit);
}

/// One milestone's inputs added to [`report`]`(None)`: `gen` (cells, visibly off) of its
/// generated suite, its catalog coverage (covered, functions), its function-scope row
/// (`failing`, of which `excepted`) and its probe row with `probe_failures` bit-level failures.
struct Inputs<'a> {
    milestone: &'a str,
    probe_row: &'a str,
    gen: (usize, usize),
    coverage: (usize, usize),
    failing: &'a str,
    excepted: bool,
    probe_failures: usize,
}

fn milestone_report(i: &Inputs<'_>) -> RunReport {
    let mut r = report(None);
    let (cells, visible) = i.gen;
    r.catalog_coverage.push(MilestoneCoverage {
        milestone: i.milestone.into(),
        covered: i.coverage.0,
        functions: i.coverage.1,
        ..MilestoneCoverage::default()
    });
    r.suites.push(SuiteSummary {
        name: format!("gen-{}", i.milestone.to_ascii_lowercase()),
        cells,
        cells_passed: cells - visible,
        closeness: Closeness {
            visible_other: visible,
            ..Closeness::default()
        },
        ..SuiteSummary::default()
    });
    let failing = vec![i.failing.to_string()];
    r.function_scope
        .as_mut()
        .unwrap()
        .rows
        .push(MilestoneScope {
            milestone: i.milestone.into(),
            excepted: if i.excepted {
                failing.clone()
            } else {
                Vec::new()
            },
            visibly_off: failing.clone(),
            failing,
            ..MilestoneScope::default()
        });
    r.probe_milestones.push(MilestoneRow {
        milestone: i.probe_row.into(),
        known_failures: i.probe_failures,
        ..MilestoneRow::default()
    });
    r
}

/// A run whose M2 inputs meet M2's exit: `gen_cells` (cells, visibly off) of gen-m2.
fn m2_report(gen_cells: (usize, usize)) -> RunReport {
    milestone_report(&Inputs {
        milestone: "M2",
        probe_row: "M0-M2",
        gen: gen_cells,
        coverage: (30, 37),
        failing: "ironcalc-calc/IF_ARRAY.xlsx!Sheet1!C11",
        excepted: true,
        probe_failures: 3,
    })
}

#[test]
fn m2_needs_ten_thousand_gen_cells_none_visibly_off_and_no_coverage() {
    let r = m2_report((M2_MIN_SCORED, 0));
    let m2 = row(&r, "M2");
    assert_eq!(
        (m2.covered, m2.functions, m2.coverage_met),
        (Some(30), Some(37), None),
        "coverage is shown, not required"
    );
    assert_eq!((m2.gen_allowance, m2.gen_met), (Some(0), Some(true)));
    assert_eq!((m2.calc_visibly_off, m2.calc_met), (Some(0), Some(true)));
    assert_eq!(m2.probe_row, "M0-M2");
    assert_eq!(
        (m2.probe_known_failures, m2.probes_met),
        (Some(3), Some(true))
    );
    assert!(m2.exit, "{m2:?}");
    // The M5 row keeps its own thresholds and reads its own probe row.
    assert_eq!(m5(&r).probe_row, "M5");
    assert_eq!(m5(&r).coverage_met, Some(true));
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains(
            "| M2 | 30/37 n/a | 10000, 1.00000, 0/0 [0] met | 0 [0] met | 0 [3] met | n/a n/a | n/a | met |"
        ),
        "{md}"
    );
    // 9,999 scored cells, or one visibly-off cell, is not met.
    let m2 = row(&m2_report((M2_MIN_SCORED - 1, 0)), "M2");
    assert_eq!(m2.gen_met, Some(false));
    assert!(!m2.exit);
    let m2 = row(&m2_report((50_000, 1)), "M2");
    assert_eq!((m2.gen_allowance, m2.gen_met), (Some(0), Some(false)));
    assert!(!m2.exit);
    // A visibly-off probe of the M0-M2 row is not met.
    let mut r = m2_report((M2_MIN_SCORED, 0));
    r.probe_milestones[1].visibly_off = 1;
    assert_eq!(row(&r, "M2").probes_met, Some(false));
}

#[test]
fn gen_visibly_off_cells_are_listed_for_exit_owners() {
    use crate::report::tests::{case, cell};
    use crate::value::{Value, Verdict};
    let mut r = m2_report((M2_MIN_SCORED, 1));
    let mut g = case("gen-m2/abc.case.json", 1);
    let close = CellResult {
        cell: "A2".into(),
        expected: Value::Number(1.0),
        actual: Value::Number(1.0 + 1e-17),
        ..cell("ABS(1)", Verdict::NumericClose)
    };
    g.cells = vec![cell("ABS(1)", Verdict::ErrorMismatch), close];
    r.cases = vec![g];
    let m2 = row(&r, "M2");
    assert_eq!(m2.gen_visibly_off_cells, ["gen-m2/abc.case.json!S!A1"]);
}

/// A run whose M7 inputs meet M7's exit: `gen_cells` (cells, visibly off) of gen-m7, an
/// ironcalc-calc M7 row with a visibly-off cell and coverage short (neither required).
fn m7_report(gen_cells: (usize, usize)) -> RunReport {
    milestone_report(&Inputs {
        milestone: "M7",
        probe_row: "M7",
        gen: gen_cells,
        coverage: (39, 40),
        failing: "c!S!A1",
        excepted: false,
        probe_failures: 2,
    })
}

/// A spill-extent cell of gen-m7, as `spill_extent.rs` adds it.
fn extent_case() -> crate::runner::CaseResult {
    use crate::report::tests::{case, cell};
    use crate::value::Verdict;
    let mut g = case("gen-m7/abc.case.json", 1);
    g.cells = vec![CellResult {
        kind: CellKind::SpillExtent,
        ..cell("SEQUENCE(2)", Verdict::TypeMismatch)
    }];
    g
}

#[test]
fn m7_needs_ten_thousand_gen_cells_at_the_rate_and_no_extent_failure() {
    let r = m7_report((M7_MIN_SCORED, 10));
    let m7 = row(&r, "M7");
    assert_eq!((m7.gen_allowance, m7.gen_met), (Some(10), Some(true)));
    assert_eq!(
        (m7.covered, m7.functions, m7.coverage_met),
        (Some(39), Some(40), None),
        "coverage is shown, not required"
    );
    assert_eq!(
        (m7.calc_failing, m7.calc_visibly_off, m7.calc_met),
        (Some(1), Some(1), None),
        "ironcalc-calc is shown, not required"
    );
    assert_eq!(m7.probe_row, "M7");
    assert_eq!(
        (m7.probe_known_failures, m7.probes_met),
        (Some(2), Some(true))
    );
    assert_eq!((m7.extent_failures, m7.extent_met), (Some(0), Some(true)));
    assert!(m7.exit, "{m7:?}");
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains(
            "| M7 | 39/40 n/a | 10000, 0.99900, 10/10 [10] met | 1 [1] n/a | 0 [2] met | 0 met | n/a | met |"
        ),
        "{md}"
    );
    // Other rows have no extent criterion.
    assert_eq!(m5(&r).extent_failures, None);
    // 9,999 scored cells is not met.
    let m7 = row(&m7_report((M7_MIN_SCORED - 1, 0)), "M7");
    assert_eq!(m7.gen_met, Some(false));
    assert!(!m7.exit);
    // The allowance plus one visibly-off cell is not met.
    let m7 = row(&m7_report((M7_MIN_SCORED, 11)), "M7");
    assert_eq!((m7.gen_allowance, m7.gen_met), (Some(10), Some(false)));
    assert!(!m7.exit);
    // A visibly-off M7 probe cell is not met.
    let mut r = m7_report((M7_MIN_SCORED, 0));
    r.probe_milestones[1].visibly_off = 1;
    assert!(!row(&r, "M7").exit);
}

#[test]
fn one_m7_extent_failure_is_not_met() {
    let mut r = m7_report((M7_MIN_SCORED, 0));
    r.cases = vec![extent_case()];
    let m7 = row(&r, "M7");
    assert_eq!((m7.extent_failures, m7.extent_met), (Some(1), Some(false)));
    assert!(!m7.exit);
    let mut r = m7_report((M7_MIN_SCORED, 0));
    r.probe_milestones[1].spill_extent_failures = 1;
    let m7 = row(&r, "M7");
    assert_eq!((m7.extent_failures, m7.extent_met), (Some(1), Some(false)));
    assert!(!m7.exit);
    // Without the gen suite or the probe row the measure is absent, never met.
    let mut r = m7_report((M7_MIN_SCORED, 0));
    r.probe_milestones.pop();
    assert_eq!(row(&r, "M7").extent_met, None);
}

/// A suite row of `cells` scored cells, `visible` of them visibly off.
fn suite(name: &str, cells: usize, visible: usize) -> SuiteSummary {
    SuiteSummary {
        name: name.into(),
        cells,
        cells_passed: cells - visible,
        closeness: Closeness {
            visible_numeric: visible,
            ..Closeness::default()
        },
        ..SuiteSummary::default()
    }
}

/// A run whose M1 inputs meet M1's exit: gen-m1 and smoke cells and the `m1-probes` list's
/// visibly-off cells (with one bit-level failure either way).
fn m1_report(gen: usize, smoke_off: usize, list_off: usize) -> RunReport {
    let mut r = report(None);
    r.suites.push(suite("gen-m1", gen, 0));
    r.suites.push(suite("smoke", 46, smoke_off));
    r.milestone_lists.push(crate::milestone_lists::ListSummary {
        name: M1_PROBE_LIST.into(),
        cases: 89,
        cases_run: 89,
        cells: 1858,
        known_failures: 1,
        visibly_off: list_off,
        ..crate::milestone_lists::ListSummary::default()
    });
    r
}

#[test]
fn m1_needs_gen_m1_and_smoke_none_visibly_off_and_the_m1_probes_list() {
    let r = m1_report(3_750, 0, 0);
    let m1 = row(&r, "M1");
    assert_eq!(m1.gen_suite, "gen-m1");
    assert_eq!((m1.gen_allowance, m1.gen_met), (Some(0), Some(true)));
    assert_eq!(m1.more_suites.len(), 1);
    assert_eq!(
        (m1.more_suites[0].name.as_str(), m1.more_suites[0].met),
        ("smoke", Some(true))
    );
    assert_eq!(m1.probe_row, M1_PROBE_LIST);
    assert_eq!(
        (m1.probe_known_failures, m1.probe_visibly_off, m1.probes_met),
        (Some(1), Some(0), Some(true))
    );
    assert_eq!((m1.coverage_met, m1.calc_met), (None, None));
    assert!(m1.exit, "{m1:?}");
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains(
            "| M1 | n/a/n/a n/a | 3750, 1.00000, 0/0 [0] met; smoke: 46, 1.00000, 0/0 [0] met | n/a [n/a] n/a | 0 [1] met | n/a n/a | n/a | met |"
        ),
        "{md}"
    );
    // gen-m1 short of 3,000 cells, a visibly-off smoke cell or a visibly-off listed cell
    // is not met.
    let m1 = row(&m1_report(M1_MIN_SCORED - 1, 0, 0), "M1");
    assert_eq!(m1.gen_met, Some(false));
    assert!(!m1.exit);
    let m1 = row(&m1_report(3_750, 1, 0), "M1");
    assert_eq!(
        (m1.gen_met, m1.more_suites[0].met),
        (Some(true), Some(false))
    );
    assert!(!m1.exit);
    let m1 = row(&m1_report(3_750, 0, 1), "M1");
    assert_eq!(m1.probes_met, Some(false));
    assert!(!m1.exit);
    // A run that did not execute the list (`--suite gen-m1`) has no probe measure.
    let mut r = m1_report(3_750, 0, 0);
    r.milestone_lists[0].cases_run = 0;
    let m1 = row(&r, "M1");
    assert_eq!(m1.probes_met, None);
    assert!(!m1.exit);
}

/// A run with `corpus` (scored cells, visibly off) of corpus-bench, when fetched, and a
/// clean M11 probe row.
fn m11_report(corpus: Option<(usize, usize)>) -> RunReport {
    let mut r = report(None);
    if let Some((cells, visible)) = corpus {
        r.suites.push(suite("corpus-bench", cells, visible));
    }
    r.probe_milestones.push(MilestoneRow {
        milestone: "M11".into(),
        ..MilestoneRow::default()
    });
    // A real report has M11 coverage and function-scope rows; M11's exit shows neither.
    r.catalog_coverage.push(MilestoneCoverage {
        milestone: "M11".into(),
        ..MilestoneCoverage::default()
    });
    r.function_scope
        .as_mut()
        .unwrap()
        .rows
        .push(MilestoneScope {
            milestone: "M11".into(),
            failing: vec!["c!S!A1".into()],
            visibly_off: vec!["c!S!A1".into()],
            ..MilestoneScope::default()
        });
    r
}

#[test]
fn m11_needs_corpus_bench_at_the_leaderboard_rate_and_is_na_without_it() {
    // 99.834% of 205,366 cells (decision 0146's numbers): the allowance is 410.
    let r = m11_report(Some((205_366, 340)));
    let m11 = row(&r, "M11");
    assert_eq!(m11.gen_suite, "corpus-bench");
    assert_eq!((m11.gen_allowance, m11.gen_met), (Some(410), Some(true)));
    assert_eq!(m11.probe_row, "M11");
    assert_eq!(m11.probes_met, Some(true));
    assert_eq!((m11.coverage_met, m11.calc_met), (None, None));
    assert!(m11.exit, "{m11:?}");
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains(
            "| M11 | n/a/n/a n/a | corpus-bench: 205366, 0.99834, 340/410 [340] met | n/a [n/a] n/a | 0 [0] met | n/a n/a | n/a | met |"
        ),
        "{md}"
    );
    // Past the allowance, or short of 200,000 cells, is not met.
    let m11 = row(&m11_report(Some((205_366, 411))), "M11");
    assert_eq!(m11.gen_met, Some(false));
    assert!(!m11.exit);
    let m11 = row(&m11_report(Some((M11_MIN_SCORED - 1, 0))), "M11");
    assert_eq!(m11.gen_met, Some(false));
    assert!(!m11.exit);
    // corpus-bench not fetched: n/a, never met.
    let r = m11_report(None);
    let m11 = row(&r, "M11");
    assert_eq!((m11.gen_scored, m11.gen_met), (None, None));
    assert!(!m11.exit);
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains(
            "| M11 | n/a/n/a n/a | n/a | n/a [n/a] n/a | 0 [0] met | n/a n/a | n/a | not met |"
        ),
        "{md}"
    );
}

/// A run with an M9 function-scope row of `cells` with `off` visibly-off cells (one of them,
/// `x!S!A1`, under an exit exception when `excepted`), two M9 functions of the checkpoint at the
/// rate and one of M8 below it, and a clean M9 probe row.
fn m9_report(cells: usize, off: usize, excepted: bool) -> RunReport {
    use crate::v15_checkpoint::V15Function;
    let mut r = report(None);
    let visibly_off: Vec<String> = (0..off).map(|i| format!("x!S!A{}", i + 1)).collect();
    r.function_scope
        .as_mut()
        .unwrap()
        .rows
        .push(MilestoneScope {
            milestone: "M9".into(),
            cells,
            deviations: 5,
            failing: visibly_off
                .iter()
                .cloned()
                .chain(["x!S!B1".into()])
                .collect(),
            excepted: if excepted {
                vec!["x!S!A1".into()]
            } else {
                Vec::new()
            },
            visibly_off,
            ..MilestoneScope::default()
        });
    r.catalog_coverage.push(MilestoneCoverage {
        milestone: "M9".into(),
        functions: 2,
        covered: 1,
        ..MilestoneCoverage::default()
    });
    let function = |name: &str, milestone: &str, meets_rate: bool| V15Function {
        name: name.into(),
        milestone: milestone.into(),
        covered: true,
        meets_rate,
        ..V15Function::default()
    };
    r.v15_checkpoint.functions = vec![
        function("PMT", "M9", true),
        function("IRR", "M9", true),
        function("SUMIFS", "M8", false),
    ];
    r.probe_milestones.push(MilestoneRow {
        milestone: "M9".into(),
        ..MilestoneRow::default()
    });
    r
}

fn m9(r: &RunReport) -> ExitRow {
    row(r, "M9")
}

#[test]
fn m9_needs_ironcalc_calc_at_99_95_every_m9_function_at_the_rate_and_clean_probes() {
    // 73,160 cells less 5 deviations: 73,155 scored, the allowance is floor(0.0005 * 73,155) = 36.
    let r = m9_report(73_160, 36, false);
    let row = m9(&r);
    assert_eq!(
        (row.calc_scored, row.calc_allowance),
        (Some(73_155), Some(36))
    );
    assert_eq!(
        (row.calc_visibly_off, row.calc_failing),
        (Some(36), Some(37))
    );
    assert_eq!(row.calc_met, Some(true));
    assert_eq!((row.rate_functions, row.below_rate), (Some(2), Some(0)));
    assert_eq!(row.rate_met, Some(true));
    assert_eq!(row.probes_met, Some(true));
    // Coverage is shown, not required, and no suite is required.
    assert_eq!((row.covered, row.coverage_met), (Some(1), None));
    assert_eq!((row.gen_suite.as_str(), row.gen_met), ("", None));
    assert!(row.exit, "{row:?}");
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains(
            "| M9 | 1/2 n/a | none required | 36/36 of 73155 [37] met | 0 [0] met | n/a n/a | 0/2 met | met |"
        ),
        "{md}"
    );
}

#[test]
fn m9_calc_past_the_allowance_fails_alone() {
    let row = m9(&m9_report(73_160, 37, false));
    assert_eq!(row.calc_met, Some(false));
    assert_eq!((row.rate_met, row.probes_met), (Some(true), Some(true)));
    assert!(!row.exit);
}

#[test]
fn m9_an_exit_exception_keeps_ironcalc_calc_met() {
    // 37 visibly off, one excepted: 36 left over 73,154 scored cells, still within 36.
    let row = m9(&m9_report(73_160, 37, true));
    assert_eq!(
        (row.calc_scored, row.calc_visibly_off),
        (Some(73_154), Some(36))
    );
    assert_eq!((row.calc_allowance, row.calc_met), (Some(36), Some(true)));
    assert!(row.exit, "{row:?}");
}

#[test]
fn m9_a_function_below_the_rate_fails_alone() {
    let mut r = m9_report(73_160, 0, false);
    r.v15_checkpoint.functions[1].meets_rate = false;
    let row = m9(&r);
    assert_eq!((row.below_rate, row.rate_met), (Some(1), Some(false)));
    assert_eq!(row.below_rate_functions, ["IRR"]);
    assert_eq!(row.calc_met, Some(true));
    assert!(!row.exit);
}

#[test]
fn m9_a_visibly_off_probe_fails_alone() {
    let mut r = m9_report(73_160, 0, false);
    r.probe_milestones.last_mut().unwrap().visibly_off = 1;
    let row = m9(&r);
    assert_eq!(row.probes_met, Some(false));
    assert_eq!((row.calc_met, row.rate_met), (Some(true), Some(true)));
    assert!(!row.exit);
}

#[test]
fn m9_without_checkpoint_functions_is_not_applicable() {
    let mut r = m9_report(73_160, 0, false);
    r.v15_checkpoint.functions.clear();
    let row = m9(&r);
    assert_eq!((row.rate_functions, row.rate_met), (None, None));
    assert!(!row.exit);
}

/// A run with gen-m8a and gen-m8b of 3,577 and 3,368 cells (`m8b_off` visibly off), a poi
/// subset of `subset` (scored, visibly off) when poi ran, and a clean M8 probe row.
fn m8_report(m8b_off: usize, subset: Option<(usize, usize)>) -> RunReport {
    use crate::poi_m8_subset::PoiM8Subset;
    let mut r = report(None);
    r.suites.push(suite("gen-m8a", 3_577, 0));
    r.suites.push(suite("gen-m8b", 3_368, m8b_off));
    r.poi_m8_subset = subset.map(|(scored, off)| PoiM8Subset {
        scored,
        visibly_off: off,
        visibly_off_cells: (0..off)
            .map(|i| format!("poi/a.xls!S!A{}", i + 1))
            .collect(),
        ..PoiM8Subset::default()
    });
    r.probe_milestones.push(MilestoneRow {
        milestone: "M8".into(),
        ..MilestoneRow::default()
    });
    r
}

fn m8(r: &RunReport) -> ExitRow {
    row(r, "M8")
}

#[test]
fn m8_needs_both_gen_suites_the_poi_subset_at_99_5_and_clean_probes() {
    // 2,000 scored subset cells: the allowance is floor(0.005 * 2,000) = 10.
    let r = m8_report(0, Some((2_000, 10)));
    let row = m8(&r);
    assert_eq!(row.gen_suite, "gen-m8a");
    assert_eq!(row.gen_met, Some(true));
    let names: Vec<&str> = row.more_suites.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["gen-m8b", M8_SUBSET]);
    let subset = &row.more_suites[1];
    assert_eq!((subset.allowance, subset.met), (Some(10), Some(true)));
    assert_eq!(subset.visibly_off_cells.len(), 10);
    assert_eq!((row.coverage_met, row.calc_met), (None, None));
    assert!(row.exit, "{row:?}");
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains(
            "| M8 | n/a/n/a n/a | gen-m8a: 3577, 1.00000, 0/0 [0] met; gen-m8b: 3368, 1.00000, 0/0 [0] met; poi M8 subset: 2000, 0.99500, 10/10 [n/a] met | n/a [n/a] n/a | 0 [0] met | n/a n/a | n/a | met |"
        ),
        "{md}"
    );
}

#[test]
fn m8_a_visibly_off_gen_m8b_cell_fails_alone() {
    let row = m8(&m8_report(1, Some((2_000, 0))));
    assert_eq!(row.more_suites[0].met, Some(false));
    assert_eq!(
        (row.gen_met, row.more_suites[1].met),
        (Some(true), Some(true))
    );
    assert!(!row.exit);
}

#[test]
fn m8_a_subset_below_99_5_fails_alone() {
    let row = m8(&m8_report(0, Some((2_000, 11))));
    assert_eq!(row.more_suites[1].met, Some(false));
    assert_eq!(row.more_suites[0].met, Some(true));
    assert!(!row.exit);
}

#[test]
fn m8_a_blocked_subset_case_fails_alone() {
    let mut r = m8_report(0, Some((2_000, 0)));
    r.poi_m8_subset.as_mut().unwrap().blocked_cases = vec!["poi/b.xlsx!*".into()];
    let row = m8(&r);
    let subset = &row.more_suites[1];
    assert_eq!(subset.met, Some(false));
    assert_eq!(subset.blocked_cases, ["poi/b.xlsx!*"]);
    assert!(!row.exit);
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains("poi M8 subset: 2000, 1.00000, 0/10 [n/a], 1 cases blocked not met"),
        "{md}"
    );
}

#[test]
fn m8_a_visibly_off_probe_fails_alone() {
    let mut r = m8_report(0, Some((2_000, 0)));
    r.probe_milestones.last_mut().unwrap().visibly_off = 1;
    let row = m8(&r);
    assert_eq!(row.probes_met, Some(false));
    assert!(row.more_suites.iter().all(|s| s.met == Some(true)));
    assert!(!row.exit);
}

#[test]
fn m8_without_poi_is_not_applicable() {
    let row = m8(&m8_report(0, None));
    let subset = &row.more_suites[1];
    assert_eq!((subset.scored, subset.met), (None, None));
    assert!(!row.exit);
}
