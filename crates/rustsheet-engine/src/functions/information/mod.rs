mod isomitted;
mod offline_errors;
pub(crate) use offline_errors::error_array;

use crate::external_reference::PickedReference;
use crate::{
    calc_result::{CalcResult, Range},
    cast::calc_result_to_array_node,
    defined_names::ParsedDefinedName,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
        utils::number_to_column,
    },
    functions::util::array_node_to_calc_result,
    get_all_timezones,
    limits::check_range_size,
    model::Model,
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

/// N of one array element: a number stays, TRUE is 1, text, FALSE and blanks are 0, an error stays.
fn n_of(value: &ArrayNode) -> ArrayNode {
    match value {
        ArrayNode::Number(n) => ArrayNode::Number(*n),
        ArrayNode::Boolean(b) => ArrayNode::Number(f64::from(u8::from(*b))),
        ArrayNode::String(_) | ArrayNode::Empty => ArrayNode::Number(0.0),
        ArrayNode::Error(error) => ArrayNode::Error(error.clone()),
    }
}

/// An argument that lifts: a multi-cell range or an array gives its values, anything else
/// (a one-cell range included) a single value.
pub(crate) enum Lifted {
    Scalar(CalcResult),
    Array(Vec<Vec<ArrayNode>>),
}

impl Lifted {
    /// Rows and columns; a single value is 1x1.
    fn dims(&self) -> (usize, usize) {
        match self {
            Lifted::Array(values) => (values.len(), values.iter().map(Vec::len).max().unwrap_or(0)),
            Lifted::Scalar(_) => (1, 1),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_system() -> String {
    std::env::consts::OS.to_string()
}

#[cfg(target_arch = "wasm32")]
fn get_system() -> String {
    "browser".to_string()
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
        args: &[Node],
        cell: CellReferenceIndex,
        test: impl Fn(&ArrayNode) -> bool,
    ) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.lifted_arg(&args[0], cell) {
            Ok(Lifted::Array(values)) => {
                CalcResult::Array(map_elements(&values, |v| ArrayNode::Boolean(test(v))))
            }
            Ok(Lifted::Scalar(other)) => {
                CalcResult::Boolean(test(&calc_result_to_array_node(other)))
            }
            Err(error) => error,
        }
    }

    /// Evaluates an argument that lifts over a multi-cell range or array.
    pub(crate) fn lifted_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Lifted, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, right } if left == right => {
                Ok(Lifted::Scalar(self.evaluate_cell(left)))
            }
            CalcResult::Range { left, right } => {
                Ok(Lifted::Array(self.evaluate_range(left, right, cell)?))
            }
            CalcResult::Array(values) => Ok(Lifted::Array(values)),
            other => Ok(Lifted::Scalar(other)),
        }
    }

    pub(crate) fn fn_isnumber(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(args, cell, |v| matches!(v, ArrayNode::Number(_)))
    }
    pub(crate) fn fn_istext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(args, cell, |v| matches!(v, ArrayNode::String(_)))
    }
    pub(crate) fn fn_isnontext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(args, cell, |v| !matches!(v, ArrayNode::String(_)))
    }
    pub(crate) fn fn_islogical(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(args, cell, |v| matches!(v, ArrayNode::Boolean(_)))
    }
    pub(crate) fn fn_isblank(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(args, cell, |v| matches!(v, ArrayNode::Empty))
    }
    pub(crate) fn fn_iserror(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(args, cell, |v| matches!(v, ArrayNode::Error(_)))
    }
    pub(crate) fn fn_iserr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(
            args,
            cell,
            |v| matches!(v, ArrayNode::Error(e) if *e != Error::NA),
        )
    }
    pub(crate) fn fn_isna(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.is_predicate(args, cell, |v| matches!(v, ArrayNode::Error(Error::NA)))
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
        let node = match self.pick_reference(node, cell) {
            PickedReference::Node(node) => node,
            PickedReference::External(areas) => return areas.is_ok(),
            PickedReference::Value(_) => return false,
        };
        // OFFSET of one is `#VALUE!`, so not a reference (probes/external-ref-no-link-index
        // A16), where OFFSET otherwise counts as one unevaluated (`kind.returns_reference()`).
        if let Node::FunctionKind {
            kind: crate::functions::Function::Offset,
            args,
        } = node
        {
            if let Some(reference) = args.first() {
                if self.external_reference(reference, cell).is_some() {
                    return false;
                }
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
            // results C3:C6, excel-graph; T-0235).
            Node::OpIntersectKind { .. }
            | Node::OpUnionKind { .. }
            | Node::FunctionKind {
                kind: crate::functions::Function::If | crate::functions::Function::Choose,
                ..
            } => matches!(self.evaluate_reference_arg(node, cell), RefArg::Ranges(_)),
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
    fn parity_predicate(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        even: bool,
    ) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::Range { left, right } if left != right => {
                CalcResult::new_error(Error::VALUE, cell, "Expecting a single value".to_string())
            }
            CalcResult::Array(values) => CalcResult::Array(
                values
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|v| {
                                let element = array_node_to_calc_result(&v, cell);
                                calc_result_to_array_node(self.parity_of(element, even, cell))
                            })
                            .collect()
                    })
                    .collect(),
            ),
            other => self.parity_of(other, even, cell),
        }
    }

    fn parity_of(&mut self, value: CalcResult, even: bool, cell: CellReferenceIndex) -> CalcResult {
        // Parity is taken on the truncated absolute value (4.5 is even, -7 is odd:
        // probes/is-functions-array-lift, Data!C3:C4); a boolean is #VALUE!, text is cast.
        let value = self.dereference(value, cell);
        if matches!(value, CalcResult::Boolean(_)) {
            return CalcResult::new_error(Error::VALUE, cell, "Expecting number".to_string());
        }
        match self.cast_to_number(value, cell) {
            Ok(f) => CalcResult::Boolean((f.abs().trunc() as i64 % 2 == 0) == even),
            Err(error) => error,
        }
    }

    pub(crate) fn fn_isodd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.parity_predicate(args, cell, false)
    }

    pub(crate) fn fn_iseven(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.parity_predicate(args, cell, true)
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

    fn cell_has_formula(&self, at: CellReferenceIndex) -> bool {
        matches!(
            self.get_cell_formula(at.sheet, at.row, at.column),
            Ok(Some(_))
        )
    }

    pub(crate) fn fn_errortype(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // Lifts over a multi-cell range or array like the IS* tests (probes/rich-errors-arg-kinds
        // F10:G10, F40:G41: `ERROR.TYPE(FIELDVALUE(Q1:R2,"Price"))` spills 13s).
        let error_type = |v: &ArrayNode| match v {
            ArrayNode::Error(error) => ArrayNode::Number(f64::from(error.error_type_code())),
            _ => ArrayNode::Error(Error::NA),
        };
        match self.lifted_arg(&args[0], cell) {
            Ok(Lifted::Array(values)) => CalcResult::Array(map_elements(&values, error_type)),
            Ok(Lifted::Scalar(CalcResult::Error { error, .. })) => {
                CalcResult::Number(f64::from(error.error_type_code()))
            }
            Ok(Lifted::Scalar(_)) => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Not an error".to_string(),
            },
            Err(error) => error,
        }
    }

    // Excel believes for some reason that TYPE(A1:A7) is an array formula
    // Although we evaluate the same as Excel we cannot, ATM import this from excel
    pub(crate) fn fn_type(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.evaluate_node_in_context(&args[0], cell) {
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
                        ParsedDefinedName::CellReference(reference) => {
                            return CalcResult::Number(reference.sheet as f64 + 1.0)
                        }
                        ParsedDefinedName::RangeReference(range) => {
                            return CalcResult::Number(range.left.sheet as f64 + 1.0)
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
                if let Ok(range) = self.get_reference(arg, cell) {
                    return CalcResult::Number(range.left.sheet as f64 + 1.0);
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
            return CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Sheets function with an argument is not implemented".to_string(),
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
        let info_type = match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::String(s) => s.into_lossy_string(),
            CalcResult::Array(types) => {
                let lifted = types
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|item| match item {
                                ArrayNode::String(name) => calc_result_to_array_node(
                                    self.cell_info(name, ranges.as_deref(), reference, cell),
                                ),
                                _ => ArrayNode::Error(Error::VALUE),
                            })
                            .collect()
                    })
                    .collect();
                return CalcResult::Array(lifted);
            }
            _ => match self.get_string(&args[0], cell) {
                Ok(s) => s,
                Err(e) => return e,
            },
        };
        self.cell_info(&info_type, ranges.as_deref(), reference, cell)
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
                if reference.sheet != cell.sheet {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "References to other sheets not implemented".to_string(),
                    };
                }
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
                CalcResult::String(addresses.join(", ").into())
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
            "FILENAME" => {
                let workbook_name = &self.workbook.name;
                let worksheet_name = match self.workbook.worksheet(reference.sheet) {
                    Ok(ws) => &ws.name,
                    Err(_) => {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Invalid sheet".to_string(),
                        }
                    }
                };
                CalcResult::String(format!("[{workbook_name}.xlsx]{worksheet_name}").into())
            }
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
    /// NB: In Excel "`text_type`" is localized. Here it is always in English.
    pub(crate) fn fn_info(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let type_text = match self.get_string(&args[0], cell) {
            Ok(s) => s.to_uppercase(),
            Err(e) => return e,
        };
        // We take the release version from the git tag at build time. (v0.7.1-202-g847240b0 for example)
        // See build.rs
        let release = env!("GIT_VERSION");
        match type_text.as_str() {
            "DIRECTORY" | "ORIGIN" | "OSVERSION" => CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "type_text not implemented".to_string(),
            },

            // For now we just show the count of sheets in the current workbook
            "NUMFILE" => CalcResult::Number(self.workbook.worksheets.len() as f64),

            // At the moment we always do automatic recalc
            "RECALC" => CalcResult::String("Automatic".into()),
            "RELEASE" => CalcResult::String(release.to_string().into()),
            "SYSTEM" => CalcResult::String(get_system().into()),
            "TIMEZONE" => CalcResult::String(self.get_timezone().into()),
            "TIMEZONES" => {
                let mut tzs = get_all_timezones();
                tzs.sort();
                let list: Vec<Vec<ArrayNode>> = tzs
                    .into_iter()
                    .map(|name| vec![ArrayNode::String(name.to_string().into())])
                    .collect();
                CalcResult::Array(list)
            }
            _ => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Invalid type_text".to_string(),
            },
        }
    }
}
