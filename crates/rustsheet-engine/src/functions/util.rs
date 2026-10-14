use crate::{
    area_budget::SLOTS_PER_WALK_STEP,
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
    reference_operators::RefArg,
};

/// Applies `f` to every element of a 2-D array, keeping its shape.
pub(crate) fn map_elements(
    values: &[Vec<ArrayNode>],
    f: impl Fn(&ArrayNode) -> ArrayNode,
) -> Vec<Vec<ArrayNode>> {
    values
        .iter()
        .map(|row| row.iter().map(&f).collect())
        .collect()
}

/// An array of `error` with the shape of `values` (an element that is already an error stays).
pub(crate) fn error_array(values: &[Vec<ArrayNode>], error: &Error) -> CalcResult {
    CalcResult::Array(map_elements(values, |v| match v {
        ArrayNode::Error(_) => v.clone(),
        _ => ArrayNode::Error(error.clone()),
    }))
}

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

/// Orders two (already case-folded) texts the way Excel's `<`/`>` and `">"` criteria do: by
/// character, with CHAR(160) right after the space and before every printable character, where code
/// points would put it after the letters. `" 1" < CHAR(160)&"1" < "0"`, `"1"&CHAR(160) > "1"`
/// (probes/text-order-nbsp-space, excel-graph, T-0244). Equal texts stay equal: only CHAR(160)
/// moves. The rest of Excel's collation (`"-1" > "0"`, `"x" >= "ü"`) is T-0220.
pub(crate) fn cmp_text_excel(left: &str, right: &str) -> std::cmp::Ordering {
    left.chars()
        .map(text_order_key)
        .cmp(right.chars().map(text_order_key))
}

fn text_order_key(c: char) -> u64 {
    const SPACE_KEY: u64 = 0x20 * 2;
    if c == '\u{a0}' {
        SPACE_KEY + 1
    } else {
        u64::from(u32::from(c)) * 2
    }
}

// In Excel there are two ways of comparing cell values.
// The old school comparison valid in formulas like D3 < D4 or HLOOKUP,... cast empty cells into empty strings or 0
// For the new formulas like XLOOKUP or SORT an empty cell is always larger than anything else.

// ..., -2, -1, 0, 1, 2, ..., A-Z, FALSE, TRUE;
pub(crate) fn compare_values(left: &CalcResult, right: &CalcResult) -> i32 {
    // An empty value (`IFERROR(<blank>,1)`, `CalcResult::EmptyArg`) compares as a blank cell:
    // `IFERROR(B1,1)=0`, `=""` and `=FALSE` are TRUE and SWITCH matches it to 0
    // (probes/iferror-blank-value D2:D5, D18, D19; T-0291).
    fn as_blank(value: &CalcResult) -> &CalcResult {
        static BLANK: CalcResult = CalcResult::EmptyCell;
        if matches!(value, CalcResult::EmptyArg) {
            &BLANK
        } else {
            value
        }
    }
    let (left, right) = (as_blank(left), as_blank(right));
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
            match cmp_text_excel(&value1, &value2) {
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

/// Whether `node` is `@IF(...)`: the `@` of an IF keeps the reference the IF picks.
pub(crate) fn is_at_if(node: &Node) -> bool {
    matches!(
        node,
        Node::ImplicitIntersection { child, .. }
            if matches!(child.ungrouped(), Node::FunctionKind { kind: Function::If, .. })
    )
}

/// Whether `node` is a multi-sheet 3-D span, possibly as a branch of IF (which hands its branch on
/// as a reference). Recursion is bounded by the parser's depth cap.
pub(crate) fn is_span_or_if_of_span(node: &Node) -> bool {
    let node = node.ungrouped();
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
        let node = node.ungrouped();
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

    /// An argument that reads one cell of a reference (MATCH's `match_type`, IFS and SWITCH's chosen
    /// value): IF, IFS, SWITCH, LET and CHOOSE hand back the cell they pick, however deeply they
    /// nest (probes/match-type-text-passthrough K1:K9, match-type-nested-passthrough K1:K17;
    /// T-1399, T-1452). Aggregates keep [`Self::evaluate_reference_arg`], where IFS and SWITCH are
    /// values (`SUM(IFS(TRUE,A1:B2))`, docs/excel-semantics/logic.md).
    pub(crate) fn evaluate_chosen_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match node.ungrouped() {
            Node::FunctionKind {
                kind: Function::If,
                args,
            } => self
                .if_impl(args, cell, IfBranchMode::Chosen)
                .into_single_area(cell),
            Node::FunctionKind {
                kind: Function::Ifs,
                args,
            } => self.ifs_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Switch,
                args,
            } => self.switch_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Let,
                args,
            } => self.let_impl(args, cell, true).into_single_area(cell),
            Node::FunctionKind {
                kind: Function::Choose,
                args,
            } => self.choose_impl(args, cell, true).into_single_area(cell),
            other => self.eval_ref_or_index(other, cell),
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
        let node = node.ungrouped();
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
            } => self.index_impl(
                args,
                cell,
                crate::functions::lookup_and_reference::IndexForm::Reference,
            ),
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

    /// [`Self::evaluate_reference_arg`] for the arguments SUBTOTAL, AREAS, SUM, COUNT, AVERAGE, MIN, MAX
    /// and AVERAGEA take (and a LET or LAMBDA binding, T-1232): XLOOKUP, TAKE, DROP and TRIMRANGE over a
    /// range are also the cells they pick (probes/subtotal-reference-operands-b H12, H15:H17, T-1175; -c
    /// H4:H14, T-1206; -d H1:H27, L1:L7, T-1211). The aggregates sharing `for_each_value` follow the
    /// same rule. The call list is [`Function::picks_reference`].
    pub(crate) fn evaluate_picked_reference_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        if let Node::FunctionKind { kind, args } = node.ungrouped() {
            if kind.picks_reference() {
                if let Some(result) = self.evaluate_reference_form(kind, args, cell) {
                    return RefArg::from(result);
                }
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

    /// Calls `f` with the value of every cell of `areas`, area by area in formula
    /// order, each in row-major order.
    ///
    /// The one range walk behind every aggregate function. A range whose corners
    /// sit on two sheets is a 3-D reference (only [`Self::evaluate_reference_arg`]
    /// builds one) and is walked sheet by sheet in tab order; whole-row and
    /// whole-column ranges are clipped to each sheet's used area so the walk
    /// stays proportional to the data. Several areas (a union) are walked over
    /// their existing cells only, so `f` is never handed a blank cell of a union
    /// ([`Self::for_each_position_in_area`]). The first error cell aborts the walk
    /// and is returned as `Err`, as does an `Err` from `f`. `f` never sees
    /// `CalcResult::Error`.
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
    /// (COUNTA counts them, COUNT skips them).
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
    /// Several areas are walked by [`Self::for_each_position_in_area`] as areas of a union: only
    /// their cells that exist, held to the formula's walk budget. An `Err` from `f` stops the
    /// walk.
    pub(crate) fn for_each_position_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let of_union = areas.len() > 1;
        for area in areas {
            self.for_each_position_in_area(area, of_union, cell, &mut f)?;
        }
        Ok(())
    }

    /// Calls `f` with the position of the cells of one area, row by row. A range that is the
    /// whole argument (`of_union` false) is walked densely, every position of its sheets' used
    /// areas, as it always was. An area of a union is walked over the cells that exist only:
    /// every caller of a union walk skips blank cells, and a union can repeat a whole column
    /// many times over (`MAX_AREAS` copies, or a LAMBDA walking one per row), so the walk must
    /// cost what the data costs (T-1497). Its work is drawn from the formula's walk budget
    /// ([`Model::spend_area_walk`]) as it goes, and the area is recorded as seen empty in one
    /// piece rather than cell by cell.
    pub(crate) fn for_each_position_in_area<F>(
        &mut self,
        area: &Range,
        of_union: bool,
        cell: CellReferenceIndex,
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let Range { left, right } = area;
        // The columns of a row found by a union walk, reused from row to row and sheet to sheet.
        let mut columns = Vec::new();
        // A 3-D reference spans the sheets `left.sheet..=right.sheet` (tab order); each one
        // is clipped to its own used area, so a span of whole columns walks only data.
        debug_assert!(left.sheet <= right.sheet, "3-D span corners out of order");
        for sheet in left.sheet..=right.sheet {
            if of_union {
                self.for_each_existing_position(sheet, (left, right), cell, &mut columns, f)?;
                continue;
            }
            let (row2, column2) = self
                .clip_to_used_area(sheet, left.row, left.column, right.row, right.column)
                .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
            for row in left.row..=row2 {
                for column in left.column..=column2 {
                    f(self, CellReferenceIndex { sheet, row, column })?;
                }
            }
        }
        Ok(())
    }

    /// The union-area walk of [`Self::for_each_position_in_area`] on one sheet: the existing
    /// cells of the rectangle `left..=right`, in row-major order like the dense walk, so the
    /// first error cell is the same one. The area, every chunk and row slot the search looks at
    /// and every cell it hands to `f` are paid for before `f` runs.
    fn for_each_existing_position<F>(
        &mut self,
        sheet: u32,
        (left, right): (&CellReferenceIndex, &CellReferenceIndex),
        cell: CellReferenceIndex,
        columns: &mut Vec<i32>,
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let sheet_error = |message| CalcResult::new_error(Error::ERROR, cell, message);
        // The positions not handed to `f` are empty. Recording the whole rectangle covers the
        // existing cells too, which is harmless: a spill can only write where nothing but its
        // own spill cells stands, and those read as empty until it has run.
        // The area itself is a step: its record must be paid for even when it finds nothing, or
        // a LAMBDA repeating a union over an empty sheet would grow the record without bound.
        self.spend_area_walk(1, cell)?;
        self.record_seen_empty_range(sheet, left.row, left.column, right.row, right.column);
        // Slots are paid for in whole steps as they add up, so a dense column costs a step per
        // cell and one more per `SLOTS_PER_WALK_STEP` rows.
        let mut slots: u64 = 0;
        let mut from = left.row;
        while from <= right.row {
            let search = self
                .workbook
                .worksheet(sheet)
                .map_err(sheet_error)?
                .sheet_data
                .next_row_between(from, right.row);
            let paid = slots.div_ceil(SLOTS_PER_WALK_STEP);
            slots += search.slots;
            let steps = search.absent_chunks + (slots.div_ceil(SLOTS_PER_WALK_STEP) - paid);
            self.spend_area_walk(steps, cell)?;
            let Some(row) = search.row else {
                break;
            };
            self.workbook
                .worksheet(sheet)
                .map_err(sheet_error)?
                .sheet_data
                .columns_in_row_between(row, left.column, right.column, columns);
            // A row costs a step even with no cell in the area's columns: finding them is a
            // search of the row.
            self.spend_area_walk((columns.len() as u64).max(1), cell)?;
            for &column in columns.iter() {
                f(self, CellReferenceIndex { sheet, row, column })?;
            }
            from = row + 1;
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
/// `n - d*floor(n/d)`, which loses precision for tiny divisors; evidence: `MOD_QUOTIENT.xlsx`
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

/// How far below the next integer a whole-number argument may fall and still count as that
/// integer: 2^-22 + 2^-33, the same at 1, 500, 1000, 30000 and 100000 (an absolute bound, not
/// relative). LEFT, RIGHT, REPLACE counts and INDEX's row and column indexes share it.
pub(crate) const NEAR_INTEGER_TOLERANCE: f64 = 1.0 / 4_194_304.0 + 1.0 / 8_589_934_592.0;

/// The whole number LEFT, RIGHT, REPLACE and INDEX read from a count, position or index: `x`
/// truncated, except that a fraction within [`NEAR_INTEGER_TOLERANCE`] of the next integer counts
/// as that integer, so `LEFT("abc",1-2^-22)` is "a". MID truncates exactly and does not use this.
// Excel: docs/excel-semantics/text.md "LEFT, RIGHT and REPLACE round counts up near an
// integer" (probes/text-count-argument-bounds) and lookup.md "INDEX rounds an index up near an
// integer" (probes/index-snap-tolerance).
pub(crate) fn near_integer_count(x: f64) -> f64 {
    let whole = x.floor();
    // `x - whole` is exact for every finite double, so the bound is not blurred at large x.
    if x - whole >= 1.0 - NEAR_INTEGER_TOLERANCE {
        whole + 1.0
    } else {
        whole
    }
}

#[cfg(test)]
mod tests {
    use super::near_integer_count;

    #[test]
    fn near_integer_count_rounds_up_within_the_tolerance() {
        // probes/text-count-argument-bounds Z10, Z9, AA1, AA2: 1 - (2^-22 + 2^-33) is 1, a hair
        // further down is 0; the bound is absolute (AA13, AA14 below 30000).
        let at = 2f64.powi(-22) + 2f64.powi(-33);
        let past = 2f64.powi(-22) + 9.0 * 2f64.powi(-36);
        assert_eq!(near_integer_count(1.0 - at), 1.0);
        assert_eq!(near_integer_count(1.0 - past), 0.0);
        assert_eq!(near_integer_count(30000.0 - at), 30000.0);
        assert_eq!(near_integer_count(30000.0 - past), 29999.0);
        assert_eq!(near_integer_count(2.5), 2.0);
        assert_eq!(near_integer_count(1e-300), 0.0);
        assert_eq!(near_integer_count(1e100), 1e100);
    }
}
