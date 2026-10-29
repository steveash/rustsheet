use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{lift::LiftCost, Function},
    model::Model,
};

impl Model {
    // ── TOCOL ─────────────────────────────────────────────────────────────────

    /// `=TOCOL(array, [ignore], [scan_by_col])`
    ///
    /// Flattens an array into a single column.
    ///   * ignore: 0=keep all (default), 1=ignore blanks, 2=ignore errors, 3=ignore both
    ///   * `scan_by_col`: FALSE=row-by-row (default), TRUE=column-by-column
    pub(crate) fn fn_tocol(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_flatten(Function::Tocol, args, cell, |flat| {
            flat.into_iter().map(|v| vec![v]).collect()
        })
    }

    // ── TOROW ─────────────────────────────────────────────────────────────────

    /// `=TOROW(array, [ignore], [scan_by_col])`
    ///
    /// Flattens an array into a single row.
    ///   * ignore: 0=keep all (default), 1=ignore blanks, 2=ignore errors, 3=ignore both
    ///   * `scan_by_col`: FALSE=row-by-row (default), TRUE=column-by-column
    pub(crate) fn fn_torow(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_flatten(Function::Torow, args, cell, |flat| vec![flat])
    }

    /// TOCOL and TOROW: the array is read once and handed to every element whole; ignore and
    /// `scan_by_column` lift over an array and a range (SPEC-0011 "Array-valued functions"; TOCOL
    /// T-1836, TOROW T-2313: `TOROW(Data!B3:E8,3,WRAPCOLS(Data!D3:D11,1))` is one element per
    /// `scan_by_column` entry, gen-m7/7d67c559ce7536fa A619:E619). Each element walks the array, so
    /// the call is charged its cells per element; `shape` lays the flat values out.
    fn lift_flatten(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        shape: fn(Vec<ArrayNode>) -> Vec<Vec<ArrayNode>>,
    ) -> CalcResult {
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        // An array that is an error is every element's result: `TOROW(<#NUM!>,3,<5x3 SCAN>)`
        // spills 15 `#NUM!`s (gen-m7/611ce3bbfab27cef A919:C923); unlifted it is the one error.
        let data = self.eval_array_arg_keep_blank(&args[0], cell);
        let cells = data.as_ref().map_or(1, |data| {
            let cells = data.iter().map(Vec::len).sum::<usize>();
            u64::try_from(cells).unwrap_or(u64::MAX).max(1)
        });
        self.lift_call_costed(
            kind,
            args,
            LiftCost::Cells(cells),
            cell,
            &mut |model, values| {
                let data = match &data {
                    Ok(data) => data,
                    Err(error) => return error.clone(),
                };
                let ignore = values.get(1).filter(|v| !matches!(v, CalcResult::EmptyArg));
                let scan_by_col = values.get(2).filter(|v| !matches!(v, CalcResult::EmptyArg));
                // An error argument comes before an invalid one: `TOCOL(Data!B1:B5,Data!B8,NA())`
                // with an invalid ignore is `#N/A` (gen-m7/8044dce50616cfe9 A1508).
                for value in [ignore, scan_by_col].into_iter().flatten() {
                    if value.is_error() {
                        return value.clone();
                    }
                }
                let ignore = match ignore {
                    None => 0,
                    Some(value) => match model
                        .cast_number_ref(value, cell)
                        .and_then(|n| ignore_mode(n, cell))
                    {
                        Ok(ignore) => ignore,
                        Err(error) => return error,
                    },
                };
                let scan_by_col = match scan_by_col {
                    None => false,
                    Some(value) => match Self::cast_bool_ref(value, cell) {
                        Ok(scan_by_col) => scan_by_col,
                        Err(error) => return error,
                    },
                };
                match flatten_data(data, ignore, scan_by_col) {
                    Some(flat) => CalcResult::Array(shape(flat)),
                    None => empty_flatten(cell),
                }
            },
        )
    }
}

/// TOCOL's and TOROW's ignore argument: 0 to 3 after truncation, else `#VALUE!`.
fn ignore_mode(n: f64, cell: CellReferenceIndex) -> Result<u8, CalcResult> {
    match n as i64 {
        n @ 0..=3 => Ok(n as u8),
        _ => Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "ignore must be 0, 1, 2, or 3".to_string(),
        )),
    }
}

/// The `#CALC!` of a flatten that kept nothing.
fn empty_flatten(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::CALC,
        cell,
        "TOCOL/TOROW returned empty array".to_string(),
    )
}

/// `data` in scan order without the ignored blanks and errors; `None` when nothing is left.
fn flatten_data(data: &[Vec<ArrayNode>], ignore: u8, scan_by_col: bool) -> Option<Vec<ArrayNode>> {
    let num_cols = data.first().map_or(0, Vec::len);
    let keep = |node: &ArrayNode| {
        let is_blank = matches!(node, ArrayNode::Empty);
        let is_error = matches!(node, ArrayNode::Error(_));
        match ignore {
            1 => !is_blank,
            2 => !is_error,
            3 => !is_blank && !is_error,
            _ => true,
        }
    };
    let mut flat = Vec::with_capacity(data.len() * num_cols);
    if scan_by_col {
        for j in 0..num_cols {
            flat.extend(data.iter().map(|row| &row[j]).filter(|n| keep(n)).cloned());
        }
    } else {
        flat.extend(data.iter().flatten().filter(|n| keep(n)).cloned());
    }
    (!flat.is_empty()).then_some(flat)
}
