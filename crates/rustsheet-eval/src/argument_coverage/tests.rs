use super::*;
use crate::catalog::{CatalogFunction, Scope};
use crate::value::Verdict;

/// Data!A1 is 0, A2 is `"12"`, A3 is `FALSE`, A4 is `#N/A`; every other cell is empty.
struct Data;
impl Evaluated for Data {
    fn value(&self, sheet: &str, addr: Addr) -> Option<Value> {
        if !sheet.eq_ignore_ascii_case("Data") && !sheet.eq_ignore_ascii_case("Calc") {
            return None;
        }
        Some(match (sheet, addr.col, addr.row) {
            ("Data", 1, 1) => Value::Number(0.0),
            ("Data", 1, 2) => Value::Text("12".into()),
            ("Data", 1, 3) => Value::Bool(false),
            ("Data", 1, 4) => Value::Error("#N/A".into()),
            _ => Value::Empty,
        })
    }
    fn stored_cells(&self, _: &str) -> Vec<Addr> {
        Vec::new()
    }
}

fn cell(formula: &str, from_excel: bool) -> CellResult {
    CellResult {
        sheet: "Calc".into(),
        cell: "B2".into(),
        formula: Some(formula.into()),
        anchor_formula: None,
        expected: Value::Empty,
        actual: Value::Empty,
        verdict: Verdict::Pass,
        from_excel,
        listed_deviation: false,
        graph_subnormal: false,
        kind: crate::runner::CellKind::Scored,
        math: None,
    }
}

fn param(name: &str, mode: &str, repeat: Option<usize>) -> CatalogParam {
    CatalogParam {
        name: name.into(),
        mode: mode.into(),
        repeat,
    }
}

fn function(name: &str, params: Vec<CatalogParam>) -> CatalogFunction {
    CatalogFunction {
        name: name.into(),
        category: "math".into(),
        milestone: "M2".into(),
        scope: Scope::In,
        reason: None,
        oracle: None,
        v15: None,
        v15_reason: None,
        params,
    }
}

fn observed(formula: &str) -> Vec<(usize, ArgClass)> {
    let sheets = vec!["Calc".to_string(), "Data".to_string()];
    observe(&Data, &sheets, &[cell(formula, true)])
        .into_iter()
        .flat_map(|call| call.args.into_iter().enumerate())
        .filter_map(|(i, a)| Some((i, a?.class)))
        .collect()
}

#[test]
fn every_value_class() {
    let cases = [
        (Value::Number(3.0), ArgClass::Number),
        (Value::Number(0.0), ArgClass::Zero),
        (Value::Number(-0.5), ArgClass::Negative),
        (Value::Number(2.5), ArgClass::Fraction),
        (Value::Text(" -1.5e3 ".into()), ArgClass::NumericText),
        (Value::Text("abc".into()), ArgClass::Text),
        (Value::Text("inf".into()), ArgClass::Text),
        (Value::Text("1/2".into()), ArgClass::Text),
        (Value::Text(String::new()), ArgClass::EmptyText),
        (Value::Bool(true), ArgClass::Boolean),
        (Value::Error("#DIV/0!".into()), ArgClass::Error),
        (Value::Empty, ArgClass::Blank),
    ];
    for (value, class) in cases {
        assert_eq!(classify_value(&value), class, "{value}");
    }
}

#[test]
fn literals_are_classified_as_written() {
    assert_eq!(
        observed(r#"=IF(TRUE,-2,"")"#),
        vec![
            (0, ArgClass::Boolean),
            (1, ArgClass::Negative),
            (2, ArgClass::EmptyText)
        ]
    );
    assert_eq!(
        observed(r#"=ROUND("7",)"#),
        vec![(0, ArgClass::NumericText), (1, ArgClass::Blank)]
    );
    assert_eq!(observed("=ABS(#N/A)"), vec![(0, ArgClass::Error)]);
    assert_eq!(observed("=SUM({1,2})"), vec![(0, ArgClass::Range)]);
}

#[test]
fn single_cell_references_are_classified_by_the_cell_value() {
    let got = observed("=SUM(Data!A1,Data!A2,Data!A3,Data!A4,Data!A5,Data!A1:A2)");
    assert_eq!(
        got,
        vec![
            (0, ArgClass::Zero),
            (1, ArgClass::NumericText),
            (2, ArgClass::Boolean),
            (3, ArgClass::Error),
            (4, ArgClass::Blank),
            (5, ArgClass::Range),
        ]
    );
    // A relative reference on the formula's own sheet.
    assert_eq!(observed("=ABS(C3)"), vec![(0, ArgClass::Blank)]);
    // ABS's nested call is left unclassified; SQRT's literal is classified.
    let sheets = vec!["Calc".to_string()];
    let got = observe(&Data, &sheets, &[cell("=ABS(SQRT(4))", true)]);
    let calls: Vec<(&str, &[Option<ArgumentSeen>])> = got
        .iter()
        .map(|c| (c.function.as_str(), c.args.as_slice()))
        .collect();
    let four = Some(ArgumentSeen {
        class: ArgClass::Number,
        reference: false,
    });
    // A call without a classified argument is left out.
    assert_eq!(calls, vec![("SQRT", &[four][..])]);
}

#[test]
fn only_excel_evidence_cells_are_observed() {
    let sheets = vec!["Calc".to_string()];
    assert!(observe(&Data, &sheets, &[cell("=ABS(1)", false)]).is_empty());
}

#[test]
fn arguments_map_to_catalog_parameters() {
    let abs = [param("number", "value", None)];
    assert_eq!(param_of(&abs, 1, 0), Some(0));
    assert_eq!(param_of(&abs, 2, 1), None);
    let sumifs = [
        param("sum_range", "range", None),
        param("criteria_range", "range", Some(2)),
        param("criteria", "criteria", None),
    ];
    assert_eq!(
        (0..5).map(|i| param_of(&sumifs, 5, i)).collect::<Vec<_>>(),
        vec![Some(0), Some(1), Some(2), Some(1), Some(2)]
    );
    // LET(name, value, ..., calculation): the last argument is the calculation.
    let let_ = [
        param("name", "name", Some(2)),
        param("value", "any", None),
        param("calculation", "any", None),
    ];
    assert_eq!(
        (0..5).map(|i| param_of(&let_, 5, i)).collect::<Vec<_>>(),
        vec![Some(0), Some(1), Some(0), Some(1), Some(2)]
    );
    // SWITCH(expression, value, result, ..., [default]).
    let switch = [
        param("expression", "value", None),
        param("value", "value", Some(2)),
        param("result", "any", None),
        param("default", "any", None),
    ];
    assert_eq!(param_of(&switch, 4, 3), Some(3));
    assert_eq!(param_of(&switch, 5, 3), Some(1));
    assert_eq!(param_of(&switch, 5, 4), Some(2));
}

#[test]
fn summary_counts_expected_classes_per_parameter() {
    let catalog = Catalog::from_functions([
        function("ABS", vec![param("number", "value", None)]),
        function(
            "SUMIF",
            vec![
                param("range", "range", None),
                param("criteria", "criteria", None),
            ],
        ),
        function("PI", Vec::new()),
    ]);
    let sheets = vec!["Calc".to_string(), "Data".to_string()];
    let cells = [
        cell("=ABS(Data!A1)", true),
        cell("=ABS(2.5)", true),
        cell("=SUMIF(Data!A1,1)", true),
    ];
    let mut result = CaseResult::empty("c", "s", crate::runner::CaseStatus::Pass);
    result.arguments = observe(&Data, &sheets, &cells);
    let rows = summarize(&catalog, &[result]);
    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, vec!["ABS", "PI", "SUMIF"]);
    assert_eq!(
        rows[0].params[0].seen,
        vec![ArgClass::Zero, ArgClass::Fraction]
    );
    assert_eq!(
        (rows[0].params[0].covered, rows[0].params[0].expected),
        (2, 11)
    );
    assert_eq!(rows[1].single, None);
    // A `range` parameter counts a single-cell reference as a range.
    assert_eq!(rows[2].params[0].seen, vec![ArgClass::Range]);
    assert_eq!(rows[2].params[1].seen, vec![ArgClass::Number]);
    assert_eq!(rows[2].single, Some(2.0 / 12.0));
}

#[test]
fn pairwise_counts_expected_class_pairs_one_call_passes_together() {
    let catalog = Catalog::from_functions([
        function(
            "ROUND",
            vec![
                param("number", "value", None),
                param("num_digits", "value", None),
            ],
        ),
        function(
            "SUMIF",
            vec![
                param("range", "range", None),
                param("criteria", "criteria", None),
                param("sum_range", "range", None),
            ],
        ),
        function("ABS", vec![param("number", "value", None)]),
    ]);
    let sheets = vec!["Calc".to_string(), "Data".to_string()];
    let cells = [
        cell("=ROUND(2.5,0)", true),
        cell("=ROUND(-1,0)", true),
        // Seen apart, (fraction, negative) is not seen together.
        cell("=ROUND(2.5,SQRT(4))", true),
        cell("=ROUND(SQRT(4),-1)", true),
        // A literal where a `range` parameter expects a reference counts for no pair.
        cell("=SUMIF(Data!A1:A2,1,5)", true),
        cell("=SUMIF(Data!A1,TRUE,Data!A3)", true),
    ];
    let mut result = CaseResult::empty("c", "s", crate::runner::CaseStatus::Pass);
    result.arguments = observe(&Data, &sheets, &cells);
    let rows = summarize(&catalog, &[result]);
    // ROUND: (fraction, zero) and (negative, zero) of 11 × 11 pairs.
    assert_eq!(rows[1].name, "ROUND");
    assert_eq!(rows[1].pairwise, Some(2.0 / 121.0));
    // SUMIF: (range, number), (range, boolean), (boolean, range), (range, range) of
    // 1·11 + 1·1 + 11·1 pairs.
    assert_eq!(rows[2].pairwise, Some(4.0 / 23.0));
    assert_eq!(
        rows[2].params[2].seen,
        vec![ArgClass::Number, ArgClass::Range]
    );
    assert_eq!(rows[2].params[2].covered, 1);
    // One counted parameter has no pairs.
    assert_eq!(rows[0].pairwise, None);
    assert!(rows[0].unseen_pairs.is_empty());
    // Every expected pair not seen is listed: 121 - 2 for ROUND, 23 - 4 for SUMIF.
    assert_eq!(rows[1].unseen_pairs.len(), 119);
    assert_eq!(rows[2].unseen_pairs.len(), 19);
    assert_eq!(
        rows[1].unseen_pairs[0],
        UnseenPair {
            params: ["number".into(), "num_digits".into()],
            classes: [ArgClass::Number, ArgClass::Number],
        }
    );
    assert!(!rows[1]
        .unseen_pairs
        .iter()
        .any(|u| u.classes == [ArgClass::Fraction, ArgClass::Zero]));
}

#[test]
fn no_pair_is_unseen_once_every_one_is_seen() {
    // A `range` parameter expects one class, so one call with two references (as SUMIF's sum_range above) sees the only pair.
    let catalog = Catalog::from_functions([function(
        "SUMSQ",
        vec![param("a", "range", None), param("b", "range", None)],
    )]);
    let sheets = vec!["Calc".to_string(), "Data".to_string()];
    let cells = [cell("=SUMSQ(Data!A1,Data!B1)", true)];
    let mut result = CaseResult::empty("c", "s", crate::runner::CaseStatus::Pass);
    result.arguments = observe(&Data, &sheets, &cells);
    let rows = summarize(&catalog, &[result]);
    assert_eq!(rows[0].pairwise, Some(1.0));
    assert!(rows[0].unseen_pairs.is_empty());
}

#[test]
fn a_repeated_parameter_pairs_with_the_others_not_itself() {
    let catalog = Catalog::from_functions([function(
        "SUMIFS",
        vec![
            param("sum_range", "range", None),
            param("criteria_range", "range", Some(2)),
            param("criteria", "criteria", None),
        ],
    )]);
    let sheets = vec!["Calc".to_string(), "Data".to_string()];
    let cells = [cell(
        "=SUMIFS(Data!A1:A2,Data!A1:A2,1,Data!A1:A2,\"x\")",
        true,
    )];
    let mut result = CaseResult::empty("c", "s", crate::runner::CaseStatus::Pass);
    result.arguments = observe(&Data, &sheets, &cells);
    let rows = summarize(&catalog, &[result]);
    // (sum, crit_range), (sum, number), (sum, text), (crit_range, number), (crit_range, text)
    // of 1·1 + 1·11 + 1·11 pairs.
    assert_eq!(rows[0].pairwise, Some(5.0 / 23.0));
}
