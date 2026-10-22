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

/// The bound of a lifted option's valid whole numbers.
#[derive(Clone, Copy)]
pub(crate) enum Bound {
    /// A fixed bound (`field_headers`: 3).
    Fixed(usize),
    /// The width of the key fields at this argument index (a total depth), read only when the
    /// option lifts.
    KeyWidth(usize),
}

impl Model {
    /// The arguments of a GROUPBY or PIVOTBY call with each of `options` (argument index, bound
    /// of its valid whole numbers) evaluated once and put in as a literal, or, when one of them is
    /// an array or a multi-cell range, `Err` with the call lifted over the first such option.
    /// Excel evaluates each element's call in full, so an element's own error or invalid value
    /// comes in its argument's place among the others: `GROUPBY(k,v,SUM,0,{1,"x",#N/A},,,1/0)` is
    /// `#DIV/0!`, `#VALUE!`, `#N/A` (probes/groupby-lifted-args A97). No argument is evaluated
    /// twice: the lifted calls get every argument but `function` frozen ([`Model::frozen`]), so a
    /// GROUPBY nested in a GROUPBY is evaluated once per level, not once per element class.
    pub(crate) fn lift_options(
        &mut self,
        args: &[Node],
        options: &[(usize, Bound)],
        function: usize,
        call: Call,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Node>, CalcResult> {
        let mut args = args.to_vec();
        for &(index, bound) in options {
            let Some(node) = args.get(index) else {
                continue;
            };
            let frozen = self.frozen(node, cell);
            // A one-element array lifts too: `GROUPBY(k,v,SUM,{3})` is one cell (A113).
            let elements = match &frozen {
                Node::ArrayKind(elements) => elements.clone(),
                Node::RangeKind { .. } => match self.evaluate_node_in_context(&frozen, cell) {
                    range @ CalcResult::Range { left, right } if left != right => {
                        self.result_to_array(range, cell)?
                    }
                    _ => continue,
                },
                _ => {
                    // Text that is no number is `#VALUE!` in the option's place, ahead of a later
                    // argument's error value, scalar or lifted (A201, A97).
                    let frozen = match frozen {
                        Node::StringKind(text)
                            if self
                                .cast_to_number(CalcResult::String(text.as_str().into()), cell)
                                .is_err() =>
                        {
                            Node::ErrorKind(Error::VALUE)
                        }
                        other => other,
                    };
                    if let Some(slot) = args.get_mut(index) {
                        *slot = frozen;
                    }
                    continue;
                }
            };
            let scalar_args: Vec<Node> = args
                .iter()
                .enumerate()
                .map(|(i, node)| {
                    if i == function || i == index {
                        node.clone()
                    } else {
                        self.frozen(node, cell)
                    }
                })
                .collect();
            let bound = match bound {
                Bound::Fixed(bound) => bound,
                Bound::KeyWidth(keys) => self.key_width(scalar_args.get(keys), cell),
            };
            let mut scalar_args = scalar_args;
            return Err(self.lift_by_class(
                &elements,
                |model, element| model.element_class(element, bound, cell),
                |model, element| {
                    // Text that is no number is `#VALUE!` in its argument's place, ahead of a
                    // later argument's error value (`{1,"x",#N/A}` over a `1/0` relationship: A97).
                    let text = model.element_class(element, bound, cell) == Class::OtherText;
                    if let Some(slot) = scalar_args.get_mut(index) {
                        *slot = if text {
                            Node::ErrorKind(Error::VALUE)
                        } else {
                            literal(element)
                        };
                    }
                    match call(model, &scalar_args, cell) {
                        CalcResult::Array(output) => at(&output, 0, 0),
                        other => calc_result_to_array_node(other),
                    }
                },
            ));
        }
        Ok(args)
    }

    /// `node` evaluated and put back as a literal: a value, or an array as an array constant.
    /// Omitted arguments, literals, references (cheap to read again, and a blank cell is not a
    /// literal) and anything that evaluates to a reference or a LAMBDA are kept as they are.
    fn frozen(&mut self, node: &Node, cell: CellReferenceIndex) -> Node {
        if is_scalar_node(node) || matches!(node, Node::RangeKind { .. } | Node::ArrayKind(_)) {
            return node.clone();
        }
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Number(number) => Node::NumberKind(number),
            CalcResult::String(text) => Node::StringKind(text.to_string()),
            CalcResult::Boolean(value) => Node::BooleanKind(value),
            CalcResult::Error { error, .. } => Node::ErrorKind(error),
            CalcResult::Array(array) => Node::ArrayKind(array),
            CalcResult::Range { .. }
            | CalcResult::EmptyCell
            | CalcResult::EmptyArg
            | CalcResult::Lambda(_) => node.clone(),
        }
    }

    /// `elements` lifted: each element's value is `run` of the first element of its class, so
    /// `run` is called once per class whatever the array's size (GROUPBY's `field_relationship`
    /// and the options of [`Model::lift_options`]).
    pub(crate) fn lift_by_class<K: PartialEq>(
        &mut self,
        elements: &[Vec<ArrayNode>],
        mut classify: impl FnMut(&mut Model, &ArrayNode) -> K,
        mut run: impl FnMut(&mut Model, &ArrayNode) -> ArrayNode,
    ) -> CalcResult {
        let mut known: Vec<(K, ArrayNode)> = Vec::new();
        let mut lifted = Vec::with_capacity(elements.len());
        for row in elements {
            let mut cells = Vec::with_capacity(row.len());
            for element in row {
                let class = classify(self, element);
                if let Some((_, value)) = known.iter().find(|(c, _)| *c == class) {
                    cells.push(value.clone());
                    continue;
                }
                let value = run(self, element);
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

    /// The width of the key fields `node`, read without expanding a range: a total depth's bound.
    fn key_width(&mut self, node: Option<&Node>, cell: CellReferenceIndex) -> usize {
        let Some(node) = node else {
            return 1;
        };
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, right } => {
                usize::try_from(right.column - left.column + 1).unwrap_or(1)
            }
            CalcResult::Array(array) => array.first().map_or(1, Vec::len),
            _ => 1,
        }
    }
}

/// Whether `node` is omitted, a literal or a one-cell reference, none of which lifts.
fn is_scalar_node(node: &Node) -> bool {
    matches!(
        node,
        Node::EmptyArgKind
            | Node::NumberKind(_)
            | Node::StringKind(_)
            | Node::BooleanKind(_)
            | Node::ErrorKind(_)
            | Node::ReferenceKind { .. }
    )
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
