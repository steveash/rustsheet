#![allow(clippy::unwrap_used, reason = "tests")]

//! Unions and multi-area intersections as aggregate arguments (SPEC-0004 step 2, T-1062).
//! Sheet1 A1 = 1, B2 = 5, C3 = 7, D4 = 9 as in the probes; expected values are Excel's from
//! probes/multiarea-* (oracle excel-graph, 2026-10-07, T-1061) unless marked otherwise. See
//! docs/excel-semantics/references.md "Multi-area references in functions".

use crate::area_budget::TOO_MANY_AREA_CELLS;
use crate::expressions::types::CellReferenceRC;
use crate::model::Model;
use crate::reference_operators::{MAX_AREAS, TOO_MANY_AREAS};
use crate::test::util::{balanced_union, new_empty_model};

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

/// LARGE, SMALL, RANK and FREQUENCY walk the areas as written (T-1068; probes/multiarea-rank
/// H1:H3, probes/multiarea-cond H1).
#[test]
fn order_statistics_take_a_union() {
    let mut model = model();
    model._set("E1", "=1/0");
    results(
        &mut model,
        &[
            ("H1", "=LARGE((A1,B2,C3),2)", "5"),
            ("H2", "=SMALL((A1,B2,C3),2)", "5"),
            ("H3", "=RANK(5,(A1,B2,C3))", "2"),
            ("H4", "=RANK.AVG(5,(A1,B2,C3),1)", "2"),
            ("H5", "=SUM(FREQUENCY((A1,B2),3))", "2"),
            // The overlap counts twice, as for SUM; k past the count is #NUM!.
            ("H6", "=LARGE((A1:B2,B2),2)", "5"),
            ("H7", "=LARGE((A1,B2),3)", "#NUM!"),
            // An error cell in an area is the function's error, as in a plain range.
            ("H8", "=LARGE((A1,E1),1)", "#DIV/0!"),
            // SMALL resolves its argument like LARGE: a cell, a typed scalar and an error cell.
            ("H9", "=SMALL(B2,1)", "5"),
            ("H10", "=SMALL(5,1)", "5"),
            ("H11", "=SMALL((A1,E1),1)", "#DIV/0!"),
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
    assert_eq!(model._get_error_message("Z2"), TOO_MANY_AREAS);
}

/// `count` rows of column A holding 1, from row 1.
fn model_with_ones_in_column_a(count: i32) -> Model {
    let mut model = new_empty_model();
    for row in 1..=count {
        model._set(&format!("A{row}"), "1");
    }
    model
}

#[test]
fn whole_columns_over_sparse_data_cost_what_the_data_costs() {
    // T-1497: a union area is walked over its existing cells, so MAX_AREAS copies of A:A over
    // one cell in row 1,000 are a step or two each, well inside the walk budget.
    let mut model = model();
    model._set("A1000", "1");
    model._set("Z1", &format!("=SUM({})", balanced_union("A:A", MAX_AREAS)));
    model._set("Z2", &format!("=SUM({})", balanced_union("A:A", 16)));
    model.evaluate();
    assert_eq!(model._get_text("Z1"), (2 * MAX_AREAS).to_string());
    assert_eq!(model._get_text("Z2"), "32");
}

#[test]
fn a_union_walked_by_a_lambda_costs_what_its_data_costs() {
    // probes/union-walk-repeated-in-lambda (excel-graph, 2026-10-13, T-1462), Big!E1:E4 and
    // Calc!E1, E6, E8: data in the last row, or 10,000 used rows with three cells, walked again
    // and again; Excel computes them all.
    let mut model = new_empty_model();
    model.add_sheet("Big").unwrap();
    for (cell, value) in [
        ("B1", "5"),
        ("B10000", "1"),
        ("C10000", "2"),
        ("Big!B1048576", "1"),
        ("Big!C1048576", "2"),
    ] {
        model._set(cell, value);
    }
    let nine = ["(B:B,C:C)"; 9].join(",");
    results(
        &mut model,
        &[
            (
                "E1",
                "=SUM(BYROW(A1:A1000,LAMBDA(r,SUM((B:B,C:C)))))",
                "8000",
            ),
            (
                "E6",
                "=SUM(MAP(A1:A10000,LAMBDA(r,SUM((B:B,C:C)))))",
                "80000",
            ),
            (
                "E8",
                "=SUM(MAP(A1:A1000,LAMBDA(r,SUM((B1:B10000,C1:C10000)))))",
                "8000",
            ),
            (
                "E11",
                "=SUM(BYROW(A1:A1000,LAMBDA(r,SUBTOTAL(9,(B:B,C:C)))))",
                "8000",
            ),
            (
                "Big!E2",
                "=SUM(BYROW(A1:A100,LAMBDA(r,SUM((B:B,C:C)))))",
                "300",
            ),
            ("Big!E4", &format!("=SUM({nine})"), "27"),
        ],
    );
}

#[test]
fn a_union_walk_costs_its_own_columns_not_the_rows_others_fill() {
    // T-1502: probes/union-walk-repeated-in-lambda Calc!E6 (excel-graph, 2026-10-13, T-1462) is
    // 80000 in Excel; the value is inferred for this sheet, which adds 9,000 rows in column G that
    // the sum does not read. With them a walk of B:B used to find every one
    // of them and pay a step each, 2*10^8 steps for the MAP; through the column index each walk
    // visits only the cells of B and C.
    let mut model = new_empty_model();
    for (cell, value) in [("B1", "5"), ("B10000", "1"), ("C10000", "2")] {
        model._set(cell, value);
    }
    for row in 1..=9_000 {
        model._set(&format!("G{row}"), "1");
    }
    results(
        &mut model,
        &[(
            "E6",
            "=SUM(MAP(A1:A10000,LAMBDA(r,SUM((B:B,C:C)))))",
            "80000",
        )],
    );
}

#[test]
fn only_a_union_area_over_more_than_a_chunk_of_rows_builds_the_column_index() {
    // A short area gains nothing from the index (its row walk is a chunk at most), so it leaves
    // the sheet without one, and its writes keep paying nothing for it.
    let mut model = new_empty_model();
    for row in 1..=3_000 {
        model._set(&format!("A{row}"), "1");
    }
    model._set("Z1", "=SUM((A1:A1000,A5))");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "1001");
    let indexed = |model: &Model| {
        model
            .workbook
            .worksheet(0)
            .unwrap()
            .sheet_data
            .has_column_index()
    };
    assert!(!indexed(&model));
    model._set("Z2", "=SUM((A:A,A5))");
    model.evaluate();
    assert_eq!(model._get_text("Z2"), "3001");
    assert!(indexed(&model));
}

#[test]
fn an_indexed_union_walk_meets_cells_in_row_major_order() {
    // The first error in row-major order is the result, as in the dense walk
    // (references.md "Union area cap"): B1 comes before A2 although A is walked first. A2000
    // makes the whole columns span more than a chunk of rows, so they take the index walk.
    let mut model = new_empty_model();
    for (cell, value) in [("A2", "=1/0"), ("B1", "=NA()"), ("A3", "4"), ("A2000", "1")] {
        model._set(cell, value);
    }
    for column in [
        "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q",
    ] {
        model._set(&format!("{column}5"), "1");
    }
    results(
        &mut model,
        &[
            ("Z1", "=SUM((A:B,A3))", "#N/A"),
            ("Z2", "=SUM((A3:B3,A3))", "8"),
            // seventeen columns with cells: the row walk, same order
            ("Z3", "=SUM((A:Q,A3))", "#N/A"),
            ("Z4", "=SUM((A3:Q5,A3))", "23"),
        ],
    );
}

#[test]
fn a_let_bound_union_shares_the_walk_budget_across_arguments() {
    // DEVIATION (references.md "Union area cap"): the formula, not each argument, may take
    // MAX_AREA_WALK steps of union walks. Over 65,536 filled rows one walk of A:A is a step for
    // the area, one to find its column in the column index, 1,025 reads of 64 rows from it and
    // 65,536 cells (66,563 steps), so sixteen copies are 1,065,008 steps: fifteen arguments of
    // them (15,975,120) fit in 2^24 and a sixteenth (17,040,128) is past it.
    let mut model = model_with_ones_in_column_a(1 << 16);
    let union = balanced_union("A:A", 16);
    let within = vec!["x"; 15].join(",");
    model._set("Z1", &format!("=LET(x,{union},SUM({within}))"));
    let past = vec!["x"; 16].join(",");
    model._set("Z2", &format!("=LET(x,{union},SUM({past}))"));
    model.evaluate();
    assert_eq!(model._get_text("Z1"), (15 * 16 * 65536).to_string());
    assert_eq!(model._get_text("Z2"), "#VALUE!");
    assert_eq!(model._get_error_message("Z2"), TOO_MANY_AREA_CELLS);
}

#[test]
fn a_dense_union_repeated_by_a_lambda_is_bounded() {
    // DEVIATION (directive 5): the walk budget is per formula run, so a LAMBDA repeating a
    // walk of dense data stops at MAX_AREA_WALK steps: 300 walks of (A:A,A:A) over 65,536
    // filled rows are 39,937,800 steps, past it.
    let mut model = model_with_ones_in_column_a(1 << 16);
    model._set("Z1", "=SUM(MAP(A1:A300,LAMBDA(r,SUM((A:A,A:A)))))");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "#VALUE!");
}

#[test]
fn aggregate_of_a_union_obeys_the_walk_budget() {
    // Same budget as SUM (`a_let_bound_union_shares_the_walk_budget_across_arguments`) for
    // AGGREGATE's reference form, which walks area by area: 256 copies of A:A over 65,536
    // filled rows are past it, sixteen are not.
    let mut model = model_with_ones_in_column_a(1 << 16);
    model._set(
        "Z1",
        &format!("=AGGREGATE(9,4,{})", balanced_union("A:A", 256)),
    );
    model._set(
        "Z2",
        &format!("=AGGREGATE(9,4,{})", balanced_union("A:A", 16)),
    );
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "#VALUE!");
    assert_eq!(model._get_error_message("Z1"), TOO_MANY_AREA_CELLS);
    assert_eq!(model._get_text("Z2"), (16 * 65536).to_string());
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

#[test]
fn subtotal_of_a_union() {
    // probes/multiarea-subtotal H1, H2 (oracle excel-graph, 2026-10-07): both 6 for (A1,B2).
    // H3 is not probed: it follows SUM, where an area is counted as written (references.md).
    let mut model = model();
    results(
        &mut model,
        &[
            ("H1", "=SUBTOTAL(9,(A1,B2))", "6"),
            ("H2", "=SUBTOTAL(109,(A1,B2))", "6"),
            ("H3", "=SUBTOTAL(9,(A1:B2,B2))", "11"),
        ],
    );
}

#[test]
fn subtotal_of_a_union_obeys_the_walk_budget() {
    // Same budget as SUM (`a_let_bound_union_shares_the_walk_budget_across_arguments`): 256
    // copies of A:A over 65,536 filled rows are past it, so SUBTOTAL answers `#VALUE!`; a
    // handful of whole columns is walked.
    let mut model = model_with_ones_in_column_a(1 << 16);
    model._set(
        "Z1",
        &format!("=SUBTOTAL(9,{})", balanced_union("A:A", 256)),
    );
    model._set("Z2", &format!("=SUBTOTAL(9,{})", balanced_union("A:A", 16)));
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "#VALUE!");
    assert_eq!(model._get_error_message("Z1"), TOO_MANY_AREA_CELLS);
    assert_eq!(model._get_text("Z2"), (16 * 65536).to_string());
}

#[test]
fn areas_counts_the_areas_of_a_reference() {
    // SPEC-0004 probe rows 6 and 20 (probes/multiarea-areas-*).
    results(
        &mut model(),
        &[
            ("H1", "=AREAS((A1,B2))", "2"),
            ("H2", "=AREAS(((A1,B2),C3))", "3"),
            ("H3", "=AREAS((A1:B2,A1:B2))", "2"),
            ("H4", "=AREAS(A1:B2)", "1"),
            ("H5", "=AREAS(Sheet1:Data!A1)", "#VALUE!"),
        ],
    );
}

#[test]
fn index_area_num_selects_an_area() {
    // SPEC-0004 probe rows 7 and 8.
    results(
        &mut model(),
        &[
            ("H1", "=INDEX((A1:B2,C3:D4),1,1,2)", "7"),
            ("H2", "=INDEX((A1:B2,C3:D4),1,1,3)", "#REF!"),
            ("H3", "=INDEX((A1:B2,C3:D4),1,1,0)", "#VALUE!"),
            ("H4", "=INDEX((A1:B2,C3:D4),1,1)", "1"),
            ("H5", "=SUM(INDEX((A1:B2,C3:D4),0,0,2))", "16"),
            ("H6", "=INDEX((A1:B2,C3:D4),1,1,1E+300)", "#REF!"),
            ("H7", "=INDEX(A1:B2,1,1,2)", "#REF!"),
            ("H8", "=INDEX(Sheet1:Data!A1,1,1,1)", "#VALUE!"),
        ],
    );
}

#[test]
fn three_d_operand_beside_and_inside_a_union() {
    // SPEC-0004 probe row 13 (probes/multiarea-3d-ref, excel-graph 2026-10-07): a 3-D
    // argument beside a union adds up (101 + 1 + 5); a 3-D operand inside a union is #NAME?
    // (H2). H3 (the 3-D operand on the right) and H4 (the bare union) are UNVERIFIED engine choices.
    // Sheet2 sits right after Sheet1, as in the probe, so the span holds just those two.
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    for (cell, value) in [
        ("A1", "1"),
        ("B2", "5"),
        ("Sheet2!A1", "100"),
        ("Sheet2!B2", "200"),
    ] {
        model._set(cell, value);
    }
    results(
        &mut model,
        &[
            ("H1", "=SUM(Sheet1:Sheet2!A1,(A1,B2))", "107"),
            ("H2", "=SUM((Sheet1:Sheet2!A1,B2))", "#NAME?"),
            ("H3", "=SUM((A1,Sheet1:Sheet2!B2))", "#NAME?"),
            ("H4", "=(Sheet1:Sheet2!A1,B2)", "#NAME?"),
        ],
    );
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
fn choose_and_let_hand_a_3d_reference_to_the_aggregates() {
    // probes/3d-ref-choose-let (oracle excel-graph, 2026-10-08, T-1064), 3d-ref-let-values and
    // 3d-ref-let-bound-span (T-1158).
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
            // probes/3d-ref-let-values C7 and 3d-ref-choose-let C3:C4: an aggregate reads a span
            // bound by LET, written or from CHOOSE (T-1158).
            ("D7", "=LET(x,CHOOSE(1,Data1:Data3!A1),SUM(x))", "111"),
            ("E3", "=LET(x,Data1:Data3!A1,SUM(x))", "111"),
            ("E4", "=SUM(LET(x,Data1:Data3!A1,x))", "111"),
            // probes/3d-ref-let-bound-span (excel-graph, 2026-10-09, T-1158): through another
            // variable or a LAMBDA too, while value readers, INDEX, ROWS and ISREF give what the
            // written span gives.
            ("F1", "=LET(x,Data1:Data3!A1,y,x,SUM(y))", "111"),
            ("F2", "=LAMBDA(x,SUM(x))(Data1:Data3!A1)", "111"),
            ("F3", "=LET(x,Data1:Data3!A1,COUNT(x))", "3"),
            ("F4", "=LET(x,Data1:Data3!A1,INDEX(x,1))", "#VALUE!"),
            ("F5", "=INDEX(Data1:Data3!A1,1)", "#VALUE!"),
            ("F6", "=LET(x,Data1:Data3!A1:A2,SUM(x))", "113"),
            ("F7", "=LET(x,Data1:Data3!A1,ROWS(x))", "#VALUE!"),
            ("F8", "=ROWS(Data1:Data3!A1)", "#VALUE!"),
            ("F9", "=LET(x,Data1:Data3!A1,ISREF(x))", "FALSE"),
            ("F10", "=ISREF(Data1:Data3!A1)", "FALSE"),
            ("F11", "=LET(x,Data1:Data3!A1,MAX(x)+0)", "100"),
            ("F12", "=LAMBDA(x,x)(Data1:Data3!A1)", "#VALUE!"),
            ("F13", "=LET(x,Data1:Data3!A1,SUM(x,x))", "222"),
            ("F14", "=LET(x,Data1:Data3!A1,SUM(IF(TRUE,x)))", "111"),
            ("F15", "=LET(x,Data1:Data3!A1,x+1)", "#VALUE!"),
            // probes/3d-ref-let-bound-span-readers (excel-graph, 2026-10-09): text readers walk
            // it like the written span, single-sheet readers refuse it.
            ("G1", "=LET(x,Data1:Data3!A1,CONCAT(x))", "110100"),
            (
                "G3",
                "=LET(x,Data1:Data3!A1,TEXTJOIN(\",\",TRUE,x))",
                "1,10,100",
            ),
            ("G5", "=LET(x,Data1:Data3!A1,AGGREGATE(9,0,x))", "#VALUE!"),
            ("G7", "=LET(x,Data1:Data3!A1,AREAS(x))", "#VALUE!"),
            ("G9", "=LET(x,Data1:Data3!A1,SUM((x,Data1!B1)))", "#VALUE!"),
            ("G10", "=SUM((Data1:Data3!A1,Data1!B1))", "#NAME?"),
            ("G11", "=LET(x,Data1:Data3!A1,COUNTBLANK(x))", "#VALUE!"),
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
    assert_eq!(model._get_error_message("Z2"), TOO_MANY_AREAS);
}

#[test]
fn plain_intersection_operands_take_the_pair_path() {
    // `name A1` and `A1:B2 B1:B3` are intersected pairwise without an area list; only an
    // operand that may stand for several areas builds one.
    let mut model = model();
    model
        .new_defined_name("box", None, "Sheet1!$A$1:$B$2")
        .unwrap();
    model.define_name("arr", None, "{1,2,3}").unwrap();
    model
        .define_name("mu", None, "Sheet1!$A$1,Sheet1!$B$2")
        .unwrap();
    model.define_name("alias", None, "mu").unwrap();
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
        // A formula-bodied name is multi-area only when its body is (T-1065).
        ("arr", false),
        ("mu A1", true),
        ("alias A1", true),
    ] {
        let node = model.parser.parse(formula, &context);
        assert_eq!(model.is_multi_area(&node), multi, "{formula}");
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

#[test]
fn union_bodied_names_stand_for_their_areas() {
    // probes/union-defined-name H1:H5; a name reaches AGGREGATE's array form, AREAS, INDEX and
    // an intersection, and another name or a LET variable passes its areas on.
    let mut model = model();
    model
        .define_name("mu", None, "Sheet1!$A$1,Sheet1!$B$2")
        .unwrap();
    model.define_name("alias", None, "mu").unwrap();
    model
        .define_name("xs", None, "Data!$A$1,Sheet1!$A$1")
        .unwrap();
    model.define_name("arr", None, "{1,2,3}").unwrap();
    model.define_name("arr_alias", None, "arr").unwrap();
    results(
        &mut model,
        &[
            ("H1", "=SUM(mu)", "6"),
            ("H2", "=mu", "#VALUE!"),
            ("H3", "=ISREF(mu)", "TRUE"),
            ("H4", "=mu A1", "1"),
            ("H5", "=COUNT(mu)", "2"),
            ("H6", "=AGGREGATE(14,0,mu,1)", "5"),
            ("H7", "=LET(x,mu,AGGREGATE(15,0,x,1))", "1"),
            ("H8", "=AREAS(mu)", "2"),
            ("H9", "=INDEX(mu,1,1,2)", "5"),
            ("H10", "=SUM(alias)", "6"),
            ("H11", "=SUM(xs)", "#VALUE!"),
            ("H12", "=AGGREGATE(14,0,IF(TRUE,(A1,B2)),2)", "1"),
            // A name whose body is not a union keeps its path (probes/union-defined-name-uses H9).
            ("H13", "=ROWS(FREQUENCY(arr,2))", "2"),
            // Behind another name, a LET variable or IF, an array is counted too (T-1196).
            ("H14", "=SUM(FREQUENCY(arr_alias,2))", "3"),
            ("H15", "=LET(x,{1,2,3},SUM(FREQUENCY(x,2)))", "3"),
            ("H16", "=SUM(FREQUENCY(IF(TRUE,{1,2,3}),2))", "3"),
        ],
    );
}

#[test]
fn union_bodied_name_over_the_area_cap() {
    // DEVIATION (references.md "Union area cap"): a name body is capped like a written union,
    // at each use: MAX_AREAS areas are summed, one more is `#VALUE!`.
    let mut model = model();
    let at_cap = balanced_union("Sheet1!$A$1", MAX_AREAS);
    let over_cap = balanced_union("Sheet1!$A$1", MAX_AREAS + 1);
    model.define_name("cap", None, &at_cap).unwrap();
    model.define_name("over", None, &over_cap).unwrap();
    results(
        &mut model,
        &[
            ("Z1", "=SUM(cap)", &MAX_AREAS.to_string()),
            ("Z2", "=SUM(over)", "#VALUE!"),
        ],
    );
    assert_eq!(model._get_error_message("Z2"), TOO_MANY_AREAS);
}

#[test]
fn self_referential_union_names_are_bounded() {
    // A name that reaches itself through a union runs out of name depth (`#NAME?`, as Excel's
    // self-reference is; probes/defined-name-formula-bodies H11) instead of recursing.
    let mut model = model();
    model.define_name("loop", None, "Sheet1!$A$1,loop").unwrap();
    model.define_name("ping", None, "Sheet1!$A$1,pong").unwrap();
    model.define_name("pong", None, "Sheet1!$B$2,ping").unwrap();
    results(
        &mut model,
        &[
            ("Z1", "=SUM(loop)", "#NAME?"),
            ("Z2", "=SUM(ping)", "#NAME?"),
            ("Z3", "=loop A1", "#NAME?"),
        ],
    );
}

#[test]
fn a_spill_into_a_union_area_is_read_after_its_anchor_runs() {
    // T-1497: a union area is recorded as seen empty in one piece. A spill written into it
    // after the union was read must restart the reader, not be missed or reported circular,
    // both when the spill appears and when it grows.
    let mut model = new_empty_model();
    model._set("A1", "=SUM((D:D,E:E))");
    model._set("B1", "=COUNT((D1:D10,E:E))");
    model._set("D1", "=SEQUENCE(C1)");
    model._set("C1", "3");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "6");
    assert_eq!(model._get_text("B1"), "3");
    model._set("C1", "5");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "15");
    assert_eq!(model._get_text("B1"), "5");
}
