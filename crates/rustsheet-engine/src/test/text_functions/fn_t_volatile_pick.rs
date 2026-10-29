#![allow(clippy::unwrap_used)]

//! T reads and tests the one argument a volatile picker picked (T-3142). Over 255 UTF-16 units,
//! a cell's text is "" and a computed text `#VALUE!`, also through IF and CHOOSE but not through
//! `+` (probes/text-255-units-reference-shapes; T-3107), so a T that read one pick and tested
//! another could give `#VALUE!` for D1.

use crate::test::util::new_empty_model;

#[test]
fn test_t_volatile_pick_reads_and_tests_one_pick() {
    let mut model = new_empty_model();
    model._set("D1", "=REPT(\"x\",256)");
    // D1 picked is "" (cell text) and "a" picked is "a". `#VALUE!` needs D1's value and the
    // reference test of "a", a second draw: 200 recalculations would show it with probability
    // 1 - 0.75^200 if each call drew twice.
    model._set("A1", "=T(CHOOSE(RANDBETWEEN(1,2),D1,\"a\"))");
    model._set("A2", "=T(IF(RANDBETWEEN(0,1)=1,D1,\"a\"))");
    model._set(
        "A3",
        "=T((CHOOSE(RANDBETWEEN(1,2),IF(RANDBETWEEN(0,1)=1,D1,\"a\"),D1)))",
    );
    for _ in 0..200 {
        model.evaluate();
        for cell in ["A1", "A2", "A3"] {
            let text = model._get_text(cell);
            assert!(text.is_empty() || text == "a", "{cell}: {text}");
        }
    }
}

#[test]
fn test_t_picked_shapes_keep_their_reading() {
    // probes/text-255-units-reference-shapes: IF and CHOOSE pick the cell.
    let mut model = new_empty_model();
    model._set("D1", "=REPT(\"x\",256)");
    model._set("A1", "=T(CHOOSE(1,D1))");
    model._set("A2", "=T(IF(TRUE,D1))");
    model._set("A3", "=T(IF(TRUE,REPT(\"x\",256)))");
    model._set("A4", "=T(CHOOSE(2,\"b\",D1))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "");
    assert_eq!(model._get_text("A2"), "");
    assert_eq!(model._get_text("A3"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "");
}
