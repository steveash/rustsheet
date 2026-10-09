use crate::{
    calc_result::{CalcResult, Range},
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{logical::IfBranchMode, Function},
    model::Model,
    number_format::cmp_excel_precision,
    reference_operators::{RefArg, MAX_AREA_CELLS, TOO_MANY_AREA_CELLS},
};

/// Converts a single array element into a scalar [`CalcResult`].
pub(crate) fn array_node_to_calc_result(node: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::Number(n) => CalcResult::Number(*n),
        ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
        ArrayNode::String(s) => CalcResult::String(s.clone()),
        ArrayNode::Error(e) => CalcResult::new_error(e.clone(), cell, String::new()),
        ArrayNode::Empty => CalcResult::EmptyCell,
    }
}

/// The top left element of an array, which is what the implicit intersection
/// operator makes of it.
pub(crate) fn top_left_of_array(array: &[Vec<ArrayNode>], cell: CellReferenceIndex) -> CalcResult {
    match array.first().and_then(|row| row.first()) {
        Some(node) => array_node_to_calc_result(node, cell),
        None => CalcResult::new_error(Error::CALC, cell, "Empty array".to_string()),
    }
}

fn error_sort_rank(e: &Error) -> u8 {
    match e {
        Error::NULL => 1,
        Error::DIV => 2,
        Error::VALUE => 3,
        Error::REF => 4,
        Error::NAME => 5,
        Error::NUM => 6,
        Error::NA => 7,
        _ => 8,
    }
}

// In Excel there are two ways of comparing cell values.
// The old school comparison valid in formulas like D3 < D4 or HLOOKUP,... cast empty cells into empty strings or 0
// For the new formulas like XLOOKUP or SORT an empty cell is always larger than anything else.

// ..., -2, -1, 0, 1, 2, ..., A-Z, FALSE, TRUE;
pub(crate) fn compare_values(left: &CalcResult, right: &CalcResult) -> i32 {
    match (left, right) {
        (CalcResult::Number(value1), CalcResult::Number(value2)) => {
            // Equality is decided on the 15-digit values alone: an absolute tolerance would equate
            // every tiny value with 0, but Excel says `EXP(-708)=0` is FALSE (docs/excel-semantics/numbers.md,
            // probes/subnormal-function-results C4-C6).
            match cmp_excel_precision(*value1, *value2) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }
        (CalcResult::Number(_value1), CalcResult::String(_value2)) => -1,
        (CalcResult::Number(_value1), CalcResult::Boolean(_value2)) => -1,
        (CalcResult::String(value1), CalcResult::String(value2)) => {
            let value1 = value1.to_uppercase();
            let value2 = value2.to_uppercase();
            match value1.cmp(&value2) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }
        (CalcResult::String(_value1), CalcResult::Boolean(_value2)) => -1,
        (CalcResult::Boolean(value1), CalcResult::Boolean(value2)) => {
            if value1 == value2 {
                return 0;
            }
            if *value1 {
                return 1;
            }
            -1
        }
        (CalcResult::EmptyCell, CalcResult::String(_value2)) => {
            compare_values(&CalcResult::String("".to_string()), right)
        }
        (CalcResult::String(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::String("".to_string()))
        }
        (CalcResult::EmptyCell, CalcResult::Number(_value2)) => {
            compare_values(&CalcResult::Number(0.0), right)
        }
        (CalcResult::Number(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::Number(0.0))
        }
        (CalcResult::EmptyCell, CalcResult::Boolean(_)) => {
            compare_values(&CalcResult::Boolean(false), right)
        }
        (CalcResult::Boolean(_), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::Boolean(false))
        }
        (CalcResult::EmptyCell, CalcResult::EmptyCell) => 0,
        // Errors sort after everything else, ordered by Excel's canonical rank
        (CalcResult::Error { error: e1, .. }, CalcResult::Error { error: e2, .. }) => {
            let r1 = error_sort_rank(e1);
            let r2 = error_sort_rank(e2);
            if r1 < r2 {
                -1
            } else if r1 > r2 {
                1
            } else {
                0
            }
        }
        (CalcResult::Error { .. }, _) => 1,
        (_, CalcResult::Error { .. }) => -1,
        (_, _) => 1,
    }
}

/// Whether `node` is a multi-sheet 3-D span, possibly as a branch of IF (which hands its branch on
/// as a reference). Recursion is bounded by the parser's depth cap.
pub(crate) fn is_span_or_if_of_span(node: &Node) -> bool {
    match node {
        Node::SheetSpanKind {
            first_index,
            last_index,
            ..
        } => first_index != last_index,
        Node::FunctionKind {
            kind: Function::If,
            args,
        } => args.iter().skip(1).any(is_span_or_if_of_span),
        _ => false,
    }
}

impl Model {
    /// Evaluates an argument that must be a single-sheet reference (INDEX, AREAS, VLOOKUP,
    /// MATCH, COUNTBLANK): a 3-D reference is `#VALUE!` there, not the `#REF!` it is as a
    /// value (probes/3d-ref-functions; docs/excel-semantics/references.md).
    pub(crate) fn evaluate_single_sheet_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match node {
            Node::SheetSpanKind {
                first_index,
                last_index,
                reference,
                ..
            } if first_index != last_index => {
                self.get_sheet_span_reference(*first_index, *last_index, reference, cell)
            }
            // IF hands its chosen branch on as a reference: a 3-D span inside it is
            // `#VALUE!` here, as it is inside CHOOSE (T-1026).
            Node::FunctionKind {
                kind: Function::If,
                args,
            } => self
                .if_impl(args, cell, IfBranchMode::SingleSheet)
                .into_single_area(cell),
            _ => self.evaluate_node_in_context(node, cell),
        }
    }

    /// Evaluates one argument of an aggregate (SUM, COUNT, AVERAGE, MIN, ...).
    /// A cell reference, and an IF or INDEX that selects one, comes back as a
    /// range, so the caller skips its text, booleans and blanks like any range
    /// cell; everything else is a value (T-0216; docs/excel-semantics/references.md).
    /// A union, and an intersection that leaves several areas, is its areas, walked one after
    /// the other with each one counted as written (probes/multiarea-agg, multiarea-intersect;
    /// SPEC-0004). A function that takes only one area turns those into `#VALUE!` with
    /// [`RefArg::into_single_area`].
    pub(crate) fn evaluate_reference_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        let result = match node {
            Node::OpUnionKind { .. } | Node::OpIntersectKind { .. } if self.is_multi_area(node) => {
                return match self.reference_ranges(node, cell) {
                    Ok(ranges) => RefArg::Ranges(ranges),
                    Err(error) => RefArg::Value(error),
                };
            }
            // `@A1:A3` is the intersected cell's reference, so text there is ignored
            // (probes/min-max-reference-forms C1-C4).
            Node::ReferenceKind { .. }
            | Node::ImplicitIntersection { .. }
            | Node::OpRangeKind { .. } => self.evaluate_node_with_reference(node, cell),
            // A name whose body is a formula, a union included, stands for what the body does
            // in reference context: `SUM(mu)` with `mu` = `Sheet1!$A$1,Sheet1!$B$2` is 6
            // (probes/union-defined-name H1).
            Node::DefinedNameKind(_) => return self.defined_name_reference(node, cell),
            Node::SheetSpanKind {
                first_index,
                last_index,
                reference,
                ..
            } => self.get_sheet_span_range(*first_index, *last_index, reference, cell),
            // IF and CHOOSE hand back the chosen branch, LET its body, in reference context,
            // a union included (probes/multiarea-ifchoose H2:H3, multiarea-letbind H8).
            Node::FunctionKind {
                kind: Function::If,
                args,
            } => return self.if_impl(args, cell, IfBranchMode::Aggregate),
            Node::FunctionKind {
                kind: Function::Choose,
                args,
            } => return self.choose_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Let,
                args,
            } => return self.let_impl(args, cell, true),
            // A LET variable bound to a union is its areas (probes/multiarea-let H1).
            Node::NamedVariableKind { .. } => return self.variable_reference(node, cell),
            Node::FunctionKind {
                kind: Function::Index,
                args,
            } => self.index_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Hlookup,
                args,
            } => self.hlookup_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Vlookup,
                args,
            } => self.vlookup_impl(args, cell, true),
            _ => self.evaluate_node_in_context(node, cell),
        };
        RefArg::from(result)
    }

    /// [`Self::evaluate_reference_arg`] for the arguments SUBTOTAL and AREAS take: XLOOKUP, TAKE,
    /// DROP and TRIMRANGE over a range are also the cells they pick (probes/subtotal-reference-
    /// operands-b H12, H15:H17, T-1175). Only these two functions are probed; SUM and the other
    /// aggregates still evaluate those calls to values. The call list is repeated in
    /// `is_read_reference` (static_analysis/aggregate.rs).
    pub(crate) fn evaluate_picked_reference_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        if let Node::FunctionKind {
            kind: kind @ (Function::Xlookup | Function::Take | Function::Drop | Function::Trimrange),
            args,
        } = node
        {
            if let Some(result) = self.evaluate_reference_form(kind, args, cell) {
                return RefArg::from(result);
            }
        }
        self.evaluate_reference_arg(node, cell)
    }

    /// The bottom-right corner of the range `row1:column1..=row2:column2` once a whole-column
    /// or whole-row range is clipped to the sheet's used area; any other range is unchanged.
    /// Reads nothing and records nothing (see [`Self::clip_to_used_area`]).
    pub(crate) fn used_area_corner(
        &self,
        sheet: u32,
        row1: i32,
        column1: i32,
        row2: i32,
        column2: i32,
    ) -> Result<(i32, i32), String> {
        let whole_rows = row1 == 1 && row2 == LAST_ROW;
        let whole_columns = column1 == 1 && column2 == LAST_COLUMN;
        if !whole_rows && !whole_columns {
            return Ok((row2, column2));
        }
        let (max_row, max_column) = self
            .workbook
            .worksheet(sheet)
            .map_err(|_| format!("Invalid worksheet index: '{sheet}'"))?
            .used_area_end();
        let last_row = if whole_rows { max_row } else { row2 };
        let last_column = if whole_columns { max_column } else { column2 };
        Ok((last_row, last_column))
    }

    /// Checks that walking `areas` stays within [`MAX_AREA_CELLS`] cells once each area is
    /// clipped to its sheet's used area: arithmetic only, before any cell is read, so
    /// `MAX_AREAS` whole columns over a populated sheet are `#VALUE!` without a walk.
    pub(crate) fn check_area_cells(
        &self,
        areas: &[Range],
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        let mut total: u64 = 0;
        for area in areas {
            let (row2, column2) = self
                .used_area_corner(
                    area.left.sheet,
                    area.left.row,
                    area.left.column,
                    area.right.row,
                    area.right.column,
                )
                .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
            let rows = u64::try_from(row2 - area.left.row + 1).unwrap_or(0);
            let columns = u64::try_from(column2 - area.left.column + 1).unwrap_or(0);
            total = total.saturating_add(rows.saturating_mul(columns));
            if total > MAX_AREA_CELLS {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    TOO_MANY_AREA_CELLS.to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Calls `f` with the value of every cell of `areas`, area by area in formula
    /// order, each in row-major order.
    ///
    /// The one range walk behind every aggregate function. A range whose corners
    /// sit on two sheets is a 3-D reference (only [`Self::evaluate_reference_arg`]
    /// builds one) and is walked sheet by sheet in tab order; whole-row and
    /// whole-column ranges are clipped to each sheet's used area so the walk
    /// stays proportional to the data. The first
    /// error cell aborts the walk and is returned as `Err`, as does an `Err`
    /// from `f`. `f` never sees `CalcResult::Error`.
    pub(crate) fn for_each_cell_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CalcResult) -> Result<(), CalcResult>,
    {
        self.for_each_cell_or_error_in_areas(areas, cell, |model, value| match value {
            error @ CalcResult::Error { .. } => Err(error),
            value => f(model, value),
        })
    }

    /// [`Self::for_each_cell_in_areas`] that also hands `f` the error cells
    /// (COUNTA counts them, COUNT and COUNTBLANK skip them).
    pub(crate) fn for_each_cell_or_error_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CalcResult) -> Result<(), CalcResult>,
    {
        self.for_each_position_in_areas(areas, cell, |model, position| {
            let value = model.evaluate_cell(position);
            f(model, value)
        })
    }

    /// Calls `f` with the position of every cell of `areas`, as
    /// [`Self::for_each_cell_in_areas`] orders them, without evaluating it: for walks that
    /// skip cells by position (AGGREGATE's hidden rows and nested SUBTOTAL/AGGREGATE cells).
    /// Several areas are first held to [`MAX_AREA_CELLS`]; one range is not, as before.
    /// An `Err` from `f` stops the walk.
    pub(crate) fn for_each_position_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        if areas.len() > 1 {
            self.check_area_cells(areas, cell)?;
        }
        for Range { left, right } in areas {
            // A 3-D reference spans the sheets `left.sheet..=right.sheet` (tab order); each one
            // is clipped to its own used area, so a span of whole columns walks only data.
            debug_assert!(left.sheet <= right.sheet, "3-D span corners out of order");
            for sheet in left.sheet..=right.sheet {
                let (row2, column2) = self
                    .clip_to_used_area(sheet, left.row, left.column, right.row, right.column)
                    .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
                for row in left.row..=row2 {
                    for column in left.column..=column2 {
                        f(self, CellReferenceIndex { sheet, row, column })?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Calls `f` with every number held by the cells of `areas`, in the order of
    /// [`Self::for_each_cell_in_areas`].
    ///
    /// This is how aggregate functions (SUM, NPV, VAR, ...) read a
    /// *reference*: numbers count, booleans, text and empty cells are skipped
    /// (unlike direct arguments, which the callers coerce themselves). See
    /// [`Self::for_each_cell_in_areas`] for errors and range clipping.
    pub(crate) fn for_each_number_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        self.for_each_cell_in_areas(areas, cell, |_, value| {
            // Booleans, text and empty cells in a reference are ignored.
            if let CalcResult::Number(value) = value {
                f(value);
            }
            Ok(())
        })
    }

    /// Like [`Self::for_each_number_in_areas`] but with the coercion of the
    /// *A functions (VARA, STDEVA, ...): text counts as 0, TRUE/FALSE as 1/0,
    /// empty cells are skipped.
    pub(crate) fn for_each_coerced_number_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        self.for_each_cell_in_areas(areas, cell, |_, value| {
            match value {
                CalcResult::Number(value) => f(value),
                CalcResult::String(_) => f(0.0),
                CalcResult::Boolean(value) => f(if value { 1.0 } else { 0.0 }),
                _ => {}
            }
            Ok(())
        })
    }
}

/// Excel's `MOD`: the exact remainder (`fmod`) shifted into the divisor's sign. Not
/// `n - d*floor(n/d)`, which loses precision for tiny divisors; evidence: MOD_QUOTIENT.xlsx
/// C20/C21/C25/C26 (docs/excel-semantics/numbers.md). `divisor` must be non-zero.
pub(crate) fn excel_mod(value: f64, divisor: f64) -> f64 {
    let remainder = value % divisor;
    if remainder == 0.0 {
        // Drop the sign of -0.0 (fmod keeps the dividend's sign).
        0.0
    } else if (remainder < 0.0) != (divisor < 0.0) {
        remainder + divisor
    } else {
        remainder
    }
}
