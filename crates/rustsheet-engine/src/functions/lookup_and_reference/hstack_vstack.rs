use super::ref_lift::{lifts_into_references, put_back};
use crate::calc_result::range_dims;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    limits::check_stack_size,
    model::Model,
    reference_operators::{RefArg, RefElement, RefGrid},
};

/// `CELL("contents", ref)` written out as a call with a literal info type.
fn is_cell_contents_call(arg: &Node) -> bool {
    match arg.ungrouped() {
        Node::FunctionKind {
            kind: Function::Cell,
            args,
        } => matches!(
            (args.first().map(Node::ungrouped), args.len()),
            (Some(Node::StringKind(info)), 2) if info.eq_ignore_ascii_case("contents")
        ),
        _ => false,
    }
}

impl Model {
    // ── HSTACK ────────────────────────────────────────────────────────────────

    /// `=HSTACK(array1, [array2], ...)`
    ///
    /// Stacks arrays horizontally (side by side) into one array.
    /// All arrays must have the same number of rows, or shorter ones are padded
    /// with #N/A to match the tallest array.
    ///
    /// Each argument is evaluated once (see VSTACK: `read_once` relies on it).
    pub(crate) fn fn_hstack(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut arrays: Vec<Vec<Vec<ArrayNode>>> = Vec::with_capacity(args.len());
        for arg in args {
            // A blank one-cell reference is a blank element, as for VSTACK:
            // `SEARCH(HSTACK(Z1),"Hello World",8)` is 8 (probes/t2436, gen-m9 T-2436).
            match self.eval_to_array_keep_blank(arg, cell) {
                Ok(a) => arrays.push(a),
                Err(e) => return e,
            }
        }

        let max_rows = arrays.iter().map(|a| a.len()).max().unwrap_or(0);
        if max_rows == 0 {
            return CalcResult::Array(vec![]);
        }

        // The padded shape is known from the arguments: refuse one above the array limit before
        // any padding or copying (DEVIATION, arrays.md "Array results above 1,048,576 cells";
        // T-1780). A column next to a row is their product. Columns past the grid are allowed
        // (`check_stack_size`, T-1803).
        let total_cols = arrays
            .iter()
            .map(|a| a.first().map_or(0, Vec::len))
            .fold(0_usize, usize::saturating_add);
        if let Some((error, message)) = check_stack_size(max_rows, total_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let na = ArrayNode::Error(Error::NA);
        let mut result: Vec<Vec<ArrayNode>> = Vec::with_capacity(max_rows);
        for r in 0..max_rows {
            let mut row: Vec<ArrayNode> = Vec::new();
            for arr in &arrays {
                if r < arr.len() {
                    row.extend(arr[r].iter().cloned());
                } else {
                    let ncols = arr.first().map(|r| r.len()).unwrap_or(0);
                    row.extend(std::iter::repeat_n(na.clone(), ncols));
                }
            }
            result.push(row);
        }

        CalcResult::Array(result)
    }

    // ── VSTACK ────────────────────────────────────────────────────────────────

    /// `=VSTACK(array1, [array2], ...)`
    ///
    /// Stacks arrays vertically (one on top of another) into one array.
    /// All arrays must have the same number of columns, or narrower ones are
    /// padded with #N/A to match the widest array.
    ///
    /// Each argument is evaluated once: a LAMBDA parameter read here is moved into the stack, not
    /// copied (`read_once` in functions/logical/lambda.rs relies on it, as for HSTACK).
    pub(crate) fn fn_vstack(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        // Excel evaluates every argument first, a reference without reading its cells, and checks
        // the stacked rows before it reads any: an error argument wins wherever it stands, and a
        // stack past the grid is `#NUM!` even when a reference holds this cell, which reading it
        // would make a cycle (`=ROWS(VSTACK(2:2,A:A))` in W2, `=ROWS(VSTACK(A:A,W8))` in W8;
        // probes/vstack-self-past-grid, scalar-lift-open-c2 W1; arrays.md "Stacked results past
        // the grid"; T-1965). A one-cell reference is read after the check (`None` here); a
        // range is evaluated to its corners, which reads nothing. Every argument's rows count.
        let mut results: Vec<Option<CalcResult>> = Vec::with_capacity(args.len());
        let mut known_rows = 0_usize;
        for arg in args {
            // `VSTACK(CELL("contents",ref),..)` is one `#VALUE!`, though CELL alone is fine
            // (probes/t2414-gen-m7-unexplained Calc!A41, A61, A81, excel-graph, T-2457).
            if is_cell_contents_call(arg) {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "CELL(\"contents\", ref) is not stackable".to_string(),
                );
            }
            if matches!(*arg.ungrouped_at(cell.row), Node::ReferenceKind { .. }) {
                known_rows = known_rows.saturating_add(1);
                results.push(None);
                continue;
            }
            match self.evaluate_node_in_context(arg, cell) {
                err @ CalcResult::Error { .. } => return err,
                result => {
                    let rows = match &result {
                        CalcResult::Range { left, right } => range_dims(*left, *right).0,
                        CalcResult::Array(array) => array.len(),
                        _ => 1,
                    };
                    known_rows = known_rows.saturating_add(rows);
                    results.push(Some(result));
                }
            }
        }
        if let Some((error, message)) = check_stack_size(known_rows, 1) {
            return CalcResult::new_error(error, cell, message);
        }

        let mut arrays: Vec<Vec<Vec<ArrayNode>>> = Vec::with_capacity(args.len());
        for (arg, result) in args.iter().zip(results) {
            let array = match result {
                Some(result) => self.result_to_array(result, cell),
                // A blank one-cell reference stacks as a blank element, not 0 (`ISBLANK(INDEX(
                // VSTACK(Z1,1),1,1))` is TRUE; probes/t2413-blank-in-stacked-arrays A13:A23).
                None => self.eval_to_array_keep_blank(arg, cell),
            };
            match array {
                Ok(a) => arrays.push(a),
                Err(e) => return e,
            }
        }

        let max_cols = arrays
            .iter()
            .filter_map(|a| a.first().map(|r| r.len()))
            .max()
            .unwrap_or(0);

        if max_cols == 0 {
            return CalcResult::Array(vec![]);
        }

        // Refuse a padded shape above the array limit before any padding (T-1780, as HSTACK); rows
        // past the grid are `#NUM!` (`check_stack_size`, T-1803).
        let total_rows = arrays
            .iter()
            .map(Vec::len)
            .fold(0_usize, usize::saturating_add);
        if let Some((error, message)) = check_stack_size(total_rows, max_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let na = ArrayNode::Error(Error::NA);
        let mut result: Vec<Vec<ArrayNode>> = Vec::with_capacity(total_rows);
        for arr in arrays {
            for row in arr {
                let mut padded = row;
                while padded.len() < max_cols {
                    padded.push(na.clone());
                }
                result.push(padded);
            }
        }

        CalcResult::Array(result)
    }

    /// VSTACK where a reference is taken (SPEC-0014 rule 7): when every argument is an array of
    /// references, the stacked grid, narrower ones padded with `#N/A` elements, so
    /// `SUM(VSTACK(OFFSET(A1,{0;1},0)))` reads each element's top-left cell, 101
    /// (probes/reference-lift-consumers-b2 E225). `None` unless every argument is an OFFSET
    /// or INDIRECT (read without evaluating); otherwise the call's value, as
    /// [`Model::fn_vstack`] gives it. Several grids, padded with `#N/A` where narrower, are
    /// unprobed (E225 has one).
    pub(crate) fn vstack_reference(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Option<RefArg> {
        if args.is_empty() || !args.iter().all(lifts_into_references) {
            return None;
        }
        let reads: Vec<RefArg> = args
            .iter()
            .map(|arg| self.evaluate_reference_arg(arg, cell))
            .collect();
        if !reads.iter().all(|read| matches!(read, RefArg::Grid(_))) {
            // Not every argument is a grid: the value, as before, from what each one gave
            // (never read twice).
            let mut put = Vec::with_capacity(reads.len());
            for read in &reads {
                match put_back(read) {
                    Some(node) => put.push(node),
                    None => {
                        return Some(RefArg::Value(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "VSTACK of a reference it cannot read".to_string(),
                        )))
                    }
                }
            }
            return Some(RefArg::from(self.fn_vstack(&put, cell)));
        }
        let grids: Vec<RefGrid> = reads
            .into_iter()
            .filter_map(|read| match read {
                RefArg::Grid(grid) => Some(grid),
                _ => None,
            })
            .collect();
        let columns = grids.iter().map(|grid| grid.dims().1).max().unwrap_or(1);
        let rows = grids
            .iter()
            .map(|grid| grid.dims().0)
            .fold(0_usize, usize::saturating_add);
        if let Some((error, message)) = check_stack_size(rows, columns) {
            return Some(RefArg::Value(CalcResult::new_error(error, cell, message)));
        }
        let mut elements = Vec::with_capacity(rows.saturating_mul(columns));
        for grid in &grids {
            let width = grid.dims().1;
            for row in grid.elements().chunks(width) {
                elements.extend_from_slice(row);
                elements.extend(std::iter::repeat_n(
                    RefElement::Error(Error::NA),
                    columns - width,
                ));
            }
        }
        Some(match RefGrid::new(rows, columns, elements) {
            Some(grid) => RefArg::Grid(grid),
            None => RefArg::Value(CalcResult::new_error(
                Error::ERROR,
                cell,
                "Empty array of references".to_string(),
            )),
        })
    }
}
