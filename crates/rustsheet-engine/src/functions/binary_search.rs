use std::cmp::Ordering;

use crate::{calc_result::CalcResult, expressions::types::CellReferenceIndex, model::Model};

/// Random access to a sorted sequence, so the binary searches also run over
/// a `SearchVector` whose empty tail is virtual (T-0127).
pub(crate) trait Sequence<T: Ord> {
    fn seq_len(&self) -> usize;
    /// The element at `index`, which must be below `seq_len`.
    fn at(&self, index: usize) -> &T;
}

impl<T: Ord> Sequence<T> for [T] {
    fn seq_len(&self) -> usize {
        self.len()
    }
    fn at(&self, index: usize) -> &T {
        &self[index]
    }
}

impl<T: Ord> Sequence<T> for Vec<T> {
    fn seq_len(&self) -> usize {
        self.len()
    }
    fn at(&self, index: usize) -> &T {
        &self[index]
    }
}

/// Excel's binary search (XMATCH/XLOOKUP `search_mode` 2/-2) over `array`, sorted ascending (or
/// `descending`). `mode` is the match mode: `Equal` (0) only takes an equal element, `Less` (-1)
/// else the next smaller one and `Greater` (1) else the next larger one.
///
/// It is the classic search on inclusive bounds probing the lower middle `(lo + hi) / 2`, and it
/// compares every element with the `Ord` of its values, so a blank or an error is larger than
/// any number and is never skipped. It remembers the last equal element probed and keeps bisecting
/// past it, to the left when ascending and to the right when descending, so a gap inside a run ends
/// it; otherwise the answer is the last probe on the wanted side. On
/// unsorted data that is whatever those probes give, as in Excel: with 5,_,10,_,20,_ in A1:A6,
/// `XMATCH(10,A1:A6,0,2)` is 3 and `XMATCH(99,A:A,-1,2)` is 1 (probes/xmatch-xlookup-blanks
/// Col!M16-M26, Err!M6-M7, Row!A18; T-1019). O(log n) probes, so a virtual tail is never walked.
pub(crate) fn binary_search<T: Ord, S: Sequence<T> + ?Sized>(
    target: &T,
    array: &S,
    descending: bool,
    mode: Ordering,
) -> Option<usize> {
    let (mut smaller, mut larger, mut equal) = (None, None, None);
    // The live range is lo..hi (hi exclusive), so the lower middle of the inclusive bounds is
    // lo + (hi - 1 - lo) / 2.
    let mut lo = 0;
    let mut hi = array.seq_len();
    while lo < hi {
        let mid = lo + (hi - 1 - lo) / 2;
        let ordering = array.at(mid).cmp(target);
        match ordering {
            // An equal element is remembered and the search goes on past it, to the left when
            // ascending and to the right when descending, so a gap inside a run ends it: with
            // FALSE,_,TRUE,_,TRUE,_ in B1:B6 `XMATCH(TRUE,B1:B6,0,-2)` is 5 and with 40,30,40,40,50
            // `XMATCH(40,R1:R5,0,2)` is 1 (probes/xmatch-text-bool-equal-runs Txt!M21, M24, Run!M1-M42;
            // probes/xmatch-blank-extensions Ext!M38-M43).
            Ordering::Equal => {
                equal = Some(mid);
                if descending {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
                continue;
            }
            Ordering::Less => smaller = Some(mid),
            Ordering::Greater => larger = Some(mid),
        }
        if (ordering == Ordering::Less) != descending {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if equal.is_some() {
        return equal;
    }
    match mode {
        Ordering::Less => smaller,
        Ordering::Equal => None,
        Ordering::Greater => larger,
    }
}

impl Model {
    /// Returns an array with the list of cell values in the range
    pub(crate) fn prepare_array(
        &mut self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
        is_row_vector: bool,
    ) -> Vec<CalcResult> {
        let n = if is_row_vector {
            right.row - left.row
        } else {
            right.column - left.column
        } + 1;
        let mut result = Vec::with_capacity(usize::try_from(n).unwrap_or(0));
        for index in 0..n {
            let row;
            let column;
            if is_row_vector {
                row = left.row + index;
                column = left.column;
            } else {
                column = left.column + index;
                row = left.row;
            }
            let value = self.evaluate_cell(CellReferenceIndex {
                sheet: left.sheet,
                row,
                column,
            });
            result.push(value);
        }
        result
    }
}
