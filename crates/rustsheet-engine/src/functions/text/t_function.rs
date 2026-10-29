// T(value): text and errors pass through, anything else is "".

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{information::map_elements, text::common::reject_over_short_limit, Function},
    model::Model,
    reference_operators::Picked,
    text::Text,
};

impl Model {
    pub(crate) fn fn_t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.t_of_argument(args, cell, false)
    }

    /// T in a legacy formula's value position (SPEC-0012 rule 3): an array argument is `#VALUE!`.
    pub(crate) fn fn_t_legacy(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.t_of_argument(args, cell, true)
    }

    /// T's kernel; `legacy` makes an array argument `#VALUE!`, whatever its size.
    fn t_of_argument(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        legacy: bool,
    ) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // A range is its top-left cell, with no lift and no implicit intersection, legacy or
        // dynamic: `T(A1:A3)` with A1 "x" is "x" from any row, and with A1 5 "" even from A2's
        // row (probes/t-n-range-top-left F2, F10, F12; t-array-lift S!E7; T-0985).
        let (result, read) = self.t_argument(&args[0], None, cell);
        // Text over 255 units read from a cell is "" however the cell is reached: a range, name,
        // INDIRECT, OFFSET, INDEX, IF, CHOOSE, LET, spill reference, sheet-qualified, parenthesised
        // or legacy reference, or structured reference. A computed value stays `#VALUE!`, and so
        // does `+D1` (probes/text-255-units-reference-shapes A1:A19; T-3107).
        match result {
            // Text over 255 UTF-16 units is #VALUE!, as in REPT, CODE and UNICODE:
            // `T(REPT("x",256))` (docs/excel-semantics/text.md "REPT, CODE, UNICODE and T refuse a
            // text argument over 255 UTF-16 units", probes/t2199-text-length-and-combin-cuts A12).
            // Read from a cell, such text is "" instead, and an array element over the limit is
            // #VALUE! (probes/text-255-units-from-cell A4, A12; -array-element A4; T-2627).
            CalcResult::String(ref text) => match reject_over_short_limit(text) {
                Ok(()) => result,
                // Checked only here: `is_reference` can evaluate the argument again, which every
                // T call would pay and nested T(IF(…T(IF(…)))) would double per level. It tests
                // the argument already picked, so a volatile pick is not drawn again (T-3142).
                Err(_) if self.reads_cell_text(read, cell) => {
                    CalcResult::String(String::new().into())
                }
                Err(error) => CalcResult::new_error(error, cell, String::new()),
            },
            error @ CalcResult::Error { .. } => error,
            // An array lifts element-wise, text and error elements kept and anything else "":
            // `T({"a",1,#N/A,TRUE})` spills "a", "", #N/A, "" (probes/t-array-lift,
            // legacy-array-result S!G1:G2; T-1729).
            // In a legacy formula's value position it is `#VALUE!` (legacy-n-t-reference-args;
            // SPEC-0012).
            CalcResult::Array(_) if legacy => {
                CalcResult::new_error(Error::VALUE, cell, String::new())
            }
            CalcResult::Array(values) => {
                CalcResult::Array(map_elements(&values, |value| match value {
                    ArrayNode::String(text) if over_short_limit(text) => {
                        ArrayNode::Error(Error::VALUE)
                    }
                    ArrayNode::String(_) | ArrayNode::Error(_) => value.clone(),
                    _ => ArrayNode::String("".into()),
                }))
            }
            _ => CalcResult::String("".into()),
        }
    }

    /// T's argument read top-left, and how to tell whether it is a cell's text. Each IF, CHOOSE,
    /// IFS and SWITCH at the argument's root is replaced by the argument it picks, its condition
    /// or index read once, so the value and the reference test see one pick:
    /// `T(CHOOSE(RANDBETWEEN(1,2),D1,REPT("x",256)))` with 256 units in D1 is "" or `#VALUE!` by
    /// the pick it made, never the value of one pick and the test of another (T-3142). A LET there
    /// is read and tested while its names are bound, and the `@` of a picker is read once, so
    /// `T(LET(i,RANDBETWEEN(1,2),CHOOSE(i,D1,…)))` and `T(@CHOOSE(RANDBETWEEN(1,2),D1,…))` see
    /// one draw too (T-3157). `outer` is the picker that picked `node`, when a LET was picked.
    fn t_argument<'a>(
        &mut self,
        node: &'a Node,
        outer: Option<&Function>,
        cell: CellReferenceIndex,
    ) -> (CalcResult, ArgumentRead<'a>) {
        let mut arm = node;
        let mut picker = outer;
        loop {
            match arm {
                // Not `ungrouped`: a unary plus makes a value.
                Node::Group { child, .. } | Node::Spaced { child, .. } => arm = child,
                Node::FunctionKind {
                    kind: Function::Let,
                    args,
                } => return self.t_let_argument(args, picker, cell),
                Node::ImplicitIntersection { child, .. } if is_picker(child) => {
                    return self.t_at_picker_argument(child, cell)
                }
                Node::FunctionKind { kind, args } => match self.pick_step(kind, args, cell) {
                    Some(Picked::Node(picked)) => {
                        picker = Some(kind);
                        arm = picked;
                    }
                    Some(Picked::Value(value)) => {
                        return (self.top_left(value), ArgumentRead::Settled(false))
                    }
                    None => break,
                },
                _ => break,
            }
        }
        match picker {
            Some(kind) => {
                let value = self.picked_value(kind, arm, cell);
                (self.top_left(value), ArgumentRead::Picked(arm))
            }
            None => (
                self.evaluate_node_top_left(arm, cell),
                ArgumentRead::Written(arm),
            ),
        }
    }

    /// T's argument when it is a LET (`args`): its body read as [`Self::t_argument`] reads an
    /// argument, and tested for cell text, while the LET's names are bound, so a volatile value
    /// is drawn once for both (T-3157). LET's own error is a value.
    fn t_let_argument(
        &mut self,
        args: &[Node],
        outer: Option<&Function>,
        cell: CellReferenceIndex,
    ) -> (CalcResult, ArgumentRead<'static>) {
        let mut cell_text = false;
        let value = self.let_scope(args, cell, |model, body| {
            let (value, read) = model.t_argument(body, outer, cell);
            // A body as written is tested as `is_reference` tests a LET, in reference context:
            // `T(LET(x,D1,+x))` keeps the reading it had (unprobed, TODO(T-3167)).
            let read = match read {
                ArgumentRead::Written(body) => ArgumentRead::Picked(body),
                read => read,
            };
            cell_text = matches!(&value, CalcResult::String(text) if over_short_limit(text))
                && model.reads_cell_text(read, cell);
            value
        });
        (
            value.unwrap_or_else(|error| error),
            ArgumentRead::Settled(cell_text),
        )
    }

    /// T's argument when it is the `@` of an IF, CHOOSE, IFS or SWITCH (`picker`): read once as
    /// `@` reads its operand, and a cell's text when that operand was a reference (T-3157).
    fn t_at_picker_argument(
        &mut self,
        picker: &Node,
        cell: CellReferenceIndex,
    ) -> (CalcResult, ArgumentRead<'static>) {
        let operand = self.evaluate_implicit_intersection_operand(picker, cell);
        let is_reference = matches!(operand, CalcResult::Range { .. });
        let value = self.implicit_intersection_to_value(operand, cell);
        (self.top_left(value), ArgumentRead::Settled(is_reference))
    }

    /// Whether T's argument, read as [`Self::t_argument`] says, hands T a cell's text.
    fn reads_cell_text(&mut self, read: ArgumentRead<'_>, cell: CellReferenceIndex) -> bool {
        match read {
            ArgumentRead::Settled(cell_text) => cell_text,
            ArgumentRead::Picked(arm) => self.picked_arm_is_reference(arm, cell),
            ArgumentRead::Written(node) => self.written_reads_cell_text(node, cell),
        }
    }

    /// Whether `node`, T's argument as written, hands T a cell's text rather than a computed
    /// value. A legacy formula's automatic `@` and a spill reference still stand for cells; a
    /// unary plus makes a value (probes/text-255-units-reference-shapes A9, A13, A18; T-3107).
    fn written_reads_cell_text(&mut self, node: &Node, cell: CellReferenceIndex) -> bool {
        let mut node = node;
        loop {
            match node {
                Node::UnaryPlus { .. } => return false,
                Node::Group { child, .. }
                | Node::Spaced { child, .. }
                | Node::ImplicitIntersection { child, .. } => node = child,
                Node::SpillRangeOperator { .. } => return true,
                _ => return self.is_reference(node, cell),
            }
        }
    }
}

/// Whether `node` is an IF, CHOOSE, IFS or SWITCH call, parentheses aside: the functions
/// [`Model::pick_step`] picks through, which this list must track.
fn is_picker(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::FunctionKind {
            kind: Function::If | Function::Choose | Function::Ifs | Function::Switch,
            ..
        }
    )
}

/// Whether `text` is over T's 255-unit limit: `#VALUE!` computed or as an array element, "" read
/// from a cell, so the only text whose cell-text test matters.
fn over_short_limit(text: &Text) -> bool {
    reject_over_short_limit(text).is_err()
}

/// How T tells whether its argument is a cell's text ([`Model::t_argument`]).
#[derive(Clone, Copy)]
enum ArgumentRead<'a> {
    /// Already decided: false for the value an IF, CHOOSE, IFS or SWITCH gives in place of an
    /// argument; for a LET or the `@` of a picker, what its single reading showed.
    Settled(bool),
    /// The argument a picker picked, tested as the picker's own reference test tests it.
    Picked(&'a Node),
    /// The argument as written.
    Written(&'a Node),
}
