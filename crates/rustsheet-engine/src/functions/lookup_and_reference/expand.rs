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

use super::wrapcols_wraprows::pad_with_of;
use super::SIZE_LIMIT_EXCLUSIVE;

impl Model {
    // ── EXPAND ────────────────────────────────────────────────────────────────

    /// `=EXPAND(array, rows, [columns], [pad_with])`
    ///
    /// Expands or pads an array to the specified dimensions.
    ///   * rows    – target row count (must be >= current rows)
    ///   * columns – target column count (must be >= current columns); defaults to current columns
    ///   * `pad_with` – value to fill new cells with; defaults to #N/A
    pub(crate) fn fn_expand(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let kind = Function::Expand;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }

        // The array is read once and handed to every element whole; rows and columns lift over
        // an array and a range, one EXPAND per element whose value is that call's top-left
        // (`ROWS(EXPAND(A1:A2,{3;4}))` is 2, probes/m7-argument-modes F7:H7,
        // probes/m7-lift-take-drop-sortby row 20). pad_with does not lift: an array there is
        // `#VALUE!` (`EXPAND(A1:A2,3,1,{0;1})`, same case row 21; T-1835), and so is a multi-cell
        // range (T-2415); a 1x1 array is unprobed (T-2365).
        let data = match self.eval_to_array(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let src_rows = data.len();
        let src_cols = data.first().map(|r| r.len()).unwrap_or(0);
        let cells = u64::try_from(src_rows.saturating_mul(src_cols))
            .unwrap_or(u64::MAX)
            .max(1);

        self.lift_call_costed(
            kind,
            args,
            LiftCost::Cells(cells),
            cell,
            &mut |model, values| {
                let target_rows = match model.cast_number_or(values.get(1), 0.0, cell) {
                    Ok(n) => n,
                    Err(e) => return e,
                };
                let target_cols = match values.get(2) {
                    None => src_cols as f64,
                    Some(value) => match model.cast_number_ref(value, cell) {
                        Ok(n) => n,
                        Err(e) => return e,
                    },
                };
                // A size of 2^31 or more is `#VALUE!`, while a smaller one past the grid is
                // `#NUM!` (probes/expand-makearray-past-grid A1:A13, -big A1:B3).
                if target_rows >= SIZE_LIMIT_EXCLUSIVE || target_cols >= SIZE_LIMIT_EXCLUSIVE {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "rows and columns must be less than 2^31".to_string(),
                    );
                }
                let (target_rows, target_cols) = (target_rows as usize, target_cols as usize);
                // An array pad_with is `#VALUE!` for the whole call, not a fill of `#VALUE!`s
                // (probes/m7-lift-take-drop-sortby F21:H21: ROWS, COLUMNS and INDEX of it are
                // `#VALUE!`).
                // A multi-cell range is `#VALUE!` for the whole call too, a row, a column or a 2-D
                // one, whether or not any cell is padded (gen-m7/2d9b48c6363e7a16 A366, A830,
                // A2686; arrays.md "EXPAND, WRAPROWS and WRAPCOLS"; T-2415). A one-cell reference
                // is its value.
                let pad = values.get(3);
                if pad.is_some_and(|pad| {
                    matches!(pad, CalcResult::Array(_) | CalcResult::Range { .. })
                        && pad.is_multi_value()
                }) {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "pad_with must be a single value".to_string(),
                    );
                }
                let pad = pad.map(|pad| match pad {
                    CalcResult::Range { .. } => model.dereference(pad.clone(), cell),
                    other => other.clone(),
                });
                expand_kernel(
                    &data,
                    target_rows,
                    target_cols,
                    pad_with_of(pad.as_ref()),
                    cell,
                )
            },
        )
    }
}

/// The least columns Excel is known to refuse with `#NUM!` (probes/expand-makearray-past-grid-big
/// A1); its exact cap lies between 16,386 and this (T-2452).
const COLUMNS_REFUSED: usize = 123_456_789;

/// One EXPAND call: `data` padded with `pad` to `target_rows` x `target_cols`.
fn expand_kernel(
    data: &[Vec<ArrayNode>],
    target_rows: usize,
    target_cols: usize,
    pad: ArrayNode,
    cell: CellReferenceIndex,
) -> CalcResult {
    let src_rows = data.len();
    let src_cols = data.first().map(|r| r.len()).unwrap_or(0);
    if target_rows < src_rows {
        return CalcResult::new_error(
            Error::VALUE,
            cell,
            "rows must be >= array row count".to_string(),
        );
    }
    if target_cols < src_cols {
        return CalcResult::new_error(
            Error::VALUE,
            cell,
            "columns must be >= array column count".to_string(),
        );
    }

    // Excel refuses columns past a cap of its own after the sizes above (16,385 are built,
    // 123,456,789 are `#NUM!` even in one row: probes/expand-makearray-past-grid-big A1, B3; a
    // negative rows is `#VALUE!` first: gen-m7/bc4a69b0b512bc04 A1430); ours refuses from the
    // smallest refused size probed, and builds below it up to the cell bound.
    if target_cols >= COLUMNS_REFUSED {
        return CalcResult::new_error(
            Error::NUM,
            cell,
            "columns exceeds EXPAND's limit".to_string(),
        );
    }

    // Rows past the grid are `#NUM!` and columns past it are allowed, as for VSTACK (arrays.md
    // "Stacked results past the grid"; T-1967); a target above the cell bound is refused before
    // padding it (DEVIATION, arrays.md "Array results above 1,048,576 cells"; T-1786).
    if let Some((error, message)) = check_stack_size(target_rows, target_cols) {
        return CalcResult::new_error(error, cell, message);
    }

    let mut result: Vec<Vec<ArrayNode>> = Vec::with_capacity(target_rows);
    for r in 0..target_rows {
        let mut row: Vec<ArrayNode> = Vec::with_capacity(target_cols);
        for c in 0..target_cols {
            let val = data
                .get(r)
                .and_then(|src| src.get(c))
                .cloned()
                .unwrap_or_else(|| pad.clone());
            row.push(val);
        }
        result.push(row);
    }

    CalcResult::Array(result)
}
