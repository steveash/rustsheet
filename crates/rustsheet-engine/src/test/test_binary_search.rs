use std::cmp::Ordering;

use crate::calc_result::CalcResult;
use crate::expressions::token::Error;
use crate::functions::binary_search::*;
use crate::test::util::new_empty_model;

/// The match for match mode -1 (`Some(-1)`), 0 (`None`) or 1 (`Some(1)`).
fn find<T: Ord>(target: &T, array: &[T], descending: bool, mode: Option<i8>) -> Option<usize> {
    let side = match mode {
        Some(-1) => Ordering::Less,
        Some(1) => Ordering::Greater,
        _ => Ordering::Equal,
    };
    binary_search_by(target, array, descending, side, T::cmp)
}

#[test]
fn test_binary_search() {
    let t = vec![1, 2, 3, 40, 55, 155];
    assert_eq!(find(&40, &t, false, Some(-1)), Some(3));
    assert_eq!(find(&40, &t, false, Some(1)), Some(3));
    assert_eq!(find(&45, &t, false, Some(-1)), Some(3));
    assert_eq!(find(&45, &t, false, Some(1)), Some(4));
    assert_eq!(find(&45, &t, false, None), None);
    assert_eq!(find(&0, &t, false, Some(-1)), None);
    assert_eq!(find(&999, &t, false, Some(1)), None);
}

#[test]
fn test_binary_search_descending() {
    let t = vec![100, 33, 23, 14, 5, -155];
    assert_eq!(find(&23, &t, true, Some(-1)), Some(2));
    assert_eq!(find(&23, &t, true, Some(1)), Some(2));
    assert_eq!(find(&25, &t, true, Some(-1)), Some(2));
    assert_eq!(find(&25, &t, true, Some(1)), Some(1));
}

#[test]
fn test_binary_search_multiple() {
    let t = vec![1, 2, 3, 40, 40, 40, 40, 55, 155];
    assert_eq!(find(&40, &t, false, Some(-1)), Some(3));
    assert_eq!(find(&39, &t, false, Some(-1)), Some(2));
    assert_eq!(find(&40, &t, false, Some(1)), Some(3));
    assert_eq!(find(&41, &t, false, Some(1)), Some(7));
}

/// Blanks and errors are larger than numbers and are probed, never skipped
/// (probes/xmatch-xlookup-blanks; T-1019).
#[test]
fn test_binary_search_over_blanks_and_errors() {
    let n = CalcResult::Number;
    let blank = || CalcResult::EmptyCell;
    // Col!A1:A6 = 5,_,10,_,20,_
    let col = vec![n(5.0), blank(), n(10.0), blank(), n(20.0), blank()];
    assert_eq!(find(&n(10.0), &col, false, None), Some(2)); // M16: 3
    assert_eq!(find(&n(20.0), &col, false, None), Some(4)); // M17: 5
    assert_eq!(find(&n(7.0), &col, false, Some(-1)), Some(0)); // M18: 1
    assert_eq!(find(&n(7.0), &col, false, Some(1)), Some(1)); // M19: 2, a blank
    assert_eq!(find(&n(10.0), &col, true, None), Some(2)); // M24: 3
    assert_eq!(find(&n(0.0), &col, true, None), None); // M26: #N/A
                                                       // A whole column: the same cells, then blanks to row 1048576.
    let mut whole = col.clone();
    whole.resize_with(1_048_576, blank);
    assert_eq!(find(&n(99.0), &whole, false, Some(-1)), Some(0)); // M20: 1
    assert_eq!(find(&n(99.0), &whole, false, Some(1)), Some(1)); // M23: 2
    assert_eq!(find(&n(0.0), &whole, false, Some(1)), Some(0)); // M22: 1
    assert_eq!(find(&n(0.0), &whole, false, None), None); // M21: #N/A
    assert_eq!(find(&n(99.0), &whole, true, Some(-1)), None); // M25: #N/A
                                                              // Err!A1:A5 = 5,#N/A,10,#DIV/0!,20
    let err = |e| CalcResult::Error {
        error: e,
        origin: crate::expressions::types::CellReferenceIndex {
            sheet: 0,
            row: 1,
            column: 1,
        },
        message: String::new(),
    };
    let errs = vec![n(5.0), err(Error::NA), n(10.0), err(Error::DIV), n(20.0)];
    assert_eq!(find(&n(10.0), &errs, false, None), Some(2)); // M6: 3
    assert_eq!(find(&n(15.0), &errs, false, Some(-1)), Some(2)); // M7: 3
}

/// Runs of equal values (probes/xmatch-text-bool-equal-runs Run!M1-M42; T-1135, T-1153): every
/// match mode returns the run's first element, a run starting at the first cell included. Long
/// runs cost O(log n) probes: an equal hit keeps the search bisecting.
#[test]
fn test_binary_search_runs() {
    let t = vec![40, 40, 40];
    assert_eq!(find(&40, &t, false, None), Some(0));
    assert_eq!(find(&40, &t, false, Some(-1)), Some(0));
    assert_eq!(find(&40, &t, false, Some(1)), Some(0));
    assert_eq!(find(&40, &t, true, None), Some(2));
    // The search goes on past an equal element: 60,40,70,40 ends at the last 40 descending,
    // 40,30,40,40,50 at the first 40 ascending (probes/xmatch-blank-extensions Ext!M38, M41).
    assert_eq!(find(&40, &[40, 30, 40, 40, 50], false, None), Some(0));
    let gap = vec![60, 40, 70, 40];
    assert_eq!(find(&40, &gap, true, None), Some(3));
    let mut long = vec![1];
    long.resize(1_048_576, 7);
    assert_eq!(find(&7, &long, false, None), Some(1));
    let mut long_desc = vec![9];
    long_desc.resize(1_048_576, 7);
    assert_eq!(find(&7, &long_desc, true, None), Some(1_048_575));
}

/// XLOOKUP's binary search walks the whole column, blanks past the used area included, as
/// XMATCH does: `XMATCH(99,A:A,-1,2)` is 1 (probes/xmatch-xlookup-blanks Col!M20; T-1019).
#[test]
fn xlookup_binary_search_over_a_whole_column() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "5"),
        ("A3", "10"),
        ("A5", "20"),
        ("B1", "a"),
        ("B2", "b"),
        ("B3", "c"),
        ("B4", "d"),
        ("B5", "e"),
        ("B6", "f"),
        ("C9", "x"), // used area past the data must not change the answer
    ] {
        model._set(cell, value);
    }
    model._set("D1", "=XLOOKUP(99,A:A,B:B,\"nf\",-1,2)");
    model._set("D2", "=XLOOKUP(99,A:A,B:B,\"nf\",1,2)");
    model._set("D3", "=XMATCH(99,A:A,-1,2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "a");
    assert_eq!(model._get_text("D2"), "b");
    assert_eq!(model._get_text("D3"), "1");
}
