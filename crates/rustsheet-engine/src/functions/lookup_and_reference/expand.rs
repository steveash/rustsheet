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
        // `#VALUE!` (`EXPAND(A1:A2,3,1,{0;1})`, same case row 21; T-1835), and a range is read as a
        // value (a 1x1 array and a multi-cell range are unprobed, T-2365).
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
                    Ok(n) => n as usize,
                    Err(e) => return e,
                };
                let target_cols = match values.get(2) {
                    None => src_cols,
                    Some(value) => match model.cast_number_ref(value, cell) {
                        Ok(n) => n as usize,
                        Err(e) => return e,
                    },
                };
                // An array pad_with is `#VALUE!` for the whole call, not a fill of `#VALUE!`s
                // (probes/m7-lift-take-drop-sortby F21:H21: ROWS, COLUMNS and INDEX of it are
                // `#VALUE!`).
                // A reference is read as a value, as the scalar argument it was before the table
                // (a one-cell `Data!E$11`; a larger range is intersected).
                let pad = values.get(3).map(|pad| match pad {
                    CalcResult::Range { .. } => model.dereference(pad.clone(), cell),
                    other => other.clone(),
                });
                if matches!(pad, Some(CalcResult::Array(_))) {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "pad_with must be a single value".to_string(),
                    );
                }
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
