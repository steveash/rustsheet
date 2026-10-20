use super::drop_take::sub_range;
use super::ref_lift::lifts;
use crate::reference_operators::RefArg;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};
use std::ops::Range;

/// Returns true if an `ArrayNode` counts as a blank/empty cell for TRIMRANGE purposes.
fn is_blank(node: &ArrayNode) -> bool {
    matches!(node, ArrayNode::Empty)
}

/// The first and last row and column of the source that hold a value, from one pass over its
/// cells, so that each element of a lifted call picks its bounds in constant time. `None` when
/// every cell is blank.
struct NonBlank {
    rows: Range<usize>,
    columns: Range<usize>,
}

impl NonBlank {
    fn of(data: &[Vec<ArrayNode>]) -> Option<NonBlank> {
        let mut found: Option<NonBlank> = None;
        for (r, row) in data.iter().enumerate() {
            for (c, node) in row.iter().enumerate() {
                if is_blank(node) {
                    continue;
                }
                found = Some(match found {
                    None => NonBlank {
                        rows: r..r + 1,
                        columns: c..c + 1,
                    },
                    Some(seen) => NonBlank {
                        rows: seen.rows.start.min(r)..seen.rows.end.max(r + 1),
                        columns: seen.columns.start.min(c)..seen.columns.end.max(c + 1),
                    },
                });
            }
        }
        found
    }
}

/// A trim argument as an integer; a fractional one (even `0.9` or `3.9`) is out of range, like
/// `4` and `-1` (docs/excel-semantics/lookup.md "TRIMRANGE with a fractional trim argument").
fn trim_mode(n: f64) -> i32 {
    if n.fract() == 0.0 && (0.0..=3.0).contains(&n) {
        n as i32
    } else {
        -1
    }
}

impl Model {
    /// `=TRIMRANGE(range, [trim_rows], [trim_cols])`
    ///
    /// Trims blank rows and/or columns from the outer edges of a range or array.
    ///
    /// `trim_rows` / `trim_cols`:
    ///   * 0 – no trimming
    ///   * 1 – trim leading (top / left) blanks only
    ///   * 2 – trim trailing (bottom / right) blanks only
    ///   * 3 – trim both leading and trailing blanks (default)
    ///
    /// Returns `#REF!` if every cell in the range is blank.
    pub(crate) fn fn_trimrange(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.trimrange_reference(args, cell, false)
            .into_single_area(cell)
    }

    /// TRIMRANGE; `as_reference` gives the trimmed part of a single-sheet range argument as a
    /// reference, as an intersection operand sees it: `=A1:A3 TRIMRANGE(A2:B2)` is 2
    /// (probes/intersection-function-operands H14; T-0165). When `trim_rows` or `trim_cols` is an
    /// array or a multi-cell range the call lifts into an array of references, each the scalar
    /// call's trimmed reference of the source range or its error (SPEC-0014; docs/excel-semantics/
    /// lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references").
    pub(crate) fn trimrange_reference(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> RefArg {
        if args.is_empty() || args.len() > 3 {
            return RefArg::Value(CalcResult::new_args_number_error(cell));
        }

        let source = if as_reference {
            self.evaluate_picked_source(&args[0], cell, Self::evaluate_node_in_context)
        } else {
            self.evaluate_node_in_context(&args[0], cell)
        };
        let reference_corner = match source {
            CalcResult::Range { left, right } if left.sheet == right.sheet => Some(left),
            _ => None,
        };
        let data = match self.result_to_array(source, cell) {
            Ok(d) => d,
            Err(e) => return RefArg::Value(e),
        };

        if data.is_empty() || data[0].is_empty() {
            return RefArg::Value(CalcResult::new_error(
                Error::REF,
                cell,
                "Range is empty".to_string(),
            ));
        }

        // A missing trim argument is 3; a written empty one is 0.
        let non_blank = NonBlank::of(&data);
        let values = self.reference_lift_values(&args[1..], cell);
        if !values.as_slice().iter().any(lifts) {
            return match self.trim_of(&data, non_blank.as_ref(), values.as_slice(), cell) {
                Ok((rows, columns)) => match reference_corner {
                    Some(left) if as_reference => RefArg::from(sub_range(left, &rows, &columns)),
                    _ => RefArg::Value(CalcResult::Array(
                        data[rows]
                            .iter()
                            .map(|row| row[columns.clone()].to_vec())
                            .collect(),
                    )),
                },
                Err(error) => RefArg::Value(error),
            };
        }
        // A one-cell reference evaluates to its value, but its trim is still a reference
        // (`INDEX(TRIMRANGE(A1,{0;1}),2,1)` is A1; probes/reference-lift-elements).
        let reference_corner = reference_corner.or_else(|| match args[0].ungrouped() {
            node @ Node::ReferenceKind { .. } => match self.get_reference(node, cell) {
                Ok(range) => Some(range.left),
                Err(_) => None,
            },
            _ => None,
        });
        let lifted = match self.reference_lift_args(values, cell) {
            Ok(lifted) => lifted,
            Err(error) => return RefArg::Value(error),
        };
        self.lift_references(lifted, cell, &mut |model, values| {
            match (
                model.trim_of(&data, non_blank.as_ref(), values, cell),
                reference_corner,
            ) {
                (Ok((rows, columns)), Some(left)) => sub_range(left, &rows, &columns),
                // The trim of an array is a value, not a reference.
                (Ok(_), None) => CalcResult::new_error(Error::VALUE, cell, String::new()),
                (Err(error), _) => error,
            }
        })
    }

    /// Whether a TRIMRANGE call lifts into an array of references, its own arguments evaluated
    /// but no element computed: ISREF's question.
    pub(crate) fn trimrange_lifts(&mut self, args: &[Node], cell: CellReferenceIndex) -> bool {
        (1..=3).contains(&args.len())
            && self
                .reference_lift_values(&args[1..], cell)
                .as_slice()
                .iter()
                .any(lifts)
    }

    /// The rows and columns of `data` that remain after trimming by the values of `trim_rows` and
    /// `trim_cols` (`values`; a missing one is 3), or the scalar call's error.
    fn trim_of(
        &mut self,
        data: &[Vec<ArrayNode>],
        non_blank: Option<&NonBlank>,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(Range<usize>, Range<usize>), CalcResult> {
        let mode = |model: &mut Self, value: Option<&CalcResult>| match value {
            Some(value) => model.cast_to_number(value.clone(), cell).map(trim_mode),
            None => Ok(3),
        };
        let trim_rows = mode(self, values.first())?;
        let trim_cols = mode(self, values.get(1))?;

        if !matches!(trim_rows, 0..=3) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "trim_rows must be 0, 1, 2, or 3".to_string(),
            ));
        }
        if !matches!(trim_cols, 0..=3) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "trim_cols must be 0, 1, 2, or 3".to_string(),
            ));
        }

        let (num_rows, num_cols) = (data.len(), data[0].len());
        // Trimming a mode removes leading blanks (1), trailing (2) or both (3); with no value
        // at all the leading edge is past the end and the trailing one before the start.
        let edges = |mode: i32, found: Option<&Range<usize>>, len: usize| {
            let start = if mode == 1 || mode == 3 {
                found.map_or(len, |found| found.start)
            } else {
                0
            };
            let end = if mode == 2 || mode == 3 {
                found.map_or(0, |found| found.end)
            } else {
                len
            };
            (start, end)
        };
        let (row_start, row_end) = edges(trim_rows, non_blank.map(|n| &n.rows), num_rows);
        let (col_start, col_end) = edges(trim_cols, non_blank.map(|n| &n.columns), num_cols);

        if row_start >= row_end || col_start >= col_end {
            return Err(CalcResult::new_error(
                Error::REF,
                cell,
                "TRIMRANGE result is empty".to_string(),
            ));
        }

        Ok((row_start..row_end, col_start..col_end))
    }
}
