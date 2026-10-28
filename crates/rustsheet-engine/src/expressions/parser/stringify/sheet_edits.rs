//! Rewrites of the sheet prefixes in a parsed formula when a sheet is renamed or moved.

use crate::expressions::parser::{for_each_chain_operand_mut, Node};
use crate::expressions::utils::rename_defined_name_sheet;

/// Calls `f` on every node of `node` that names a sheet (a reference, a 3-D span, a wrong
/// reference or a defined name) and walks through every other node to reach them.
fn for_each_sheet_reference_mut(node: &mut Node, f: &mut impl FnMut(&mut Node)) {
    match node {
        Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::ThisRowKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::DefinedNameKind(_) => f(node),

        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            for_each_sheet_reference_mut(left, f);
            for_each_sheet_reference_mut(right, f);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| for_each_sheet_reference_mut(operand, f));
        }
        Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
            for arg in args {
                for_each_sheet_reference_mut(arg, f);
            }
        }
        Node::UnaryKind { right: child, .. }
        | Node::ImplicitIntersection { child, .. }
        | Node::SpillRangeOperator { child }
        | Node::Group { child, .. }
        | Node::UnaryPlus { child }
        | Node::Spaced { child, .. }
        | Node::StructuredReferenceKind { lowered: child, .. }
        | Node::LambdaDefKind { body: child, .. } => for_each_sheet_reference_mut(child, f),
        Node::LambdaCallKind { lambda, args } => {
            for_each_sheet_reference_mut(lambda, f);
            for arg in args {
                for_each_sheet_reference_mut(arg, f);
            }
        }

        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::ArrayKind(_)
        | Node::NamedVariableKind { .. }
        | Node::EmptyArgKind => {}
    }
}

/// Writes `new_name` for the sheet at `sheet_index` everywhere `node` names it.
pub(crate) fn rename_sheet_in_node(node: &mut Node, sheet_index: u32, new_name: &str) {
    for_each_sheet_reference_mut(node, &mut |node| match node {
        Node::SheetSpanKind {
            first_sheet,
            last_sheet,
            first_index,
            last_index,
            ..
        } => {
            if *first_index == sheet_index {
                new_name.clone_into(first_sheet);
            }
            if *last_index == sheet_index {
                new_name.clone_into(last_sheet);
            }
        }
        Node::WrongReferenceKind { sheet_name, .. } => {
            if let Some(name) = sheet_name {
                if name.to_uppercase() == new_name.to_uppercase() {
                    *sheet_name = Some(name.to_owned());
                }
            }
        }
        Node::WrongRangeKind { sheet_name, .. } => {
            if sheet_name.is_some() {
                *sheet_name = Some(new_name.to_owned());
            }
        }
        Node::DefinedNameKind((n, s, _)) => rename_defined_name_sheet(n, *s, sheet_index, new_name),
        _ => node.rename_reference_sheet(sheet_index, new_name),
    });
}

/// Points every reference of `node` written without a sheet prefix at the sheet `to`. A
/// D-function's computed criterion reads its unqualified references on the database's sheet, not
/// its own (probes/db-computed-criteria-cross-sheet, criteria.md "D* computed criteria and an
/// unknown header in the gen-m9 database sweep (T-2258)").
pub(crate) fn retarget_unqualified_references(node: &mut Node, to: u32) {
    for_each_sheet_reference_mut(node, &mut |node| {
        if let Node::ReferenceKind {
            sheet_name: None,
            sheet_index,
            ..
        }
        | Node::RangeKind {
            sheet_name: None,
            sheet_index,
            ..
        }
        | Node::ThisRowKind {
            sheet_name: None,
            sheet_index,
            ..
        } = node
        {
            *sheet_index = to;
        }
    });
}

/// Rewrites the 3-D spans of `node` for moving the sheet at `from` to `to`. Both indices and
/// `sheet_names` (in tab order) are from before the move.
///
/// Excel rewrites a span whose moved end lands past its other end to the sheets still between
/// the ends, and a span left with one sheet to a plain reference to it: with Calc, Jan, Feb, Mar,
/// moving Mar first turns `Jan:Mar!A1` into `Jan:Feb!A1`, and moving Feb first turns `Jan:Feb!A1`
/// into `Jan!A1` (docs/excel-semantics/references.md, "3-D references when sheets move or are
/// renamed"). Every other move keeps the ends, which then resolve by name in the new order.
pub(crate) fn move_sheet_in_node(node: &mut Node, from: u32, to: u32, sheet_names: &[String]) {
    for_each_sheet_reference_mut(node, &mut |node| {
        let Node::SheetSpanKind {
            first_sheet,
            last_sheet,
            first_index,
            last_index,
            ..
        } = node
        else {
            return;
        };
        let (low, high) = if first_index <= last_index {
            (*first_index, *last_index)
        } else {
            (*last_index, *first_index)
        };
        // After the move, the last sheet lands before the first one when it goes to `low` or
        // earlier; the first sheet lands after the last one when it goes to `high` or later
        // (the last sheet is at `high - 1` once the first sheet is taken out).
        let (low, high) = if low == high {
            return;
        } else if from == high && to <= low {
            (low, high - 1)
        } else if from == low && to >= high {
            (low + 1, high)
        } else {
            return;
        };
        let (Some(first), Some(last)) = (
            sheet_names.get(low as usize),
            sheet_names.get(high as usize),
        ) else {
            return;
        };
        first.clone_into(first_sheet);
        last.clone_into(last_sheet);
        *first_index = low;
        *last_index = high;
    });
}
