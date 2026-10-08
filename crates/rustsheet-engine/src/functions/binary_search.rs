use std::cmp::Ordering;

use crate::{calc_result::CalcResult, expressions::types::CellReferenceIndex, model::Model};

/// Random access to a sorted sequence, so the binary searches also run over
/// a `SearchVector` whose empty tail is virtual (T-0127).
pub(crate) trait Sequence<T: Ord> {
    fn seq_len(&self) -> usize;
    /// The element at `index`, which must be below `seq_len`.
    fn at(&self, index: usize) -> &T;
    /// The index from which every element equals the last one (a virtual run
    /// of empty cells), or `seq_len` if there is no such run. It lets the walks
    /// over equal elements jump the run instead of stepping a million cells.
    fn uniform_tail_start(&self) -> usize {
        self.seq_len()
    }
}

/// From `m`, whose element equals `target`, the start of the run of equal
/// elements (never below 1, as before T-0127).
fn walk_back<T: Ord, S: Sequence<T> + ?Sized>(target: &T, array: &S, mut m: usize) -> usize {
    let tail = array.uniform_tail_start();
    if m > tail {
        m = tail.max(1);
    }
    while m > 1 && array.at(m - 1) == target {
        m -= 1;
    }
    m
}

/// From `index`, whose element equals `target`, the end of the run of equal
/// elements.
fn walk_forward<T: Ord, S: Sequence<T> + ?Sized>(target: &T, array: &S, mut index: usize) -> usize {
    let n = array.seq_len();
    let tail = array.uniform_tail_start();
    while index + 1 < n && array.at(index + 1) == target {
        if index + 1 >= tail {
            return n - 1;
        }
        index += 1;
    }
    index
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

/// What a binary search (XMATCH/XLOOKUP search_mode 2/-2) found: an equal element, else the
/// last probed element smaller and larger than the target.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct BinaryProbe {
    pub exact: Option<usize>,
    pub smaller: Option<usize>,
    pub larger: Option<usize>,
}

impl BinaryProbe {
    /// The match for an exact (0), exact-or-smaller (-1) or exact-or-larger (1) match mode.
    pub(crate) fn pick(&self, smaller: bool, larger: bool) -> Option<usize> {
        self.exact
            .or(if smaller { self.smaller } else { None })
            .or(if larger { self.larger } else { None })
    }
}

/// Excel's binary search over `array`, sorted ascending (or `descending`): the classic search on
/// inclusive bounds probing the lower middle `(lo + hi) / 2`, comparing every element with the
/// `Ord` of its values, so a blank or an error is larger than any number and is never skipped.
/// It ends at the first equal element, moved to the start of its run of equal elements (the
/// end when `descending`); otherwise it reports the last probes on either side. On unsorted data
/// the answer is whatever those probes give, as in Excel: with 5,_,10,_,20,_ in A1:A6,
/// `XMATCH(10,A1:A6,0,2)` is 3 and `XMATCH(99,A:A,-1,2)` is 1 (probes/xmatch-xlookup-blanks
/// Col!M16-M26, Err!M6-M7, Row!A18; T-1019). O(log n) probes, so a virtual tail is never walked.
pub(crate) fn binary_search<T: Ord, S: Sequence<T> + ?Sized>(
    target: &T,
    array: &S,
    descending: bool,
) -> BinaryProbe {
    let mut probe = BinaryProbe::default();
    // The live range is lo..hi (hi exclusive), so the lower middle of the inclusive bounds is
    // lo + (hi - 1 - lo) / 2.
    let mut lo = 0;
    let mut hi = array.seq_len();
    while lo < hi {
        let mid = lo + (hi - 1 - lo) / 2;
        let ordering = array.at(mid).cmp(target);
        match ordering {
            Ordering::Equal => {
                probe.exact = Some(if descending {
                    walk_forward(target, array, mid)
                } else {
                    walk_back(target, array, mid)
                });
                return probe;
            }
            Ordering::Less => probe.smaller = Some(mid),
            Ordering::Greater => probe.larger = Some(mid),
        }
        if (ordering == Ordering::Less) != descending {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    probe
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
