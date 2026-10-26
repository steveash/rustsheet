//! VARA, VARPA, STDEVA and STDEVPA take a union argument, MODE and MODE.SNGL give `#VALUE!` for
//! one (T-1184). Expected values are Excel's,
//! probes/multiarea-foreach H8, H9, H13, H14, H17, H18.

use crate::test::util::new_empty_model;

#[test]
fn union_argument_to_the_a_variance_family_and_mode() {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("B2", "5"), ("C3", "7"), ("D4", "9")] {
        model._set(cell, value);
    }
    let cases = [
        ("H1", "=VARA((A1,B2,C3,D4))", "11.66666667"),
        ("H2", "=VARPA((A1,B2,C3,D4))", "8.75"),
        ("H3", "=STDEVA((A1,B2,C3,D4))", "3.415650255"),
        ("H4", "=STDEVPA((A1,B2,C3,D4))", "2.958039892"),
        ("H5", "=MODE((A1,B2,C3,D4))", "#VALUE!"),
        ("H6", "=MODE.SNGL((A1,B2,C3,D4))", "#VALUE!"),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cases {
        assert_eq!(model._get_text(cell), expected, "{cell}: {formula}");
    }
}

/// A one-cell range reference to an empty cell is `#VALUE!` in MODE, MODE.SNGL and MODE.MULT,
/// beside other arguments too; a one-cell range holding a number is unchanged (gen-m9, T-2471).
#[test]
fn mode_one_cell_range_to_an_empty_cell_is_value() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "2");
    for (cell, formula) in [
        ("C1", "=MODE(Z1:Z1)"),
        ("C2", "=MODE.SNGL(1,1,Z1:Z1)"),
        ("C3", "=MODE.MULT(Z1:Z1,{5})"),
        ("C4", "=MODE(A1:A1,A1:A2)"),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, want) in [
        ("C1", "#VALUE!"),
        ("C2", "#VALUE!"),
        ("C3", "#VALUE!"),
        ("C4", "2"),
    ] {
        assert_eq!(model._get_text(cell), *want, "{cell}");
    }
}
