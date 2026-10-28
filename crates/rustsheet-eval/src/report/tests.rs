use super::*;

/// A passing one-cell case `id` in the suite its id names (shared with sibling tests).
pub(crate) fn case(id: &str, millis: u64) -> CaseResult {
    CaseResult {
        id: id.to_string(),
        suite: id.split('/').next().unwrap_or(id).to_string(),
        status: CaseStatus::Pass,
        message: None,
        expected_source: String::new(),
        checked: 1,
        passed: 1,
        deviations: 0,
        skipped_volatile: 0,
        skipped_add_in: 0,
        millis,
        tags: Vec::new(),
        cells: Vec::new(),
        text: Vec::new(),
        defined_names: BTreeMap::new(),
        arguments: Vec::new(),
    }
}

#[test]
fn slowest_cases_are_ordered_by_time_then_id() {
    let results = [
        case("s/b", 5),
        case("s/a", 5),
        case("s/c", 9),
        case("s/d", 1),
    ];
    let ids: Vec<String> = slowest(&results, 3).into_iter().map(|c| c.id).collect();
    assert_eq!(ids, ["s/c", "s/a", "s/b"]);
}

#[test]
fn real_world_calls_count_formula_cells_of_the_corpora_only() {
    let mut calipers = case("calipers/a", 1);
    calipers.cells = vec![
        cell("SUM(A2:A3)+ABS(1)", Verdict::Pass),
        cell("SUM(1)", Verdict::NumericMismatch),
        CellResult {
            formula: None,
            anchor_formula: Some("SUM(1)".into()),
            ..cell("", Verdict::Pass)
        },
    ];
    let mut poi = case("poi/b", 1);
    poi.cells = vec![cell("SUM(2)", Verdict::Deviation)];
    let mut probes = case("probes/c", 1);
    probes.cells = vec![cell("SUM(3)", Verdict::Pass)];
    let report = RunReport::build("e", &[calipers, poi, probes], &[], &Catalog::default());
    let calls = |name: &str| {
        report
            .functions
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.real_world_calls)
    };
    // An array member without a formula of its own is not a call; a deviation still is.
    assert_eq!(calls("SUM"), Some(3));
    assert_eq!(calls("ABS"), Some(1));
}

/// A cell with `formula` and `verdict`, expecting and giving 1 (shared with sibling tests).
pub(crate) fn cell(formula: &str, verdict: Verdict) -> CellResult {
    CellResult {
        sheet: "S".into(),
        cell: "A1".into(),
        formula: Some(formula.into()),
        anchor_formula: None,
        expected: crate::value::Value::Number(1.0),
        actual: crate::value::Value::Number(1.0),
        verdict,
        from_excel: false,
        listed_deviation: false,
        graph_subnormal: false,
        kind: crate::runner::CellKind::Scored,
    }
}

#[test]
fn near_misses_count_toward_the_close_rate_only() {
    let mut r = case("corpus-bench/a", 1);
    r.cells = vec![
        cell("ABS(1)", Verdict::NumericUlp),
        cell("ABS(2)", Verdict::NumericClose),
        cell("ABS(3)", Verdict::NumericMismatch),
    ];
    (r.checked, r.passed) = (4, 1);
    let report = RunReport::build("e", &[r], &[], &Catalog::default());
    let s = &report.suites[0];
    assert!((s.cell_pass_rate() - 0.25).abs() < 1e-12);
    assert!((s.cell_close_rate() - 0.75).abs() < 1e-12);
}

#[test]
fn deviations_leave_the_bit_exact_rate_and_stale_lines_are_listed() {
    let listed = |verdict| CellResult {
        listed_deviation: true,
        ..cell("ROWS(SEQUENCE(1048576))", verdict)
    };
    let mut r = case("probes/a", 1);
    r.cells = vec![
        cell("ABS(1)", Verdict::Pass),
        cell("ABS(2)", Verdict::NumericMismatch),
        listed(Verdict::Deviation),
        listed(Verdict::Pass),
    ];
    (r.checked, r.passed, r.deviations) = (4, 2, 1);
    let report = RunReport::build("e", &[r], &[], &Catalog::default());
    let s = &report.suites[0];
    assert_eq!((s.cells, s.deviations), (4, 1));
    assert!((s.cell_pass_rate() - 2.0 / 3.0).abs() < 1e-12);
    assert_eq!(s.stale_deviations, ["probes/a\tS!A1"]);
    // the by-design cell is neither a failing case cell nor a function statistic
    assert_eq!(report.cases[0].cells.len(), 1);
    let sequence = report.functions.iter().find(|f| f.name == "SEQUENCE");
    assert_eq!(sequence.map(|f| (f.cells, f.passed)), Some((1, 1)));
    let md = report.to_markdown(1);
    assert!(
        md.contains("| probes | 1 | 1 | 0 | 0 | 0 | 0 | 4 | 1 | 66.67 |"),
        "{md}"
    );
    assert!(md.contains("- `probes/a\tS!A1`"));
}

/// Each function counts its scored cells per closeness bucket, which sum to its cells minus
/// the unsupported ones (T-2036).
#[test]
fn function_closeness_buckets_sum_to_the_scored_cells() {
    let mut r = case("probes/a", 1);
    let ulp = |n: u64| CellResult {
        actual: crate::value::Value::Number(f64::from_bits(1.0_f64.to_bits() + n)),
        ..cell("ABS(1)+SIGN(1)", Verdict::NumericUlp)
    };
    r.cells = vec![
        cell("ABS(1)", Verdict::Pass),
        ulp(1),
        ulp(3),
        cell("ABS(2)", Verdict::NumericClose),
        CellResult {
            actual: crate::value::Value::Number(2.0),
            ..cell("ABS(3)", Verdict::NumericMismatch)
        },
        cell("ABS(4)", Verdict::TypeMismatch),
        cell("ABS(5)", Verdict::Unsupported),
    ];
    let report = RunReport::build("e", &[r], &[], &Catalog::default());
    let abs = report.functions.iter().find(|f| f.name == "ABS");
    let abs = abs.map(|f| (f.cells, f.closeness));
    let expected = Closeness {
        pass: 1,
        ulp1: 1,
        ulp4: 1,
        digits15: 1,
        visible_numeric: 1,
        visible_other: 1,
    };
    assert_eq!(abs, Some((7, expected)));
    let sign = report.functions.iter().find(|f| f.name == "SIGN");
    assert_eq!(sign.map(|f| f.closeness.scored()), Some(2));
}

#[test]
fn cells_passed_by_decision_0042_are_listed() {
    let mut r = case("probes/a", 1);
    r.cells = vec![
        CellResult {
            graph_subnormal: true,
            ..cell("3E-308-2.5E-308", Verdict::Pass)
        },
        cell("ABS(1)", Verdict::Pass),
    ];
    let report = RunReport::build("e", &[r], &[], &Catalog::default());
    assert_eq!(report.suites[0].graph_subnormal_passes, ["probes/a\tS!A1"]);
    assert!(report.to_markdown(1).contains("- `probes/a\tS!A1`"));
}

fn excel_cell(formula: &str, verdict: Verdict) -> CellResult {
    CellResult {
        from_excel: true,
        ..cell(formula, verdict)
    }
}

#[test]
fn evidence_counts_only_excel_sourced_cells_and_lists_unevidenced_functions() {
    let mut oracle = case("probes/a", 1);
    oracle.expected_source = "excel-graph".into();
    oracle.cells = vec![
        excel_cell("SUM(A1:A2)", Verdict::Pass),
        excel_cell("SUM(A1)+ABS(1)", Verdict::NumericMismatch),
        // A hand-written cell beside a partly covering sidecar is not evidence.
        cell("SUM(A3)+IF(TRUE,1,0)", Verdict::Pass),
    ];
    let mut cached = case("poi/b", 1);
    cached.expected_source = "excel-cached (Microsoft Excel)".into();
    cached.cells = vec![
        excel_cell("_xlfn.XLOOKUP(1,A:A,B:B)", Verdict::Pass),
        // Non-Excel cached values (e.g. LibreOffice) do not count.
        cell("TRIM(\"a\")", Verdict::Pass),
    ];
    let mut hand = case("smoke/c", 1);
    hand.expected_source = "hand".into();
    hand.cells = vec![cell("IF(TRUE,SUM(1),0)", Verdict::Pass)];
    let implemented = ["ABS", "IF", "SUM", "TRIM", "XLOOKUP"].map(String::from);
    let report = RunReport::build(
        "e",
        &[oracle, cached, hand],
        &implemented,
        &Catalog::default(),
    );
    let got: Vec<(&str, usize, Vec<&str>)> = report
        .evidence
        .iter()
        .map(|f| {
            (
                f.name.as_str(),
                f.cells,
                f.suites.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            ("ABS", 1, vec!["probes"]),
            ("XLOOKUP", 1, vec!["poi"]),
            ("SUM", 2, vec!["probes"])
        ]
    );
    assert_eq!(report.unevidenced, ["IF", "TRIM"]);
    let md = report.to_markdown(1);
    assert!(md.contains("Excel evidence: 0 functions with ≥20 oracle/cached cells, 3 with fewer, 2 implemented with none, 0 in-scope catalog functions with none"));
    assert!(md.contains("Implemented, no Excel evidence (2): IF, TRIM"));
    assert!(md.contains("| SUM | 2 | probes | yes |"));
}

#[test]
fn out_of_scope_functions_leave_parity_counts_and_list_with_their_reason() {
    use crate::catalog::{CatalogFunction, OracleSupport, Scope};
    let entry = |name: &str, scope, reason: Option<&str>, oracle| CatalogFunction {
        name: name.into(),
        category: "math".into(),
        milestone: "M5".into(),
        scope,
        reason: reason.map(String::from),
        oracle,
        v15: None,
        v15_reason: None,
        params: Vec::new(),
    };
    let catalog = Catalog::from_functions([
        entry("ABS", Scope::In, None, None),
        entry("COS", Scope::In, None, None),
        entry("INFO", Scope::In, None, Some(OracleSupport::Unavailable)),
        entry("RTD", Scope::Out, Some("real-time server"), None),
        entry("WEBSERVICE", Scope::Out, Some("web | service"), None),
    ]);
    let mut probe = case("probes/a", 1);
    probe.cells = vec![
        excel_cell("ABS(1)", Verdict::NumericMismatch),
        excel_cell("WEBSERVICE(\"x\")", Verdict::ErrorMismatch),
    ];
    let implemented = ["ABS", "INFO", "SUM", "WEBSERVICE"].map(String::from);
    let report = RunReport::build("e", &[probe], &implemented, &catalog);
    let parity: Vec<&str> = report.functions.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(parity, ["ABS"]);
    let evidenced: Vec<&str> = report.evidence.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(evidenced, ["ABS"]);
    // SUM is not in this catalog, so it stays in scope; INFO needs another source.
    assert_eq!(report.unevidenced, ["SUM"]);
    assert_eq!(report.catalog_unevidenced, ["COS"]);
    assert_eq!(report.needs_other_evidence, ["INFO"]);
    let out: Vec<(&str, usize, usize)> = report
        .out_of_scope
        .iter()
        .map(|f| (f.name.as_str(), f.cells, f.failing))
        .collect();
    assert_eq!(out, [("RTD", 0, 0), ("WEBSERVICE", 1, 1)]);
    let md = report.to_markdown(1);
    assert!(md.contains("## Functions with failing cells (1)"), "{md}");
    assert!(md.contains("In-scope catalog functions with no Excel evidence (gaps) (1): COS"));
    assert!(md.contains("(`oracle: none`) (1): INFO"));
    assert!(md.contains("## Out of scope (2)"));
    assert!(
        md.contains("| WEBSERVICE | 1 | 1 | web \\| service |"),
        "{md}"
    );
    assert!(!md.contains("### WEBSERVICE"));
}

#[test]
fn add_in_cells_are_counted_per_suite_and_listed_out_of_scope() {
    // T-0462: cells calling `_xll.` are not scored; the out-of-scope section counts them.
    let mut a = case("calipers/a", 1);
    a.skipped_add_in = 246;
    let mut b = case("calipers/b", 1);
    b.skipped_add_in = 246;
    let report = RunReport::build("e", &[a, b, case("probes/c", 1)], &[], &Catalog::default());
    let calipers = report.suites.iter().find(|s| s.name == "calipers");
    assert_eq!(calipers.map(|s| s.skipped_add_in), Some(492));
    let md = report.to_markdown(1);
    assert!(
        md.contains("- calipers: 492 cells call an add-in function (`_xll.`) and are not scored."),
        "{md}"
    );
    assert!(!md.contains("- probes:"), "{md}");
}

#[test]
fn report_sums_suite_time_and_renders_timing() {
    let report = RunReport::build(
        "e",
        &[case("s/a", 5), case("s/b", 7), case("t/c", 1)],
        &[],
        &Catalog::default(),
    );
    assert_eq!(report.suites[0].millis, 12);
    let md = report.to_markdown(1);
    assert!(
        md.contains("## Timing") && md.contains("| s/b | 7 |"),
        "{md}"
    );
}

#[test]
fn timing_does_not_change_the_gate() {
    use crate::baseline::{diff, Baseline};
    use crate::suite::BaselineMode;
    let fast = [case("s/a", 1), case("s/b", 2)];
    let slow = [case("s/a", 900), case("s/b", 5000)];
    let base = |r: &[CaseResult]| {
        let refs: Vec<&CaseResult> = r.iter().collect();
        Baseline::from_results(BaselineMode::Cells, &refs)
    };
    assert_eq!(base(&fast), base(&slow));
    assert!(!diff(&base(&fast), &base(&slow)).is_dirty());
}
