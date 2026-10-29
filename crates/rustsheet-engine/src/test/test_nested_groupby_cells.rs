#![allow(clippy::unwrap_used)]

//! GROUPBY and PIVOTBY cells that hold their function's array result (SPEC-0018, T-2593). Excel:
//! probes/t2592-nested-groupby-cells, probes/t2360-array-lambda-holidays P
//! (docs/excel-semantics/arrays.md "A LAMBDA array result is shown as `#VALUE!`").

use crate::test::util::new_empty_model;
use crate::Model;

/// The D sheet of probes/t2592-nested-groupby-cells, on Sheet1: keys a, a, b, b under `k`, values
/// 1-4 under `u` and 10-40 under `w`. `GROUPBY(A1:A5,C1:D5,LAMBDA(v,v))` is 3x3: a, b and the
/// Total, a's cells `{1;2}` and `{10;20}`, b's `{3;4}` and `{30;40}`, the Total's 4x1 ones.
fn groups() -> Model {
    let mut model = new_empty_model();
    let rows = [
        ("k", "u", "w"),
        ("a", "1", "10"),
        ("a", "2", "20"),
        ("b", "3", "30"),
        ("b", "4", "40"),
    ];
    for (i, (key, u, w)) in rows.iter().enumerate() {
        model._set(&format!("A{}", i + 1), key);
        model._set(&format!("C{}", i + 1), u);
        model._set(&format!("D{}", i + 1), w);
    }
    model
}

const NESTED: &str = "GROUPBY(A1:A5,C1:D5,LAMBDA(v,v))";

#[test]
fn index_returns_the_nested_array() {
    // A25: `64|2x1|3`; P!B21: the Total `64|4x1`; a key cell stays a scalar (P!A3 `2|a`).
    let mut model = groups();
    model._set("F1", &format!("=TYPE(INDEX({NESTED},2,2))"));
    model._set("F2", &format!("=ROWS(INDEX({NESTED},2,2))"));
    model._set("F3", &format!("=INDEX(INDEX({NESTED},2,2),1,1)"));
    model._set("F4", &format!("=ROWS(INDEX({NESTED},3,3))"));
    model._set("F5", &format!("=TYPE(INDEX({NESTED},1,1))"));
    model._set(
        "F6",
        &format!("=LET(x,INDEX({NESTED},2,2)+0,TYPE(x)&ROWS(x))"),
    );
    model.evaluate();
    assert_eq!(model._get_text("F1"), "64");
    assert_eq!(model._get_text("F2"), "2");
    assert_eq!(model._get_text("F3"), "3");
    assert_eq!(model._get_text("F4"), "4");
    assert_eq!(model._get_text("F5"), "2");
    // A16: an operator on the cell INDEX took out works on its array.
    assert_eq!(model._get_text("F6"), "642");
}

#[test]
fn a_spilled_nested_cell_shows_value_error() {
    // Rule 2: each nested cell shows `#VALUE!`; `INDEX(ZZ1#,2,2)` reads the stored error (A20).
    let mut model = groups();
    model._set("F1", &format!("={NESTED}"));
    model._set("J1", "=ERROR.TYPE(INDEX(F1#,2,2))");
    model.evaluate();
    assert_eq!(model._get_text("F1"), "a");
    assert_eq!(model._get_text("G1"), "#VALUE!");
    assert_eq!(model._get_text("H3"), "#VALUE!");
    assert_eq!(model._get_text("J1"), "3");
}

#[test]
fn computed_and_one_cell_results_are_nested() {
    // A33 (`LAMBDA(v,{7})` `64|1x1|7`), A22 (two parameters, `64|2x1|3`), A21 (a one-cell
    // reference is read by INDEX as its value, `1|10`).
    let mut model = groups();
    model._set("F1", "=TYPE(INDEX(GROUPBY(A1:A5,C1:C5,LAMBDA(v,{7})),2,2))");
    model._set(
        "F2",
        "=INDEX(INDEX(GROUPBY(A1:A5,C1:C5,LAMBDA(v,{7})),2,2),1,1)",
    );
    model._set("F3", "=ROWS(INDEX(GROUPBY(A1:A5,C1:D5,LAMBDA(x,y,x)),2,2))");
    model._set(
        "F4",
        "=LET(x,INDEX(GROUPBY(A1:A5,C1:D5,LAMBDA(x,D2)),2,2),TYPE(x)&\"|\"&x)",
    );
    model.evaluate();
    assert_eq!(model._get_text("F1"), "64");
    assert_eq!(model._get_text("F2"), "7");
    assert_eq!(model._get_text("F3"), "2");
    assert_eq!(model._get_text("F4"), "1|10");
}

#[test]
fn nested_cells_nest_again() {
    // Rule 8 (A23, A24): the inner GROUPBY's 3x3 result, whose own cells are nested.
    let mut model = groups();
    let outer = "GROUPBY(A1:A5,C1:D5,LAMBDA(v,GROUPBY(A1:A5,C1:D5,LAMBDA(w,w))))";
    model._set("F1", &format!("=ROWS(INDEX({outer},2,2))"));
    model._set("F2", &format!("=INDEX(INDEX({outer},2,2),1,1)"));
    model._set("F3", &format!("=TYPE(INDEX(INDEX({outer},2,2),2,2))"));
    model._set("F4", &format!("=ROWS(INDEX(INDEX({outer},2,2),2,2))"));
    model.evaluate();
    assert_eq!(model._get_text("F1"), "3");
    assert_eq!(model._get_text("F2"), "a");
    assert_eq!(model._get_text("F3"), "64");
    assert_eq!(model._get_text("F4"), "2");
}

#[test]
fn element_wise_lifts_read_the_top_left() {
    // Rule 4 (P!C1, P!C33): only a top-left error counts. `{1;#N/A}`, `{3;4}` and the Total
    // `{1;#N/A;3;4}` hold no top-left error; `{#N/A;2}` and the Total `{#N/A;2;3;4}` do.
    let mut model = groups();
    model._set(
        "F1",
        "=SUM(--ISERROR(GROUPBY(A1:A5,C1:C5,LAMBDA(v,IF(v=2,NA(),v)))))",
    );
    model._set(
        "F2",
        "=SUM(--ISERROR(GROUPBY(A1:A5,C1:C5,LAMBDA(v,IF(v=1,NA(),v)))))",
    );
    model._set(
        "F3",
        "=SUM(--ISERROR(PIVOTBY(A1:A5,D1:D5,C1:C5,LAMBDA(v,v))))",
    );
    model.evaluate();
    assert_eq!(model._get_text("F1"), "0");
    assert_eq!(model._get_text("F2"), "2");
    // t2311 P!C12: PIVOTBY's nested cells are no errors either.
    assert_eq!(model._get_text("F3"), "0");
}

#[test]
fn kept_cells_are_capped_per_call() {
    // Each call keeps at most 1,048,576 nested cells, inner levels included: past it the whole
    // result is `#ERROR!` (DEVIATION, arrays.md "Nested GROUPBY and PIVOTBY cells: bounds").
    let mut model = groups();
    model._set("F1", "=GROUPBY(A1:A5,C1:C5,LAMBDA(v,SEQUENCE(1048576)))");
    model._set(
        "G1",
        "=ROWS(INDEX(GROUPBY(A1:A5,C1:C5,LAMBDA(w,SEQUENCE(300000))),3,2))",
    );
    model._set(
        "H1",
        "=GROUPBY(A1:A5,C1:C5,LAMBDA(v,GROUPBY(A1:A5,C1:C5,LAMBDA(w,SEQUENCE(300000)))))",
    );
    model.evaluate();
    assert_eq!(model._get_text("F1"), "#ERROR!");
    // Three 300,000-cell cells fit one call; two of those results kept by an outer call do not.
    assert_eq!(model._get_text("G1"), "300000");
    assert_eq!(model._get_text("H1"), "#ERROR!");
}

#[test]
fn reading_copies_of_a_nested_cell_is_bounded() {
    // CHOOSEROWS copies the nested cell 200 times for free; reading each copy's 1,048,512 cells
    // (64 rows of 16,383) through INDEX passes the walk budget (2^27 cells) at the 129th read:
    // `#ERROR!`, in time bounded by the budget (DEVIATION, arrays.md "Nested GROUPBY and PIVOTBY
    // cells: bounds").
    let mut model = groups();
    model._set(
        "F1",
        "=LET(r,GROUPBY(A2,C2,LAMBDA(v,SEQUENCE(64,16383)),0,0),s,CHOOSEROWS(r,SEQUENCE(200)^0),\
         REDUCE(0,SEQUENCE(200),LAMBDA(a,i,a+ROWS(INDEX(s,i,2)))))",
    );
    // Within the budget, every copy reads in full.
    model._set(
        "G1",
        "=LET(r,GROUPBY(A2,C2,LAMBDA(v,SEQUENCE(1000)),0,0),s,CHOOSEROWS(r,SEQUENCE(200)^0),\
         REDUCE(0,SEQUENCE(200),LAMBDA(a,i,a+ROWS(INDEX(s,i,2)))))",
    );
    model.evaluate();
    assert_eq!(model._get_text("F1"), "#ERROR!");
    assert_eq!(model._get_text("G1"), "200000");
}

#[test]
fn map_and_reduce_hand_the_lambda_each_nested_cell() {
    // probes/t2592-nested-groupby-cells A100:A105, A15 (T-2594): ROWS 1 for a key, 2 for group
    // b's cell, 4 for the Total's; SUM 7 for `{3;4}`; a step returning `x` is `#CALC!`.
    let mut model = groups();
    model._set(
        "F1",
        &format!("=INDEX(MAP({NESTED},LAMBDA(x,ROWS(x))),1,1)"),
    );
    model._set(
        "F2",
        &format!("=INDEX(MAP({NESTED},LAMBDA(x,ROWS(x))),2,2)"),
    );
    model._set(
        "F3",
        &format!("=INDEX(MAP({NESTED},LAMBDA(x,ROWS(x))),3,3)"),
    );
    model._set(
        "F4",
        &format!("=INDEX(MAP({NESTED},LAMBDA(x,TYPE(x))),2,2)"),
    );
    model._set("F5", &format!("=INDEX(MAP({NESTED},LAMBDA(x,SUM(x))),2,2)"));
    model._set("F6", &format!("=MAP({NESTED},LAMBDA(x,x))"));
    model._set("F7", &format!("=REDUCE(0,{NESTED},LAMBDA(a,x,a+ROWS(x)))"));
    model.evaluate();
    assert_eq!(model._get_text("F1"), "1");
    assert_eq!(model._get_text("F2"), "2");
    assert_eq!(model._get_text("F3"), "4");
    assert_eq!(model._get_text("F4"), "64");
    assert_eq!(model._get_text("F5"), "7");
    assert_eq!(model._get_text("F6"), "#CALC!");
    assert_eq!(model._get_text("F7"), "19");
}

#[test]
fn aggregates_flatten_or_are_not_available() {
    // probes/t2592-nested-groupby-cells A6:A10, probes/t2594-nested-aggregates (T-2594): SUM,
    // COUNT and COUNTA flatten the nested cells; AVERAGE, MIN, MAX and PRODUCT are `#N/A`;
    // ARRAYTOTEXT is `#CALC!`.
    let mut model = groups();
    model._set("F1", &format!("=SUM({NESTED})"));
    model._set("F2", &format!("=COUNT({NESTED})"));
    model._set("F3", &format!("=COUNTA({NESTED})"));
    model._set("F4", &format!("=AVERAGE({NESTED})"));
    model._set("F5", &format!("=MAX({NESTED})"));
    model._set("F6", &format!("=PRODUCT({NESTED})"));
    model._set("F7", &format!("=ARRAYTOTEXT({NESTED})"));
    model._set("F8", &format!("=SUM(1,{NESTED},2)"));
    model.evaluate();
    assert_eq!(model._get_text("F1"), "220");
    assert_eq!(model._get_text("F2"), "16");
    assert_eq!(model._get_text("F3"), "19");
    assert_eq!(model._get_text("F4"), "#N/A");
    assert_eq!(model._get_text("F5"), "#N/A");
    assert_eq!(model._get_text("F6"), "#N/A");
    assert_eq!(model._get_text("F7"), "#CALC!");
    assert_eq!(model._get_text("F8"), "223");
}

#[test]
fn flattening_and_handing_over_copies_is_bounded() {
    // SUM over 200 free copies of a 1,048,512-cell nested cell, and REDUCE handing each copy to
    // its LAMBDA, pass the walk budget (2^27 cells) at the 129th copy: `#ERROR!`, in time bounded
    // by the budget (DEVIATION, arrays.md "Nested GROUPBY and PIVOTBY cells: bounds").
    let mut model = groups();
    let copies = "LET(r,GROUPBY(A2,C2,LAMBDA(v,SEQUENCE(64,16383)),0,0),\
                  CHOOSECOLS(CHOOSEROWS(r,SEQUENCE(200)^0),2))";
    model._set("F1", &format!("=SUM({copies})"));
    model._set("G1", &format!("=REDUCE(0,{copies},LAMBDA(a,x,a+1))"));
    // Within the budget, every copy is read in full.
    model._set(
        "H1",
        "=SUM(CHOOSECOLS(CHOOSEROWS(GROUPBY(A2,C2,LAMBDA(v,SEQUENCE(1000)),0,0),\
         SEQUENCE(200)^0),2))",
    );
    model.evaluate();
    assert_eq!(model._get_text("F1"), "#ERROR!");
    assert_eq!(model._get_text("G1"), "#ERROR!");
    assert_eq!(model._get_text("H1"), "100100000");
}

#[test]
fn scan_over_a_groupby_call_is_one_cell() {
    // probes/scan-over-groupby (T-2398), Data a,1; b,2; a,3; c,4: SCAN over a GROUPBY call is one cell, 0 for
    // a SUM GROUPBY and `#N/A` for nested `LAMBDA(w,w)` cells, `#CALC!` when the LAMBDA uses its accumulator;
    // MAP still sees the whole 4x2 result.
    let mut model = new_empty_model();
    for (i, (key, value)) in [("a", "1"), ("b", "2"), ("a", "3"), ("c", "4")]
        .iter()
        .enumerate()
    {
        model._set(&format!("A{}", i + 1), key);
        model._set(&format!("B{}", i + 1), value);
    }
    model._set(
        "F1",
        "=ROWS(SCAN(GROUPBY(A1:A4,B1:B4,_xleta.SUM),0,_xleta.MIN))",
    );
    model._set(
        "F2",
        "=INDEX(SCAN(GROUPBY(A1:A4,B1:B4,_xleta.SUM),0,_xleta.MIN),1,1)",
    );
    model._set(
        "F3",
        "=ERROR.TYPE(SCAN(GROUPBY(A1:A4,B1:B4,LAMBDA(w,w)),0,_xleta.MIN))",
    );
    model._set(
        "F4",
        "=ERROR.TYPE(ROWS(SCAN(GROUPBY(A1:A4,B1:B4,_xleta.SUM),0,LAMBDA(a,v,a+1))))",
    );
    model._set(
        "F5",
        "=ROWS(MAP(GROUPBY(A1:A4,B1:B4,_xleta.SUM),LAMBDA(x,TYPE(x))))",
    );
    model.evaluate();
    for (cell, value) in [
        ("F1", "1"),
        ("F2", "0"),
        ("F3", "7"),
        ("F4", "14"),
        ("F5", "4"),
    ] {
        assert_eq!(model._get_text(cell), value, "{cell}");
    }
}
