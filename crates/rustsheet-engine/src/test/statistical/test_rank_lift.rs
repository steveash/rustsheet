#![allow(clippy::unwrap_used)]

//! RANK, RANK.EQ and RANK.AVG lift `number` and `order` over a `ref` read once, and a `ref`
//! that is not a reference broadcasts its errors (SPEC-0011, T-1999). Excel values:
//! probes/stat-test-lift P20-P33, probes/rank-direct-boolean, probes/rank-nonref-errors
//! (docs/excel-semantics/statistics.md "RANK, FORECAST, PROB, Z.TEST and T.TEST").

use crate::functions::Function;
use crate::test::util::new_empty_model;

/// Each formula of `cases` in its own cell of column W, checked against its Excel value, over
/// A1:A8 = 1,2,4,3,7,5,8,6 and D1:D3 = 1, #N/A, 3.
fn assert_cells(cases: &[(&str, &str)]) {
    let mut model = new_empty_model();
    for (row, value) in [1, 2, 4, 3, 7, 5, 8, 6].iter().enumerate() {
        model._set(&format!("A{}", row + 1), &value.to_string());
    }
    for (cell, value) in [("D1", "1"), ("D2", "=NA()"), ("D3", "3")] {
        model._set(cell, value);
    }
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("W{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("W{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

/// stat-test-lift P20-P27, P32, P33: each element is the scalar call over the data.
#[test]
fn number_and_order_lift_over_the_data() {
    assert_cells(&[
        ("=INDEX(RANK({4;\"x\"},A1:A8),2)", "#VALUE!"),
        ("=INDEX(RANK({4;99},A1:A8),2)", "#N/A"),
        ("=INDEX(RANK(4,A1:A8,{0;1}),2)", "4"),
        ("=ROWS(RANK(4,A1:A8,{0;1}))", "2"),
        ("=INDEX(RANK({1;3},D1:D3),1)", "#N/A"),
        ("=INDEX(RANK({4;TRUE},A1:A8),2)", "8"),
        ("=INDEX(RANK.AVG({4;5},A1:A8,{1;0}),2)", "4"),
        ("=INDEX(RANK.EQ({4;\"2\"},A1:A8),2)", "7"),
        ("=INDEX(RANK(4,A1:A8,{0;\"x\"}),2)", "#VALUE!"),
        ("=TYPE(RANK({4},A1:A8))", "64"),
    ]);
}

/// stat-test-lift P28-P31, rank-nonref-errors: a non-reference `ref` broadcasts as its own
/// shape, after `number`'s error and before `order`'s.
#[test]
fn a_non_reference_ref_broadcasts_its_errors() {
    assert_cells(&[
        ("=ROWS(RANK({4;\"x\"},IF(A1:C1>0,A1:C1)))", "2"),
        ("=COLUMNS(RANK({4;\"x\"},IF(A1:C1>0,A1:C1)))", "3"),
        ("=INDEX(RANK({4;\"x\"},IF(A1:C1>0,A1:C1)),2,1)", "#VALUE!"),
        ("=INDEX(RANK({4;\"x\"},IF(A1:C1>0,A1:C1)),1,1)", "#VALUE!"),
        ("=INDEX(RANK(NA(),IF(A1:C1>0,A1:C1)),1,2)", "#N/A"),
        (
            "=INDEX(RANK(5,IF({7,1,5}>1,1/0,{7,1,5}),\"x\"),1,1)",
            "#DIV/0!",
        ),
        (
            "=INDEX(RANK(5,IF({7,1,5}>1,1/0,{7,1,5}),\"x\"),1,2)",
            "#VALUE!",
        ),
    ]);
}

/// rank-direct-boolean: a boolean `number` or `order` coerces, a text `order` is `#VALUE!`,
/// and `order` is read before the data's error.
#[test]
fn scalar_arguments_coerce_as_excel_reads_them() {
    assert_cells(&[
        ("=RANK(TRUE,A1:A8)", "8"),
        ("=RANK.AVG(TRUE,A1:A8)", "8"),
        ("=RANK(\"4\",A1:A8)", "5"),
        ("=RANK(4,A1:A8,TRUE)", "4"),
        ("=RANK(4,A1:A8,\"1\")", "#VALUE!"),
        ("=RANK(4,D1:D3,\"x\")", "#VALUE!"),
        ("=RANK(4,D1:D3,1/0)", "#DIV/0!"),
        ("=RANK(4,D1:D3)", "#N/A"),
        ("=RANK(FALSE,A1:A8)", "#N/A"),
    ]);
}

/// A lifted call searches the numbers sorted once; each element equals the scalar call, which
/// walks them, ties and -0 included.
#[test]
fn sorted_elements_equal_the_scalar_walk() {
    let mut model = new_empty_model();
    for (row, value) in ["3", "-1", "3", "0", "=-0", "7", "3", "x", "TRUE", "0.5"]
        .iter()
        .enumerate()
    {
        model._set(&format!("A{}", row + 1), value);
    }
    let mut cells = Vec::new();
    for function in ["RANK", "RANK.AVG"] {
        for order in ["0", "1"] {
            for (index, number) in ["3", "-1", "0", "-0", "7", "0.5", "2", "1"]
                .iter()
                .enumerate()
            {
                let row = index + 1;
                let lifted =
                    format!("=INDEX({function}({{3;-1;0;-0;7;0.5;2;1}},A1:A10,{order}),{row})");
                let scalar = format!("={function}({number},A1:A10,{order})");
                let (l, r) = (
                    format!("C{}", cells.len() + 1),
                    format!("D{}", cells.len() + 1),
                );
                model._set(&l, &lifted);
                model._set(&r, &scalar);
                cells.push((l, r));
            }
        }
    }
    model.evaluate();
    for (lifted, scalar) in &cells {
        assert_eq!(
            model._get_text(lifted),
            model._get_text(scalar),
            "{lifted} vs {scalar}"
        );
    }
    assert_eq!(model._get_text("C1"), "2");
}

/// `fn_rank_eq` and `fn_rank_avg` read RANK.EQ's table entry for all three functions.
#[test]
fn the_rank_functions_share_one_table_entry() {
    let entry = Function::RankEq.arg_kinds().map(std::ptr::from_ref);
    assert!(entry.is_some());
    for function in [Function::RankCompat, Function::RankAvg] {
        assert_eq!(
            function.arg_kinds().map(std::ptr::from_ref),
            entry,
            "{function:?}"
        );
    }
}
