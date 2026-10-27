//! What one formula's texts hold in memory (T-2806; docs/excel-semantics/arrays.md "Copies of a
//! text share its bytes"): a text copied into many cells shares one buffer, and the `&` operator
//! over an array pays for each text it makes. Every test checks sharing or counts units, never
//! memory or time (T-2282).

use std::sync::Arc;

use crate::area_budget::{CELL_UNITS, NODE_UNITS, TEXT_BYTES_PER_UNIT, TOO_MUCH_WORK};
use crate::calc_result::CalcResult;
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
use crate::test::util::new_empty_model;
use crate::text::Text;

/// Whether `a` and `b` hold the same buffer, not equal copies.
fn shares_bytes(a: &Text, b: &Text) -> bool {
    match (a, b) {
        (Text::Utf8(x), Text::Utf8(y)) => Arc::ptr_eq(x, y),
        (Text::Lone(x), Text::Lone(y)) => Arc::ptr_eq(x, y),
        _ => false,
    }
}

/// The texts of the array `formula` gives in B1 of a sheet whose A1 holds a 1,000-byte text.
fn array_texts(formula: &str) -> Vec<Text> {
    let mut model = new_empty_model();
    model._set("A1", r#"=REPT("a",1000)"#);
    model.evaluate();
    let context = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 2,
    };
    let cell = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 2,
    };
    let node = model.parse_internal_formula(formula, &context);
    let CalcResult::Array(array) = model.evaluate_outside_cell(&node, cell) else {
        panic!("{formula} is not an array");
    };
    array
        .into_iter()
        .flatten()
        .map(|node| match node {
            ArrayNode::String(text) => text,
            other => panic!("{formula} holds {other:?}"),
        })
        .collect()
}

#[test]
fn a_clone_shares_the_bytes_and_an_edit_copies_them() {
    let lone = Text::from_utf16(vec![u16::from(b'a'), 0xD83D]);
    for original in [Text::from("abc"), lone] {
        let copy = original.clone();
        assert!(shares_bytes(&original, &copy));
        let mut pushed = copy.clone();
        pushed.push(&Text::from("d"));
        let mut cut = copy.clone();
        assert!(cut.truncate_utf16(1));
        // The edits are the copies' own; the original and its other copies keep their text.
        assert_eq!(original, copy);
        assert!(shares_bytes(&original, &copy));
        assert_eq!(pushed.len_utf16(), original.len_utf16() + 1);
        assert_eq!(&*cut, "a");
        // A text of exactly the limit is not cut, and still shares its bytes.
        let mut whole = copy.clone();
        assert!(!whole.truncate_utf16(original.len_utf16()));
        assert!(shares_bytes(&original, &whole));
    }
}

#[test]
fn a_text_copied_into_many_cells_is_one_buffer() {
    // Each shape copies A1's text, or one REPT result, into every cell of its result: owned
    // copies of a 32,767-byte text over 2^20 cells would be 32 GB.
    for formula in [
        "IF(SEQUENCE(8,8),A1)",
        r#"IF(SEQUENCE(8,8),REPT("a",1000))"#,
        "IFERROR(SEQUENCE(8,8)/0,A1)",
        "IFNA(SEQUENCE(8,8)+NA(),A1)",
        "EXPAND(A1,8,8,A1)",
        "INDEX(A1,SEQUENCE(8,8)*0+1)",
        "CHOOSE(SEQUENCE(8,8)*0+1,A1)",
    ] {
        let texts = array_texts(formula);
        assert_eq!(texts.len(), 64, "{formula}");
        let first = &texts[0];
        assert_eq!(first.len(), 1000, "{formula}");
        assert!(
            texts.iter().all(|text| shares_bytes(first, text)),
            "{formula}"
        );
    }
}

#[test]
fn concatenating_over_an_array_pays_for_each_text_it_makes() {
    const N: u64 = NODE_UNITS;
    const C: u64 = CELL_UNITS;
    let mut model = new_empty_model();
    model._set("A1", r#"=ROWS(SEQUENCE(4)&"abcdefg")"#);
    model.evaluate();
    // ROWS, `&`, SEQUENCE, 4 and the literal (7 bytes): five nodes and the literal's text; the
    // four cells SEQUENCE builds and the four `&` builds; and the four texts `&` makes, "1abcdefg"
    // to "4abcdefg", 8 bytes each.
    assert_eq!(
        model.area_budget.last_spent,
        5 * N + 7_u64.div_ceil(TEXT_BYTES_PER_UNIT) + 8 * C + 4 * 8 / TEXT_BYTES_PER_UNIT
    );
}

#[test]
fn concatenating_past_the_budget_is_the_budget_error() {
    // 1,000 texts of 1,001 bytes: about 250,000 units of text, past a limit of 100,000.
    let mut model = new_empty_model();
    model.area_budget.limit = 100_000;
    model._set("A1", r#"=IFERROR(ROWS(SEQUENCE(1000)&REPT("a",1000)),0)"#);
    model._set("A2", r#"=ROWS(SEQUENCE(10)&REPT("a",1000))"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_error_message("A1"), TOO_MUCH_WORK);
    assert_eq!(model._get_text("A2"), "10");
}
