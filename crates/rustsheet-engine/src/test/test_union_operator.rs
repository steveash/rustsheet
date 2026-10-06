#![allow(clippy::unwrap_used)]

//! The union operator, a comma inside parentheses (`(A1,B2)`), T-0134. Expected values are
//! Excel's, from gen-m1/c940d3cfb0d3b083 (oracle excel-graph, 2026-10-05) unless marked
//! UNVERIFIED; see docs/excel-semantics/references.md "Union (comma) reference in a value
//! context is #VALUE!".

use crate::model::Model;
use crate::test::util::new_empty_model;

/// Sheet1 (the formulas) and `Data`, with values like the gen-m1 case's.
fn model() -> Model {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model.add_sheet("Other").unwrap();
    for (cell, value) in [
        ("Data!A7", "=NA()"),
        ("Data!A8", "0.5"),
        ("Data!A9", "ABC"),
        ("Data!B1", "1E-15"),
        ("Data!B8", "FALSE"),
        ("Data!C2", "a"),
        ("Data!C12", "' 2 "),
        ("Data!D4", "3.25"),
        ("Data!D10", "=1/0"),
        ("Data!D11", "a"),
        ("Data!E4", "7"),
        ("Data!E6", "TRUE"),
        ("Data!E8", "=1/0"),
    ] {
        model._set(cell, value);
    }
    model
}

#[test]
fn union_in_a_value_context_is_value_error() {
    let mut model = model();
    for (cell, formula) in [
        ("A1", "=(Data!E4,Data!E4)"),
        ("A2", "=(Data!B6,Data!E11)"),
        ("A3", "=(Data!E12,Data!A9)/1"),
        ("A4", "=(Data!C7,Data!A1)+1"),
        ("A5", "=(Data!C12,Data!E9)&\"\""),
        ("A6", "=((Data!A2,Data!E8),Data!B6)"),
        // Cells holding errors do not matter: the union itself is not a value.
        ("A7", "=(Data!C9,Data!E8)*1"),
        ("A8", "=(Data!A7,Data!D10)"),
        ("A9", "=-(Data!E4,Data!B1)"),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
}

#[test]
fn union_cut_by_a_cell_gives_that_cell() {
    let mut model = model();
    for (cell, formula) in [
        ("A1", "=(Data!C1:C12,Data!E1:E12) Data!C9"),
        ("A2", "=(Data!D1:D12,Data!A1:A12) Data!D11"),
        ("A3", "=(Data!A1:A12,Data!C1:C12) Data!A7"),
        ("A4", "=(Data!D1:D12,Data!A1:A12) Data!D4"),
        ("A5", "=(Data!C1:C12,Data!E1:E12) Data!C12"),
        ("A6", "=(Data!E1:E12,Data!B1:B12) Data!E6"),
        ("A7", "=(Data!B1:B12,Data!D1:D12) Data!B8"),
        ("A8", "=(Data!D1:D12,Data!A1:A12) Data!D10"),
        ("A9", "=(Data!A1:A12,Data!C1:C12) Data!A9"),
        ("A10", "=(Data!B1:B12,Data!D1:D12) Data!B1"),
        // The cut may come first, and the result is an operand like any cell.
        ("A11", "=Data!A8 (Data!A1:A12,Data!C1:C12)"),
        ("A12", "=(Data!D1:D12,Data!A1:A12) Data!D4+1"),
        ("A13", "=(Data!C1:C12,Data!E1:E12) Data!C9&\"\""),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, expected) in [
        ("A1", "0"),
        ("A2", "a"),
        ("A3", "#N/A"),
        ("A4", "3.25"),
        ("A5", " 2 "),
        ("A6", "TRUE"),
        ("A7", "FALSE"),
        ("A8", "#DIV/0!"),
        ("A9", "ABC"),
        ("A10", "1E-15"),
        ("A11", "0.5"),
        ("A12", "4.25"),
        ("A13", ""),
    ] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
}

#[test]
fn intersection_leaving_several_areas_or_none() {
    let mut model = model();
    // Two areas left, even of the same cell: `#VALUE!` (probes/intersection-function-operands H5).
    model._set("A1", "=Data!A1:A3 (Data!A2:B2,Data!A2)");
    // No area left: `#NULL!`, as for two plain areas (UNVERIFIED for a union).
    model._set("A2", "=(Data!A1,Data!B2) Data!C3");
    // Several areas, cut again down to one (UNVERIFIED).
    model._set("A3", "=(Data!A1:A12,Data!D1:D12) Data!A4:D4 Data!D4");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#NULL!");
    assert_eq!(model._get_text("A3"), "3.25");
}

#[test]
fn union_errors() {
    let mut model = model();
    // An operand's own error comes first (UNVERIFIED).
    model._set("A1", "=(Data!A1,#REF!)");
    model._set("A2", "=(Data!A1,#REF!) Data!A1");
    // Areas on different sheets cut by a cell: `#VALUE!`, as for two plain areas (UNVERIFIED).
    model._set("A3", "=(Data!A1:A3,Other!A1:A3) Data!A2");
    // A value that is not a reference cannot be in a union: Excel-style refusal.
    model._set("A4", "=(1,Data!A1)");
    model._set("A5", "=Data!A1,Data!B1");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#REF!");
    assert_eq!(model._get_text("A2"), "#REF!");
    assert_eq!(model._get_text("A3"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "#ERROR!");
    assert_eq!(model._get_text("A5"), "#ERROR!");
}

#[test]
fn union_is_a_reference() {
    let mut model = model();
    // UNVERIFIED: Excel's ISREF of a union is not probed (references.md).
    model._set("A1", "=ISREF((Data!A1,Data!B1))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
}

#[test]
fn union_formula_text_round_trips() {
    let mut model = model();
    model._set("A1", "=(Data!E4,Data!E4)");
    model._set("A2", "=((Data!A2,Data!E8),Data!B6)");
    model._set("A3", "=(Data!C1:C12,Data!E1:E12) Data!C9");
    model._set("A4", "=SUM((Data!A1,Data!B2),1)");
    model._set("A5", "=FORMULATEXT(A3)");
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=(Data!E4,Data!E4)");
    assert_eq!(model._get_formula("A2"), "=(Data!A2,Data!E8,Data!B6)");
    assert_eq!(
        model._get_formula("A3"),
        "=(Data!C1:C12,Data!E1:E12) Data!C9"
    );
    assert_eq!(model._get_formula("A4"), "=SUM((Data!A1,Data!B2),1)");
    assert_eq!(model._get_text("A5"), "=(Data!C1:C12,Data!E1:E12) Data!C9");
}

#[test]
fn union_survives_row_insertion() {
    let mut model = model();
    model._set("A1", "=(Data!D1:D12,Data!A1:A12) Data!D11");
    model.evaluate();
    model.insert_rows(1, 1, 1).unwrap();
    model.evaluate();
    assert_eq!(
        model._get_formula("A1"),
        "=(Data!D2:D13,Data!A2:A13) Data!D12"
    );
    assert_eq!(model._get_text("A1"), "a");
}
