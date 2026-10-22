//! SORTBY's sort orders lift over an array or a range; its array and by arrays stay whole
//! (SPEC-0011 "Array-valued functions" rule 4, T-1839; probes/m7-argument-modes F26:H26,
//! probes/sort-error-args H97-H99, probes/m7-lift-take-drop-sortby sheet Sort).

use crate::functions::lift::ArgKind;
use crate::functions::Function;
use crate::test::util::new_empty_model;

fn check(cells: &[(&str, &str, &str)]) {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "3"),
        ("A2", "1"),
        ("A3", "2"),
        ("C1", "2"),
        ("C2", "1"),
        ("C3", "3"),
    ] {
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
fn the_table_alternates_by_array_and_sort_order() {
    let Some(kinds) = Function::Sortby.arg_kinds() else {
        panic!("SORTBY has a lifting table entry");
    };
    let whole = Some(ArgKind::WholeByCaller);
    let lift = Some(ArgKind::Lift);
    let got: Vec<_> = (0..6).map(|index| kinds.kind(index)).collect();
    assert_eq!(got, vec![whole, whole, lift, whole, lift, whole]);
    // 2, 3, then odd counts from 5, as `fn_sortby` accepted before the entry.
    let accepted: Vec<usize> = (0..10).filter(|&count| kinds.accepts(count)).collect();
    assert_eq!(accepted, vec![2, 3, 5, 7, 9]);
}

#[test]
fn an_unlifted_call_is_unchanged() {
    check(&[
        ("E1", "=SORTBY(A1:A3,C1:C3)", "1"),
        ("E2", "", "3"),
        ("E3", "", "2"),
        ("F1", "=SORTBY(A1:A3,C1:C3,-1,A1:A3,1)", "2"),
        ("G1", "=SORTBY(A1:A3,C1:C3,1,A1:A3)", "#ERROR!"),
    ]);
}

#[test]
fn sort_orders_lift_by_the_first_element_rule() {
    check(&[
        // Later elements are empty unless every entry is equal.
        ("E1", "=SORTBY(A1:A3,C1:C3,{1;-1;1})", "1"),
        ("E2", "", "0"),
        ("E3", "", "0"),
        ("F1", "=SORTBY(A1:A3,C1:C3,{-1;-1})", "2"),
        ("F2", "", "2"),
        // A second pair makes them #VALUE!; a lifted second sort_order lifts the same way.
        ("G1", "=SORTBY(A1:A3,C1:C3,{1;-1},A1:A3,1)", "1"),
        ("G2", "", "#VALUE!"),
        ("H1", "=SORTBY(A1:A3,C1:C3,1,A1:A3,{1;1})", "1"),
        ("H2", "", "1"),
        // An entry that is not 1 or -1 is #VALUE! in its own slot.
        ("I1", "=SORTBY(A1:A3,C1:C3,{1;2})", "1"),
        ("I2", "", "#VALUE!"),
        // A by_array written as an array literal is read whole, not lifted.
        ("J1", "=ROWS(SORTBY(A1:A3,{2;1;3}))", "3"),
    ]);
}

/// Each element sorts the whole array, so it is charged the array's and the by array's cells and
/// is `#ERROR!` past `limits::MAX_LIFTED_CELL_VISITS` (DEVIATION, docs/excel-semantics/arrays.md
/// "Array results above 1,048,576 cells").
#[test]
fn a_lifted_call_over_a_large_array_is_charged_per_element() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=ROWS(SORTBY(SEQUENCE(131072),SEQUENCE(131072),SEQUENCE(1100,1,1,0)))",
    );
    model._set("B1", "=ROWS(SORTBY(SEQUENCE(4),SEQUENCE(4),{1;1}))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("B1"), "2");
}
