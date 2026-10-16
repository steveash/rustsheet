//! The two arguments of a function that pairs values by position: SUMXMY2 and its siblings,
//! COVARIANCE, CHISQ.TEST, PEARSON, CORREL, the regression family and paired T.TEST.
//!
//! A whole-column range of an almost empty sheet has millions of cells, so the lists must not
//! hold one entry per cell (T-1681, directive 5). Past the used area every cell is blank, and
//! every paired caller treats a pair with a blank side as no data, so the lists stop where the
//! data of both arguments stops: they are the same length, and pairing them by position gives the
//! result the full lists gave.

use crate::calc_result::{CalcResult, Range};
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;

/// One argument of a paired function, as evaluated and not yet flattened.
pub(crate) enum PairedArg {
    /// A reference: walked here, only as far as its sheet's used area.
    Range(Range),
    /// An array value: flattened here, so its error comes after the first argument's.
    Array(Vec<Vec<ArrayNode>>),
    /// Values the caller flattened itself (a scalar typed in place).
    Values(Vec<Option<f64>>),
    /// An error value: returned when its turn comes, after the first argument's errors.
    Error(CalcResult),
}

/// The flattened arguments of a paired function, row-major: a number is `Some`, anything else
/// `None`. Both lists have the same length, and every position left out is blank in at least one
/// argument (or both, when the sizes match).
pub(crate) struct PairedValues {
    pub(crate) first: Vec<Option<f64>>,
    pub(crate) second: Vec<Option<f64>>,
    /// Whether the arguments have the same number of cells. When they do not, the lists pair the
    /// positions the shorter argument has, as zipping full lists would.
    pub(crate) same_size: bool,
}

/// How far an argument reaches: its cells, and the row-major prefix of them that can hold data.
struct Extent {
    cells: u64,
    data: u64,
}

impl Model {
    /// The two samples of COVARIANCE.P, COVARIANCE.S or T.TEST (`args[0]` and `args[1]`, which
    /// the caller checked exist), paired by position. Each must be a range or an array: anything
    /// else is `#VALUE!`, and an array holding an error is `#VALUE!`, each when its turn comes.
    pub(crate) fn paired_samples(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<PairedValues, CalcResult> {
        let first = self.sample_arg(&args[0], "First", cell);
        let second = self.sample_arg(&args[1], "Second", cell);
        self.paired_values(first, second, cell, |index, error| {
            let which = if index == 0 { "first" } else { "second" };
            CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("Error in {which} array: {error:?}"),
            )
        })
    }

    fn sample_arg(&mut self, arg: &Node, which: &str, cell: CellReferenceIndex) -> PairedArg {
        match self.evaluate_node_in_context(arg, cell) {
            CalcResult::Range { left, right } => PairedArg::Range(Range { left, right }),
            CalcResult::Array(array) => PairedArg::Array(array),
            _ => PairedArg::Error(CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("{which} argument must be a range or array"),
            )),
        }
    }

    /// Flattens `first` and `second` for a function that pairs them by position. The first
    /// argument is read before the second, so its first error is the one returned; an array's
    /// error is mapped by `array_error` (argument index, element error). Every cell of a range's
    /// used area is read, so an error past the paired positions still surfaces.
    pub(crate) fn paired_values(
        &mut self,
        first: PairedArg,
        second: PairedArg,
        cell: CellReferenceIndex,
        array_error: impl Fn(usize, Error) -> CalcResult,
    ) -> Result<PairedValues, CalcResult> {
        let mut args = [first, second];
        let mut extents = [Extent { cells: 0, data: 0 }, Extent { cells: 0, data: 0 }];
        for (arg, extent) in args.iter_mut().zip(&mut extents) {
            *extent = self.paired_extent(arg, cell)?;
        }
        let same_size = extents[0].cells == extents[1].cells;
        let paired = extents[0].cells.min(extents[1].cells);
        let keep = paired.min(extents[0].data.max(extents[1].data));
        let [first, second] = args;
        let first = self.paired_list(first, keep, cell, |error| array_error(0, error))?;
        let second = self.paired_list(second, keep, cell, |error| array_error(1, error))?;
        Ok(PairedValues {
            first,
            second,
            same_size,
        })
    }

    fn paired_extent(
        &mut self,
        arg: &mut PairedArg,
        cell: CellReferenceIndex,
    ) -> Result<Extent, CalcResult> {
        let count = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
        Ok(match arg {
            PairedArg::Range(Range { left, right }) if left.sheet == right.sheet => {
                let span = |from: i32, to: i32| u64::try_from(to - from + 1).unwrap_or(0);
                let (last_row, _) = self
                    .used_area_corner(left.sheet, left.row, left.column, right.row, right.column)
                    .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
                let columns = span(left.column, right.column);
                Extent {
                    cells: span(left.row, right.row) * columns,
                    data: span(left.row, last_row) * columns,
                }
            }
            // A 3-D reference is walked sheet by sheet over each sheet's used area, so its list
            // is all it has (T-0196); it is read now, to know its size.
            PairedArg::Range(Range { left, right }) => {
                let values = self.values_from_range(*left, *right)?;
                let n = count(values.len());
                *arg = PairedArg::Values(values);
                Extent { cells: n, data: n }
            }
            PairedArg::Array(array) => {
                let n = count(array.iter().map(Vec::len).sum());
                Extent { cells: n, data: n }
            }
            PairedArg::Values(values) => {
                let n = count(values.len());
                Extent { cells: n, data: n }
            }
            PairedArg::Error(_) => Extent { cells: 0, data: 0 },
        })
    }

    /// The first `keep` positions of `arg`, row-major.
    fn paired_list(
        &mut self,
        arg: PairedArg,
        keep: u64,
        cell: CellReferenceIndex,
        array_error: impl Fn(Error) -> CalcResult,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        let keep = usize::try_from(keep).unwrap_or(usize::MAX);
        let mut values = match arg {
            PairedArg::Range(Range { left, right }) => {
                return self.range_prefix(left, right, keep, cell);
            }
            PairedArg::Array(array) => self.values_from_array(array).map_err(array_error)?,
            PairedArg::Values(values) => values,
            PairedArg::Error(error) => return Err(error),
        };
        values.truncate(keep);
        Ok(values)
    }

    /// The first `keep` cells of a single-sheet range, row-major. Cells past the used area are
    /// blank and not read; every cell inside it is, so its first error is found.
    fn range_prefix(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        keep: usize,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        // The rows and columns past the used area are recorded as read empty (dynamic arrays).
        let (last_row, last_column) = self
            .clip_to_used_area(left.sheet, left.row, left.column, right.row, right.column)
            .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
        let mut values = Vec::with_capacity(keep);
        // Two exits: once `keep` entries are held, a row below the used area ends the walk, and a
        // column right of it ends its row; until then those blank cells are pushed unread.
        for row in left.row..=right.row {
            if row > last_row && values.len() >= keep {
                break;
            }
            for column in left.column..=right.column {
                let value = if row > last_row || column > last_column {
                    None
                } else {
                    match self.evaluate_cell(CellReferenceIndex {
                        sheet: left.sheet,
                        row,
                        column,
                    }) {
                        CalcResult::Number(v) => Some(v),
                        error @ CalcResult::Error { .. } => return Err(error),
                        _ => None,
                    }
                };
                if values.len() < keep {
                    values.push(value);
                } else if row > last_row || column > last_column {
                    break;
                }
            }
        }
        Ok(values)
    }
}
