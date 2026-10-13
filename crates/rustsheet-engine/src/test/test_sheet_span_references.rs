#![allow(clippy::unwrap_used)]

// 3-D references (`First:Last!A1`, `First:Last!A1:B2`) cover the same cells on every sheet
// from First to Last in tab order. The aggregates read them through the shared range walk;
// where a value is needed they are #REF!, and where one sheet's reference is needed #VALUE!
// (probes/3d-ref-aggregates, 3d-ref-values, 3d-ref-functions; references.md).

use std::time::{Duration, Instant};

use crate::test::util::new_empty_model;
use crate::Model;

// Sheet1 holds the formulas; Data1..Data3 and "Q 4" hold A1 = 1, 10, 100, 1000.
fn sample() -> Model {
    let mut model = new_empty_model();
    for name in ["Data1", "Data2", "Data3", "Q 4"] {
        model.add_sheet(name).unwrap();
    }
    model._set("Data1!A1", "1");
    model._set("Data1!A2", "5x");
    model._set("Data1!A3", "TRUE");
    model._set("Data2!A1", "10");
    model._set("Data3!A1", "100");
    model._set("Data3!A2", "2");
    model._set("Q 4!A1", "1000");
    model
}

fn eval(model: &mut Model, formula: &str) -> String {
    model._set("A1", formula);
    model.evaluate();
    model._get_text("A1")
}

#[test]
fn aggregates_read_every_sheet_of_the_span() {
    let mut model = sample();
    assert_eq!(eval(&mut model, "=SUM(Data1:Data3!A1)"), "111");
    assert_eq!(eval(&mut model, "=SUM(Data1:Data3!A1:A3)"), "113");
    assert_eq!(eval(&mut model, "=COUNT(Data1:Data3!A1:A3)"), "4");
    assert_eq!(eval(&mut model, "=COUNTA(Data1:Data3!A1:A3)"), "6");
    assert_eq!(eval(&mut model, "=AVERAGE(Data1:Data3!A1)"), "37");
    assert_eq!(eval(&mut model, "=MIN(Data1:Data3!A1:A2)"), "1");
    assert_eq!(eval(&mut model, "=MAX(Data1:Data3!A1,5000)"), "5000");
}

#[test]
fn span_order_and_quoting() {
    let mut model = sample();
    // Written backwards it is the same span (3d-ref-aggregates C2).
    assert_eq!(eval(&mut model, "=SUM(Data3:Data1!A1)"), "111");
    assert_eq!(eval(&mut model, "=SUM('Data2:Q 4'!A1)"), "1110");
    // The formula's own sheet may be in the span.
    assert_eq!(eval(&mut model, "=SUM(Sheet1:Data2!A1)"), "#CIRC!");
    model._set("B1", "=SUM(Sheet1:Data2!A1)");
    model._set("A1", "7");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "18");
}

#[test]
fn formula_text_round_trips() {
    let mut model = sample();
    model._set("A1", "=SUM(Data1:Data3!A1:B2)");
    model._set("A2", "=SUM('Data2:Q 4'!$A$1)");
    assert_eq!(model._get_formula("A1"), "=SUM(Data1:Data3!A1:B2)");
    assert_eq!(model._get_formula("A2"), "=SUM('Data2:Q 4'!$A$1)");
    model.rename_sheet("Data3", "Last").unwrap();
    assert_eq!(model._get_formula("A1"), "=SUM(Data1:Last!A1:B2)");
}

#[test]
fn an_error_cell_in_the_span_wins() {
    let mut model = sample();
    model._set("Data2!B1", "=1/0");
    assert_eq!(eval(&mut model, "=SUM(Data1:Data3!B1)"), "#DIV/0!");
}

#[test]
fn a_value_context_is_ref_error() {
    let mut model = sample();
    assert_eq!(eval(&mut model, "=Data1:Data3!A1"), "#REF!");
    assert_eq!(eval(&mut model, "=Data1:Data3!A1+1"), "#REF!");
    assert_eq!(eval(&mut model, "=ABS(Data1:Data3!A1)"), "#REF!");
    assert_eq!(eval(&mut model, "=ERROR.TYPE(Data1:Data3!A1)"), "4");
    // A span of one sheet is that sheet's reference (3d-ref-values C9).
    assert_eq!(eval(&mut model, "=Data2:Data2!A1"), "10");
}

#[test]
fn a_one_sheet_reference_context_is_value_error() {
    let mut model = sample();
    assert_eq!(eval(&mut model, "=ROWS(Data1:Data3!A1:A3)"), "#VALUE!");
    assert_eq!(eval(&mut model, "=ROW(Data1:Data3!A2)"), "#VALUE!");
    assert_eq!(eval(&mut model, "=SUMIF(Data1:Data3!A1,\">0\")"), "#VALUE!");
}

#[test]
fn corners_on_two_sheets_are_not_a_span() {
    let mut model = sample();
    assert_eq!(eval(&mut model, "=SUM(Data1!A1:Data3!A1)"), "#VALUE!");
}

#[test]
fn whole_columns_walk_only_the_used_area() {
    let mut model = sample();
    for index in 0..20 {
        model.add_sheet(&format!("Extra{index}")).unwrap();
    }
    model._set("Extra19!A5", "3");
    let start = Instant::now();
    assert_eq!(eval(&mut model, "=SUM(Data1:Extra19!A:C)"), "1116");
    assert!(start.elapsed() < Duration::from_secs(10));
}

// Two-input functions pair their lists by position, so a whole column must keep one entry per
// cell even when the two sheets' used areas differ (review of T-0196).
#[test]
fn paired_whole_columns_keep_positions() {
    let mut model = sample();
    assert_eq!(eval(&mut model, "=SUMXMY2(Data1!A:A,Data2!A:A)"), "81");
    // A length mismatch must not panic.
    let result = eval(&mut model, "=CHISQ.TEST(Data1!A:A,SEQUENCE(1048576))");
    assert!(!result.is_empty());
}

// Where one sheet's reference is needed a 3-D reference is #VALUE! (T-0197,
// probes/3d-ref-functions C1, C5, C6, C10, C13).
#[test]
fn single_sheet_lookup_functions_give_value_error() {
    let mut model = sample();
    for formula in [
        "=INDEX(Data1:Data3!A1:A2,1)",
        "=AREAS(Data1:Data3!A1)",
        "=VLOOKUP(1,Data1:Data3!A1:B2,1,FALSE)",
        "=MATCH(1,Data1:Data3!A1:A2,0)",
        "=COUNTBLANK(Data1:Data3!A1:A2)",
    ] {
        assert_eq!(eval(&mut model, formula), "#VALUE!", "{formula}");
    }
    // A span of one sheet is that sheet's reference.
    assert_eq!(eval(&mut model, "=AREAS(Data1:Data1!A1)"), "1");
    assert_eq!(eval(&mut model, "=COUNTBLANK(Data1:Data1!A1:A5)"), "2");
}

// The error of a prefix that names no sheet: #REF! for a plain name, #NAME? for a span, and
// #BLOCKED! for another workbook (probes/external-ref-no-link A1, A2, A3; T-0124).
#[test]
fn unknown_sheet_prefix_errors() {
    let mut model = sample();
    assert_eq!(eval(&mut model, "=SUM(Nope!A1:B1)"), "#REF!");
    assert_eq!(eval(&mut model, "=SUM(Data1:Nope!A1)"), "#NAME?");
    assert_eq!(
        eval(&mut model, "=SUM('[other.xlsx]Sheet1'!A1:B1)"),
        "#BLOCKED!"
    );
    assert_eq!(eval(&mut model, "='[other.xlsx]Sheet1'!A1"), "#BLOCKED!");
    assert_eq!(
        eval(&mut model, "=ERROR.TYPE('[other.xlsx]Sheet1'!A1)"),
        "11"
    );
}

// A reference into another workbook with no cached data is still a reference of the written
// shape: ISREF and the shape functions read it, its cells are #BLOCKED!, and the unquoted
// `[book]Sheet!` prefix parses (probes/external-ref-no-link A4, A7, A10; external-ref-no-link-
// shapes; T-1381).
#[test]
fn external_reference_keeps_its_shape() {
    let mut model = sample();
    assert_eq!(eval(&mut model, "=ISREF('[other.xlsx]Sheet1'!A1)"), "TRUE");
    assert_eq!(
        eval(&mut model, "=ISREF('[other.xlsx]Sheet1'!A1:B3)"),
        "TRUE"
    );
    assert_eq!(eval(&mut model, "=ROWS('[other.xlsx]Sheet1'!A1:B3)"), "3");
    assert_eq!(
        eval(&mut model, "=COLUMNS('[other.xlsx]Sheet1'!A1:C2)"),
        "3"
    );
    assert_eq!(
        eval(&mut model, "=ROWS('[other.xlsx]Sheet1'!A:A)"),
        "1048576"
    );
    assert_eq!(eval(&mut model, "=AREAS('[other.xlsx]Sheet1'!A1:C2)"), "1");
    assert_eq!(eval(&mut model, "=ROW('[other.xlsx]Sheet1'!B5)"), "5");
    assert_eq!(eval(&mut model, "=COLUMN('[other.xlsx]Sheet1'!B5)"), "2");
    assert_eq!(eval(&mut model, "=COUNT('[other.xlsx]Sheet1'!A1:B1)"), "0");
    // The unquoted prefix.
    assert_eq!(
        eval(&mut model, "=SUM([other.xlsx]Sheet1!A1:B1)"),
        "#BLOCKED!"
    );
    assert_eq!(eval(&mut model, "=ROWS([other.xlsx]Sheet1!A1:B3)"), "3");
    assert_eq!(eval(&mut model, "=ISREF([other.xlsx]Sheet1!A1)"), "TRUE");
    // A 3-D prefix naming another workbook is not a reference.
    assert_eq!(
        eval(&mut model, "=SUM('[other.xlsx]S1:S2'!A1)"),
        "#BLOCKED!"
    );
    assert_eq!(eval(&mut model, "=ISREF('[other.xlsx]S1:S2'!A1)"), "FALSE");
    // A missing sheet of this workbook stays #REF!, not a reference.
    assert_eq!(eval(&mut model, "=ISREF(Nope!A1)"), "FALSE");
    assert_eq!(eval(&mut model, "=ROWS(Nope!A1:B3)"), "#REF!");
}

// Span edge forms (T-1170; probes/3d-ref-span-edges A1, A2, B1-B3, C4, C5).
#[test]
fn span_edge_forms_match_excel() {
    let mut model = sample();
    // A span whose end is not a sheet is #NAME?, not #REF!.
    assert_eq!(eval(&mut model, "=SUM(Data1:Nope!A1)"), "#NAME?");
    assert_eq!(eval(&mut model, "=SUM(Nope:Data3!A1)"), "#NAME?");
    assert_eq!(eval(&mut model, "=SUM(Data1:Nope!A1:B2)"), "#NAME?");
    // An explicit `@` of a span over several sheets is #REF!; of one sheet an ordinary reference.
    assert_eq!(eval(&mut model, "=@Data1:Data3!A1"), "#REF!");
    assert_eq!(eval(&mut model, "=SUM(@Data1:Data3!A1:A2)"), "#REF!");
    assert_eq!(eval(&mut model, "=@Data1:Data1!A1"), "1");
    // IF-wrapped spans are #VALUE! for HLOOKUP and COUNTBLANK too.
    assert_eq!(
        eval(&mut model, "=HLOOKUP(1,IF(TRUE,Data1:Data3!A1:B2),2,FALSE)"),
        "#VALUE!"
    );
    assert_eq!(
        eval(&mut model, "=COUNTBLANK(IF(TRUE,Data1:Data3!A1:B3))"),
        "#VALUE!"
    );
    assert_eq!(
        eval(&mut model, "=COUNTBLANK(IF(TRUE,Data1:Data1!A1:A5))"),
        "2"
    );
}
