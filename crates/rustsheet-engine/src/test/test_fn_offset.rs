#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_offset_reference() {
    let mut model = new_empty_model();
    model._set("B1", "12");
    model._set("B2", "13");
    model._set("B3", "15");

    model._set("A1", "=SUM(B1:OFFSET($B$1,3,0))");
    model._set("A2", "=SUM(OFFSET(A1, 1, 1):B3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"40");
    assert_eq!(model._get_text("A2"), *"28");
}

// `@` meets OFFSET in the two directions that route through `get_reference`'s
// fallback arm (cast.rs): the `@` node must reach it as a reference (the
// intersected 1x1 range), not be dereferenced to a value.

#[test]
fn fn_offset_of_at_range() {
    // `@` UNDER OFFSET: the ref argument is an @-wrapped range. In D2, @B1:B4
    // intersects to B2; OFFSET(B2, 1, 0) -> B3.
    let mut model = new_empty_model();
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");
    model._set("B4", "40");
    model._set("D2", "=OFFSET(@B1:B4, 1, 0)");
    model.evaluate();

    assert_eq!(model._get_text("D2"), *"30");
}

#[test]
fn fn_rows_of_at_offset() {
    // `@` OVER OFFSET: OFFSET(B1,0,0,4,1) -> B1:B4; `@` in D2 intersects it to
    // B2, a single cell, so ROWS is 1 (not 4).
    let mut model = new_empty_model();
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");
    model._set("B4", "40");
    model._set("D2", "=ROWS(@OFFSET(B1, 0, 0, 4, 1))");
    model.evaluate();

    assert_eq!(model._get_text("D2"), *"1");
}

// An empty height or width argument keeps the reference's size
// (poi/FormulaEvalTestData_Copy.xlsx EverythingTests row 1008).
#[test]
fn fn_offset_empty_height_width() {
    let mut model = new_empty_model();
    model._set("K7", "1");
    model._set("L7", "1.001");
    model._set("K8", "2");
    model._set("L8", "2.9999");
    model._set("A1", "=SUM(OFFSET(K7:L8, 0,0,1,))");
    model._set("A2", "=SUM(OFFSET(K7:L8, 0,0,,1))");
    model._set("A3", "=SUM(OFFSET(K7:L8, 0,0,,))");
    model._set("A4", "=SUM(OFFSET(K7:L8, 0,0,))");
    model._set("A5", "=SUM(OFFSET(K7:L8, 0,0,1,1))");
    model._set("A6", "=SUM(OFFSET(K7:L8, 0,0,1))");
    model._set("A7", "=SUM(OFFSET(K7:L8, 0,0))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2.001");
    assert_eq!(model._get_text("A2"), *"3");
    assert_eq!(model._get_text("A3"), *"7.0009");
    assert_eq!(model._get_text("A4"), *"7.0009");
    assert_eq!(model._get_text("A5"), *"1");
    assert_eq!(model._get_text("A6"), *"2.001");
    assert_eq!(model._get_text("A7"), *"7.0009");
}

// Huge arguments saturate instead of overflowing i32 (untrusted input must not panic).
#[test]
fn fn_offset_huge_arguments_are_ref_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=OFFSET(B2,1E10,0)");
    model._set("A2", "=OFFSET(B2,-1E10,0)");
    model._set("A3", "=OFFSET(B2,0,1E10)");
    model._set("A4", "=OFFSET(B2,0,-1E10)");
    model._set("A5", "=OFFSET(B2,1,0,1E10)");
    model._set("A6", "=OFFSET(B2,1,0,1,1E10)");
    model._set("A7", "=OFFSET(B2,1,0,-1E10)");
    model._set("A8", "=OFFSET(B2,1,0,1,-1E10)");
    model._set("A9", "=OFFSET(B2,1E300,1E300,1E300,1E300)");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8"] {
        assert_eq!(model._get_text(cell), *"#REF!", "{cell}");
    }
    // A9 only pins "no panic": the result for 1E300 arguments has no oracle evidence.
}

// A negative height/width extends the area up/left from the anchor; fractions and 0 are quirky.
// Excel: probes/offset-negative-height-width (docs/excel-semantics/references.md).
#[test]
fn fn_offset_negative_height_width() {
    let mut model = new_empty_model();
    for r in 1..=12 {
        model._set(&format!("A{r}"), &r.to_string());
        model._set(&format!("B{r}"), &(r * 100).to_string());
    }
    model._set("D1", "=SUM(OFFSET(A10,0,0,-3,1))");
    model._set("D2", "=SUM(OFFSET(B10,0,0,1,-2))");
    model._set("D3", "=SUM(OFFSET(B10,0,0,-2,-2))");
    model._set("D4", "=SUM(OFFSET(A5:B6,1,1,-2,-2))");
    model._set("D5", "=SUM(OFFSET(A2,0,0,-3,1))");
    model._set("D6", "=OFFSET(A10,0,0,0,1)");
    model._set("D7", "=SUM(OFFSET(A10,0,0,0.5,1))");
    model._set("D8", "=SUM(OFFSET(A10,0,0,-0.5,1))");
    model._set("D9", "=SUM(OFFSET(A10,0,0,-2.9,1))");
    model._set("D10", "=SUM(OFFSET(A5:B6,0,0,,-2))");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"27");
    assert_eq!(model._get_text("D2"), *"1010");
    assert_eq!(model._get_text("D3"), *"1919");
    assert_eq!(model._get_text("D4"), *"1111");
    assert_eq!(model._get_text("D5"), *"#REF!");
    assert_eq!(model._get_text("D6"), *"#REF!");
    assert_eq!(model._get_text("D7"), *"19");
    assert_eq!(model._get_text("D8"), *"21");
    assert_eq!(model._get_text("D9"), *"19");
    assert_eq!(model._get_text("D10"), *"#REF!");
}

// SPEC-0014: OFFSET lifted over an array is an array of references. It never reaches a cell:
// spilled, every reference element is `#VALUE!` and an error element stays itself.
#[test]
fn fn_offset_lifted_spills_its_value_view() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("C1", "=OFFSET(A1,{0;-1;1},0)");
    model._set("D1", "=ROWS(OFFSET(A1,{0;1},NA()))");
    model._set("E1", "=INDEX(OFFSET(A1,{0;1},0),2,1)");
    model._set("F1", "=ISREF(OFFSET(A1,{0;1},0))");
    model._set("G1", "=ISREF(INDEX(OFFSET(A1,{0;1},0),1,1))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"#VALUE!");
    assert_eq!(model._get_text("C2"), *"#REF!");
    assert_eq!(model._get_text("C3"), *"#VALUE!");
    assert_eq!(model._get_text("D1"), *"2");
    assert_eq!(model._get_text("E1"), *"2");
    assert_eq!(model._get_text("F1"), *"FALSE");
    assert_eq!(model._get_text("G1"), *"TRUE");
}

// The broadcast shape is checked before any element is computed (CLAUDE.md directive 5).
#[test]
fn fn_offset_lift_past_the_array_bound_is_an_error() {
    let mut model = new_empty_model();
    model._set("A1", "=ROWS(OFFSET(B1,SEQUENCE(2000),SEQUENCE(1,1000)))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn ref_element_is_small() {
    // SPEC-0014 "Bounds": no larger than the `ArrayNode` it stands in for.
    assert!(std::mem::size_of::<crate::reference_operators::RefElement>() <= 32);
}

// SPEC-0014 rules 4-5 (T-2063): SUM, AVERAGE, MIN, MAX and PRODUCT read each element's top-left
// cell; COUNT counts no element, COUNTA every one; SUMPRODUCT reads an element as 0 and an error
// element as its error (probes/reference-lift-consumers E1-E29, -b2 E117-E189).
#[test]
fn fn_offset_lifted_into_top_left_aggregates() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "100");
    model._set("A3", "10000");
    model._set("A4", "'5");
    model._set("A6", "TRUE");
    model._set("A7", "7");
    model._set("C1", "=SUM(OFFSET(A1,0,0,{1;2}))");
    model._set("C2", "=AVERAGE(OFFSET(A1,{0;1},0))");
    model._set("C3", "=MAX(OFFSET(A1,0,0,{1;3}))");
    model._set("C4", "=MIN(OFFSET(A4,0,0,{1;3}))");
    model._set("C5", "=PRODUCT(OFFSET(A1,{0;1},0))");
    model._set("C6", "=SUM(OFFSET(A1,{0;1},0),1)");
    model._set("C7", "=AVERAGE(OFFSET(A1,{0;-1},0))");
    model._set("C8", "=COUNT(OFFSET(A1,0,0,{1;2}))");
    model._set("C9", "=COUNTA(OFFSET(A5,0,0,{1;2}))");
    model._set("C10", "=SUMPRODUCT(OFFSET(A1,{0;1},0))");
    model._set("C11", "=SUMPRODUCT(OFFSET(A1,{0;-1},0))");
    model._set("C12", "=SUM(OFFSET(A4,{0;3},0))");
    model._set("C13", "=SUM(OFFSET(A6,0,0,{1;2}))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"2");
    assert_eq!(model._get_text("C2"), *"50.5");
    assert_eq!(model._get_text("C3"), *"1");
    assert_eq!(model._get_text("C4"), *"0");
    assert_eq!(model._get_text("C5"), *"100");
    assert_eq!(model._get_text("C6"), *"102");
    assert_eq!(model._get_text("C7"), *"#REF!");
    assert_eq!(model._get_text("C8"), *"0");
    assert_eq!(model._get_text("C9"), *"2");
    assert_eq!(model._get_text("C10"), *"0");
    assert_eq!(model._get_text("C11"), *"#REF!");
    assert_eq!(model._get_text("C12"), *"7");
    assert_eq!(model._get_text("C13"), *"0");
}

// The aggregates SPEC-0014 rules 4-5 do not name keep the value view, every reference element
// `#VALUE!`, until their Excel results are known (T-2065).
#[test]
fn fn_offset_lifted_into_other_aggregates_is_its_value_view() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "100");
    model._set("C1", "=MEDIAN(OFFSET(A1,{0;1},0))");
    model._set("C2", "=AVERAGEA(OFFSET(A1,{0;1},0))");
    model._set("C3", "=STDEV.S(OFFSET(A1,{0;1},0))");
    model._set("C4", "=MAXA(OFFSET(A1,{0;1},0))");
    model._set("C5", "=SUMSQ(OFFSET(A1,{0;1},0))");
    model.evaluate();
    for cell in ["C1", "C2", "C3", "C4", "C5"] {
        assert_eq!(model._get_text(cell), *"#VALUE!", "{cell}");
    }
}

// LET, a LAMBDA parameter, IF and CHOOSE hand an array of references on, and OFFSET over one
// lifts again (SPEC-0014 rule 7; probes/reference-lift-consumers E89, E145, E149,
// reference-lift-pass-through).
#[test]
fn fn_offset_grid_passes_through_let_if_choose_and_nested_offset() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "100");
    model._set("A3", "10000");
    model._set("C1", "=LET(x,OFFSET(A1,{0;1},0),INDEX(x,2,1))");
    model._set(
        "C2",
        "=LET(x,OFFSET(A1,{0;1},0),y,x,SUM(INDEX(y,2,1),INDEX(x,1,1)))",
    );
    model._set("C3", "=LET(f,LAMBDA(r,INDEX(r,2,1)),f(OFFSET(A1,{0;1},0)))");
    model._set("C4", "=ROWS(OFFSET(OFFSET(A1,{0;1},0),1,0))");
    model._set("C5", "=SUM(OFFSET(OFFSET(A1,{0;1},0),{0,1},0))");
    model._set(
        "C6",
        "=ERROR.TYPE(INDEX(OFFSET(OFFSET(A1,{0;-1},0),1,0),2,1))",
    );
    model._set("C7", "=SUM(OFFSET(LET(x,OFFSET(A1,{0;1},0),x),1,0))");
    model._set("C8", "=SUMPRODUCT(IF(TRUE,OFFSET(A1,{0;1},0)))");
    model._set("C9", "=LET(x,OFFSET(A1,{0;1},0),ISREF(x))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"100");
    assert_eq!(model._get_text("C2"), *"101");
    assert_eq!(model._get_text("C3"), *"100");
    assert_eq!(model._get_text("C4"), *"2");
    assert_eq!(model._get_text("C5"), *"10201");
    assert_eq!(model._get_text("C6"), *"4");
    assert_eq!(model._get_text("C7"), *"10100");
    assert_eq!(model._get_text("C8"), *"0");
    assert_eq!(model._get_text("C9"), *"FALSE");
}

// Each level of a deep OFFSET nest over an array of references is one lift of the grid's
// elements, so the work grows with depth times elements, not exponentially (SPEC-0014 "Bounds").
#[test]
fn fn_offset_deep_nest_over_a_grid_stays_bounded() {
    let mut model = new_empty_model();
    let depth = 60;
    let mut formula = "OFFSET(A1,SEQUENCE(1000)-1,0)".to_string();
    for _ in 0..depth {
        formula = format!("OFFSET({formula},{{0,1}},0)");
    }
    model._set("C1", &format!("=ROWS({formula})"));
    model._set("C2", &format!("=COLUMNS({formula})"));
    model._set("C3", &format!("=COUNTA({formula})"));
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"1000");
    assert_eq!(model._get_text("C2"), *"2");
    assert_eq!(model._get_text("C3"), *"2000");
}
