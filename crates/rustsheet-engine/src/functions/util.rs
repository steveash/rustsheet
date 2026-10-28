pub(crate) mod near_integer;

use crate::external_reference::PickedReference;
use crate::{
    calc_result::{CalcResult, Range},
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    external_links::CachedBlanks,
    functions::{logical::IfBranchMode, lookup_and_reference::Edge, Function},
    model::Model,
    number_format::cmp_excel_precision,
    reference_operators::{Picked, RefArg},
};

pub(crate) mod aggregate_args;
pub(crate) mod collation;
mod existing_walk;

pub(crate) use existing_walk::StoredWalk;
use existing_walk::{span, WalkBudget};

use collation::cmp_text;

/// Whether `value`, the value of the argument `node`, is a number that ERROR.TYPE or
/// `CELL("col"|"row",..)` returned straight to the argument (parentheses aside): the "mark" some
/// array-taking arguments reject where the same number from anywhere else is accepted. F.TEST
/// and the paired functions give `#NUM!` for it (`FTEST({1;2;3},ERROR.TYPE(#REF!))`, statistics.md
/// "ERROR.TYPE and CELL results are #NUM! in the paired functions") and IRR's flows `#VALUE!`
/// (`IRR(CELL("row",D2))`, financial.md "IRR's error order", T-2728). `+0`, `@`, SINGLE, MAX or
/// a cell holding the value make it a plain number. LET, IF, CHOOSE, IFERROR, a LAMBDA call and
/// INDEX keep it in Excel and are not followed here: TODO(T-2766).
pub(crate) fn is_marked_number(node: &Node, value: &CalcResult, row: i32) -> bool {
    if !matches!(value, CalcResult::Number(_)) {
        return false;
    }
    match &*node.ungrouped_at(row) {
        Node::FunctionKind {
            kind: Function::ErrorType,
            ..
        } => true,
        Node::FunctionKind {
            kind: Function::Cell,
            args,
        } => matches!(
            args.first(),
            Some(Node::StringKind(info)) if info.eq_ignore_ascii_case("col") || info.eq_ignore_ascii_case("row")
        ),
        _ => false,
    }
}

/// The number a referenced cell gives an aggregate: numbers count, booleans, text and empty cells
/// are skipped (SPEC-0009 `CellNumbers::Numbers`).
pub(crate) fn cell_number(value: &CalcResult) -> Option<f64> {
    match value {
        CalcResult::Number(value) => Some(*value),
        _ => None,
    }
}

/// The number a referenced cell gives the A functions (VARA, AVERAGEA, MAXA, ...): text counts
/// as 0, TRUE/FALSE as 1/0, empty cells are skipped (SPEC-0009 `CellNumbers::CoercedA`).
pub(crate) fn coerced_cell_number(value: &CalcResult) -> Option<f64> {
    match value {
        CalcResult::Number(value) => Some(*value),
        CalcResult::String(_) => Some(0.0),
        CalcResult::Boolean(value) => Some(if *value { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Converts a single array element into a scalar [`CalcResult`]. A LAMBDA element is read as a
/// value, `#VALUE!` (docs/excel-semantics/arrays.md "Arrays of LAMBDAs");
/// [`array_element_value`] keeps it.
pub(crate) fn array_node_to_calc_result(node: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::Lambda(_) => CalcResult::new_error(Error::VALUE, cell, String::new()),
        other => array_element_value(other, cell),
    }
}

/// [`array_node_to_calc_result`] for a caller that can take a LAMBDA, MAP's and REDUCE's
/// arguments and the element INDEX picks: a LAMBDA element is that LAMBDA (T-2039).
pub(crate) fn array_element_value(node: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::Number(n) => CalcResult::Number(*n),
        ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
        ArrayNode::String(s) => CalcResult::String(s.clone()),
        ArrayNode::Error(e) => CalcResult::new_error(e.clone(), cell, String::new()),
        ArrayNode::Empty => CalcResult::EmptyCell,
        ArrayNode::Lambda(id) => CalcResult::Lambda(*id),
        // Only INDEX reads its contents (`NestedArray::value`); others see `#VALUE!` (SPEC-0018).
        ArrayNode::Nested(_) => CalcResult::new_error(Error::VALUE, cell, String::new()),
    }
}

/// The top left element of an array, which is what the implicit intersection operator makes of it.
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
            // Case-insensitive, accent- and hyphen-aware (T-0220).
            match cmp_text(value1, value2) {
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
            compare_values(&CalcResult::String("".into()), right)
        }
        (CalcResult::String(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::String("".into()))
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

/// Whether `node` is `@XLOOKUP(...)` or `@TAKE(...)`: the `@` keeps the reference they pick.
pub(crate) fn is_at_xlookup_take(node: &Node) -> bool {
    matches!(
        node,
        Node::ImplicitIntersection { child, .. }
            if matches!(
                child.ungrouped(),
                Node::FunctionKind { kind: Function::Xlookup | Function::Take, .. }
            )
    )
}

/// Which cells of a sheet a range walk hands on ([`Model::walk_area`]). The two dense walks
/// differ only for an explicit block reaching past the used area: `ClipBlocks` stops at the
/// used area ([`Model::clip_block_to_used_area`]), `ClipWholeLines` walks the block whole and
/// clips only whole rows and columns ([`Model::clip_to_used_area`]).
#[derive(Clone, Copy)]
enum AreaWalk {
    /// Every position of the range within the sheet's used area: the cells past it are blank.
    ClipBlocks,
    /// Every position of the range, but a whole-row or whole-column range only within the
    /// used area, for walkers that read a blank cell as something.
    ClipWholeLines,
    /// The cells that exist only, for the areas of a union.
    Existing,
}

/// What a range walk hands its walker ([`Model::walk_area`]): a position to read, or, for an
/// `AreaWalk::ClipWholeLines` walk over a mostly blank area, a run of blank positions handed on
/// as their count, in row-major order between the positions around it.
enum AreaStep {
    Position(CellReferenceIndex),
    Blanks(u64),
}

/// What [`Model::for_each_cell_or_error_in_whole_range`] hands its walker, in row-major order:
/// a cell's value, or a run of blank cells as their count, so a mostly blank range costs its
/// data, not its positions (T-2710).
pub(crate) enum RangeItem {
    Cell(CalcResult),
    Blanks(u64),
}

impl Model {
    /// MATCH's, VLOOKUP's and HLOOKUP's table argument: [`Model::external_lookup_table`] for a
    /// table written into a linked workbook, else [`Model::evaluate_single_sheet_arg`].
    pub(crate) fn evaluate_lookup_table_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.external_lookup_table(node, cell)
            .unwrap_or_else(|| self.evaluate_single_sheet_arg(node, cell))
    }

    /// Evaluates an argument that must be a single-sheet reference (INDEX, AREAS, VLOOKUP,
    /// MATCH, COUNTBLANK): a 3-D reference is `#VALUE!` there, not the `#REF!` it is as a
    /// value (probes/3d-ref-functions; docs/excel-semantics/references.md). A one-cell reference,
    /// written or picked by IF or INDEX, stays a one-cell range (T-1862).
    pub(crate) fn evaluate_single_sheet_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // A `#This Row` area is its cell or range on the formula's row (SPEC-0016 step 2).
        let node: &Node = &node.ungrouped_at(cell.row);
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
            // One cell is a one-cell range, searched as one: `MATCH("b",A2,0)` is 1 and
            // `VLOOKUP(2,B2,1,FALSE)` 2 where B2 is 2, also picked by IF or INDEX
            // (probes/match-single-value-lookup-array U2, V2, W2, Z2, AA2; T-1862).
            // A name for one cell is the same one-cell range
            // (probes/match-single-cell-forms-and-texts O2:R2, T-1939).
            Node::ReferenceKind { .. } | Node::DefinedNameKind(_) => {
                self.evaluate_node_with_reference(node, cell)
            }
            Node::FunctionKind {
                kind: Function::Index,
                ..
            } => self.eval_ref_or_index(node, cell),
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
        self.reference_arg_reading(node, CachedBlanks::Kept, cell)
    }

    /// [`Self::evaluate_reference_arg`] with a cached area of a linked workbook read as `blanks`
    /// says ([`Model::external_aggregate_value`]).
    fn reference_arg_reading(
        &mut self,
        node: &Node,
        blanks: CachedBlanks,
        cell: CellReferenceIndex,
    ) -> RefArg {
        // An IF or CHOOSE that may pick a reference into another workbook is the argument it
        // picks (`SUM(IF(TRUE,…!A1))` is `#BLOCKED!`; probes/external-ref-no-link-ifchoose A3).
        // An aggregate reading cells of another workbook reads what its externalLink cache
        // holds, as reference cells (fixtures/external-link-cached-ranges; T-1460), and is
        // `#BLOCKED!` when there is no cache, however the reference was reached
        // (`SUM(INDEX(…!A1:C4,2,2))`, `SUM((…!A1,…!B2))`, `SUMPRODUCT(…!A1:B2)`;
        // probes/external-ref-no-link-index A7, A11, A15, A22, A27). COUNTA, COUNTBLANK and the
        // reference functions read [`Model::pick_reference`] first.
        let node = match self.pick_reference(node, cell) {
            PickedReference::Node(node) => node,
            PickedReference::Value(value) => return RefArg::Value(value),
            PickedReference::Read(read) => return read,
            PickedReference::External(areas) => {
                return RefArg::Value(match areas {
                    Ok(areas) => self.external_aggregate_value(&areas, blanks, cell),
                    Err(error) => error,
                })
            }
        };
        // A `#This Row` area is its cell or range on the formula's row (SPEC-0016 step 2).
        let node: &Node = &node.ungrouped_at(cell.row);
        // A 3-D span of a linked workbook's cached sheets is read sheet by sheet, as a span of
        // this workbook is (fixtures/external-link-cached-3d-span Refs!B2:B6; T-1654).
        if let Some(rows) = self.cached_span_cells(node, cell) {
            return RefArg::Value(CalcResult::Array(rows));
        }
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
            Node::LambdaCallKind { lambda, args } => {
                return self.call_callee_as_reference(lambda, args, cell)
            }
            // A lifted OFFSET, INDIRECT, TRIMRANGE, TAKE or DROP is an array of references, and
            // so is a VSTACK of them (SPEC-0014; T-1838, T-2064); unlifted, each gives its value.
            Node::FunctionKind {
                kind:
                    kind @ (Function::Offset
                    | Function::Indirect
                    | Function::Trimrange
                    | Function::Take
                    | Function::Drop
                    | Function::Vstack),
                args,
            } => return self.reference_producer(kind, args, cell),
            Node::FunctionKind {
                kind: Function::Index,
                args,
            } => self.index_reference(args, cell),
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
    /// range, also as the arm a scalar IF, CHOOSE, IFS or SWITCH picks or a LET body (T-2009,
    /// T-2014), are the cells they pick (probes/subtotal-reference-operands-b H12, H15:H17,
    /// T-1175; -c H4:H14, T-1206; -d H1:H27, L1:L7, T-1211). The aggregates sharing `for_each_value` follow the
    /// same rule. The call list is [`Function::picks_reference`].
    pub(crate) fn evaluate_picked_reference_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        self.picked_reference_arg_reading(node, CachedBlanks::Kept, cell)
    }

    /// [`Self::evaluate_picked_reference_arg`] with a cached area of a linked workbook read as
    /// `blanks` says.
    fn picked_reference_arg_reading(
        &mut self,
        node: &Node,
        blanks: CachedBlanks,
        cell: CellReferenceIndex,
    ) -> RefArg {
        // The arm a scalar IF, CHOOSE, IFS or SWITCH picks and a LET body follow the same rule,
        // however deeply they nest: `SUBTOTAL(9,IF(TRUE,XLOOKUP(1,J1:J2,H1:I2)))` is 9 and
        // `AREAS(LET(v,1,XLOOKUP(..)))` 1 (probes/subtotal-if-choose-over-xlookup A10-A42,
        // T-2009); `SUBTOTAL(9,IFS(TRUE,XLOOKUP(..)))` is 9 and `SUBTOTAL(9,SWITCH(1,1,(H1,I2)))`
        // 11 (probes/subtotal-ifs-switch-reference-picks, T-2014). Each condition, index, case
        // and binding is read once, as `reference_arg_reading` reads them.
        let Node::FunctionKind { kind, args } = node.ungrouped() else {
            return self.reference_arg_reading(node, blanks, cell);
        };
        if let Some(picked) = self.pick_step(kind, args, cell) {
            return match picked {
                Picked::Node(arm) => self.picked_reference_arg_reading(arm, blanks, cell),
                Picked::Value(value) => RefArg::Value(value),
            };
        }
        match kind {
            Function::Let => self
                .let_scope(args, cell, |model, body| {
                    model.picked_reference_arg_reading(body, blanks, cell)
                })
                .unwrap_or_else(RefArg::Value),
            // A lifted TRIMRANGE keeps its array of references, which SUM reads by its
            // top-left cells (`SUM(TRIMRANGE(A1:B5,{0;1}))` is 10 with A1 = 5;
            // probes/trimrange-lift-sources D27).
            Function::Trimrange => self.trimrange_reference(args, cell, true),
            // So does a lifted TAKE or DROP over a range (T-1838).
            Function::Take => self.take_drop_reference(args, cell, Edge::Take, true),
            Function::Drop => self.take_drop_reference(args, cell, Edge::Drop, true),
            _ if kind.picks_reference() => match self.evaluate_reference_form(kind, args, cell) {
                Some(result) => RefArg::from(result),
                None => self.reference_arg_reading(node, blanks, cell),
            },
            _ => self.reference_arg_reading(node, blanks, cell),
        }
    }

    /// [`Self::evaluate_picked_reference_arg`] for an aggregate that skips the blanks of a range
    /// (SUM, COUNT, AVERAGE, AVERAGEA, MIN, MAX and the like): a cached area of a linked
    /// workbook reads only the cells its cache spans ([`CachedBlanks::Skipped`]), so
    /// `SUM([1]Uses!$A$2:$B$1048576)` reads the cache instead of tripping the array bound
    /// (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells"; T-1550).
    pub(crate) fn evaluate_aggregate_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        self.picked_reference_arg_reading(node, CachedBlanks::Skipped, cell)
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
        let (max_row, max_column) = self.used_area_end_of(sheet)?;
        let last_row = if whole_rows { max_row } else { row2 };
        let last_column = if whole_columns { max_column } else { column2 };
        Ok((last_row, last_column))
    }

    /// Calls `f` with the value of every cell of `areas`, area by area in formula
    /// order, each in row-major order.
    ///
    /// The one range walk behind every aggregate function. A range whose corners
    /// sit on two sheets is a 3-D reference (only [`Self::evaluate_reference_arg`]
    /// builds one) and is walked sheet by sheet in tab order; every range is
    /// clipped to each sheet's used area so the walk stays proportional to the
    /// data, and `f` is not handed the blank cells past it (T-0405). A walker
    /// that reads those blanks uses
    /// [`Self::for_each_cell_or_error_in_whole_range`]. Several areas (a union) are walked over
    /// their existing cells only, so `f` is never handed a blank cell of a union
    /// ([`Self::for_each_position_in_area`]). The first error cell aborts the walk
    /// and is returned as `Err`, as does an `Err` from `f`. `f` never sees `CalcResult::Error`.
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

    /// [`Self::for_each_cell_or_error_in_areas`] over one range that also hands `f` the blank
    /// cells of an explicit block past the used area, for the walkers that read a blank as
    /// something: TEXTJOIN with `ignore_empty` FALSE adds a delimiter for it. (GCD and LCM read
    /// it as 0, `LCM(F1:F3)` with 4, blank, 6 is 0, docs/excel-semantics/coercion.md, but any
    /// number of blanks as one: [`Self::for_each_cell_or_error_in_range_noting_blanks`].)
    /// Whole-row and whole-column ranges are still clipped to the used area. A mostly blank
    /// area ([`Self::walks_sparsely`]) hands its blank cells on as runs, [`RangeItem::Blanks`],
    /// so the walk costs the cells that exist (T-2710).
    pub(crate) fn for_each_cell_or_error_in_whole_range<F>(
        &mut self,
        range: &Range,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, RangeItem) -> Result<(), CalcResult>,
    {
        self.walk_area(
            range,
            AreaWalk::ClipWholeLines,
            cell,
            &mut |model, step| match step {
                AreaStep::Position(position) => {
                    let value = model.evaluate_cell(position);
                    f(model, RangeItem::Cell(value))
                }
                AreaStep::Blanks(count) => f(model, RangeItem::Blanks(count)),
            },
        )
        .map(|_| ())
    }

    /// [`Self::for_each_cell_or_error_in_whole_range`] for a walker to which any number of blank
    /// cells reads as one: an explicit block is cut at the used area as
    /// [`Self::for_each_cell_or_error_in_areas`] cuts it, and the result says whether the cut
    /// left out blank cells that the whole-range walk hands on. GCD and LCM read them as one 0,
    /// so `LCM(A1:Z1048575)` costs the data, not 27 million cells (T-0438).
    pub(crate) fn for_each_cell_or_error_in_range_noting_blanks<F>(
        &mut self,
        range: &Range,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<bool, CalcResult>
    where
        F: FnMut(&mut Self, CalcResult) -> Result<(), CalcResult>,
    {
        self.walk_area(range, AreaWalk::ClipBlocks, cell, &mut |model, step| {
            let AreaStep::Position(position) = step else {
                // A `ClipBlocks` walk hands on no blank runs: its blanks are left out.
                return Ok(());
            };
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
    /// whole argument (`of_union` false) is walked densely: every position of the range within
    /// each sheet's used area, for an explicit block as for whole rows and columns (T-0405),
    /// unless a far cell leaves that area mostly blank, when it is walked like a union's area
    /// ([`Self::walks_sparsely`], T-1690). An area of a union is walked over the cells that
    /// exist only: every caller of a union walk skips blank cells, and a union can repeat a
    /// whole column many times over (`MAX_AREAS` copies, or a LAMBDA walking one per row), so
    /// the walk must cost what the data costs (T-1497). Its work is drawn from the formula's
    /// walk budget ([`Model::spend_area_walk`]) as it goes, and the area is recorded as seen
    /// empty in one piece rather than cell by cell.
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
        let walk = if of_union {
            AreaWalk::Existing
        } else {
            AreaWalk::ClipBlocks
        };
        self.walk_area(area, walk, cell, &mut |model, step| match step {
            AreaStep::Position(position) => f(model, position),
            // Only a `ClipWholeLines` walk hands on blank runs.
            AreaStep::Blanks(_) => Ok(()),
        })
        .map(|_| ())
    }

    /// [`Self::for_each_position_in_area`] with the cells of each sheet chosen as `walk` says.
    /// Returns whether `ClipBlocks` cut an explicit block at the used area: whether it left out
    /// blank cells that `ClipWholeLines` would have handed on.
    fn walk_area<F>(
        &mut self,
        area: &Range,
        walk: AreaWalk,
        cell: CellReferenceIndex,
        f: &mut F,
    ) -> Result<bool, CalcResult>
    where
        F: FnMut(&mut Self, AreaStep) -> Result<(), CalcResult>,
    {
        let Range { left, right } = area;
        // The columns of a row found by a union walk, reused from row to row and sheet to sheet.
        let mut columns = Vec::new();
        let mut blanks_left_out = false;
        // A 3-D reference spans the sheets `left.sheet..=right.sheet` (tab order); each one
        // is clipped to its own used area, so a span of whole columns, or of a tall block,
        // walks only data (T-0405).
        debug_assert!(left.sheet <= right.sheet, "3-D span corners out of order");
        for sheet in left.sheet..=right.sheet {
            let clipped = match walk {
                AreaWalk::Existing => {
                    self.for_each_existing_position(
                        sheet,
                        (left, right),
                        cell,
                        (&mut columns, WalkBudget::Charged),
                        &mut |model, position| f(model, AreaStep::Position(position)),
                    )?;
                    continue;
                }
                AreaWalk::ClipBlocks => self
                    .clip_block_to_used_area(sheet, left.row, left.column, right.row, right.column)
                    .and_then(|corner| {
                        let whole_lines = self.used_area_corner(
                            sheet,
                            left.row,
                            left.column,
                            right.row,
                            right.column,
                        )?;
                        blanks_left_out |= corner != whole_lines;
                        Ok(corner)
                    }),
                AreaWalk::ClipWholeLines => {
                    self.clip_to_used_area(sheet, left.row, left.column, right.row, right.column)
                }
            };
            let (row2, column2) =
                clipped.map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
            // A far cell stretches the used area: `SUM(A:XFD)` with data in A1 and XFD1048576
            // is 1.7e10 positions. A mostly blank area is walked over its cells that exist, as
            // a union's areas are, so the walk costs the data (T-1690, T-2710). The blank cells
            // it skips are ones the dense walk hands on, so `ClipBlocks` reports them left out
            // and `ClipWholeLines` hands them on as runs between the cells it visits.
            if self.walks_sparsely(sheet, left, (row2, column2)) {
                blanks_left_out = true;
                let corner = CellReferenceIndex {
                    sheet,
                    row: row2,
                    column: column2,
                };
                let runs = matches!(walk, AreaWalk::ClipWholeLines);
                let width = span(left.column, column2);
                // The row-major index of the next position not yet handed on. Every position
                // the walk visits lies inside the area, so its index is below the area's size.
                let mut next: u64 = 0;
                self.for_each_existing_position(
                    sheet,
                    (left, &corner),
                    cell,
                    (&mut columns, WalkBudget::Free),
                    &mut |model, position| {
                        let index = span(left.row, position.row - 1) * width
                            + span(left.column, position.column - 1);
                        if runs && index > next {
                            f(model, AreaStep::Blanks(index - next))?;
                        }
                        next = index + 1;
                        f(model, AreaStep::Position(position))
                    },
                )?;
                let positions = span(left.row, row2) * width;
                if runs && positions > next {
                    f(self, AreaStep::Blanks(positions - next))?;
                }
                continue;
            }
            for row in left.row..=row2 {
                for column in left.column..=column2 {
                    f(
                        self,
                        AreaStep::Position(CellReferenceIndex { sheet, row, column }),
                    )?;
                }
            }
        }
        Ok(blanks_left_out)
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
            if let Some(value) = cell_number(&value) {
                f(value);
            }
            Ok(())
        })
    }
}

/// Evaluates `c[0] + c[1]*y + c[2]*y^2 + ...` by Horner's rule in plain doubles, from the top
/// coefficient.
pub(crate) fn poly(c: &[f64], y: f64) -> f64 {
    c.iter().rev().fold(0.0, |acc, &a| acc * y + a)
}

/// Excel's `MOD`: the exact remainder (`fmod`) shifted into the divisor's sign. Not
/// `n - d*floor(n/d)`, which loses precision for tiny divisors; evidence: `MOD_QUOTIENT.xlsx`
/// C20/C21/C25/C26 (docs/excel-semantics/numbers.md). `divisor` must be non-zero. `None` is
/// Excel's `#NUM!` for a remainder that passes below the smallest normal double.
pub(crate) fn excel_mod(value: f64, divisor: f64) -> Option<f64> {
    let remainder = value % divisor;
    if remainder == 0.0 {
        // Drop the sign of -0.0 (fmod keeps the dividend's sign).
        return Some(0.0);
    }
    // Excel: a subnormal remainder, or a subnormal shifted result, is #NUM!; when both are
    // subnormal the result is `d - fmod` if |d| is exactly 2^-1022 and #NUM! otherwise
    // (`MOD(-(2^-1021*(1+2^-52)),2^-1022)` = 2^-1022+2^-1073, probes/t2714-mod-near-smallest-
    // normal, -b, -c; numbers.md "MOD near the smallest normal").
    let is_subnormal = |x: f64| x.abs() < f64::MIN_POSITIVE;
    if (remainder < 0.0) == (divisor < 0.0) {
        return (!is_subnormal(remainder)).then_some(remainder);
    }
    let shifted = remainder + divisor;
    match (is_subnormal(remainder), is_subnormal(shifted)) {
        (true, true) if divisor.abs() == f64::MIN_POSITIVE => Some(divisor - remainder),
        (false, false) => {
            // A shift that rounds back to the divisor is 0: `MOD(1,-(2^54))` = 0 but
            // `MOD(1,-(2^53))` = 1-2^53 (probes/mod-huge-opposite-divisor; numbers.md "MOD of
            // a small number by a huge opposite-sign divisor is 0").
            Some(if shifted == divisor { 0.0 } else { shifted })
        }
        _ => None,
    }
}

impl Function {
    /// XLOOKUP, TAKE, DROP and TRIMRANGE: over a range they evaluate to the cells they pick, so
    /// SUBTOTAL, AREAS and a LET or LAMBDA binding read a reference
    /// (`Model::evaluate_picked_reference_arg`, probes/subtotal-reference-operands-b H12,
    /// H15:H17; probes/let-name-uncalled-picked-refs). The one list of these calls, with the
    /// index of the argument each picks from: XLOOKUP's `return_array` (2), the array (0) of
    /// the others.
    pub(crate) fn picked_reference_arg(&self) -> Option<usize> {
        match self {
            Function::Xlookup => Some(2),
            Function::Take | Function::Drop | Function::Trimrange => Some(0),
            _ => None,
        }
    }

    /// Whether the call is one of [`Function::picked_reference_arg`]'s.
    pub(crate) fn picks_reference(&self) -> bool {
        self.picked_reference_arg().is_some()
    }

    /// The built-ins whose result can be a reference they pick or build: INDEX, OFFSET and INDIRECT, the arm a
    /// scalar IF, CHOOSE, SWITCH or IFS takes, a LET body, and [`Function::picks_reference`]'s.
    /// The parser takes them as reference operands (`is_reference_operand`) and FORMULATEXT
    /// reads the reference they pick (`fn_formulatext`, T-2726).
    pub(crate) fn may_be_reference(&self) -> bool {
        matches!(
            self,
            Function::Index
                | Function::Offset
                | Function::Indirect
                | Function::If
                | Function::Choose
                | Function::Switch
                | Function::Ifs
                | Function::Let
        ) || self.picks_reference()
    }
}
