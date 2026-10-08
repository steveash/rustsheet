use crate::calc_result::CalcResult;
use crate::functions::binary_search::*;

/// The match for match mode -1 (`Some(-1)`), 0 (`None`) or 1 (`Some(1)`).
fn find<T: Ord>(target: &T, array: &[T], descending: bool, mode: Option<i8>) -> Option<usize> {
    binary_search(target, array, descending).pick(mode == Some(-1), mode == Some(1))
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
    use crate::expressions::token::Error;
    let errs = vec![n(5.0), err(Error::NA), n(10.0), err(Error::DIV), n(20.0)];
    assert_eq!(find(&n(10.0), &errs, false, None), Some(2)); // M6: 3
    assert_eq!(find(&n(15.0), &errs, false, Some(-1)), Some(2)); // M7: 3
}
