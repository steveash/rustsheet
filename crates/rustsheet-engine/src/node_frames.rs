//! What `Model::evaluate_node_in_context` evaluates out of line: the nodes no nested call goes
//! through, the call of a name and the operator chain. Kept apart so that its frame, on the stack
//! at every nesting level of a formula or a LAMBDA call, holds none of their locals (T-0444,
//! decision 0100).

use std::sync::Arc;

use crate::arithmetic::{divide, minus, plus, power, times};
use crate::calc_result::CalcResult;
use crate::defined_names::ParsedDefinedName;
use crate::expressions::parser::{left_chain, value_operands, Node};
use crate::expressions::token::{Error, OpProduct, OpSum};
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;

impl Model {
    /// [`Self::evaluate_node_in_context`] of the nodes no nested call goes through.
    #[inline(never)]
    pub(crate) fn evaluate_cold_node(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        use Node::*;
        match node {
            StringKind(value) => CalcResult::String(value.replace(r#""""#, r#"""#).into()),
            WrongReferenceKind { .. } | WrongRangeKind { .. } => {
                self.wrong_reference_value(node, cell)
            }
            OpRangeKind { left, right } => self.get_range(left, right, cell),
            OpIntersectKind { left, right, .. } => self.get_intersection(left, right, cell),
            SheetSpanKind {
                first_index,
                last_index,
                reference,
                ..
            } => self.get_sheet_span_value(*first_index, *last_index, reference, cell),
            OpUnionKind { left, right, .. } => self.get_union(left, right, cell),
            ArrayKind(s) => CalcResult::Array(s.to_owned()),
            DefinedNameKind((name, scope, _)) => self.evaluate_defined_name(name, *scope, cell),
            // Their operand is turned into a number, element by element on a range or an array.
            UnaryKind { .. } => self.apply_unary_chain(node, cell),
            ErrorKind(kind) => CalcResult::new_error(kind.clone(), cell, "".to_string()),
            ParseErrorKind {
                formula, message, ..
            } => CalcResult::new_error(
                Error::ERROR,
                cell,
                format!("Error parsing {formula}: {message}"),
            ),
            // Unreachable: peeled above. Answered without recursing so no input can deepen the stack.
            // `ungrouped` above sees through all three.
            Group { .. } | UnaryPlus { .. } | Spaced { .. } | StructuredReferenceKind { .. } => {
                CalcResult::new_error(
                    Error::ERROR,
                    cell,
                    "Unpeeled group, unary plus or structured reference".to_string(),
                )
            }
            // The cell, range or `#VALUE!` it is on the cell's row (SPEC-0016 step 2, T-1699).
            ThisRowKind { .. } => match node.this_row_at(cell.row) {
                Some(area) => self.evaluate_node_in_context(&area, cell),
                None => CalcResult::new_error(Error::ERROR, cell, "Unresolved row".to_string()),
            },
            SpillRangeOperator { child } => self.evaluate_spill_range_operator(child, cell),
            ImplicitIntersection { child, .. } => {
                let result = self.evaluate_implicit_intersection_operand(child, cell);
                self.implicit_intersection_to_value(result, cell)
            }
            LambdaDefKind { parameters, body } => {
                let definition = Arc::new((parameters.clone(), *body.clone()));
                CalcResult::Lambda(self.store_lambda(definition))
            }
            LambdaCallKind { lambda, args } => {
                // With a reference kept as one (`evaluate_callee`): a called reference is `#REF!`
                // (`call_callee`), and a called value, what a call returned (`LAMBDA(x,x)(1)(2)`),
                // is called like a LET-bound one (`call_variable`; probes/chained-calls-lambda
                // B4-B8, T-2038).
                let callee = self.evaluate_callee(lambda, cell);
                self.call_variable(callee, args, cell)
            }
            // Unreachable by construction: `evaluate_node_in_context` answers these itself, so
            // this is at most one bounce.
            OpSumKind { .. }
            | OpProductKind { .. }
            | OpPowerKind { .. }
            | OpConcatenateKind { .. }
            | CompareKind { .. }
            | NumberKind(_)
            | BooleanKind(_)
            | ReferenceKind { .. }
            | RangeKind { .. }
            | FunctionKind { .. }
            | NamedFunctionKind { .. }
            | NamedVariableKind { .. }
            | EmptyArgKind => self.evaluate_node_in_context(node, cell),
        }
    }

    /// A call of a name, `f(..)`: a LET variable or LAMBDA parameter, else a defined-name LAMBDA.
    #[inline(never)]
    pub(crate) fn evaluate_named_function_call(
        &mut self,
        name: &str,
        args: &[Node],
        id: Option<u32>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let lambda_result = if let Some(var_id) = id {
            // Bound by LET or LAMBDA: what it holds is called, a value too.
            match self.variable_callee(var_id, name, cell) {
                Ok(callee) => callee,
                Err(error) => return error,
            }
        } else {
            // Not bound by LET — look up as a defined-name Lambda.
            // Prefer sheet-local (current sheet) over global (scope = None),
            // matching Excel's name resolution order.
            let name_lower = name.to_lowercase();
            let found = self
                .parsed_defined_names
                .get(&(Some(cell.sheet), name_lower.clone()))
                .or_else(|| self.parsed_defined_names.get(&(None, name_lower)))
                .cloned();
            match found {
                Some(ParsedDefinedName::LambdaDefinition(definition)) => {
                    self.defined_name_lambda(definition)
                }
                _ => {
                    return CalcResult::new_error(
                        Error::NAME,
                        cell,
                        format!("Invalid function: {name}"),
                    )
                }
            }
        };
        self.call_variable(lambda_result, args, cell)
    }

    /// The value of a defined name used in a formula.
    fn evaluate_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if let Some(value) = self.evaluate_external_name(name, cell) {
            return value;
        }
        if let Some(parsed_defined_name) = self.parsed_defined_name(name, scope).cloned() {
            match parsed_defined_name {
                ParsedDefinedName::CellReference(reference) => {
                    self.evaluate_cell(reference.at(cell).left)
                }
                ParsedDefinedName::RangeReference(reference) => reference.range_at(cell),
                ParsedDefinedName::LambdaDefinition(definition) => {
                    self.defined_name_lambda(definition)
                }
                ParsedDefinedName::ErrorLiteral(error) => CalcResult::new_error(
                    error,
                    cell,
                    format!("Defined name \"{name}\" is an error."),
                ),
                ParsedDefinedName::Formula(body) => {
                    self.evaluate_defined_name_formula(name, &body, cell)
                }
                ParsedDefinedName::InvalidDefinedNameFormula => CalcResult::new_error(
                    Error::NAME,
                    cell,
                    format!("Defined name \"{name}\" does not parse."),
                ),
            }
        } else {
            CalcResult::new_error(
                Error::NAME,
                cell,
                format!("Defined name \"{name}\" not found."),
            )
        }
    }

    /// The spill range operator, `A1#`.
    fn evaluate_spill_range_operator(
        &mut self,
        child: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match self.spill_anchor_operand(child, cell) {
            CalcResult::Range { left, right } => {
                if left != right {
                    return Self::spill_anchor_range_error(cell);
                }
                // The spill range depends on the *shape* of the anchor, which is
                // only known once the anchor has been evaluated. Evaluating it here
                // also records the dependency in `readers`.
                if let CalcResult::Error {
                    error: Error::CIRC, ..
                } = self.evaluate_cell(left)
                {
                    return CalcResult::new_error(
                        Error::CIRC,
                        cell,
                        "Circular reference detected".to_string(),
                    );
                }
                let sheet = left.sheet;
                let row = left.row;
                let column = left.column;
                let worksheet = match self.workbook.worksheet(sheet) {
                    Ok(s) => s,
                    Err(e) => {
                        return CalcResult::new_error(
                            Error::REF,
                            cell,
                            format!("Sheet index {sheet} not found: {e}"),
                        );
                    }
                };
                match worksheet.get_cell_spill(row, column) {
                    Ok((width, height)) => CalcResult::Range {
                        left: CellReferenceIndex { sheet, row, column },
                        right: CellReferenceIndex {
                            sheet,
                            row: row + height - 1,
                            column: column + width - 1,
                        },
                    },
                    Err(e) => CalcResult::new_error(
                        Error::REF,
                        cell,
                        format!("Cell {sheet}!{row},{column} not found: {e}"),
                    ),
                }
            }
            // An `@` that finds no cell is `#VALUE!`, and ANCHORARRAY passes it on
            // (`=ANCHORARRAY((@A1:B2))` in D20, probes/anchorarray-at-operand, T-1115).
            error @ CalcResult::Error { .. }
                if matches!(child.ungrouped(), Node::ImplicitIntersection { .. }) =>
            {
                error
            }
            _ => Self::spill_anchor_error(child, cell),
        }
    }

    /// Evaluates a chain of value operators (`=A1+A2*3&"x"`) in a loop down its
    /// left operands (T-0128): the first operand, then each operator over the
    /// result so far and its right operand, the same order and results as
    /// recursing into each left operand, on one stack frame however long the
    /// chain is. Out of line, so its locals stay off `evaluate_node_in_context`'s frame.
    #[inline(never)]
    pub(crate) fn evaluate_operator_chain(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (links, first) = left_chain(node);
        let mut value = self.evaluate_node_in_context(first, cell);
        for link in links {
            value = self.apply_operator_link(value, link, cell);
        }
        value
    }

    /// The operator of `link` (of [`left_chain`]) over `value` and its right operand. Out of
    /// line, so the chain's frame, held while a nested call in its first operand runs, stays small.
    #[inline(never)]
    fn apply_operator_link(
        &mut self,
        value: CalcResult,
        link: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // `left_chain` links are value operators only.
        let Some((_, right)) = value_operands(link) else {
            return value;
        };
        let right = self.evaluate_node_in_context(right, cell);
        self.apply_operator(value, link, right, cell)
    }

    /// The operator of `link` over its evaluated operands `value` and `right`.
    pub(crate) fn apply_operator(
        &mut self,
        value: CalcResult,
        link: &Node,
        right: CalcResult,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match link {
            Node::OpSumKind { kind, .. } => match kind {
                OpSum::Add => self.handle_arithmetic(value, right, cell, &plus),
                OpSum::Minus => self.handle_arithmetic(value, right, cell, &minus),
            },
            Node::OpProductKind { kind, .. } => match kind {
                OpProduct::Times => self.handle_arithmetic(value, right, cell, &times),
                OpProduct::Divide => self.handle_arithmetic(value, right, cell, &divide),
            },
            Node::OpPowerKind { .. } => self.handle_arithmetic(value, right, cell, &power),
            Node::OpConcatenateKind { .. } => self.handle_concatenate(value, right, cell),
            Node::CompareKind { kind, .. } => self.handle_comparison(value, right, cell, kind),
            _ => value,
        }
    }
}
