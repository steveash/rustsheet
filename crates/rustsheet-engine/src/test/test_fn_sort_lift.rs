//! SORT's `by_col` lifts over an array, one call per element whose value is that call's top-left;
//! the array and the `sort_index` and `sort_order` lists stay whole (SPEC-0011 "Array-valued
//! functions", T-2319; gen-m7/7307132bb8820f17 A1076).

use crate::test::util::new_empty_model;

fn check(cells: &[(&str, &str, &str)]) {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "3"), ("A2", "1"), ("A3", "2")] {
        model._set(cell, value);
    }
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), *expected, "{cell} {formula}");
    }
}

#[test]
fn by_col_lifts_over_an_array() {
    check(&[
        ("C1", "=COLUMNS(SORT(A1:A3,1,1,{FALSE,TRUE}))", "2"),
        // By rows the sorted column starts at 1; by columns the one column is unchanged.
        // probes/t2366-sort-lifted-by-col: later elements are not SORTBY's empties.
        ("C2", "=INDEX(SORT(A1:A3,1,1,{FALSE,TRUE}),1,1)", "1"),
        (
            "C3",
            "=ISBLANK(INDEX(SORT(A1:A3,1,1,{FALSE,TRUE}),1,2))",
            "FALSE",
        ),
    ]);
}

#[test]
fn an_element_error_stays_in_its_position() {
    // sort_index 3 is past the one column (`#VALUE!`) but not past the three rows.
    check(&[
        ("C1", "=INDEX(SORT(A1:A3,3,1,{FALSE,TRUE}),1,1)", "#VALUE!"),
        // probes/t2366-sort-lifted-by-col I11.
        ("C2", "=INDEX(SORT(A1:A3,3,1,{FALSE,TRUE}),1,2)", "3"),
        ("C3", "=INDEX(SORT(A1:A3,1,1,{FALSE,\"x\"}),1,2)", "#VALUE!"),
    ]);
}

/// Every sorting element past the first is `#VALUE!`, but for the next one with `sort_index` 2,
/// which is empty, whatever the rows and the `by_col` values (T-3090; probes/t3090-sort-lifted-
/// by-col-broken-lead and the t2929 probes refilled on 2026-10-29).
#[test]
fn later_elements_are_empty_or_value_errors() {
    check(&[
        // sort_index 1: every element past the first sorting one is `#VALUE!` (t2366 I1, H18).
        ("C1", "=INDEX(SORT(A1:A3,1,1,{FALSE,TRUE}),1,2)", "#VALUE!"),
        ("C2", "=INDEX(SORT(A1:A3,1,1,{FALSE;TRUE}),2,1)", "#VALUE!"),
        // sort_index 2 (by_col TRUE, so a row): the second sorting element is empty, the next not.
        (
            "C3",
            "=ISBLANK(INDEX(SORT(A1:A3,2,1,{TRUE,TRUE,TRUE}),1,2))",
            "TRUE",
        ),
        (
            "C4",
            "=INDEX(SORT(A1:A3,2,1,{TRUE,TRUE,TRUE}),1,3)",
            "#VALUE!",
        ),
        // Down a column too, equal by_col values or not (t3090 K137, K71).
        (
            "C5",
            "=ISBLANK(INDEX(SORT({1,2,3;3,1,2;2,3,1},2,1,{TRUE;FALSE;FALSE}),2,1))",
            "TRUE",
        ),
        (
            "C6",
            "=INDEX(SORT({1,2,3;3,1,2;2,3,1},2,1,{TRUE;FALSE;FALSE}),3,1)",
            "#VALUE!",
        ),
        // Equal rows do not keep their call's top-left (t2929-index P1, t3090 K155); an invalid
        // element is skipped (t2929-column S6).
        (
            "C7",
            "=INDEX(SORT(A1:A3,1,1,{TRUE;TRUE;TRUE}),3,1)",
            "#VALUE!",
        ),
        ("C8", "=INDEX(SORT(A1:A3,1,1,{\"x\";TRUE;TRUE}),2,1)", "3"),
        (
            "C9",
            "=INDEX(SORT(A1:A3,1,1,{\"x\";TRUE;TRUE}),3,1)",
            "#VALUE!",
        ),
        (
            "C10",
            "=INDEX(SORT(A1:A3,2,1,{TRUE,TRUE;TRUE,TRUE}),2,1)",
            "#VALUE!",
        ),
    ]);
}

#[test]
fn unlifted_calls_are_unchanged() {
    check(&[
        ("C1", "=INDEX(SORT(A1:A3),2,1)", "2"),
        ("C2", "=INDEX(SORT(A1:A3,1,-1),1,1)", "3"),
        ("C3", "=ROWS(SORT(A1:A3,1,1,FALSE))", "3"),
        ("C4", "=SORT(A1:A3,2)", "#VALUE!"),
    ]);
}
