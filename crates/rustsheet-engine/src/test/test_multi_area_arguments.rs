#![allow(clippy::unwrap_used, reason = "tests")]

//! Unions and multi-area intersections as aggregate arguments (SPEC-0004 step 2, T-1062).
//! Sheet1 A1 = 1, B2 = 5, C3 = 7, D4 = 9 as in the probes; expected values are Excel's from
//! probes/multiarea-* (oracle excel-graph, 2026-10-07, T-1061) unless marked otherwise. See
//! docs/excel-semantics/references.md "Multi-area references in functions".

use crate::model::Model;
use crate::reference_operators::{MAX_AREAS, TOO_MANY_AREAS, TOO_MANY_AREA_CELLS};
use crate::test::util::new_empty_model;
use crate::types::{Cell, FormulaValue};

fn model() -> Model {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    for (cell, value) in [
        ("A1", "1"),
        ("B2", "5"),
        ("C3", "7"),
        ("D4", "9"),
        ("Data!A1", "11"),
    ] {
        model._set(cell, value);
    }
    model
}

fn results(model: &mut Model, cases: &[(&str, &str, &str)]) {
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cases {
        assert_eq!(model._get_text(cell), *expected, "{cell}: {formula}");
    }
}

fn error_message(model: &Model, cell: &str) -> String {
    match model._get_cell(cell) {
        Cell::CellFormula {
            v: FormulaValue::Error { m, .. },
            ..
        } => m.clone(),
        other => format!("not an error: {other:?}"),
    }
}

/// A union of `leaves` copies of `leaf`, nested as a balanced tree to stay under the parser's
/// depth cap.
fn balanced_union(leaf: &str, leaves: usize) -> String {
    if leaves <= 1 {
        return leaf.to_string();
    }
    let half = leaves / 2;
    format!(
        "({},{})",
        balanced_union(leaf, half),
        balanced_union(leaf, leaves - half)
    )
}

#[test]
fn aggregates_walk_each_area_as_written() {
    let mut model = model();
    results(
        &mut model,
        &[
            // probes/multiarea-agg H1:H9: the overlap is counted twice.
            ("H1", "=SUM((A1,B2))", "6"),
            ("H2", "=SUM((A1,B2),C3)", "13"),
            ("H3", "=SUM((A1:B2,B2))", "11"),
            ("H4", "=COUNT((A1,B2))", "2"),
            ("H5", "=COUNTA((A1,B2))", "2"),
            ("H6", "=AVERAGE((A1,B2))", "3"),
            ("H7", "=MIN((A1,B2))", "1"),
            ("H8", "=MAX((A1,B2))", "5"),
            ("H9", "=PRODUCT((A1,B2))", "5"),
            // probes/multiarea-intersect H1: an intersection that leaves two areas.
            ("H10", "=SUM((A1:A3,B1:B3) A1:B2)", "6"),
            // probes/multiarea-rank H5 (STDEV.S walks through the shared aggregate path).
            ("H11", "=STDEV.S((A1,B2,C3))", "3.055050463"),
            // An aggregate of a union is no longer an error, so IFERROR keeps it.
            ("H12", "=IFERROR(SUM((A1,B2)),0)", "6"),
            // probes/multiarea-iserror H1:H3: in a value context a union stays `#VALUE!`.
            ("H13", "=ISERROR((A1,B2))", "TRUE"),
            ("H14", "=IFERROR((A1,B2),\"x\")", "x"),
            ("H15", "=ISNUMBER((A1,B2))", "FALSE"),
        ],
    );
}

#[test]
fn union_argument_errors() {
    let mut model = model();
    model._set("E1", "=1/0");
    results(
        &mut model,
        &[
            // An operand's own error comes first (probes/union-accepted H9 for the reference).
            ("H1", "=SUM((A1,#REF!))", "#REF!"),
            // An error cell in an area is the aggregate's error, as in a plain range; COUNT
            // skips it and COUNTA counts it (probes/multiarea-more H3:H5).
            ("H2", "=SUM((A1,E1))", "#DIV/0!"),
            ("H3", "=COUNT((A1,E1))", "1"),
            ("H4", "=COUNTA((A1,E1))", "2"),
            // Areas on two sheets: probes/multiarea-xsheet H1.
            ("H5", "=SUM((Data!A1,A1))", "#VALUE!"),
            // ISREF agrees (probes/multiarea-more H1).
            ("H6", "=ISREF((Data!A1,A1))", "FALSE"),
        ],
    );
}

#[test]
fn whole_column_union() {
    let mut model = model();
    model._set("C1", "2");
    results(&mut model, &[("H1", "=SUM((A:A,C:C))", "10")]);
}

#[test]
fn union_at_and_over_the_area_cap() {
    // DEVIATION (references.md "Union area cap"): MAX_AREAS areas are summed, one more is
    // `#VALUE!` before any area is walked.
    let mut model = model();
    model._set("Z1", &format!("=SUM({})", balanced_union("A1", MAX_AREAS)));
    model._set(
        "Z2",
        &format!("=SUM({})", balanced_union("A1", MAX_AREAS + 1)),
    );
    model.evaluate();
    assert_eq!(model._get_text("Z1"), MAX_AREAS.to_string());
    assert_eq!(model._get_text("Z2"), "#VALUE!");
    assert_eq!(error_message(&model, "Z2"), TOO_MANY_AREAS);
}

#[test]
fn whole_columns_over_the_cell_bound_are_not_walked() {
    // DEVIATION (references.md "Union area cap"): MAX_AREAS copies of A:A over 1,000 used
    // rows are 65,536,000 cells, past MAX_AREA_CELLS, so the sum is `#VALUE!` from the bound
    // alone. A walk would read every cell; the message shows the bound answered first.
    let mut model = model();
    model._set("A1000", "1");
    model._set("Z1", &format!("=SUM({})", balanced_union("A:A", MAX_AREAS)));
    // A handful of whole columns stays under the bound and is walked.
    model._set("Z2", &format!("=SUM({})", balanced_union("A:A", 16)));
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "#VALUE!");
    assert_eq!(error_message(&model, "Z1"), TOO_MANY_AREA_CELLS);
    assert_eq!(model._get_text("Z2"), "32");
}

#[test]
fn countblank_rows_columns_of_a_union() {
    // probes/multiarea-count H1:H3 (oracle excel-graph, 2026-10-07): #VALUE!, #REF!, #REF!.
    let mut model = model();
    results(
        &mut model,
        &[
            ("H1", "=COUNTBLANK((A1,B2))", "#VALUE!"),
            ("H2", "=ROWS((A1,B2))", "#REF!"),
            ("H3", "=COLUMNS((A1,B2))", "#REF!"),
        ],
    );
}
