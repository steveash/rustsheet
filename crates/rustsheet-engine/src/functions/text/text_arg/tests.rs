use std::cell::Cell;

use super::{broadcast_text, TextArg};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
};

/// The array limit is checked before `compute` runs for any element (T-1771): a broadcast above
/// it costs nothing but the operands.
#[test]
fn the_size_is_checked_before_any_element_is_computed() {
    let cell = CellReferenceIndex {
        sheet: 0,
        column: 1,
        row: 1,
    };
    let column = TextArg::Array(vec![vec![ArrayNode::Number(1.0)]; 2000]);
    let row = TextArg::Array(vec![vec![ArrayNode::Number(1.0); 1000]]);
    let calls = Cell::new(0_usize);

    let result = broadcast_text(cell, &[&column, &row], |_, _| {
        calls.set(calls.get() + 1);
        Ok(ArrayNode::Number(0.0))
    });

    assert!(matches!(
        result,
        CalcResult::Error {
            error: Error::ERROR,
            ..
        }
    ));
    assert_eq!(calls.get(), 0);
}
