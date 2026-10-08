#![allow(clippy::unwrap_used, reason = "tests")]

//! Unions and multi-area intersections as aggregate arguments (SPEC-0004 step 2, T-1062).
//! Sheet1 A1 = 1, B2 = 5, C3 = 7, D4 = 9 as in the probes; expected values are Excel's from
//! probes/multiarea-* (oracle excel-graph, 2026-10-07, T-1061) unless marked otherwise. See
//! docs/excel-semantics/references.md "Multi-area references in functions".

use crate::expressions::types::CellReferenceRC;
use crate::model::Model;
use crate::reference_operators::{is_multi_area, MAX_AREAS, TOO_MANY_AREAS, TOO_MANY_AREA_CELLS};
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
        }
        | Cell::ArrayFormula {
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
fn unions_through_let_if_and_choose() {
    let mut model = model();
    results(
        &mut model,
        &[
            // probes/multiarea-let H1:H3 and multiarea-ifchoose H1:H3.
            ("H1", "=LET(x,(A1,B2),SUM(x))", "6"),
            ("H2", "=LET(x,(A1,B2),x A1)", "1"),
            ("H3", "=LET(x,(A1,B2),x)", "#VALUE!"),
            ("H4", "=IF(TRUE,(A1,B2)) A1", "1"),
            ("H5", "=SUM(IF(TRUE,(A1,B2)))", "6"),
            ("H6", "=SUM(CHOOSE(2,A1,(A1,B2)))", "6"),
            // probes/multiarea-letbind H5:H6: a nested LET shadowing the union binding.
            ("H7", "=LET(x,(A1,B2),LET(x,C3,SUM(x)))", "7"),
            ("H8", "=LET(x,(A1,B2),SUM(x,LET(x,C3,x)))", "13"),
            // A single-range binding and plain intersections are unchanged.
            ("H9", "=LET(x,A1:B2,x B2)", "5"),
            ("H10", "=SUM(A1:B2 B1:B3)", "5"),
            // A value of the union outside reference context stays `#VALUE!`.
            ("H11", "=IF(TRUE,(A1,B2))", "#VALUE!"),
            ("H12", "=CHOOSE(1,(A1,B2))", "#VALUE!"),
        ],
    );
}

#[test]
fn unions_bound_through_variables_functions_and_lambda() {
    let mut model = model();
    // probes/multiarea-letbind (oracle excel-graph, 2026-10-08, T-1064).
    results(
        &mut model,
        &[
            ("H1", "=LET(x,(A1,B2),y,x,SUM(y))", "6"),
            ("H2", "=LET(x,IF(TRUE,(A1,B2)),SUM(x))", "6"),
            ("H3", "=LET(x,CHOOSE(1,(A1,B2)),SUM(x))", "6"),
            ("H4", "=LET(x,(A1,B2),y,x,y A1)", "1"),
            ("H7", "=LAMBDA(x,SUM(x))((A1,B2))", "6"),
            ("H8", "=SUM(LET(x,(A1,B2),x))", "6"),
            ("H9", "=SUM((IF(TRUE,(A1,B2)),C3))", "13"),
            ("H10", "=SUM(IF(FALSE,1,(A1,B2)))", "6"),
            ("H11", "=SUM(CHOOSE(1.9,(A1,B2),C3))", "6"),
            ("H12", "=SUM(IF(TRUE,CHOOSE(1,(A1,B2))))", "6"),
            ("H13", "=IF(TRUE,IF(TRUE,(A1,B2))) A1", "1"),
            ("H14", "=CHOOSE(2,C3,(A1,B2)) B2", "5"),
            ("H15", "=LET(x,(A1,B2),y,x,y)", "#VALUE!"),
            ("H16", "=LET(x,(A1,B2),x) A1", "1"),
        ],
    );
}

#[test]
fn choose_hands_a_3d_reference_to_the_aggregates() {
    // probes/3d-ref-choose-let C1, C2, C5 and C6 (oracle excel-graph, 2026-10-08, T-1064).
    let mut model = new_empty_model();
    for sheet in ["Data1", "Data2", "Data3"] {
        model.add_sheet(sheet).unwrap();
    }
    for (cell, value) in [
        ("Data1!A1", "1"),
        ("Data1!A2", "'5"),
        ("Data2!A1", "10"),
        ("Data3!A1", "100"),
        ("Data3!A2", "2"),
        ("K1", "abc"),
    ] {
        model._set(cell, value);
    }
    results(
        &mut model,
        &[
            ("C1", "=SUM(CHOOSE(1,Data1:Data3!A1:A2))", "113"),
            ("C2", "=COUNT(CHOOSE(1,Data1:Data3!A1))", "3"),
            ("C5", "=SUM(IF(TRUE,CHOOSE(1,Data1:Data3!A1)))", "111"),
            ("C6", "=CHOOSE(1,Data1:Data3!A1)", "#VALUE!"),
            // probes/3d-ref-let-values C1:C6, C8:C10: a span from IF or CHOOSE bound by LET or
            // LAMBDA is `#VALUE!` to every value reader; text behind IF is skipped by MAX.
            ("D1", "=LET(x,CHOOSE(1,Data1:Data3!A1),x)", "#VALUE!"),
            ("D2", "=LET(x,IF(TRUE,Data1:Data3!A1),x)", "#VALUE!"),
            ("D3", "=LAMBDA(x,x)(CHOOSE(1,Data1:Data3!A1))", "#VALUE!"),
            ("D4", "=LET(x,CHOOSE(1,Data1:Data3!A1),x+1)", "#VALUE!"),
            (
                "D5",
                "=ROWS(LET(x,CHOOSE(1,Data1:Data3!A1:A2),x))",
                "#VALUE!",
            ),
            (
                "D6",
                "=INDEX(LET(x,IF(TRUE,Data1:Data3!A1:A2),x),2)",
                "#VALUE!",
            ),
            ("D8", "=LET(x,IF(TRUE,K1),MAX(x))", "0"),
            ("D9", "=LAMBDA(x,MAX(x))(IF(TRUE,K1))", "0"),
            ("D10", "=IF(TRUE,Data1:Data3!A1) A1", "#VALUE!"),
        ],
    );
}

#[test]
fn union_bindings_held_at_once_are_capped() {
    // x_1..x_15 hold 2^16 - 2 areas between them and each y_i a fresh union of 2^15 + 1, so the
    // sixth y_i would pass the cap of four `MAX_AREAS` held together and binds `#VALUE!` instead
    // (references.md "Union area cap").
    let mut model = model();
    let mut formula = String::from("=LET(x_1,(A1,A1)");
    for i in 2..=15 {
        formula.push_str(&format!(",x_{i},(x_{j},x_{j})", j = i - 1));
    }
    for i in 0..6 {
        formula.push_str(&format!(",y_{i},(x_15,A1)"));
    }
    let last = format!("{formula},SUM(y_4))");
    let over = format!("{formula},SUM(y_5))");
    let x15 = format!("{formula},SUM(x_15))");
    results(
        &mut model,
        &[
            ("Z1", &last, &(MAX_AREAS / 2 + 1).to_string()),
            ("Z2", &over, "#VALUE!"),
            ("Z3", &x15, &(MAX_AREAS / 2).to_string()),
        ],
    );
    assert_eq!(error_message(&model, "Z2"), TOO_MANY_AREAS);
}

#[test]
fn plain_intersection_operands_take_the_pair_path() {
    // `name A1` and `A1:B2 B1:B3` are intersected pairwise without an area list; only an
    // operand that may stand for several areas builds one.
    let mut model = model();
    model
        .new_defined_name("box", None, "Sheet1!$A$1:$B$2")
        .unwrap();
    let context = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 8,
    };
    for (formula, multi) in [
        ("box A1", false),
        ("A1:B2 B1:B3", false),
        ("(A1,B2) A1", true),
        ("IF(TRUE,A1) A1", true),
        ("CHOOSE(1,A1) A1", true),
        ("LET(x,A1,x) A1", true),
    ] {
        let node = model.parser.parse(formula, &context);
        assert_eq!(is_multi_area(&node), multi, "{formula}");
    }
}

#[test]
fn deep_if_chain_over_a_union() {
    // Each IF recurses through `evaluate_reference_arg`, so the chain is bounded by the
    // parser's depth cap. Excel's own nesting limit (64 levels) runs on the stack sizes of
    // parser/tests/test_nesting_depth.rs: 1 MiB optimized, 16 MiB unoptimized.
    let stack = if cfg!(debug_assertions) {
        16 << 20
    } else {
        1 << 20
    };
    let handle = std::thread::Builder::new()
        .stack_size(stack)
        .spawn(|| {
            let mut model = model();
            let depth = 64;
            let formula = format!(
                "=SUM({}(A1,B2){})",
                "IF(TRUE,".repeat(depth),
                ")".repeat(depth)
            );
            results(&mut model, &[("H1", &formula, "6")]);
        })
        .unwrap();
    handle.join().unwrap();
}
