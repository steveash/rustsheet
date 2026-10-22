//! GROUPBY and PIVOTBY over an array `field_headers` or total depth: Excel lifts the call over the
//! array, element by element. Each element is the top-left cell of the call with that element in
//! the argument's place, or that call's error (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY
//! lift over an array `field_headers` or total depth", probes/groupby-lifted-args, T-2359).

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

use super::grouping::at;

/// The calls an element can make: its result depends only on this class, so a call over an
/// array of any size runs the function once per class. A number is its own class while it is a
/// whole number no larger than `bound` (a valid header mode or depth); any other number, and any
/// text that is no number, is invalid (`#VALUE!` from the check, after the error values of the
/// other arguments), so one call answers for all of them.
#[derive(PartialEq)]
enum Class {
    Whole(i64),
    OtherNumber,
    Boolean(bool),
    OtherText,
    Error(Error),
}

/// A scalar GROUPBY or PIVOTBY call, as `Model::fn_groupby` and `Model::fn_pivotby` are.
type Call = fn(&mut Model, &[Node], CellReferenceIndex) -> CalcResult;

impl Model {
    /// The call lifted over the first of `options` (argument index, bound of its valid whole
    /// numbers) whose argument is an array or a multi-cell range, or `None` when none is. Excel
    /// evaluates each element's call in full, so an element's own error or invalid value comes in
    /// its argument's place among the others: `GROUPBY(k,v,SUM,0,{1,"x",#N/A},,,1/0)` is
    /// `#DIV/0!`, `#VALUE!`, `#N/A` (probes/groupby-lifted-args A97).
    pub(crate) fn lifted_option(
        &mut self,
        args: &[Node],
        options: &[(usize, usize)],
        call: Call,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        for &(index, bound) in options {
            let Some(node) = args.get(index) else {
                continue;
            };
            if matches!(node, Node::EmptyArgKind) {
                continue;
            }
            let value = self.evaluate_node_in_context(node, cell);
            // A one-element array lifts too: `GROUPBY(k,v,SUM,{3})` is one cell (A113).
            if !(matches!(value, CalcResult::Array(_)) || value.is_multi_value()) {
                continue;
            }
            let elements = match self.result_to_array(value, cell) {
                Ok(elements) => elements,
                Err(error) => return Some(error),
            };
            return Some(self.lift_over(args, index, bound, &elements, call, cell));
        }
        None
    }

    fn lift_over(
        &mut self,
        args: &[Node],
        index: usize,
        bound: usize,
        elements: &[Vec<ArrayNode>],
        call: Call,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut known: Vec<(Class, ArrayNode)> = Vec::new();
        let mut scalar_args = args.to_vec();
        let mut lifted = Vec::with_capacity(elements.len());
        for row in elements {
            let mut cells = Vec::with_capacity(row.len());
            for element in row {
                let class = self.element_class(element, bound, cell);
                if let Some((_, value)) = known.iter().find(|(c, _)| *c == class) {
                    cells.push(value.clone());
                    continue;
                }
                if let Some(slot) = scalar_args.get_mut(index) {
                    // Text that is no number is `#VALUE!` in its argument's place, ahead of a later
                    // argument's error value (`{1,"x",#N/A}` over a `1/0` field_relationship: A97).
                    *slot = if class == Class::OtherText {
                        Node::ErrorKind(Error::VALUE)
                    } else {
                        literal(element)
                    };
                }
                let value = match call(self, &scalar_args, cell) {
                    CalcResult::Array(output) => at(&output, 0, 0),
                    other => calc_result_to_array_node(other),
                };
                known.push((class, value.clone()));
                cells.push(value);
            }
            lifted.push(cells);
        }
        CalcResult::Array(lifted)
    }

    fn element_class(
        &mut self,
        element: &ArrayNode,
        bound: usize,
        cell: CellReferenceIndex,
    ) -> Class {
        let number = match element {
            ArrayNode::Error(error) => return Class::Error(error.clone()),
            ArrayNode::Boolean(value) => return Class::Boolean(*value),
            ArrayNode::Number(number) => *number,
            // A blank element is read as 0, as a blank one-cell reference is.
            ArrayNode::Empty => 0.0,
            ArrayNode::String(text) => {
                match self.cast_to_number(CalcResult::String(text.clone()), cell) {
                    Ok(number) => number,
                    Err(_) => return Class::OtherText,
                }
            }
        };
        // A whole number within the bound, so the cast is exact.
        if number.fract() == 0.0 && number.abs() <= bound as f64 {
            Class::Whole(number as i64)
        } else {
            Class::OtherNumber
        }
    }
}

/// The bound of a total depth's valid whole numbers: the width of its key fields `node`, read
/// without expanding a range (GROUPBY and PIVOTBY check the depth against it).
pub(crate) fn depth_bound(
    model: &mut Model,
    node: Option<&Node>,
    cell: CellReferenceIndex,
) -> usize {
    let Some(node) = node else {
        return 1;
    };
    match model.evaluate_node_in_context(node, cell) {
        CalcResult::Range { left, right } => {
            usize::try_from(right.column - left.column + 1).unwrap_or(1)
        }
        CalcResult::Array(array) => array.first().map_or(1, Vec::len),
        _ => 1,
    }
}

/// The literal that stands for `element` in the scalar call.
fn literal(element: &ArrayNode) -> Node {
    match element {
        ArrayNode::Boolean(value) => Node::BooleanKind(*value),
        ArrayNode::Number(number) => Node::NumberKind(*number),
        ArrayNode::String(text) => Node::StringKind(text.to_string()),
        ArrayNode::Error(error) => Node::ErrorKind(error.clone()),
        ArrayNode::Empty => Node::NumberKind(0.0),
    }
}
