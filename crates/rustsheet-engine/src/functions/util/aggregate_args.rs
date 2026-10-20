//! The one walker over an aggregate function's arguments (SPEC-0009,
//! docs/specs/SPEC-0009-aggregate-argument-walker.md).
//!
//! [`Model::walk_aggregate_args`] reads each argument, sorts it by shape (references, arrays,
//! direct values), walks ranges and arrays, and applies the error-precedence rule of
//! docs/excel-semantics/syntax.md "Aggregate error precedence". A consumer states how it differs
//! from the others as an [`AggregatePolicy`]. The number layer
//! ([`Model::for_each_aggregate_number`]) adds the coercion of the consumers that fold numbers.

use crate::calc_result::CalcResult;
use crate::calc_result::Range;
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::{cell_number, coerced_cell_number};
use crate::model::Model;
use crate::reference_operators::{RefArg, RefElement, RefGrid};

/// Where an item the walker hands on came from. The coercion rules depend on it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ItemSource {
    /// A direct value: a literal, a computed scalar, a function's value result.
    Direct,
    /// A value a reference node gave without areas (a cached cell of a linked workbook). Today
    /// that is only ever an array or an error, which the walker handles itself, so no consumer
    /// sees one yet. The rules for it keep the old dispatches' `ReferenceKind` guards.
    ReferenceValue,
    /// A cell of a range, reference, union or 3-D span.
    ReferencedCell,
    /// An element of an array value.
    ArrayElement,
}

/// How the walker orders the errors it meets.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorRule {
    /// A direct error returns at once. A referenced error is held and the leftmost wins, so a
    /// direct error to its right still beats it. An error inside an array ranks with the
    /// referenced errors (SUM: probes/aggregate-error-precedence C50, C53–C55, C61, C77; PRODUCT:
    /// probes/product-array-2d-and-scalar A7–A17, T-1287; MIN, MAX, AVERAGE, MEDIAN:
    /// probes/aggregate-array-error-vs-ref-error, T-1615; the AVERAGE family and DEVSQ, AVEDEV,
    /// SUMSQ: probes/aggregate-error-precedence-stat-family, T-1439; VARA, VARPA, STDEVA, STDEVPA:
    /// probes/aggregate-error-precedence-a-variants, T-1696).
    Ranked,
    /// The first error in argument order returns at once (AND, OR, XOR).
    First,
    /// Errors are items for the visitor, which counts or ignores them (COUNT ignores them, COUNTA
    /// counts them; syntax.md). Only a walk-budget error returns.
    Items,
}

/// How the walker reads an argument.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArgReading {
    /// `evaluate_aggregate_arg`: picked references, cached blanks skipped (SUM, AVERAGE, MIN, …).
    Aggregate,
    /// `evaluate_picked_reference_arg`: cached blanks kept (COUNTA).
    PickedKeepingBlanks,
}

/// How the walker reads an array of references (SPEC-0014, `RefArg::Grid`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GridReading {
    /// Its value view: every reference element is `#VALUE!`, an error element itself
    /// ([`RefGrid::to_values`]). This is also SPEC-0014 rule 5 for COUNT and COUNTA, which
    /// see an element as an error item: COUNT counts none, COUNTA every one
    /// (`COUNT(OFFSET(A1,0,0,{1;2}))` 0, `COUNTA(..)` 2; probes/reference-lift-consumers E13,
    /// E17, -b2 E169). The aggregates whose Excel result is not known yet keep it (T-2065).
    Values,
    /// Each element's top-left cell, read as a range cell, so text, booleans and blanks are
    /// skipped; an error element ranks as a referenced error (SPEC-0014 rule 4: SUM, AVERAGE,
    /// MIN, MAX, PRODUCT; `SUM(OFFSET(A1,0,0,{1;2}))` is 2, `AVERAGE(OFFSET(A1,{0;-1},0))`
    /// `#REF!`; probes/reference-lift-consumers E1-E25, -b2 E117-E121, E181, E185).
    TopLeft,
}

/// What sets one consumer's argument walk apart (SPEC-0009, "Consumer mapping").
#[derive(Clone, Copy)]
pub(crate) struct AggregatePolicy {
    /// How an argument is read.
    pub(crate) reading: ArgReading,
    /// How errors are ordered.
    pub(crate) errors: ErrorRule,
    /// DEVSQ, AVEDEV, SUMSQ: an empty argument is not an item (TODO(T-1748): Excel counts it as 0 for AVEDEV and DEVSQ).
    pub(crate) skip_empty_args: bool,
    /// How an array of references is read.
    pub(crate) grids: GridReading,
}

/// How a direct value (and, under `Cast`, a [`ItemSource::ReferenceValue`]) becomes a number.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectNumbers {
    /// Every direct value through `cast_aggregate_arg_to_number` (SUM).
    Cast,
    /// A number, boolean, text (`cast_aggregate_arg_to_number`) or empty argument (0); a LAMBDA
    /// is `#VALUE!`; other direct values are skipped (the AVERAGE family).
    CastScalars,
    /// A number, boolean or text; text that does not cast is `#VALUE!` and an empty argument is
    /// skipped (AVERAGEA, MAXA, MINA).
    CastA,
}

/// How a referenced cell (and, except under [`DirectNumbers::Cast`], a
/// [`ItemSource::ReferenceValue`]) becomes a number.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CellNumbers {
    /// Only numbers count ([`cell_number`]).
    Numbers,
    /// Text is 0 and TRUE/FALSE 1/0 ([`coerced_cell_number`]; the A-variants).
    CoercedA,
}

/// How an array element becomes a number. Booleans in an array are always skipped.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrayNumbers {
    /// Only numbers count.
    Numbers,
    /// Text is 0 (AVERAGEA; statistics.md "AVERAGEA, MAXA and MINA over an array").
    TextZero,
}

/// The coercion of a number-folding consumer, by source.
#[derive(Clone, Copy)]
pub(crate) struct NumberRules {
    /// Direct values.
    pub(crate) direct: DirectNumbers,
    /// Referenced cells.
    pub(crate) cells: CellNumbers,
    /// Array elements.
    pub(crate) array: ArrayNumbers,
}

impl NumberRules {
    /// SUM's coercion, which MIN, MAX and PRODUCT share (probes/min-max-value-args,
    /// docs/excel-semantics/coercion.md): every direct value is cast; referenced cells and arrays
    /// give only their numbers.
    pub(crate) const SUM_LIKE: NumberRules = NumberRules {
        direct: DirectNumbers::Cast,
        cells: CellNumbers::Numbers,
        array: ArrayNumbers::Numbers,
    };
}

/// Error-precedence state across one call's arguments.
pub(crate) struct AggregateWalk {
    policy: AggregatePolicy,
    held: Option<CalcResult>,
}

impl AggregateWalk {
    /// A walk with nothing held yet.
    pub(crate) fn new(policy: AggregatePolicy) -> Self {
        AggregateWalk { policy, held: None }
    }

    /// The held referenced error, if any, once every argument is walked.
    pub(crate) fn finish(self) -> Result<(), CalcResult> {
        self.held.map_or(Ok(()), Err)
    }

    /// A referenced error under `Ranked` is held if nothing is held yet (the leftmost wins);
    /// under `First` it returns at once.
    pub(crate) fn referenced_error(&mut self, error: CalcResult) -> Result<(), CalcResult> {
        match self.policy.errors {
            ErrorRule::Ranked => {
                if self.held.is_none() {
                    self.held = Some(error);
                }
                Ok(())
            }
            ErrorRule::First | ErrorRule::Items => Err(error),
        }
    }

    /// The `#VALUE!` of a direct LAMBDA ranks with the referenced errors: it is held when nothing
    /// is held yet, so a later direct error beats it and an earlier or later referenced error does
    /// not (probes/aggregate-direct-lambda-average-family D14-D31, excel-graph 2026-10-15, T-1602).
    /// Under `ErrorRule::First` an error cell read before the LAMBDA returns at once and wins,
    /// while one after it is never read (probes/and-or-xor-picked-text J5/J6, T-1685).
    fn lambda_error(&mut self, error: CalcResult) {
        if self.held.is_none() {
            self.held = Some(error);
        }
    }

    /// An error inside an array. Its `CalcResult` is built only when it is kept or returned, so
    /// the later errors of a large array cost nothing once one is held.
    fn array_error(&mut self, error: Error, cell: CellReferenceIndex) -> Result<(), CalcResult> {
        let value = || CalcResult::Error {
            error,
            origin: cell,
            message: "Error in array".to_string(),
        };
        match self.policy.errors {
            ErrorRule::Ranked => {
                if self.held.is_none() {
                    self.held = Some(value());
                }
                Ok(())
            }
            ErrorRule::First | ErrorRule::Items => Err(value()),
        }
    }
}

/// An array element as an item value, by value so text is moved, not cloned
/// ([`crate::functions::util::array_node_to_calc_result`] borrows), or its error.
fn array_item(node: ArrayNode) -> Result<CalcResult, Error> {
    Ok(match node {
        ArrayNode::Number(n) => CalcResult::Number(n),
        ArrayNode::Boolean(b) => CalcResult::Boolean(b),
        ArrayNode::String(s) => CalcResult::String(s),
        ArrayNode::Error(error) => return Err(error),
        ArrayNode::Empty => CalcResult::EmptyCell,
    })
}

impl Model {
    /// Walks every argument of `args` with `policy`, then returns the held error, if any.
    #[inline]
    pub(crate) fn walk_aggregate_args<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        policy: AggregatePolicy,
        mut visit: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, ItemSource, CalcResult) -> Result<(), CalcResult>,
    {
        let mut walk = AggregateWalk::new(policy);
        for arg in args {
            self.walk_aggregate_arg(arg, cell, &mut walk, &mut visit)?;
        }
        walk.finish()
    }

    /// One argument of a walk, for a consumer with a step of its own before it (the A-variants'
    /// linked-workbook cells).
    ///
    /// Errors never reach `visit`: they follow `walk`'s [`ErrorRule`]. An `Err` from `visit` is a
    /// direct error and returns at once, whatever the item's source.
    #[inline]
    pub(crate) fn walk_aggregate_arg<F>(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        walk: &mut AggregateWalk,
        visit: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, ItemSource, CalcResult) -> Result<(), CalcResult>,
    {
        if walk.policy.skip_empty_args && matches!(arg, Node::EmptyArgKind) {
            return Ok(());
        }
        // References (also via IF, INDEX, XLOOKUP, TAKE …) arrive as ranges.
        let read = match walk.policy.reading {
            ArgReading::Aggregate => self.evaluate_aggregate_arg(arg, cell),
            ArgReading::PickedKeepingBlanks => self.evaluate_picked_reference_arg(arg, cell),
        };
        let is_reference = matches!(arg.ungrouped(), Node::ReferenceKind { .. });
        self.walk_aggregate_read(read, is_reference, cell, walk, visit)
    }

    /// One argument already read, for a consumer that picks its argument itself (COUNTA);
    /// `is_reference` says the argument is a `Node::ReferenceKind`.
    #[inline]
    pub(crate) fn walk_aggregate_read<F>(
        &mut self,
        read: RefArg,
        is_reference: bool,
        cell: CellReferenceIndex,
        walk: &mut AggregateWalk,
        visit: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, ItemSource, CalcResult) -> Result<(), CalcResult>,
    {
        let items = walk.policy.errors == ErrorRule::Items;
        let read = match read {
            RefArg::Grid(grid) if walk.policy.grids == GridReading::TopLeft => {
                return self.walk_top_left_cells(&grid, cell, walk, visit);
            }
            read => read.into_ranges_or_value(),
        };
        match read {
            Ok(areas) => self.walk_areas(&areas, cell, walk, visit),
            Err(CalcResult::Array(array)) => {
                for node in array.into_iter().flatten() {
                    match array_item(node) {
                        Ok(value) => visit(self, ItemSource::ArrayElement, value)?,
                        // Only the first array or referenced error is held (`Ranked`).
                        Err(error) if items => visit(
                            self,
                            ItemSource::ArrayElement,
                            CalcResult::new_error(error, cell, String::new()),
                        )?,
                        Err(error) => walk.array_error(error, cell)?,
                    }
                }
                Ok(())
            }
            Err(error @ CalcResult::Error { .. }) if !items => Err(error),
            Err(value) => {
                let source = if is_reference {
                    ItemSource::ReferenceValue
                } else {
                    ItemSource::Direct
                };
                let is_lambda = matches!(value, CalcResult::Lambda(_));
                match visit(self, source, value) {
                    Err(error) if is_lambda => {
                        walk.lambda_error(error);
                        Ok(())
                    }
                    result => result,
                }
            }
        }
    }

    /// The cells of `areas`, walked as referenced cells: blanks skipped, the first error cell
    /// held as a referenced error.
    fn walk_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        walk: &mut AggregateWalk,
        visit: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, ItemSource, CalcResult) -> Result<(), CalcResult>,
    {
        // Once an error is held, later references are not read at all, as before.
        if walk.held.is_some() {
            return Ok(());
        }
        let items = walk.policy.errors == ErrorRule::Items;
        // The first error cell ends the walk of these areas; a visitor error is told apart so
        // it returns at once.
        let mut visitor_failed = false;
        let walked =
            self.for_each_cell_or_error_in_areas(areas, cell, |model, value| match value {
                CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(()),
                error @ CalcResult::Error { .. } if !items => Err(error),
                value => {
                    let result = visit(model, ItemSource::ReferencedCell, value);
                    visitor_failed = result.is_err();
                    result
                }
            });
        match walked {
            Ok(()) => Ok(()),
            Err(error) if visitor_failed => Err(error),
            // An error cell, or the walk budget, ranks as a referenced error.
            Err(error) => walk.referenced_error(error),
        }
    }

    /// [`GridReading::TopLeft`]: each element of `grid` in row-major order, a reference as its
    /// top-left cell (one cell read per element, so the work is the grid's size) and an error
    /// as a referenced error.
    fn walk_top_left_cells<F>(
        &mut self,
        grid: &RefGrid,
        cell: CellReferenceIndex,
        walk: &mut AggregateWalk,
        visit: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, ItemSource, CalcResult) -> Result<(), CalcResult>,
    {
        for element in grid.elements() {
            match element {
                RefElement::Reference(range) => {
                    let top_left = Range {
                        left: range.left,
                        right: range.left,
                    };
                    self.walk_areas(std::slice::from_ref(&top_left), cell, walk, visit)?;
                }
                RefElement::Error(error) => walk.referenced_error(CalcResult::new_error(
                    error.clone(),
                    cell,
                    String::new(),
                ))?,
            }
        }
        Ok(())
    }

    /// Calls `f` with every number `args` give under `policy` and `rules`.
    #[inline]
    pub(crate) fn for_each_aggregate_number<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        policy: AggregatePolicy,
        rules: NumberRules,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        self.walk_aggregate_args(args, cell, policy, |model, source, value| {
            // The hot path: a number in a referenced cell counts under every rule.
            if let (ItemSource::ReferencedCell, CalcResult::Number(number)) = (source, &value) {
                f(*number);
                return Ok(());
            }
            if let Some(number) = model.aggregate_number(rules, source, value, cell)? {
                f(number);
            }
            Ok(())
        })
    }

    /// The number one item gives under `rules`, `None` when it is skipped.
    #[inline]
    pub(crate) fn aggregate_number(
        &mut self,
        rules: NumberRules,
        source: ItemSource,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Option<f64>, CalcResult> {
        match source {
            ItemSource::ReferenceValue if rules.direct == DirectNumbers::Cast => {
                self.cast_aggregate_arg_to_number(value, cell).map(Some)
            }
            ItemSource::ReferencedCell | ItemSource::ReferenceValue => match rules.cells {
                CellNumbers::Numbers => Ok(cell_number(&value)),
                // A cell holding a range is not a value (the A-variants' walk said so before).
                CellNumbers::CoercedA if matches!(value, CalcResult::Range { .. }) => Err(
                    CalcResult::new_error(Error::ERROR, cell, "Unexpected Range".to_string()),
                ),
                CellNumbers::CoercedA => Ok(coerced_cell_number(&value)),
            },
            ItemSource::ArrayElement => Ok(match value {
                CalcResult::Number(n) => Some(n),
                CalcResult::String(_) if rules.array == ArrayNumbers::TextZero => Some(0.0),
                _ => None,
            }),
            ItemSource::Direct => self.direct_aggregate_number(rules.direct, value, cell),
        }
    }

    fn direct_aggregate_number(
        &mut self,
        direct: DirectNumbers,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Option<f64>, CalcResult> {
        match (direct, value) {
            (DirectNumbers::Cast, value) => {
                self.cast_aggregate_arg_to_number(value, cell).map(Some)
            }
            (_, CalcResult::Number(n)) => Ok(Some(n)),
            (_, CalcResult::Boolean(b)) => Ok(Some(if b { 1.0 } else { 0.0 })),
            // An empty argument (`AVERAGE(5,)`) counts as 0 (probes/average-value-args F2, F6,
            // F9, F15, F17).
            (DirectNumbers::CastScalars, CalcResult::EmptyArg) => Ok(Some(0.0)),
            // A direct LAMBDA is #VALUE!, not skipped, for AVERAGE and MEDIAN (probes/
            // aggregate-picked-direct-a-variants D11, D12), and so is the rest of the family
            // (probes/aggregate-direct-lambda-average-family D1-D12; docs/excel-semantics/syntax.md)
            // and the A variants AVERAGEA, MAXA and MINA, ranked with the referenced errors
            // (probes/aggregate-array-error-vs-ref-error-average-family D73-D87, T-1629).
            (
                DirectNumbers::CastScalars,
                value @ (CalcResult::String(_) | CalcResult::Lambda(_)),
            )
            | (DirectNumbers::CastA, value @ CalcResult::Lambda(_)) => {
                self.cast_aggregate_arg_to_number(value, cell).map(Some)
            }
            (DirectNumbers::CastA, CalcResult::String(text)) => {
                if let Some(error) = Self::aggregate_text_too_long(&text, cell) {
                    return Err(error);
                }
                match self.cast_number(&text) {
                    Some(parsed) => Ok(Some(parsed)),
                    None => Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Argument cannot be cast into number".to_string(),
                    )),
                }
            }
            // Everything else is ignored.
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::util::new_empty_model;

    const RANKED: AggregatePolicy = AggregatePolicy {
        reading: ArgReading::Aggregate,
        errors: ErrorRule::Ranked,
        skip_empty_args: false,
        grids: GridReading::Values,
    };

    /// Z1, where every test formula sits.
    const Z1: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 26,
    };

    /// A model with `cells` and `formula` in Z1, and the arguments of Z1's call.
    fn setup(cells: &[(&str, &str)], formula: &str) -> (Model, Vec<Node>) {
        let mut model = new_empty_model();
        for (a1, value) in cells {
            model._set(a1, value);
        }
        model._set("Z1", formula);
        model.evaluate();
        // Z1's formula is the sheet's last parsed formula.
        let args = match model.parsed_formulas[0]
            .last()
            .map(|(node, _)| node.as_ref())
        {
            Some(Node::FunctionKind { args, .. }) => args.clone(),
            _ => Vec::new(),
        };
        (model, args)
    }

    /// The items the walker hands on for `formula`'s arguments, or the error it returns.
    fn items(
        cells: &[(&str, &str)],
        formula: &str,
        policy: AggregatePolicy,
    ) -> Result<Vec<ItemSource>, String> {
        let (mut model, args) = setup(cells, formula);
        let mut seen = Vec::new();
        let walked = model.walk_aggregate_args(&args, Z1, policy, |_, source, _| {
            seen.push(source);
            Ok(())
        });
        match walked {
            Ok(()) => Ok(seen),
            Err(CalcResult::Error { error, .. }) => Err(error.to_string()),
            Err(_) => Err("not an error value".to_string()),
        }
    }

    fn err(text: &str) -> Result<Vec<ItemSource>, String> {
        Err(text.to_string())
    }

    #[test]
    fn a_direct_error_beats_a_held_referenced_error_either_side() {
        let cells = [("A1", "=NA()")];
        assert_eq!(items(&cells, "=SUM(A1,1/0)", RANKED), err("#DIV/0!"));
        assert_eq!(items(&cells, "=SUM(1/0,A1)", RANKED), err("#DIV/0!"));
        assert_eq!(items(&cells, "=SUM(A1,2)", RANKED), err("#N/A"));
    }

    #[test]
    fn the_leftmost_referenced_error_wins_and_ends_the_reading() {
        let cells = [("A1", "=NA()"), ("A2", "=1/0"), ("A3", "5"), ("B1", "7")];
        assert_eq!(items(&cells, "=SUM(A1:A3,A2)", RANKED), err("#N/A"));
        // After the held error neither A3 nor the later range is handed on.
        let (mut model, args) = setup(&cells, "=SUM(A1:A3,B1)");
        let mut count = 0;
        let walked = model.walk_aggregate_args(&args, Z1, RANKED, |_, _, _| {
            count += 1;
            Ok(())
        });
        assert!(walked.is_err());
        assert_eq!(count, 0);
    }

    #[test]
    fn array_errors_rank_with_referenced_errors_under_ranked() {
        let cells = [("A1", "=NA()")];
        assert_eq!(
            items(&cells, "=SUM({1,#DIV/0!},A1)", RANKED),
            err("#DIV/0!")
        );
        assert_eq!(items(&cells, "=SUM(A1,{1,#DIV/0!})", RANKED), err("#N/A"));
    }

    #[test]
    fn first_returns_the_first_error_and_skip_empty_drops_empty_args() {
        let cells = [("A1", "=NA()")];
        let first = AggregatePolicy {
            reading: ArgReading::Aggregate,
            errors: ErrorRule::First,
            skip_empty_args: true,
            grids: GridReading::Values,
        };
        assert_eq!(items(&cells, "=SUM(A1,1/0)", first), err("#N/A"));
        assert_eq!(items(&[], "=SUM(5,)", first).map(|v| v.len()), Ok(1));
        assert_eq!(items(&[], "=SUM(5,)", RANKED).map(|v| v.len()), Ok(2));
    }

    #[test]
    fn items_come_in_argument_cell_and_element_order() {
        // A2 is blank and never handed on; an empty range hands on nothing.
        let cells = [("A1", "1"), ("A3", "x")];
        assert_eq!(
            items(&cells, "=SUM(A1:A3,{2,\"y\"},3,C1:C2)", RANKED),
            Ok(vec![
                ItemSource::ReferencedCell,
                ItemSource::ReferencedCell,
                ItemSource::ArrayElement,
                ItemSource::ArrayElement,
                ItemSource::Direct,
            ])
        );
    }

    #[test]
    fn items_rule_hands_errors_on_as_items() {
        // COUNT and COUNTA see error cells, direct errors and array errors as items; nothing
        // is held or returned.
        let policy = AggregatePolicy {
            reading: ArgReading::Aggregate,
            errors: ErrorRule::Items,
            skip_empty_args: false,
            grids: GridReading::Values,
        };
        let cells = [("A1", "=NA()"), ("A2", "3")];
        assert_eq!(
            items(&cells, "=COUNT(A1:A2,1/0,{#N/A,2})", policy),
            Ok(vec![
                ItemSource::ReferencedCell,
                ItemSource::ReferencedCell,
                ItemSource::Direct,
                ItemSource::ArrayElement,
                ItemSource::ArrayElement,
            ])
        );
    }

    #[test]
    fn vara_casts_direct_values_keeps_array_numbers_and_coerces_cells() {
        // VARA's rules: a direct boolean and numeric text cast (TRUE 1, "7" 7), an array keeps
        // its numbers, and a referenced cell coerces (text 0, TRUE 1).
        let policy = AggregatePolicy {
            reading: ArgReading::Aggregate,
            errors: ErrorRule::Ranked,
            skip_empty_args: false,
            grids: GridReading::Values,
        };
        let rules = NumberRules {
            direct: DirectNumbers::CastA,
            cells: CellNumbers::CoercedA,
            array: ArrayNumbers::Numbers,
        };
        let (mut model, args) = setup(
            &[("A1", "x"), ("A2", "TRUE")],
            "=VARA(5,TRUE,\"7\",{2,\"y\",FALSE},A1:A2)",
        );
        let mut seen = Vec::new();
        let walked =
            model.for_each_aggregate_number(&args, Z1, policy, rules, |number| seen.push(number));
        assert!(walked.is_ok());
        assert_eq!(seen, vec![5.0, 1.0, 7.0, 2.0, 0.0, 1.0]);
    }

    #[test]
    fn a_visitor_error_returns_at_once() {
        let (mut model, args) = setup(&[("A1", "=NA()")], "=SUM(A1,\"x\")");
        let rules = NumberRules {
            direct: DirectNumbers::CastScalars,
            cells: CellNumbers::Numbers,
            array: ArrayNumbers::Numbers,
        };
        let walked = model.for_each_aggregate_number(&args, Z1, RANKED, rules, |_| {});
        assert!(matches!(
            walked,
            Err(CalcResult::Error {
                error: Error::VALUE,
                ..
            })
        ));
    }

    #[test]
    fn an_empty_array_element_is_handed_on() {
        assert_eq!(
            items(&[("B1", "")], "=SUM(IFERROR(B1:B2,1),{1})", RANKED).map(|v| v.len()),
            Ok(3)
        );
    }
}
