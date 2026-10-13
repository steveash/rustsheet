#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel counts LEN, MID, FIND, SEARCH and REPLACE positions in UTF-16 code units, LEFT and RIGHT
// in characters (probes/text-surrogate-positions, excel-graph; docs/excel-semantics/text.md
// "Text positions count UTF-16 code units, except in LEFT and RIGHT").

#[test]
fn positions_count_utf16_units() {
    let mut model = new_empty_model();
    model._set("A1", "=LEN(\"a😀b\")");
    model._set("A2", "=MID(\"a😀b\",2,2)");
    model._set("A3", "=MID(\"a😀b\",4,1)");
    model._set("A4", "=FIND(\"b\",\"😀b😀b\",4)");
    model._set("A5", "=FIND(\"b\",\"😀b\",2)");
    model._set("A6", "=SEARCH(\"?b\",\"😀b\")");
    model._set("A7", "=SEARCH(\"a?b\",\"a😀b\")");
    model._set("A8", "=SEARCH(\"a??b\",\"a😀b\")");
    model._set("A9", "=REPLACE(\"a😀b\",2,2,\"x\")");
    model._set("A10", "=REPLACE(\"a😀b\",4,1,\"x\")");
    model._set("A11", "=FIND(\"😀\",\"a😀\",3)");
    model._set("A12", "=LENB(\"𝐀\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
    assert_eq!(model._get_text("A2"), "😀");
    assert_eq!(model._get_text("A3"), "b");
    assert_eq!(model._get_text("A4"), "6");
    assert_eq!(model._get_text("A5"), "3");
    assert_eq!(model._get_text("A6"), "2");
    assert_eq!(model._get_text("A7"), "#VALUE!");
    assert_eq!(model._get_text("A8"), "1");
    assert_eq!(model._get_text("A9"), "axb");
    assert_eq!(model._get_text("A10"), "a😀x");
    assert_eq!(model._get_text("A11"), "#VALUE!");
    assert_eq!(model._get_text("A12"), "2");
}

#[test]
fn left_and_right_count_code_points() {
    let mut model = new_empty_model();
    model._set("A1", "=LEFT(\"a😀b\",3)");
    model._set("A2", "=RIGHT(\"a😀b\",2)");
    model._set("A3", "=LEFT(\"👍🏽x\",1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a😀b");
    assert_eq!(model._get_text("A2"), "😀b");
    assert_eq!(model._get_text("A3"), "👍");
}

#[test]
fn split_pair_is_dropped_whole() {
    // Excel keeps the lone surrogate; a Rust String cannot hold one yet (T-1483).
    let mut model = new_empty_model();
    model._set("A1", "=MID(\"a😀b\",3,2)");
    model._set("A2", "=MID(\"a😀b\",2,1)");
    model._set("A3", "=REPLACE(\"a😀b\",3,1,\"x\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
    assert_eq!(model._get_text("A2"), "");
    assert_eq!(model._get_text("A3"), "axb");
}

#[test]
fn find_start_one_past_the_end() {
    let mut model = new_empty_model();
    model._set("A1", "=FIND(\"\",\"abc\",4)");
    model._set("A2", "=FIND(\"\",\"abc\",5)");
    model._set("A3", "=FIND(\"\",\"\",1)");
    model._set("A4", "=FIND(\"c\",\"abc\",4)");
    model._set("A5", "=SEARCH(\"\",\"abc\",4)");
    model._set("A6", "=FIND(\"\",\"😀b\",4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A3"), "1");
    assert_eq!(model._get_text("A4"), "#VALUE!");
    assert_eq!(model._get_text("A5"), "#VALUE!");
    assert_eq!(model._get_text("A6"), "4");
}
