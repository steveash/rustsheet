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
        let kind = Function::Tocol;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        // The array is read once and handed to every element whole; ignore and scan_by_column
        // lift over an array and a range (SPEC-0011 "Array-valued functions", T-1836). Each
        // element walks the array, so the call is charged its cells per element.
        let data = match self.eval_to_array(&args[0], cell) {
            Ok(data) => data,
            Err(error) => return error,
        };
        let cells = data.iter().map(Vec::len).sum::<usize>();
        let cells = u64::try_from(cells).unwrap_or(u64::MAX).max(1);
        self.lift_call_costed(
            kind,
            args,
            LiftCost::Cells(cells),
            cell,
            &mut |model, values| {
                let ignore = match values.get(1) {
                    None | Some(CalcResult::EmptyArg) => Ok(0),
                    Some(value) => model
                        .cast_number_ref(value, cell)
                        .and_then(|n| ignore_mode(n, cell)),
                };
                let scan_by_col = match values.get(2) {
                    None | Some(CalcResult::EmptyArg) => Ok(false),
                    Some(value) => Self::cast_bool_ref(value, cell),
                };
                match (ignore, scan_by_col) {
                    (Ok(ignore), Ok(scan_by_col)) => match flatten_data(&data, ignore, scan_by_col)
                    {
                        Some(flat) => {
                            CalcResult::Array(flat.into_iter().map(|v| vec![v]).collect())
                        }
                        None => empty_flatten(cell),
                    },
                    (Err(error), _) | (_, Err(error)) => error,
                }
            },
        )
    }

    // ── TOROW ─────────────────────────────────────────────────────────────────

    /// `=TOROW(array, [ignore], [scan_by_col])`
    ///
    /// Flattens an array into a single row.
    ///   * ignore: 0=keep all (default), 1=ignore blanks, 2=ignore errors, 3=ignore both
    ///   * `scan_by_col`: FALSE=row-by-row (default), TRUE=column-by-column
    pub(crate) fn fn_torow(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        match self.flatten_array_impl(args, cell) {
            Ok(flat) => CalcResult::Array(vec![flat]),
            Err(e) => e,
        }
    }

    fn flatten_array_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<ArrayNode>, CalcResult> {
        if args.is_empty() || args.len() > 3 {
            return Err(CalcResult::new_args_number_error(cell));
        }

        let data = self.eval_to_array(&args[0], cell)?;

        let ignore: u8 = if args.len() >= 2 {
            let n = self.get_number(&args[1], cell)?;
            ignore_mode(n, cell)?
        } else {
            0
        };

        let scan_by_col: bool = if args.len() >= 3 {
            self.get_boolean(&args[2], cell)?
        } else {
            false
        };

        flatten_data(&data, ignore, scan_by_col).ok_or_else(|| empty_flatten(cell))
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
