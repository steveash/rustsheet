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

/// A blank `sort_index` or `sort_order` is `#VALUE!` before a later argument's error, while a
/// text or FALSE one loses to it (T-3507; gen-m7/91b6b88061b805b1 A5308, A11103, A1709;
/// 8217549f0b417328 A611, A1770; 75534bc7e1ce6474 A1526; docs/excel-semantics/arrays.md "TOCOL,
/// TOROW and SORT: which bad argument wins"). Z1 is blank.
#[test]
fn a_blank_key_beats_a_later_error_and_text_loses() {
    check(&[
        ("C1", "=SORT(1,Z1,1/0)", "#VALUE!"),
        ("C2", "=SORT(A1:A3,Z1,#DIV/0!,#NUM!)", "#VALUE!"),
        ("C3", "=SORT(A1:A3,1,Z1,#NAME?)", "#VALUE!"),
        ("C4", "=SORT(A1:A3,FALSE,Z1,#NULL!)", "#VALUE!"),
        ("C5", "=SORT(A1,\"abc\",0,#NUM!)", "#NUM!"),
        ("C6", "=SORT(A1:A3,\"\",#DIV/0!)", "#DIV/0!"),
        ("C7", "=SORT(A1:A3,\"abc\",#NULL!)", "#NULL!"),
        ("C8", "=SORT(A1:A3,FALSE,1,#N/A)", "#N/A"),
        // With no later error a text key is still `#VALUE!`.
        ("C9", "=SORT(A1:A3,\"abc\")", "#VALUE!"),
        ("C10", "=SORT(A1:A3,1,\"x\")", "#VALUE!"),
        // An error key still comes first, left to right (probes/sort-error-args H29).
        ("C11", "=SORT(A1:A3,NA(),1/0)", "#N/A"),
    ]);
}

/// An error in the array, `sort_index` or `sort_order` is every lifted call's result, so it fills
/// a multi-cell `by_col`'s shape (gen-m7/8217549f0b417328 A855:B856, d2945e939d6fe742
/// A5979:B5980; arrays.md "SORT's lifted by_col spreads an argument error", T-3508).
#[test]
fn an_argument_error_fills_the_lifted_shape() {
    check(&[
        ("C1", "=ROWS(SORT(A1:A3,1,#VALUE!,{TRUE;FALSE}))", "2"),
        (
            "C2",
            "=INDEX(SORT(A1:A3,1,#VALUE!,{TRUE;FALSE}),2,1)",
            "#VALUE!",
        ),
        ("C3", "=INDEX(SORT(#REF!,1,1,{TRUE,FALSE}),1,2)", "#REF!"),
        (
            "C4",
            "=INDEX(SORT(A1:A3,\"abc\",1,{TRUE,FALSE}),1,2)",
            "#VALUE!",
        ),
        // A 2x2 by_col is a 2x2 of the error (75534bc7e1ce6474 A3051:B3052).
        ("C5", "=ROWS(SORT(A1,#NUM!,1,{TRUE,FALSE;FALSE,TRUE}))", "2"),
        (
            "C6",
            "=INDEX(SORT(A1,#NUM!,1,{TRUE,FALSE;FALSE,TRUE}),2,2)",
            "#NUM!",
        ),
    ]);
}
