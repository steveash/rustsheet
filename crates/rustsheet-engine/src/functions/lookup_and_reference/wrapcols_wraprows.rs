use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{lift::LiftCost, Function},
    limits::check_stack_size,
    model::Model,
};

use crate::limits::SIZE_LIMIT_EXCLUSIVE;

impl Model {
    // ── WRAPCOLS ──────────────────────────────────────────────────────────────

    /// `=WRAPCOLS(vector, wrap_count, [pad_with])`
    ///
    /// Wraps the provided row or column vector by columns after a specified
    /// number of values. The vector is read top-to-bottom then left-to-right.
    ///   * `wrap_count` – number of values per column (must be >= 1)
    ///   * `pad_with`   – value used to pad the last column; defaults to #N/A
    pub(crate) fn fn_wrapcols(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_wrap(Function::Wrapcols, args, cell, wrapcols_kernel)
    }

    // ── WRAPROWS ──────────────────────────────────────────────────────────────

    /// `=WRAPROWS(vector, wrap_count, [pad_with])`
    ///
    /// Wraps the provided row or column vector by rows after a specified number
    /// of values. The vector is read top-to-bottom then left-to-right.
    ///   * `wrap_count` – number of values per row (must be >= 1)
    ///   * `pad_with`   – value used to pad the last row; defaults to #N/A
    pub(crate) fn fn_wraprows(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_wrap(Function::Wraprows, args, cell, wraprows_kernel)
    }

    // ── Shared helpers ────────────────────────────────────────────────────────

    /// Flatten the first argument of WRAPCOLS/WRAPROWS into a 1-D vector.
    /// Returns an error if the first argument is not a 1-D vector or range.
    fn flatten_vector(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<ArrayNode>, VectorError> {
        let result = self.evaluate_node_in_context(&args[0], cell);
        // A single empty cell is not a vector: `WRAPCOLS(Z1:Z1,2)` and `WRAPCOLS(Z1,2)` are `#VALUE!`
        // whatever pad_with is, while two empty cells are two 0s (probes/wrapcols-wraprows-pad-with
        // Z1 rows; it was this, not the error pad_with, behind gen-m9/675763979c1defb6 A6).
        let empty = match &result {
            CalcResult::EmptyCell | CalcResult::EmptyArg => true,
            CalcResult::Range { left, right } if left == right => {
                matches!(self.evaluate_cell(*left), CalcResult::EmptyCell)
            }
            _ => false,
        };
        if empty {
            // Unlike a 2-D array, it is `#VALUE!` even beside an error wrap_count
            // (`WRAPCOLS(Z1,NA())`, `WRAPROWS(Z1,1/0)`: probes/t2479-wrap-notvector-badcount G9, G10,
            // G35, G36), so it is every element's result like an error vector.
            return Err(VectorError::Error(CalcResult::new_error(
                Error::VALUE,
                cell,
                "An empty cell is not a vector".to_string(),
            )));
        }
        let data = self
            .result_to_array(result, cell)
            .map_err(VectorError::Error)?;
        let is_row_vec = data.len() == 1;
        let is_col_vec = data.iter().all(|r| r.len() == 1);
        if !is_row_vec && !is_col_vec {
            return Err(VectorError::NotVector(CalcResult::new_error(
                Error::VALUE,
                cell,
                "First argument must be a 1-D vector".to_string(),
            )));
        }
        let flat: Vec<ArrayNode> = if is_row_vec {
            data.into_iter().next().unwrap_or_default()
        } else {
            data.into_iter().map(|mut r| r.remove(0)).collect()
        };
        Ok(flat)
    }

    /// WRAPCOLS and WRAPROWS: the vector is read once and handed to every element whole; `wrap_count`
    /// and `pad_with` lift over an array and a range, one call per element whose value is that
    /// call's top-left (`ROWS(WRAPCOLS(A1:A3,{1;2}))` is 2, probes/m7-argument-modes F8:H8, G42,
    /// probes/m7-lift-take-drop-sortby rows 22-23; SPEC-0011 "Array-valued functions", T-1835).
    /// Each element builds the wrapped vector, so the call is charged its cells per element.
    fn lift_wrap(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        wrap: fn(Vec<ArrayNode>, usize, ArrayNode, CellReferenceIndex) -> CalcResult,
    ) -> CalcResult {
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        // A vector that is an error (or not a vector) is every element's result, before
        // wrap_count is read: it fills the lifted shape of the other arguments. A WRAPROWS whose
        // vector is `..-SIGN("ABC")`, wrap_count -1 and pad_with a 6x4 array is a 6x4 grid of
        // `#VALUE!`s in Excel, not one `#VALUE!` or `#NUM!` (gen-m7/90de8755a2e565e6 A889:D894);
        // with nothing lifted it stays the single error. A vector that is an error comes before
        // wrap_count, but a not-a-vector `#VALUE!` comes after an error wrap_count, which is the
        // result instead (probes/t2365-wrap-expand-tocol-error-lift G2, G12-G14; arrays.md).
        let flat = self.flatten_vector(args, cell);
        let cells = flat.as_ref().map_or(1, |flat| {
            u64::try_from(flat.len()).unwrap_or(u64::MAX).max(1)
        });
        self.lift_call_costed(
            kind,
            args,
            LiftCost::Cells(cells),
            cell,
            &mut |model, values| {
                let flat = match &flat {
                    Ok(flat) => flat,
                    Err(VectorError::Error(error)) => return error.clone(),
                    Err(VectorError::NotVector(error)) => {
                        return match values.get(1) {
                            Some(count) if count.is_error() => count.clone(),
                            _ => error.clone(),
                        };
                    }
                };
                let wrap_count = match model.wrap_count_of(values.get(1), flat.len(), cell) {
                    Ok(wrap_count) => wrap_count,
                    Err(error) => return error,
                };
                // A written empty pad_with is the missing one, `#N/A`
                // (`ISNA(INDEX(WRAPCOLS({1,2,3},2,),2,2))` is TRUE), while the empty value an
                // argument evaluates to pads blanks (`WRAPCOLS(..,IFERROR(Z1,1))`'s pad is
                // ISBLANK TRUE; probes/t2461-blank-array-sources A34, A35, A40:A42, A45).
                let pad = match args.get(2) {
                    Some(Node::EmptyArgKind) => None,
                    _ => values.get(2),
                };
                wrap(flat.clone(), wrap_count, pad_with_of(pad), cell)
            },
        )
    }

    /// The `wrap_count` value (2nd arg of WRAPCOLS/WRAPROWS) for a vector of `len` values: at
    /// least 1, and a count above the vector's length does not pad, it is the length
    /// (probes/dynamic-array-reshape-and-isomitted, arrays.md "WRAPROWS and WRAPCOLS"; T-1786).
    fn wrap_count_of(
        &self,
        value: Option<&CalcResult>,
        len: usize,
        cell: CellReferenceIndex,
    ) -> Result<usize, CalcResult> {
        // Below 1 is checked before truncation: 0.5 and 0 are `#NUM!` (arrays.md "WRAPCOLS of a
        // single value is not padded"; probes/dynamic-array-reshape-and-isomitted C20/C36/D20/D36 (0, -1),
        // gen-m7/f68fafa0ca2eba26 A39, gen-m9/50f10c4062fb2e98 A21).
        // A count of 2^31 or more is `#NUM!` too, before any clamp: 2^31 - 0.5
        // wraps, 2^31 does not (arrays.md "WRAPROWS and WRAPCOLS with a huge wrap_count";
        // probes/t2312-wrap-count-threshold).
        let wrap_count = self.cast_number_or(value, 0.0, cell)?;
        if wrap_count.is_nan() || !(1.0..SIZE_LIMIT_EXCLUSIVE).contains(&wrap_count) {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "wrap_count must be >= 1 and below 2^31".to_string(),
            ));
        }
        Ok((wrap_count as usize).min(len.max(1)))
    }
}

/// Why the vector argument of WRAPCOLS/WRAPROWS is not a flat vector.
enum VectorError {
    /// The argument is an error value, which is every element's result.
    Error(CalcResult),
    /// The `#VALUE!` for a 2-D array, which an error `wrap_count` replaces.
    NotVector(CalcResult),
}

/// The `pad_with` value (3rd arg of WRAPCOLS/WRAPROWS, 4th of EXPAND). An error `pad_with` is
/// a value like another: it fills the padded cells, and when nothing is padded the array is
/// returned (probes/wrapcols-wraprows-pad-with, `WRAPROWS({1,2,3,4},4,VALUE("x"))`). A missing
/// one is `#N/A`; any other non-scalar value (the callers never pass one) is `#VALUE!`. A blank
/// cell pads with blank elements, not 0: `ISBLANK` of them is TRUE for all three functions
/// (probes/t2413-blank-in-stacked-arrays A32, t2461-blank-array-sources A20, A22; T-2461), and
/// so does the empty value (`IFERROR(Z1,1)`, A40:A45). WRAPCOLS and WRAPROWS pass a written
/// empty argument as missing.
pub(super) fn pad_with_of(value: Option<&CalcResult>) -> ArrayNode {
    match value {
        None => ArrayNode::Error(Error::NA),
        Some(CalcResult::Number(n)) => ArrayNode::Number(*n),
        Some(CalcResult::Boolean(b)) => ArrayNode::Boolean(*b),
        Some(CalcResult::String(s)) => ArrayNode::String(s.clone()),
        Some(CalcResult::EmptyCell | CalcResult::EmptyArg) => ArrayNode::Empty,
        Some(CalcResult::Error { error, .. }) => ArrayNode::Error(error.clone()),
        Some(_) => ArrayNode::Error(Error::VALUE),
    }
}

/// One WRAPCOLS call over the flat vector.
fn wrapcols_kernel(
    flat: Vec<ArrayNode>,
    wrap_count: usize,
    pad: ArrayNode,
    cell: CellReferenceIndex,
) -> CalcResult {
    let n = flat.len();
    let num_cols = n.div_ceil(wrap_count);
    // The padded shape is wrap_count rows: refuse one above the cell bound before
    // allocating it (DEVIATION, arrays.md "Array results above 1,048,576 cells"; T-1786);
    // columns past the grid are allowed (arrays.md "Stacked results past the grid"; T-1967).
    if let Some((error, message)) = check_stack_size(wrap_count, num_cols) {
        return CalcResult::new_error(error, cell, message);
    }

    let mut result: Vec<Vec<ArrayNode>> = vec![Vec::with_capacity(num_cols); wrap_count];
    for (i, val) in flat.into_iter().enumerate() {
        result[i % wrap_count].push(val);
    }
    // Pad the last (partial) column
    for row in &mut result {
        while row.len() < num_cols {
            row.push(pad.clone());
        }
    }

    CalcResult::Array(result)
}

/// One WRAPROWS call over the flat vector.
fn wraprows_kernel(
    flat: Vec<ArrayNode>,
    wrap_count: usize,
    pad: ArrayNode,
    cell: CellReferenceIndex,
) -> CalcResult {
    // The padded shape is wrap_count columns: refuse one above the cell bound before
    // allocating a row (T-1786, as WRAPCOLS).
    if let Some((error, message)) = check_stack_size(flat.len().div_ceil(wrap_count), wrap_count) {
        return CalcResult::new_error(error, cell, message);
    }

    let mut result: Vec<Vec<ArrayNode>> = Vec::new();
    let mut chunk = Vec::with_capacity(wrap_count);
    for val in flat {
        chunk.push(val);
        if chunk.len() == wrap_count {
            result.push(chunk);
            chunk = Vec::with_capacity(wrap_count);
        }
    }
    if !chunk.is_empty() {
        while chunk.len() < wrap_count {
            chunk.push(pad.clone());
        }
        result.push(chunk);
    }

    CalcResult::Array(result)
}
