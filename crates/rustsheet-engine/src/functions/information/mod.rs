mod isomitted;
mod offline_errors;

use crate::external_reference::{may_be_external, PickedReference};
use crate::{
    calc_result::{CalcResult, Range},
    cast::{calc_result_to_array_node, MAX_SHORT_TEXT_UNITS},
    defined_names::ParsedDefinedName,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
        utils::number_to_column,
    },
    functions::Function,
    limits::check_range_size,
    model::Model,
    reference_operators::{Ranges, RefArg},
};

/// Whether `node` is `A1:Sheet!B1`, an unqualified cell joined to a sheet-qualified one. Over
/// sheets named like cells an unquoted span reads this way: written on sheet S1,
/// `SHEETS(S1:S3!A1)` is `S1:(S3!A1)`, which cannot join across sheets and is `#VALUE!`, while
/// `SHEETS(S3:S1!A1)` joins on S1 and is 1 (probes/sheet-names-like-cells-3d-spans B3:B4,
/// excel-graph; T-2193). A range operator between two sheet-qualified cells
/// (`Alpha!A1:Beta!B1`) is unprobed: the oracle canary refuses it.
fn joins_a_cell_to_a_sheet_qualified_one(node: &Node) -> bool {
    matches!(node, Node::OpRangeKind { left, right }
        if matches!(left.ungrouped(), Node::ReferenceKind { sheet_name: None, .. })
            && matches!(right.ungrouped(), Node::ReferenceKind { sheet_name: Some(_), .. }))
}

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

/// N of one array element: a number stays, TRUE is 1, text, FALSE and blanks are 0, an error stays.
fn n_of(value: &ArrayNode) -> ArrayNode {
    match value {
        ArrayNode::Number(n) => ArrayNode::Number(*n),
        ArrayNode::Boolean(b) => ArrayNode::Number(f64::from(u8::from(*b))),
        ArrayNode::String(_) | ArrayNode::Empty => ArrayNode::Number(0.0),
        ArrayNode::Error(error) => ArrayNode::Error(error.clone()),
        ArrayNode::Lambda(_) | ArrayNode::Nested(_) => ArrayNode::Error(Error::VALUE),
    }
}

impl Model {
    /// Shared body of the IS* type tests (ISNUMBER, ISERROR, ...).
    ///
    /// A multi-cell range or array argument is tested element by element and
    /// the result spills with the same shape (probes/is-functions-array-lift,
    /// excel-graph; docs/excel-semantics/information.md). A single value
    /// yields one boolean.
    fn is_predicate(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        test: impl Fn(&ArrayNode) -> bool,
    ) -> CalcResult {
        let is_nontext = kind == Function::Isnontext;
        self.lift_call(kind, args, cell, &mut |_, values| {
            let [value] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            // An uncalled LAMBDA or function name is a value of its own, neither an error nor
            // text: only ISNONTEXT is TRUE (`ISERROR(_xleta.SUM)` is FALSE; probes/eta-reduced-names
            // E93, probes/lambda-value-arguments A1-A14, A26; arrays.md "An uncalled LAMBDA is one
            // value").
            if matches!(value, CalcResult::Lambda(_)) {
                return CalcResult::Boolean(is_nontext);
            }
            CalcResult::Boolean(test(&calc_result_to_array_node(value.clone())))
        })
    }

    pub(crate) fn fn_isnumber(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(Function::Isnumber, args, cell, |v| {
            matches!(v, ArrayNode::Number(_))
        })
    }
    pub(crate) fn fn_istext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(Function::Istext, args, cell, |v| {
            matches!(v, ArrayNode::String(_))
        })
    }
    pub(crate) fn fn_isnontext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(Function::Isnontext, args, cell, |v| {
            !matches!(v, ArrayNode::String(_))
        })
    }
    pub(crate) fn fn_islogical(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(Function::Islogical, args, cell, |v| {
            matches!(v, ArrayNode::Boolean(_))
        })
    }
    pub(crate) fn fn_isblank(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(Function::Isblank, args, cell, |v| {
            matches!(v, ArrayNode::Empty)
        })
    }
    pub(crate) fn fn_iserror(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(Function::Iserror, args, cell, |v| {
            matches!(v, ArrayNode::Error(_))
        })
    }
    pub(crate) fn fn_iserr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(
            Function::Iserr,
            args,
            cell,
            |v| matches!(v, ArrayNode::Error(e) if *e != Error::NA),
        )
    }
    pub(crate) fn fn_isna(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(Function::Isna, args, cell, |v| {
            matches!(v, ArrayNode::Error(Error::NA))
        })
    }

    // Returns true if it is a reference or evaluates to a reference
    // But DOES NOT evaluate
    pub(crate) fn fn_isref(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::Boolean(self.is_reference(&args[0], cell))
    }

    /// Whether `node` stands for a reference (ISREF).
    fn is_reference(&mut self, node: &Node, cell: CellReferenceIndex) -> bool {
        // `ISREF(IF(TRUE,…!A1))` is TRUE (probes/external-ref-no-link-ifchoose A9, A10, A31).
        // A reference into another workbook, written or through INDEX, a union, an
        // intersection or LET (probes/external-ref-no-link-shapes A4, external-ref-no-link-index
        // A28, A33, external-ref-no-link-let A1, A10); one into two workbooks is `#VALUE!`, so
        // not a reference.
        // So is one a LET whose body may pick it picks: `ISREF(LET(c,TRUE,IF(c,…!A1,D1)))` is
        // TRUE (probes/external-ref-no-link-let-index A6).
        let node = match self.pick_reference(node, cell) {
            PickedReference::Node(node) => node,
            PickedReference::External(areas) => return areas.is_ok(),
            PickedReference::Read(read) => return matches!(read, RefArg::Ranges(_)),
            PickedReference::Value(_) => return false,
        };
        // A `#This Row` area is the cell or range on the formula's row, `#VALUE!` off the data rows
        // (SPEC-0016 step 2).
        let node: &Node = &node.ungrouped_at(cell.row);
        // OFFSET of one is `#VALUE!`, so not a reference (probes/external-ref-no-link-index
        // A16), also of one a LET picks (probes/external-ref-no-link-let-index A3), where OFFSET
        // otherwise counts as one unevaluated (`kind.returns_reference()`).
        if let Node::FunctionKind {
            kind: crate::functions::Function::Offset,
            args,
        } = node
        {
            if args.first().is_some_and(may_be_external) {
                return matches!(self.offset_reference(args, cell), RefArg::Ranges(_));
            }
            // A lifted OFFSET is an array of references, not a reference: `ISREF(OFFSET(A1,{0;1},0))`
            // is FALSE (probes/reference-lift-elements F15).
            if self.offset_lifts(args, cell) {
                return false;
            }
        }
        // So is a lifted INDIRECT or TRIMRANGE (probes/reference-lift-elements).
        if let Node::FunctionKind { kind, args } = node {
            let lifted = match kind {
                crate::functions::Function::Indirect => self.indirect_lifts(args, cell),
                crate::functions::Function::Trimrange => self.trimrange_lifts(args, cell),
                _ => false,
            };
            if lifted {
                return false;
            }
        }
        match node {
            Node::ReferenceKind { .. } | Node::RangeKind { .. } => true,
            // A range whose corners are on two sheets is `#VALUE!`, so not a reference:
            // `ISREF(Data!A1:Other!B2)` is FALSE (probes/sheet-qualified-corner-order Calc!A7).
            Node::OpRangeKind { left, right } => {
                matches!(self.get_range(left, right, cell), CalcResult::Range { .. })
            }
            // An intersection with no shared cell is `#NULL!`, not a reference: `ISREF(A1 B2)` is
            // FALSE and `ISREF(A1 A1)` TRUE (probes/isref-union-operands H3, H7; excel-graph).
            // A union with an error operand is that error, not a reference: `ISREF((A1,#REF!))`
            // is FALSE (probes/union-accepted H9).
            // IF and CHOOSE are references when the argument they pick is one: `ISREF(IF(TRUE,
            // A1))` and `ISREF(CHOOSE(1,A1))` are TRUE, with a value FALSE (probes/isref-function-
            // results C3:C6, excel-graph; T-0235). So is a LET whose result is a variable bound to
            // a reference: `ISREF(LET(x,A1:A3,x))` is TRUE (probes/written-parens-reference-only
            // E40, E45, excel-graph; T-1106), and a LAMBDA call that returns one:
            // `ISREF(LAMBDA(x,x)(A1))` is TRUE (probes/let-lambda-reference-results C1, C2).
            // IFS and SWITCH too, and an XLOOKUP or TAKE any of them picks over a range:
            // `ISREF(IFS(TRUE,H1))`, `ISREF(CHOOSE(1,XLOOKUP(1,J1:J2,H1:I2)))` are TRUE
            // (probes/subtotal-ifs-switch-reference-picks, subtotal-if-choose-over-xlookup A94;
            // T-2014).
            Node::OpIntersectKind { .. }
            | Node::OpUnionKind { .. }
            | Node::LambdaCallKind { .. }
            | Node::FunctionKind {
                kind:
                    crate::functions::Function::If
                    | crate::functions::Function::Choose
                    | crate::functions::Function::Ifs
                    | crate::functions::Function::Switch
                    | crate::functions::Function::Let,
                ..
            } => matches!(
                self.evaluate_picked_reference_arg(node, cell),
                RefArg::Ranges(_)
            ),
            // The `@` of an XLOOKUP or TAKE that picks one cell keeps the reference too
            // (probes/isref-implicit-xlookup-take, T-1509).
            Node::ImplicitIntersection { child, .. }
                if matches!(
                    child.ungrouped(),
                    Node::FunctionKind {
                        kind: crate::functions::Function::Xlookup
                            | crate::functions::Function::Take,
                        ..
                    }
                ) =>
            {
                matches!(
                    self.eval_at_picked_reference(node, cell),
                    Some(CalcResult::Range { .. })
                )
            }
            // The `@` of an INDEX that picks one cell keeps the reference: `ISREF(@INDEX(B1:B3,2))`
            // is TRUE (probes/let-bind-implicit-index H10, excel-graph; T-1506).
            Node::ImplicitIntersection { child, .. }
                if matches!(
                    child.ungrouped(),
                    Node::FunctionKind {
                        kind: crate::functions::Function::Index,
                        ..
                    }
                ) =>
            {
                matches!(self.eval_ref_or_index(node, cell), CalcResult::Range { .. })
            }
            // INDEX, XLOOKUP, TAKE, DROP and TRIMRANGE are references when they pick cells of a
            // range, and not when they fail or work on an array (`ISREF(INDEX(A1,1,2,2))` is
            // FALSE; probes/isref-reference-lookups).
            Node::FunctionKind { kind, args } => {
                match self.evaluate_reference_form(kind, args, cell) {
                    Some(result) => matches!(result, CalcResult::Range { .. }),
                    // INDIRECT of text that is no reference is `#REF!`, not a reference:
                    // `ISREF(INDIRECT("R0"))` is FALSE (probes/r1c1-name-use-r0, excel-graph; T-1657).
                    None if matches!(kind, crate::functions::Function::Indirect) => matches!(
                        self.evaluate_node_with_reference(node, cell),
                        CalcResult::Range { .. }
                    ),
                    None => kind.returns_reference(),
                }
            }
            // A defined name is a reference if what it stands for is one, a single cell included
            // (Excel-cached: upstream ISREF.xlsx Sheet1!A14-A18; probes/isref-names-filter-if-empty),
            // and so is a name whose body is a reference formula (`=INDEX(...)`, an intersection)
            // but not one whose body is a value (`=SUM(...)`, `=total*2`): probes/defined-name-
            // intersection H2, H9 and probes/defined-name-formula-bodies H7, H8, H10, H16.
            Node::DefinedNameKind((name, scope, _)) => {
                match self.parsed_defined_name(name, *scope) {
                    Some(
                        ParsedDefinedName::CellReference(_) | ParsedDefinedName::RangeReference(_),
                    ) => true,
                    Some(ParsedDefinedName::Formula(body)) => {
                        let body = std::sync::Arc::clone(body);
                        self.within_defined_name(|model| model.is_reference(&body, cell))
                            .unwrap_or(false)
                    }
                    _ => false,
                }
            }
            // A LET variable or LAMBDA parameter is a reference if it was bound to one
            // (upstream MIN_MAX_extras.xlsx A14-A16).
            Node::NamedVariableKind { .. } => matches!(
                self.evaluate_node_with_reference(node, cell),
                CalcResult::Range { .. }
            ),
            _ => false,
        }
    }

    /// Shared body of ISEVEN / ISODD.
    ///
    /// A multi-cell range argument is `#VALUE!`, with no implicit intersection and no spill,
    /// wherever the formula sits; an array constant lifts element by element
    /// (probes/is-functions-array-lift J2, K2, L3, M30, M32; excel-graph;
    /// docs/excel-semantics/information.md).
    fn lifted_parity(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        even: bool,
    ) -> CalcResult {
        self.lift_call(kind, args, cell, &mut |model, values| {
            let [value] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            model.parity_of(value.clone(), even, cell)
        })
    }

    fn parity_of(&mut self, value: CalcResult, even: bool, cell: CellReferenceIndex) -> CalcResult {
        // Parity is taken on the truncated absolute value (4.5 is even, -7 is odd:
        // probes/is-functions-array-lift, Data!C3:C4); a boolean is #VALUE!, text is cast.
        // The remainder is taken in f64: every double from 2^53 up is an even integer, so
        // ISODD(INT(1E+21)) is FALSE (an i64 cast would saturate to odd; gen-m5 ee1667411bbe2ccc).
        let value = self.dereference(value, cell);
        if matches!(value, CalcResult::Boolean(_)) {
            return CalcResult::new_error(Error::VALUE, cell, "Expecting number".to_string());
        }
        // `ISEVEN(LAMBDA(x,x))` is `#VALUE!` (probes/lambda-value-arguments A23).
        if matches!(value, CalcResult::Lambda(_)) {
            return Self::lambda_as_number_error(cell);
        }
        // Excel adds 1E-10 to the absolute value before truncating, so a value at most 1E-10 below
        // an integer counts as that integer: ISODD(7-1E-10) is TRUE, ISODD(7-1.01E-10) FALSE, at 7,
        // 1001 and 1000001 alike, and ISEVEN(POWER(49,0.5-1E-15)) (6.999999999999972) is FALSE
        // (probes/iseven-isodd-below-integer, -rounding-threshold, -rounding-window, excel-graph;
        // docs/excel-semantics/information.md "ISEVEN of a value just below an integer", T-2382).
        match self.cast_to_number(value, cell) {
            Ok(f) => CalcResult::Boolean(((f.abs() + 1E-10).trunc() % 2.0 == 0.0) == even),
            Err(error) => error,
        }
    }

    pub(crate) fn fn_isodd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lifted_parity(Function::Isodd, args, cell, false)
    }

    pub(crate) fn fn_iseven(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lifted_parity(Function::Iseven, args, cell, true)
    }

    // ISFORMULA arg needs to be a reference or something that evaluates to a reference
    pub(crate) fn fn_isformula(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let (left, right) = match self.get_reference(&args[0], cell) {
            Ok(range) => (range.left, range.right),
            Err(error) => return error,
        };
        if left.sheet != right.sheet {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "3D ranges not supported".to_string(),
            };
        }
        if left == right {
            return CalcResult::Boolean(self.cell_has_formula(left));
        }
        // A multi-cell reference lifts, one flag per cell (probes/info-logic-argument-modes A11:B11,
        // excel-graph); cells outside the used area hold no formula.
        if let Some((error, message)) = check_range_size(&left, &right) {
            return CalcResult::new_error(error, cell, message);
        }
        let (last_row, last_column) = match self.clip_to_used_area(
            left.sheet,
            left.row,
            left.column,
            right.row,
            right.column,
        ) {
            Ok(last) => last,
            Err(message) => return CalcResult::new_error(Error::ERROR, cell, message),
        };
        let flags = (left.row..=right.row)
            .map(|row| {
                (left.column..=right.column)
                    .map(|column| {
                        let inside = row <= last_row && column <= last_column;
                        let at = CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        };
                        ArrayNode::Boolean(inside && self.cell_has_formula(at))
                    })
                    .collect()
            })
            .collect();
        CalcResult::Array(flags)
    }

    /// A data table's result cells count as formulas (`{=TABLE(,A1)}`; recalc.md).
    fn cell_has_formula(&self, at: CellReferenceIndex) -> bool {
        self.data_table_at(at).is_some()
            || matches!(
                self.get_cell_formula(at.sheet, at.row, at.column),
                Ok(Some(_))
            )
    }

    /// Lifts over a multi-cell range or array like the IS* tests (probes/rich-errors-arg-kinds
    /// F10:G10, F40:G41: `ERROR.TYPE(FIELDVALUE(Q1:R2,"Price"))` spills 13s).
    pub(crate) fn fn_errortype(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(
            Function::ErrorType,
            args,
            cell,
            &mut |_, values| match values {
                [CalcResult::Error { error, .. }] => {
                    CalcResult::Number(f64::from(error.error_type_code()))
                }
                // Excel: text over 255 UTF-16 units reads as #VALUE! here though ISERROR is
                // FALSE for it (probes/type-long-text B4-B10, C3).
                [CalcResult::String(text)] if text.len_utf16() > MAX_SHORT_TEXT_UNITS => {
                    CalcResult::Number(f64::from(Error::VALUE.error_type_code()))
                }
                [_] => CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Not an error".to_string(),
                },
                _ => CalcResult::new_args_number_error(cell),
            },
        )
    }

    // Excel believes for some reason that TYPE(A1:A7) is an array formula
    // Although we evaluate the same as Excel we cannot, ATM import this from excel
    pub(crate) fn fn_type(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let mut value = self.evaluate_node_in_context(&args[0], cell);
        // Excel: a one-cell range is its cell's type, `TYPE(D7:D7)` is 1 (docs/excel-semantics/information.md
        // "TYPE of a one-cell range", gen-m5/2abdc1524e5aa685 Calc!A371).
        if let CalcResult::Range { left, right } = &value {
            if left == right {
                value = self.evaluate_cell(*left);
            }
        }
        match value {
            // Excel: text over 255 UTF-16 units is 64, not 2 (probes/type-long-text A4-A10, C7).
            CalcResult::String(text) if text.len_utf16() > MAX_SHORT_TEXT_UNITS => {
                CalcResult::Number(64.0)
            }
            CalcResult::String(_) => CalcResult::Number(2.0),
            CalcResult::Number(_) => CalcResult::Number(1.0),
            CalcResult::Boolean(_) => CalcResult::Number(4.0),
            CalcResult::Error { .. } => CalcResult::Number(16.0),
            CalcResult::Range { .. } => CalcResult::Number(64.0),
            // `TYPE(IFERROR(<blank>,1))` is 1 (probes/iferror-blank-value D14, T-0291).
            CalcResult::EmptyCell | CalcResult::EmptyArg => CalcResult::Number(1.0),
            CalcResult::Array(_) => CalcResult::Number(64.0),
            // `TYPE(LAMBDA(x,x))` is 128 (probes/is-functions-scalar-kinds Calc!F24).
            CalcResult::Lambda(_) => CalcResult::Number(128.0),
        }
    }
    pub(crate) fn fn_sheet(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if arg_count == 0 {
            // Sheets are 0-indexed`
            return CalcResult::Number(cell.sheet as f64 + 1.0);
        }
        // The arg could be a defined name or a table
        match args[0].ungrouped() {
            Node::DefinedNameKind((name, scope, _)) => {
                // Let's see if it is a defined name
                if let Some(defined_name) = self.parsed_defined_name(name, *scope) {
                    match defined_name {
                        ParsedDefinedName::CellReference(reference)
                        | ParsedDefinedName::RangeReference(reference) => {
                            return CalcResult::Number(f64::from(reference.sheet()) + 1.0)
                        }
                        // Excel: SHEET of a name whose body is an error literal is that error
                        // (docs/excel-semantics/references.md "A sheet-qualified #REF! is #REF!").
                        ParsedDefinedName::ErrorLiteral(error) => {
                            return CalcResult::new_error(
                                error.clone(),
                                cell,
                                format!("Defined name \"{name}\" is an error."),
                            );
                        }
                        ParsedDefinedName::InvalidDefinedNameFormula
                        | ParsedDefinedName::Formula(_)
                        | ParsedDefinedName::LambdaDefinition(..) => {
                            return CalcResult::Error {
                                error: Error::ERROR,
                                origin: cell,
                                message: "Invalid name".to_string(),
                            };
                        }
                    }
                } else {
                    // This should never happen
                    return CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "Invalid name".to_string(),
                    };
                }
            }
            Node::NamedVariableKind { name, id: _ } => {
                return CalcResult::Error {
                    error: Error::NAME,
                    origin: cell,
                    message: format!("Name not found: {name}"),
                }
            }
            arg => {
                // A reference (a range included) gives its sheet's number, with no spill
                // (probes/info-logic-argument-modes A12, excel-graph).
                // Reached through INDEX, IF or a union it is the same: a union of areas on one
                // sheet gives that sheet (probes/volatile-and-info-rand-true-false-sheet D18, D19, D26).
                match self.evaluate_reference_arg(arg, cell) {
                    RefArg::Ranges(ranges) => {
                        if let Some(first) = ranges.first() {
                            return CalcResult::Number(first.left.sheet as f64 + 1.0);
                        }
                    }
                    // An array of references is neither lifted nor read as a value: one #N/A
                    // (`SHEET(OFFSET(A1,{0;1},0))`, probes/reference-lift-consumers-b1d E101;
                    // SPEC-0014 rule 2).
                    RefArg::Grid(_) => {
                        return CalcResult::new_error(
                            Error::NA,
                            cell,
                            "SHEET of an array of references".to_string(),
                        )
                    }
                    RefArg::Value(_) => {}
                }
                // Now it should be the name of a sheet
                let sheet_name = match self.get_string(arg, cell) {
                    Ok(s) => s,
                    Err(e) => return e,
                };
                if let Some(sheet_index) = self.get_sheet_index_by_name(&sheet_name) {
                    return CalcResult::Number(sheet_index as f64 + 1.0);
                }
            }
        }
        CalcResult::Error {
            error: Error::NA,
            origin: cell,
            message: "Invalid name".to_string(),
        }
    }

    pub(crate) fn fn_n(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.n_of_argument(args, cell, false)
    }

    /// N in a legacy formula's value position (SPEC-0012 rule 3): an array argument is `#VALUE!`.
    pub(crate) fn fn_n_legacy(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.n_of_argument(args, cell, true)
    }

    /// N's kernel; `legacy` makes an array argument `#VALUE!`, whatever its size.
    fn n_of_argument(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        legacy: bool,
    ) -> CalcResult {
        let arg_count = args.len();
        if arg_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // A range is its top-left cell, with no lift and no implicit intersection
        // (probes/info-logic-argument-modes A6:A7, excel-graph).
        let argument = self.evaluate_node_top_left(&args[0], cell);
        let value = match argument {
            CalcResult::Number(n) => n,
            CalcResult::String(_) => 0.0,
            CalcResult::Boolean(f) => f64::from(u8::from(f)),
            CalcResult::EmptyCell | CalcResult::EmptyArg => 0.0,
            error @ CalcResult::Error { .. } => return error,
            // An array lifts element-wise, an error element kept: `N({1,#N/A})` spills 1, #N/A,
            // `N(IF(1,{2;3}))` 2, 3, and legacy `SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))` is 5
            // (probes/n-array-lift, legacy-index-array-index L1, L10; T-1616).
            // In a legacy formula's value position it is `#VALUE!`, `{2}` included
            // (probes/legacy-n-t-reference-args N10; SPEC-0012).
            CalcResult::Array(_) if legacy => {
                return CalcResult::new_error(Error::VALUE, cell, String::new())
            }
            CalcResult::Array(values) => return CalcResult::Array(map_elements(&values, n_of)),
            CalcResult::Range { .. } | CalcResult::Lambda(_) => {
                return CalcResult::Error {
                    error: Error::NIMPL,
                    origin: cell,
                    message: "Arrays not supported yet".to_string(),
                }
            }
        };

        CalcResult::Number(value)
    }

    pub(crate) fn fn_sheets(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if arg_count == 1 {
            // A reference counts the sheets between its corners, in either order
            // (probes/info-sheets-info-types D1:D24, excel-graph; T-2181).
            // Only a written reference is read: a union, a name, OFFSET, INDIRECT, INDEX, an
            // IF/CHOOSE pick and a non-reference argument are unprobed (T-2113): left `#N/IMPL!`. So are a
            // range operator over function operands (`A1:OFFSET(..)`) and a cross-sheet one
            // other than [`joins_a_cell_to_a_sheet_qualified_one`] (`Alpha!A1:Beta!B1`).
            // A `#This Row` area is its cell or range on the formula's row (SPEC-0016 step 2).
            let argument: &Node = &args[0].ungrouped_at(cell.row);
            if matches!(
                argument,
                Node::ReferenceKind { .. }
                    | Node::RangeKind { .. }
                    | Node::OpRangeKind { .. }
                    | Node::SheetSpanKind { .. }
            ) {
                match self.evaluate_reference_arg(&args[0], cell) {
                    RefArg::Ranges(Ranges::One(area)) => {
                        return CalcResult::Number(
                            f64::from(area.left.sheet.abs_diff(area.right.sheet)) + 1.0,
                        );
                    }
                    RefArg::Value(error @ CalcResult::Error { .. })
                        if joins_a_cell_to_a_sheet_qualified_one(argument) =>
                    {
                        return error;
                    }
                    _ => {}
                }
            }
            return CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Sheets function with this argument is not implemented".to_string(),
            };
        }
        let sheet_count = self.workbook.worksheets.len() as f64;
        CalcResult::Number(sheet_count)
    }

    /// `CELL(info_type`, [reference])
    /// NB: In Excel "`info_type`" is localized. Here it is always in English.
    pub(crate) fn fn_cell(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count == 0 || arg_count > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        // A union is read through its first area for every info type but "address", which lists
        // them all (probes/multiarea-cellfn-types H1:H7).
        let ranges = if arg_count == 2 {
            match self.reference_ranges(&args[1], cell) {
                Ok(ranges) => Some(ranges),
                Err(error) => return error,
            }
        } else {
            None
        };
        let reference = match ranges.as_deref().and_then(<[_]>::first) {
            // we just take the top-left cell of the reference
            Some(range) => range.left,
            None => cell,
        };
        // An array info_type lifts: one result per element (CELL({"row","col"},B2) spills 2, 2;
        // probes/info-logic-argument-modes A27:B27, excel-graph).
        // A multi-cell range lifts the same way (`CELL(INDEX(D2:E9,0,1),A9)` is `#SPILL!` under a
        // filled column; gen-m5/c0651f0e0fb544d0 Calc!A293, excel-graph, information.md).
        let info_type = match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::String(s) => s.into_lossy_string(),
            CalcResult::Range { left, right } if left != right && left.sheet == right.sheet => {
                return match self.evaluate_range(left, right, cell) {
                    Ok(types) => self.lifted_cell(&types, ranges.as_deref(), reference, cell),
                    Err(error) => error,
                };
            }
            CalcResult::Array(types) => {
                return self.lifted_cell(&types, ranges.as_deref(), reference, cell);
            }
            _ => match self.get_string(&args[0], cell) {
                Ok(s) => s,
                Err(e) => return e,
            },
        };
        self.cell_info(&info_type, ranges.as_deref(), reference, cell)
    }

    /// One `CELL` result per element of an `info_type` grid; an error element passes through and
    /// any other non-text element (number, blank) is `#VALUE!` (probes/
    /// info-cell-info-type-grid-errors, excel-graph, T-2232).
    fn lifted_cell(
        &mut self,
        types: &[Vec<ArrayNode>],
        ranges: Option<&[Range]>,
        reference: CellReferenceIndex,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let lifted = types
            .iter()
            .map(|row| {
                row.iter()
                    .map(|item| match item {
                        ArrayNode::String(name) => {
                            calc_result_to_array_node(self.cell_info(name, ranges, reference, cell))
                        }
                        ArrayNode::Error(error) => ArrayNode::Error(error.clone()),
                        _ => ArrayNode::Error(Error::VALUE),
                    })
                    .collect()
            })
            .collect();
        CalcResult::Array(lifted)
    }

    /// The workbook's file name as CELL names it (`Budget.xlsx`), `None` for an unnamed workbook
    /// (an empty `Workbook::name`, which holds the name without its extension).
    fn workbook_file_name(&self) -> Option<String> {
        let name = &self.workbook.name;
        (!name.is_empty()).then(|| format!("{name}.xlsx"))
    }

    /// The value of one CELL `info_type` for `reference`; `ranges` holds every area of a union.
    fn cell_info(
        &mut self,
        info_type: &str,
        ranges: Option<&[Range]>,
        reference: CellReferenceIndex,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match info_type.to_uppercase().as_str() {
            "ADDRESS" => {
                // A reference on another sheet is prefixed `[<file>]<sheet>!`, the sheet name never
                // quoted (`[b.xlsx]My Sheet!$B$2`, `[b.xlsx]Bob's!$A$1`; probes/
                // info-cell-address-cross-sheet, excel-graph, T-2042).
                let prefix = if reference.sheet == cell.sheet {
                    String::new()
                } else {
                    let unimplemented = |message: &str| CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: message.to_string(),
                    };
                    // Excel's form for an unsaved workbook is unprobed.
                    let Some(file_name) = self.workbook_file_name() else {
                        return unimplemented("Unnamed workbook");
                    };
                    // A union cannot span sheets in Excel; one on another sheet is unprobed.
                    if ranges.is_some_and(|areas| areas.len() > 1) {
                        return unimplemented("Union on another sheet not implemented");
                    }
                    match self.workbook.worksheet(reference.sheet) {
                        Ok(ws) => format!("[{file_name}]{}!", ws.name),
                        Err(_) => return unimplemented("Invalid sheet"),
                    }
                };
                let lefts: Vec<CellReferenceIndex> = match ranges {
                    Some(areas) => areas.iter().map(|range| range.left).collect(),
                    None => vec![reference],
                };
                let mut addresses = Vec::with_capacity(lefts.len());
                for area in &lefts {
                    let Some(column) = number_to_column(area.column) else {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Invalid column".to_string(),
                        };
                    };
                    addresses.push(format!("${}${}", column, area.row));
                }
                CalcResult::String(format!("{prefix}{}", addresses.join(", ")).into())
            }
            "COL" => CalcResult::Number(reference.column as f64),
            "COLOR" | "FORMAT" | "PARENTHESES" | "PREFIX" | "PROTECT" | "WIDTH" => {
                CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "info_type not implemented".to_string(),
                }
            }
            // The top-left cell of the reference taken on the formula's own sheet, whatever sheet
            // the reference names, with no implicit intersection: `CELL("contents",Data!A2)` on
            // Calc reads Calc!A2 (probes/info-cell-contents-range, excel-graph, T-1230). Desktop
            // Excel is unverified (T-1075 item 9); excel-graph is the oracle of record (decision 0039).
            "CONTENTS" => self.evaluate_cell(CellReferenceIndex {
                sheet: cell.sheet,
                ..reference
            }),
            // Excel for the web gives #VALUE! for every form (probes/info-cell-address-cross-sheet
            // Calc!B1:B4, excel-graph, T-2052); desktop's full-path form is M14.
            "FILENAME" => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "CELL(\"filename\") is #VALUE! in Excel for the web".to_string(),
            },
            "ROW" => CalcResult::Number(reference.row as f64),
            // Like "contents", "type" reads the reference's address on the formula's own sheet
            // (`CELL("type",Data!A7)` is "b" when Calc!A7 is blank although Data!A7 is text;
            // probes/info-cell-legacy-range J1:K3, excel-graph, T-1433).
            "TYPE" => {
                let cell_type = match self.evaluate_cell(CellReferenceIndex {
                    sheet: cell.sheet,
                    ..reference
                }) {
                    CalcResult::EmptyCell => "b",
                    CalcResult::String(_) => "l",
                    CalcResult::Number(_) => "v",
                    CalcResult::Boolean(_) => "v",
                    CalcResult::Error { .. } => "v",
                    CalcResult::Range { .. } => "v",
                    CalcResult::EmptyArg => "v",
                    CalcResult::Array(_) | CalcResult::Lambda(_) => "v",
                };
                CalcResult::String(cell_type.to_string().into())
            }
            _ => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Invalid info_type".to_string(),
            },
        }
    }

    /// `INFO(text_type)`
    ///
    /// `#VALUE!` for every `type_text` (valid, legacy, empty or unknown, in any case) in Excel for
    /// the web, the oracle of record (decision 0039 rule 2): probes/info-sheets-info-types
    /// Alpha!D25:D49, excel-graph, 2026-10-20. The desktop values are T-2182 (M14).
    pub(crate) fn fn_info(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if let Err(e) = self.get_string(&args[0], cell) {
            return e;
        }
        CalcResult::Error {
            error: Error::VALUE,
            origin: cell,
            message: "INFO is not available".to_string(),
        }
    }
}
