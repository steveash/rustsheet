//! The two arguments of a function that pairs values by position: SUMXMY2 and its siblings,
//! COVARIANCE, CHISQ.TEST, PEARSON, CORREL, the regression family and paired T.TEST.
//!
//! A whole-column range has a million cells per column, and an untrusted workbook can put one cell
//! far away to stretch the used area, so the lists must not hold an entry per cell (T-1681,
//! directive 5). They hold an entry for each position where either argument has a non-blank
//! cell: a position blank in both is a pair every paired caller treats as no data.

use crate::calc_result::{CalcResult, Range};
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;

/// One argument of a paired function, as evaluated and not yet flattened.
pub(crate) enum PairedArg {
    /// A reference: walked here, over its sheet's used area.
    Range(Range),
    /// An array value: flattened here, so its error comes after the first argument's.
    Array(Vec<Vec<ArrayNode>>),
    /// Values the caller flattened itself (a scalar typed in place).
    Values(Vec<Option<f64>>),
    /// An error value: returned when its turn comes, after the first argument's errors.
    Error(CalcResult),
}

impl PairedArg {
    /// The argument a range, array, number or error value stands for. A number is an array of one
    /// value and an error is itself (statistics.md "Array arguments given a scalar, a single cell
    /// or an error"); any other value is handed back for the caller's own fallback.
    pub(crate) fn from_result(value: CalcResult) -> Result<Self, CalcResult> {
        match value {
            CalcResult::Range { left, right } => Ok(Self::Range(Range { left, right })),
            CalcResult::Array(array) => Ok(Self::Array(array)),
            CalcResult::Number(n) => Ok(Self::Values(vec![Some(n)])),
            error @ CalcResult::Error { .. } => Ok(Self::Error(error)),
            other => Err(other),
        }
    }

    /// As [`Self::from_result`] for the value of `node`: a single cell holding text, a boolean
    /// or nothing is an array of one cell with no number (`CHISQ.TEST(B4,C5:D11)` with B4 text is
    /// `#N/A`, gen-m9 a2e51f48717af2a1 A34), which a typed text or boolean is not (unprobed).
    pub(crate) fn from_node_result(node: &Node, value: CalcResult) -> Result<Self, CalcResult> {
        match Self::from_result(value) {
            Err(CalcResult::String(_) | CalcResult::Boolean(_) | CalcResult::EmptyCell)
                if matches!(node.ungrouped(), Node::ReferenceKind { .. }) =>
            {
                Ok(Self::Values(vec![None]))
            }
            other => other,
        }
    }
}

/// The flattened arguments of a paired function, aligned by row-major position: a number is
/// `Some`, anything else `None`. Positions blank in both arguments are left out, so the lists
/// hold every number of both arguments and have the same length.
pub(crate) struct PairedValues {
    pub(crate) first: Vec<Option<f64>>,
    pub(crate) second: Vec<Option<f64>>,
    /// Whether the arguments have the same number of cells.
    pub(crate) same_size: bool,
    /// How many leading entries are at positions both arguments have (all of them when the
    /// sizes match): the pairs that zipping the full lists would make.
    pub(crate) paired: usize,
}

/// The non-blank entries of one argument: (row-major position, value), in position order.
type Sparse = Vec<(u64, Option<f64>)>;

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
        // A typed text is an array of one cell with no number, so against a longer range the sizes
        // differ and the result is `#N/A`: `COVAR(A1:A6,"a")`, COVARIANCE.P and .S too (statistics.md
        // "Paired regression functions", probes/covariance-pairs-skip L8, L16; T.TEST's is unprobed).
        let value = self.evaluate_node_in_context(arg, cell);
        if matches!(value, CalcResult::String(_)) {
            return PairedArg::Values(vec![None]);
        }
        PairedArg::from_node_result(arg, value).unwrap_or_else(|_| {
            PairedArg::Error(CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("{which} argument must be a range or array"),
            ))
        })
    }

    /// Flattens `first` and `second` for a function that pairs them by position. The first
    /// argument is read before the second, so its first error is the one returned; an array's
    /// error is mapped by `array_error` (argument index, element error).
    pub(crate) fn paired_values(
        &mut self,
        first: PairedArg,
        second: PairedArg,
        cell: CellReferenceIndex,
        array_error: impl Fn(usize, Error) -> CalcResult,
    ) -> Result<PairedValues, CalcResult> {
        let (first_cells, first) = self.sparse_values(first, cell, |e| array_error(0, e))?;
        let (second_cells, second) = self.sparse_values(second, cell, |e| array_error(1, e))?;
        let shared = first_cells.min(second_cells);
        // Merge the two position lists; a position missing from one side is blank there.
        let mut values = PairedValues {
            first: Vec::new(),
            second: Vec::new(),
            same_size: first_cells == second_cells,
            paired: 0,
        };
        let (mut a, mut b) = (first.into_iter().peekable(), second.into_iter().peekable());
        loop {
            let (position, x, y) = match (a.peek(), b.peek()) {
                (None, None) => break,
                (Some(&(i, x)), Some(&(j, _))) if i < j => {
                    a.next();
                    (i, x, None)
                }
                (Some(&(i, x)), Some(&(j, y))) if i == j => {
                    a.next();
                    b.next();
                    (i, x, y)
                }
                (_, Some(&(j, y))) => {
                    b.next();
                    (j, None, y)
                }
                (Some(&(i, x)), None) => {
                    a.next();
                    (i, x, None)
                }
            };
            if position < shared {
                values.paired += 1;
            }
            values.first.push(x);
            values.second.push(y);
        }
        Ok(values)
    }

    /// The number of cells of `arg` and its non-blank entries. A range is walked over its
    /// sheet's used area, so a blank costs time, as in SUM, but no memory.
    fn sparse_values(
        &mut self,
        arg: PairedArg,
        cell: CellReferenceIndex,
        array_error: impl Fn(Error) -> CalcResult,
    ) -> Result<(u64, Sparse), CalcResult> {
        let dense = |values: Vec<Option<f64>>| {
            let cells = u64::try_from(values.len()).unwrap_or(u64::MAX);
            (cells, (0..).zip(values).collect())
        };
        match arg {
            PairedArg::Range(Range { left, right }) if left.sheet == right.sheet => {
                let span = |from: i32, to: i32| u64::try_from(to - from + 1).unwrap_or(0);
                let width = span(left.column, right.column);
                let mut entries = Vec::new();
                self.for_each_position_in_areas(&[Range { left, right }], cell, |model, at| {
                    let value = match model.evaluate_cell(at) {
                        CalcResult::EmptyCell => return Ok(()),
                        CalcResult::Number(v) => Some(v),
                        error @ CalcResult::Error { .. } => return Err(error),
                        _ => None,
                    };
                    let offset = |from: i32, to: i32| u64::try_from(to - from).unwrap_or(0);
                    let position =
                        offset(left.row, at.row) * width + offset(left.column, at.column);
                    entries.push((position, value));
                    Ok(())
                })?;
                Ok((span(left.row, right.row) * width, entries))
            }
            // A 3-D reference is walked sheet by sheet over each sheet's existing cells (T-0196):
            // its list is all it has, so positions count in that list.
            PairedArg::Range(Range { left, right }) => {
                Ok(dense(self.values_from_range(left, right)?))
            }
            PairedArg::Array(array) => {
                Ok(dense(self.values_from_array(array).map_err(array_error)?))
            }
            PairedArg::Values(values) => Ok(dense(values)),
            PairedArg::Error(error) => Err(error),
        }
    }
}
