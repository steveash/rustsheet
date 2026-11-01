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

#[test]
fn test_t_let_and_at_volatile_picks_read_and_test_one_draw() {
    // T-3157: a LET's body and the `@` of a picker are read and tested on one draw, as a root
    // picker is (T-3142). A second draw would give `#VALUE!` for D1's value with "a"'s test.
    let mut model = new_empty_model();
    model._set("D1", "=REPT(\"x\",256)");
    model._set("A1", "=T(LET(i,RANDBETWEEN(1,2),CHOOSE(i,D1,\"a\")))");
    model._set("A2", "=T(@CHOOSE(RANDBETWEEN(1,2),D1,\"a\"))");
    // A LET a root picker picked.
    model._set(
        "A3",
        "=T(IF(TRUE,LET(i,RANDBETWEEN(1,2),CHOOSE(i,D1,\"a\"))))",
    );
    model._set("A4", "=T(LET(x,CHOOSE(RANDBETWEEN(1,2),D1,\"a\"),x))");
    model._set("A5", "=T(@IFS(RANDBETWEEN(0,1)=1,D1,TRUE,\"a\"))");
    model._set("A6", "=T(@SWITCH(RANDBETWEEN(1,2),1,D1,\"a\"))");
    for _ in 0..200 {
        model.evaluate();
        for cell in ["A1", "A2", "A3", "A4", "A5", "A6"] {
            let text = model._get_text(cell);
            assert!(text.is_empty() || text == "a", "{cell}: {text}");
        }
    }
}

#[test]
fn test_t_let_and_at_shapes_keep_their_reading() {
    // The readings these shapes had before T-3157, unprobed (T-3167 asks the oracle): a cell a
    // LET or a picker reaches is the cell's text, a computed text `#VALUE!`, and the `@` of a
    // picker reads like the picker, IFS and SWITCH included.
    let mut model = new_empty_model();
    model._set("D1", "=REPT(\"x\",256)");
    model._set("A1", "=T(LET(x,D1,x))");
    model._set("A2", "=T(LET(x,REPT(\"x\",256),x))");
    model._set("A4", "=T(@CHOOSE(1,D1))");
    model._set("A5", "=T(@IF(TRUE,REPT(\"x\",256)))");
    model._set("A6", "=T(LET(c,TRUE,IF(c,D1)))");
    model._set("A7", "=T(LET(x,1,x,2,x))");
    model._set("A8", "=T(@IFS(TRUE,D1))");
    model._set("A9", "=T(@SWITCH(1,1,D1))");
    model._set("A10", "=T(@IF(TRUE,+D1))");
    model._set("A11", "=T(IF(TRUE,@IF(TRUE,D1)))");
    model._set("A12", "=T(IF(TRUE,@IF(TRUE,+D1)))");
    model._set("A13", "=T(CHOOSE(1,@IFS(TRUE,D1)))");
    model._set("A14", "=T(@IF(TRUE,@IF(TRUE,D1)))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "");
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "");
    assert_eq!(model._get_text("A5"), "#VALUE!");
    assert_eq!(model._get_text("A6"), "");
    assert_eq!(model._get_text("A7"), "#ERROR!");
    assert_eq!(model._get_text("A8"), "");
    assert_eq!(model._get_text("A9"), "");
    assert_eq!(model._get_text("A10"), "");
    for cell in ["A11", "A12", "A13", "A14"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
}
