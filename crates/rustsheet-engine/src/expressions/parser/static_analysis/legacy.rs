//! Where a legacy (pre-dynamic-array) formula implicitly intersects an argument that a dynamic
//! formula reads whole: the rules behind the automatic `@`s of `add_implicit_intersection` and
//! their mirror in `remove_redundant_implicit_intersection`. Excel:
//! docs/excel-semantics/references.md "Legacy formulas intersect the ranges in value arguments".
//!
//! The same walk places the legacy N/T marker ([`mark_legacy_reading`], SPEC-0012), so
//! the value-position flag `arrays` ([`legacy_array_context`], [`returns_an_argument`]) decides
//! evaluation results as well as where an `@` goes: a change to it needs a probe's legacy values,
//! not only its FORMULATEXT (SPEC-0012 "Placement", item 4).

use super::lifted::lifts_arrays_at;
use super::{
    add_legacy_intersection, for_each_chain_operand_mut, remove_redundant_legacy_intersection,
    run_static_analysis_on_node, static_analysis_on_function, Node, Signature, StaticResult,
};
use crate::functions::Function;

/// Legacy formulas evaluate SUMPRODUCT's arguments as arrays, with no implicit intersection inside
/// (`SUMPRODUCT(SUMIFS(B1:B6,A1:A6,D1:D2))` lifts the criterion where `SUM` intersects it;
/// probes/ifs-legacy-sumproduct, excel-graph, T-0278), except for the conditions of
/// [`for_each_array_context_site`].
pub(super) fn is_array_context(kind: &Function) -> bool {
    matches!(kind, Function::Sumproduct)
}

/// Adds the automatic `@`s inside the arguments of an array-context function
/// ([`is_array_context`]).
pub(super) fn add_in_array_context(args: &mut [Node]) {
    for arg in args {
        for_each_array_context_site(arg, &mut |node, place| match place {
            Place::Condition => add_condition_intersection(node),
            Place::IndexNumber => {
                add_index_array_at(&Function::Index, 1, node);
                add_condition_intersection(node);
            }
        });
    }
}

/// Drops the `@`s [`add_in_array_context`] would add again on import; any other `@` inside the
/// arguments of an array-context function is meaningful and kept.
pub(super) fn remove_in_array_context(args: &mut [Node]) {
    for arg in args {
        for_each_array_context_site(arg, &mut |node, place| {
            if place == Place::IndexNumber {
                remove_index_array_at(&Function::Index, 1, node);
            }
            remove_redundant_legacy_intersection(node, true, false);
        });
    }
}

/// Where [`for_each_array_context_site`] found a node a legacy formula intersects inside an
/// array context.
#[derive(PartialEq, Eq, Clone, Copy)]
enum Place {
    /// The condition of IF or the value of IFERROR and IFNA, intersected as a whole.
    Condition,
    /// INDEX's `row_num`, `column_num` or `area_num`, intersected as a whole and in its array
    /// constants ([`add_index_array_at`]).
    IndexNumber,
}

/// The `@` a legacy formula puts on a condition or an INDEX index under an array context: on the
/// whole of it unless it is a scalar.
fn add_condition_intersection(node: &mut Node) {
    if !matches!(run_static_analysis_on_node(node), StaticResult::Scalar) {
        add_legacy_intersection(node, true, false);
    }
}

/// Whether `index` is one of INDEX's `row_num`, `column_num` or `area_num`.
fn is_index_number(kind: &Function, index: usize) -> bool {
    matches!(kind, Function::Index) && index > 0
}

/// A legacy formula reads an array constant in INDEX's `row_num`, `column_num` or `area_num` as
/// its first element, also inside SUMPRODUCT and inside arithmetic, and Excel shows that `@`:
/// `SUMPRODUCT(INDEX(B1:B6,{1;2;3}))` is 1 and `=SUM(INDEX(B1:B6,{2;3}))` shows as
/// `=SUM(INDEX(B1:B6,@{2;3}))`, `{2;3}+0` as `@{2;3}+0` (probes/array-context-legacy-sumproduct
/// G, T; probes/legacy-index-array-index, excel-graph, T-0294). An array constant inside a
/// function call is that function's: `INDEX(B1:B6,N(IF(1,{2;3})))` shows no `@` (the only call
/// probed; an array-returning one such as `ROW(B1:B2)` gets the `@` of any scalar argument, unprobed). Ranges there intersect through
/// [`legacy_intersects`] like any scalar argument.
pub(super) fn add_index_array_at(kind: &Function, index: usize, arg: &mut Node) {
    if is_index_number(kind, index) {
        for_each_value_operand(arg, &mut |operand| {
            if matches!(operand, Node::ArrayKind(_)) {
                *operand = Node::automatic_at(operand.clone());
            }
        });
    }
}

/// Drops the `@`s [`add_index_array_at`] adds again on import.
pub(super) fn remove_index_array_at(kind: &Function, index: usize, arg: &mut Node) {
    if is_index_number(kind, index) {
        for_each_value_operand(arg, &mut |operand| {
            if let Node::ImplicitIntersection { child, .. } = operand {
                if matches!(child.as_ref(), Node::ArrayKind(_)) {
                    *operand = child.as_ref().clone();
                }
            }
        });
    }
}

/// Whether `node` is an array constant of one element, `{5}`.
fn is_one_element_array(node: &Node) -> bool {
    matches!(node, Node::ArrayKind(rows) if rows.len() == 1 && rows[0].len() == 1)
}

/// Whether a legacy formula shows an `@` on the array constant `node` in a scalar argument, one
/// in a value position of the cell (`arrays`, [`legacy_array_context`]) or a one-element one
/// anywhere: `=ABS({5})` is shown as `=ABS(@{5})` (probes/legacy-one-cell-shape-intersection M3,
/// excel-graph, T-1633), `=N({2;3})` as `=N(@{2;3})` and `=ABS({-2;3})` as `=ABS(@{-2;3})`, 2
/// (probes/legacy-array-result E, F, T-1713; probes/t-n-range-top-left M, O, T-0985), while
/// `=SUM(N({1,2}))` is 3 with none (probes/n-array-lift A3). An array constant in an operator
/// (`N({2;3}+0)`) is unprobed and left alone. TYPE reads its argument whole and shows none:
/// `=TYPE({5})` stays `=TYPE({5})` (probes/legacy-choose-arm-aggregate-and-array-constant-at
/// L3, excel-graph, T-1638).
fn shows_scalar_array_at(signature: Option<&Signature>, arrays: bool, node: &Node) -> bool {
    matches!(signature, Some(Signature::Scalar))
        && (is_one_element_array(node) || arrays && matches!(node, Node::ArrayKind(_)))
}

/// The `@`s a legacy formula puts on the array constants of argument `index` of `kind`, `arrays`
/// when it is in a value position of the cell ([`legacy_array_context`]).
pub(super) fn add_array_at(
    kind: &Function,
    signature: Option<&Signature>,
    index: usize,
    arg: &mut Node,
    arrays: bool,
) {
    add_index_array_at(kind, index, arg);
    if shows_scalar_array_at(signature, arrays, arg) {
        *arg = Node::automatic_at(arg.clone());
    }
}

/// Drops the `@`s [`add_array_at`] adds again on import.
pub(super) fn remove_array_at(
    kind: &Function,
    signature: Option<&Signature>,
    index: usize,
    arg: &mut Node,
    arrays: bool,
) {
    remove_index_array_at(kind, index, arg);
    if let Node::ImplicitIntersection { child, .. } = arg {
        if shows_scalar_array_at(signature, arrays, child) {
            *arg = child.as_ref().clone();
        }
    }
}

/// Whether argument `arg` of `kind` with `signature` is still in a value position of the cell
/// when `kind`'s call is (`arrays`), so that a legacy formula puts its `@` on an array-valued call
/// or array constant there rather than on an enclosing call: a scalar argument of a function that
/// computes from it (`=ABS(@IF(1,{-2;3}))`, `=LEN(@IF(1,{"ab";"c"}))`, `=N(@TRANSPOSE({2,3}))`,
/// `=N(@IFERROR({2;3},0))`; probes/legacy-array-result, legacy-n-t-reference-args D, T-1713).
/// Not the arguments of IF, IFERROR, IFNA, CHOOSE (but its index, [`is_choose_index`]), IFS or
/// SWITCH, whose result is one of them: `=@IF(1,{2;3})` and `=@IF(1,N(IF(1,{2;3})))` take the
/// `@` on the call (probes/legacy-array-result D, legacy-n-t-reference-args P); nor N or T over IF,
/// SWITCH or a CHOOSE, INDEX or IFS of array constants ([`leaves_call_whole`]); nor
/// anything under a vector argument, `=SUM(N({1,2}))` (probes/n-array-lift A3) or LET, LAMBDA.
pub(super) fn legacy_array_context(
    kind: &Function,
    signature: Option<&Signature>,
    index: usize,
    arg: &Node,
    arrays: bool,
) -> bool {
    arrays
        && matches!(signature, Some(Signature::Scalar))
        && (!returns_an_argument(kind) || is_choose_index(kind, index))
        && !matches!(kind, Function::Let | Function::Lambda)
        && !leaves_call_whole(kind, arg)
}

/// CHOOSE's index is the one argument it computes from rather than returns, so a legacy formula
/// intersects an array constant there as in any scalar argument: `=CHOOSE({1,2},H1:H2,I1:I2)` is
/// shown as `=@CHOOSE(@{1,2},H1:H2,I1:I2)`, 3 on the arms' first row and `#VALUE!` off them
/// (probes/legacy-choose-array-index, excel-graph, T-1882).
fn is_choose_index(kind: &Function, index: usize) -> bool {
    matches!(kind, Function::Choose) && index == 0
}

/// Whether `kind`'s result is one of its arguments as it is: IF, IFERROR, IFNA, CHOOSE, IFS and
/// SWITCH.
fn returns_an_argument(kind: &Function) -> bool {
    matches!(
        kind,
        Function::If
            | Function::Iferror
            | Function::Ifna
            | Function::Choose
            | Function::Ifs
            | Function::Switch
    )
}

/// Whether a legacy formula puts the `@` on the array-valued call `kind(args)` in a value
/// position of the cell, its arguments already rewritten: when the array is one of its arguments
/// ([`returns_an_argument`], `=@IF(1,{2;3})`) or comes from a non-scalar one
/// (`=N(@TRANSPOSE({2,3}))`). A scalar argument that is still an array is one the rewrite left
/// whole, so the call is not intersected either: `=ABS(N(IF(1,{2;3})))` shows no `@`
/// (probes/legacy-n-t-reference-args E20), and `=ABS({2;3}+0)`, unprobed, is left alone.
pub(super) fn legacy_array_call_at(
    kind: &Function,
    signature: &[Signature],
    args: &[Node],
) -> bool {
    returns_an_argument(kind)
        || args.iter().enumerate().any(|(index, arg)| {
            !matches!(signature.get(index), Some(Signature::Scalar))
                && !matches!(run_static_analysis_on_node(arg), StaticResult::Scalar)
        })
}

/// Whether a legacy formula puts no `@` on the call `arg` of `kind`, a value it would otherwise
/// intersect: N and T over IF or SWITCH (`=N(IF(1,{2;3}))`, `=T(IF(1,{"a";"b"}))`,
/// `=N(SWITCH(1,1,{2;3}))`, the last spilling 2, 3; probes/legacy-array-result A, G, I,
/// legacy-n-t-reference-args M, T-1713), and over a CHOOSE, INDEX or IFS that returns only array
/// constants (`=N(CHOOSE(1,{2;3}))`, `=N(INDEX({2;3},0,1))`, `=N(IFS(1,{2;3}))`, A20, B20, L20),
/// or INDEX of a plain reference ([`is_plain_reference`], T-1908: `=N(INDEX(Z1:Z2,1))`, which is
/// read as a reference and not marked), where IFERROR gets one (`=N(@IFERROR({2;3},0))`, D). An
/// explicit `@` on the call is looked through, so the export keeps it.
pub(super) fn leaves_call_whole(kind: &Function, arg: &Node) -> bool {
    let call = match arg {
        Node::ImplicitIntersection { child, .. } => child.as_ref(),
        other => other,
    };
    kind.is_n_or_t()
        && match call {
            Node::FunctionKind {
                kind: Function::If | Function::Switch,
                ..
            } => true,
            // INDEX of a reference keeps its indices' own `@`: `=N(INDEX(Z1:Z2,@W1:W2))`
            // (probes/legacy-n-t-reference-args-2 L20, M20, T-1908).
            Node::FunctionKind {
                kind: Function::Index,
                args,
            } if args.first().is_some_and(is_plain_reference) => true,
            Node::FunctionKind { kind, args } => returns_array_constants(kind, args),
            _ => false,
        }
}

/// Whether `kind(args)` is a CHOOSE, INDEX or IFS whose every argument it can return
/// ([`is_returned_argument`]) is an array constant: the one shape of them probed under N and T
/// (probes/legacy-n-t-reference-args A, B, K, L). Over a reference they read the reference,
/// unmarked (T-1908); a CHOOSE mixing references and array constants is
/// [`chooses_between_references_and_array_constants`]; other mixed arms keep their `@`.
fn returns_array_constants(kind: &Function, args: &[Node]) -> bool {
    let mut returned = args
        .iter()
        .enumerate()
        .filter(|(index, _)| is_returned_argument(kind, *index))
        .peekable();
    (matches!(kind, Function::Choose | Function::Index | Function::Ifs)
        && returned.peek().is_some()
        && returned.all(|(_, arg)| matches!(arg, Node::ArrayKind(_))))
        || chooses_between_references_and_array_constants(kind, args)
}

/// Whether `kind(args)` is a CHOOSE with a number or boolean literal index whose values are
/// plain references and array constants, at least one of each kind and no one-element array
/// (unprobed, it keeps its `@`): Excel shows no `@` on the call
/// and N or T read the arm the index picks, a reference as its top-left cell and an array constant
/// as `#VALUE!` (`=N(CHOOSE(1,Z1,{2;3}))` is 7, `=N(CHOOSE(2,Z1,{2;3}))` and
/// `=N(CHOOSE(1,{2;3},Z1))` are `#VALUE!`, `=N(CHOOSE(1,Z1:Z2,{2;3}))` is 7:
/// probes/legacy-n-t-choose-index-shapes L, O, P, Q, R, T-1918).
fn chooses_between_references_and_array_constants(kind: &Function, args: &[Node]) -> bool {
    let [index, arms @ ..] = args else {
        return false;
    };
    matches!(kind, Function::Choose)
        && matches!(index, Node::NumberKind(_) | Node::BooleanKind(_))
        && arms
            .iter()
            .any(|arm| matches!(arm, Node::ArrayKind(_)) && !is_one_element_array(arm))
        && arms
            .iter()
            .any(|arm| !matches!(arm, Node::ArrayKind(_)) && is_plain_reference(arm))
        && arms
            .iter()
            .all(|arm| matches!(arm, Node::ArrayKind(_)) || is_plain_reference(arm))
}

/// Whether argument `index` of `kind` is one its result can be, as it is or as a part of it: the
/// arms of IF, the values of CHOOSE and IFS, and INDEX's `array`.
fn is_returned_argument(kind: &Function, index: usize) -> bool {
    match kind {
        Function::If | Function::Choose => index > 0,
        Function::Ifs => index % 2 == 1,
        Function::Index => index == 0,
        _ => false,
    }
}

/// Marks the N or T call `kind(args)`, in a value position of a legacy cell and its arguments
/// already rewritten, as one that reads an array argument as `#VALUE!` (the marker of SPEC-0012,
/// rule 2). Its argument must be a bare IF (no `@`, no parentheses), or a CHOOSE, INDEX or IFS
/// that returns only array constants ([`returns_array_constants`]), and not statically a scalar.
/// `=N(IF(1,{2;3}))`, `=T(IF(1,{"a";"b"}))`, `=N(CHOOSE(1,{2;3}))`,
/// `=N(CHOOSE({1;2},{2;3},{4;5}))` (no `@` on the index either), `=N(INDEX({2,3;4,5},1))`,
/// `=N(IF(1,{2;3}+0))`, `=N(IF(1,{2;3},Z1))`, `=ABS(N(IF(1,{2;3})))` and
/// `=LEN(N(IF(1,{2;3})))` are `#VALUE!` in a legacy cell, while `=N(SWITCH(1,1,{2;3}))` spills,
/// `=N(@IFERROR({2;3},0))` is 2 and `=N(INDEX({2},1,1))` 2 (probes/legacy-n-t-reference-args,
/// legacy-n-t-marker-shapes, legacy-array-result A, G, I, n-array-lift D4; T-1713). A scalar IF
/// reads the same either way, so `=N(IF(A1,1,2))` gets no marker and its export does not change,
/// nor does one the rewrite made scalar, `=N(IF(@W1:W2,Z1,Z2))` (legacy-n-t-reference-args-2 H).
///
/// The decision runs on the rewritten argument, looking through the automatic `@` that
/// [`shows_scalar_array_at`] put on a one-element array constant the argument returns, and the
/// marked call drops it: Excel shows `=N(IF(1,{2}))`, `=T(IF(1,{"a"}))`, `=N(CHOOSE(1,{2}))` and
/// `=N(IFS(1,{2}))` with none, all `#VALUE!` (legacy-n-t-reference-args N, legacy-n-t-marker-shapes
/// A-D; SPEC-0012 "Placement", item 6). Those `@`s are taken off in place and put back if the call
/// is not marked, so nothing is cloned; the rewrite's own display text has none, so the decision is
/// the same on it.
pub(super) fn mark_legacy_reading(kind: &mut Function, args: &mut [Node]) {
    let [Node::FunctionKind {
        kind: call,
        args: call_args,
    }] = args
    else {
        return;
    };
    if !kind.is_n_or_t()
        || !matches!(
            call,
            Function::If | Function::Choose | Function::Index | Function::Ifs
        )
    {
        return;
    }
    let mut dropped = Vec::new();
    for (index, returned) in call_args.iter_mut().enumerate() {
        if let Node::ImplicitIntersection {
            automatic: true,
            child,
        } = returned
        {
            if is_returned_argument(call, index) && is_one_element_array(child) {
                *returned = std::mem::replace(child.as_mut(), Node::EmptyArgKind);
                dropped.push(index);
            }
        }
    }
    let marked = (matches!(call, Function::If) || returns_array_constants(call, call_args))
        && !matches!(
            static_analysis_on_function(call, call_args),
            StaticResult::Scalar
        );
    if marked {
        *kind = kind.legacy_reading();
    } else {
        for index in dropped {
            if let Some(returned) = call_args.get_mut(index) {
                let constant = std::mem::replace(returned, Node::EmptyArgKind);
                *returned = Node::automatic_at(constant);
            }
        }
    }
}

/// Calls `f` on `node`, or on each operand of the operators, parentheses and signs it is made
/// of, at any depth (bounded by the parser's depth cap).
fn for_each_value_operand(node: &mut Node, f: &mut impl FnMut(&mut Node)) {
    match node {
        Node::UnaryKind { right: child, .. } | Node::Group { child, .. } => {
            for_each_value_operand(child, f);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| for_each_value_operand(operand, f));
        }
        _ => f(node),
    }
}

/// Adds the `@`s a legacy formula puts in `arg`, the argument of `kind` with `signature`, `arrays`
/// when it is in a value position of the cell ([`legacy_array_context`]); none where N or T reads
/// a reference whole ([`reads_reference_whole`]) and none on the call [`leaves_call_whole`].
pub(super) fn add_argument_intersection(
    kind: &Function,
    signature: Option<&Signature>,
    index: usize,
    arg: &mut Node,
    arrays: bool,
) {
    if !reads_reference_whole(kind, arg) {
        let add = legacy_intersects(kind, signature, index, arg) && !leaves_call_whole(kind, arg);
        add_legacy_intersection(arg, add, arrays);
    }
}

/// Whether a legacy formula intersects `arg`, argument `index` of `kind` with `signature`: a scalar
/// argument that is not a scalar, `IFERROR(B1:B6*1,100)` and `IF(A1:A6="a",1,0)` included
/// (probes/legacy-value-argument-intersection I10, array-context-legacy-sumproduct R10), or an
/// argument [`legacy_context_intersects`] makes a value.
fn legacy_intersects(
    kind: &Function,
    signature: Option<&Signature>,
    index: usize,
    arg: &Node,
) -> bool {
    (is_scalar_context(kind, signature) && !reads_reference(kind, arg))
        || legacy_value_operand(kind, index, arg)
}

/// Whether `arg` of `kind` needs no intersection although `kind` takes a scalar: a scalar, or
/// under N or T, which read a reference and give its top-left cell, a range or an IF or OFFSET
/// of one ([`is_plain_reference`]): a legacy `=N(Z1:Z2)`,
/// `=N(OFFSET(Z1:Z2,0,0))` and `=N(IF(1,Z1:Z2))` are 7 off the range's rows and show no `@`
/// (probes/legacy-n-t-reference-args C, J, Q, T-1728), and so are T's (probes/t-n-range-top-left
/// F10, P10, Q10; inside the rows too, F2; T-0985). An array constant or any other call keeps
/// its `@` (T-1713, T-1727).
fn reads_reference(kind: &Function, arg: &Node) -> bool {
    match run_static_analysis_on_node(arg) {
        StaticResult::Scalar => true,
        StaticResult::Range(..) | StaticResult::Unknown => reads_reference_whole(kind, arg),
        StaticResult::Array(..) => false,
    }
}

/// Whether `arg` is an argument of N or T that is a reference it reads whole, with nothing in
/// it to intersect: the `@` of a range or of an IF arm in it is not added either.
pub(super) fn reads_reference_whole(kind: &Function, arg: &Node) -> bool {
    kind.is_n_or_t() && is_plain_reference(arg)
}

/// Whether `node` is a reference N or T reads whole, in the shapes probed: a range or cell, an
/// OFFSET of one with number literals for its offsets, or an IF with a number or boolean literal
/// condition whose arms are such references. A range in a scalar slot (`IF(A1:A2,Z1,Z2)`), an
/// array constant, an operator or any other call is not (`N(B1:B3*1)` is `#VALUE!` off the
/// rows: probes/n-array-lift D7); CHOOSE with a literal index over such references and INDEX of
/// one ([`leaves_call_whole`]) are too (probes/legacy-n-t-reference-args-2, T-1908;
/// docs/excel-semantics/references.md "Legacy N and T over a reference-returning call"). An
/// explicit stack keeps the walk off the call stack: a formula's tree is untrusted input.
fn is_plain_reference(node: &Node) -> bool {
    let is_literal = |node: &Node| matches!(node, Node::NumberKind(_) | Node::BooleanKind(_));
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        match node {
            Node::ReferenceKind { .. } | Node::RangeKind { .. } => {}
            Node::FunctionKind {
                kind: Function::If,
                args,
            } if args.len() > 1 && is_literal(&args[0]) => pending.extend(&args[1..]),
            Node::FunctionKind {
                kind: Function::Offset,
                args,
            } if args.len() > 2 && args[1..].iter().all(is_literal) => pending.push(&args[0]),
            Node::FunctionKind {
                kind: Function::Choose,
                args,
            } if args.len() > 1 && is_literal(&args[0]) => pending.extend(&args[1..]),
            _ => return false,
        }
    }
    true
}

/// Whether the argument `arg` (number `index`) of `kind` with `signature` is read as a value,
/// where an `@` is redundant because [`legacy_intersects`] adds it again on import.
pub(super) fn legacy_context_intersects(
    kind: &Function,
    signature: Option<&Signature>,
    index: usize,
    arg: &Node,
) -> bool {
    is_scalar_context(kind, signature) || legacy_value_operand(kind, index, arg)
}

/// A scalar argument, or any argument of a function whose arguments a legacy formula reads as
/// scalars although a dynamic one takes them whole: legacy `TYPE(B1:B6)` is 1 on the range's rows
/// and 16 (`#VALUE!`) off them, where a dynamic one is 64 (probes/legacy-type-intersection,
/// fixtures/external-link-cached-ranges Refs!B62; T-1526).
fn is_scalar_context(kind: &Function, signature: Option<&Signature>) -> bool {
    matches!(signature, Some(Signature::Scalar)) || matches!(kind, Function::Type)
}

/// Calls `f` on each site under an array context ([`is_array_context`]) that a legacy formula
/// still intersects, with its [`Place`]: a condition, or an index of INDEX. The conditions are the
/// condition of IF and the value of IFERROR and IFNA, at any depth through the operators and
/// functions that keep the array context. `SUMPRODUCT(IF(B1:B6>1,B1:B6,0))`,
/// `SUMPRODUCT(ABS(IF(B1:B6>1,B1:B6,0)))` and `SUMPRODUCT(IFERROR(B1:B6,100))` are `#VALUE!`,
/// `#VALUE!` and 100 off the range's rows, while IF's value branches stay arrays
/// (`SUMPRODUCT(IF(TRUE,B1:B6))` 12; probes/array-context-legacy-sumproduct F10, S10, U10,
/// legacy-value-argument-intersection J10, K10, L10, N10; T-0293, T-1526), and on INDEX's
/// indices, which still intersect ([`add_index_array_at`]) while its first argument keeps the
/// array context. Recursion is bounded by the
/// parser's depth cap.
fn for_each_array_context_site(node: &mut Node, f: &mut impl FnMut(&mut Node, Place)) {
    match node {
        Node::FunctionKind {
            kind: Function::Index,
            args,
        } => {
            let mut args = args.iter_mut();
            if let Some(array) = args.next() {
                for_each_array_context_site(array, f);
            }
            for index in args {
                f(index, Place::IndexNumber);
            }
        }
        Node::FunctionKind {
            kind: Function::If | Function::Iferror | Function::Ifna,
            args,
        } => {
            let mut args = args.iter_mut();
            if let Some(condition) = args.next() {
                f(condition, Place::Condition);
            }
            for arg in args {
                for_each_array_context_site(arg, f);
            }
        }
        Node::FunctionKind { args, .. } => {
            for arg in args {
                for_each_array_context_site(arg, f);
            }
        }
        Node::UnaryKind { right: child, .. } | Node::Group { child, .. } => {
            for_each_array_context_site(child, f);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| {
                for_each_array_context_site(operand, f);
            });
        }
        _ => {}
    }
}

/// Whether `arg` of `kind` is arithmetic or a comparison that a legacy formula evaluates as a
/// value, intersecting the ranges in it, although `kind` takes a range there: `SUM(B1:B6+0)`,
/// `SUM(-B1:B6)`, `MAX(B1:B6*1)` and `LARGE(B1:B6*1,1)` are `#VALUE!` off the range's rows and
/// `COUNT(B1:B6*1)` 0, while `SUM(B1:B6)` and `SUM((B1:B6))` read the range
/// (probes/legacy-value-argument-intersection; `[1]Uses!$A$2:$A$3+0` in
/// fixtures/external-link-cached-ranges Refs!B44; T-1526), as for the other aggregates in the match
/// below (probes/legacy-value-argument-intersection-b, T-1551, and -c, T-1556); MODE, MODE.MULT, MODE.SNGL, MMULT, INDEX and LOOKUP
/// evaluate such an argument as an array (that probe S; probes/array-context-legacy-sumproduct
/// N, P, Q). docs/excel-semantics/references.md has the result per function. A `LiftArrays`
/// argument of the lifting table is one too ([`lifts_arrays_at`]: `=EDATE(A1:A60+0,1)` shows
/// `=EDATE(@A1:A60+0,1)`, probes/scalar-lift-shapes-c F, Q).
fn legacy_value_operand(kind: &Function, index: usize, arg: &Node) -> bool {
    (matches!(
        kind,
        Function::Sum
            | Function::Average
            | Function::Averagea
            | Function::Max
            | Function::Count
            | Function::Counta
            | Function::Large
            | Function::Median
            | Function::Min
            | Function::MinA
            | Function::MaxA
            | Function::Product
            | Function::Sumsq
            | Function::StDevS
            | Function::StDevP
            | Function::VarS
            | Function::VarP
            | Function::Devsq
            | Function::Avedev
            | Function::Geomean
            | Function::Harmean
            | Function::Small
            | Function::PercentileInc
            | Function::QuartileInc
            | Function::Kurt
            | Function::Skew
            | Function::Gcd
            | Function::Lcm
            | Function::Concat
            | Function::Textjoin
            | Function::Stdev
            | Function::StDevPCompat
            | Function::VarCompat
            | Function::VarPCompat
            | Function::Stdeva
            | Function::Stdevpa
            | Function::VarA
            | Function::VarpA
            | Function::PercentileCompat
            | Function::QuartileCompat
            | Function::PercentileExc
            | Function::QuartileExc
            | Function::PercentrankCompat
            | Function::PercentrankInc
            // CHOOSE's value arms: `CHOOSE(1,E1:E2+1)` shows `CHOOSE(1,@E1:E2+1)`, the `@` on the
            // arm's range operand rather than on the call (probes/legacy-one-cell-shape-intersection
            // H3, P3, excel-graph, T-1633).
            | Function::Choose
    ) || lifts_arrays_at(kind, index)
        || is_lookup_table(kind, index))
        && is_value_operation(arg)
}

/// Whether `index` is MATCH's `lookup_array` or the table of VLOOKUP or HLOOKUP, which a legacy
/// formula reads as a value when it is an operation: `=MATCH(1,(A1:A6="b")*(B1:B6=2),0)` shows
/// as `=MATCH(1,(@A1:A6="b")*(@B1:B6=2),0)` and is `#VALUE!` off the ranges' rows, and on them
/// looks up in one value (probes/legacy-match-lookup-array-intersection, excel-graph, T-1862;
/// corpus-bench/49300). XMATCH's `lookup_array` stays an array.
fn is_lookup_table(kind: &Function, index: usize) -> bool {
    matches!(
        kind,
        Function::Match | Function::Vlookup | Function::Hlookup
    ) && index == 1
}

/// Whether `node` is arithmetic, concatenation or a comparison, possibly in parentheses:
/// `SUM((B1:B6*1))` intersects like `SUM(B1:B6*1)`, unlike `SUM((B1:B6))`
/// (probes/legacy-value-argument-intersection-b Z2, Z10; T-1551).
fn is_value_operation(node: &Node) -> bool {
    match node {
        Node::Group { child, .. } => is_value_operation(child),
        Node::UnaryKind { .. }
        | Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => true,
        _ => false,
    }
}

/// Whether a legacy formula leaves the `@` off the call `kind(args)` although it may return a
/// range or an array. N or T over a reference it reads whole or over a call it leaves whole
/// ([`leaves_call_whole`]): `=N(IF(1,{2;3}))` shows no `@` at all. INDEX of a CHOOSE whose arms
/// are all arithmetic reads a value: `=INDEX(CHOOSE(1,B1:B6*1),3)` shows as
/// `=INDEX(CHOOSE(1,@B1:B6*1),3)` (probes/legacy-choose-arm-aggregate-and-array-constant-at H3,
/// excel-graph, T-1643). Only the `@` placement changes: a dynamic formula
/// of that shape still spills.
pub(super) fn legacy_call_reads_a_value(kind: &Function, args: &[Node]) -> bool {
    args.first()
        .is_some_and(|arg| reads_reference_whole(kind, arg) || leaves_call_whole(kind, arg))
        || matches!(kind, Function::Index)
            && matches!(
                args.first().map(Node::ungrouped),
                Some(Node::FunctionKind {
                    kind: Function::Choose,
                    args: arms,
                }) if arms.len() > 1 && arms[1..].iter().all(is_value_operation)
            )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::mark_legacy_reading;
    use crate::expressions::parser::stringify::to_rc_format;
    use crate::expressions::parser::{new_parser_english, Node};
    use crate::expressions::types::CellReferenceRC;
    use crate::functions::Function;

    /// `formula` parsed in Sheet1!A1, with the automatic `@` the rewrite puts on a one-element
    /// array constant wrapped around argument `arm` of the call that is N's argument.
    fn with_automatic_at(formula: &str, arm: usize) -> Node {
        let context = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };
        let mut parser = new_parser_english(vec!["Sheet1".to_string()], vec![], HashMap::new());
        let mut node = parser.parse(formula, &context);
        let Node::FunctionKind { args, .. } = &mut node else {
            panic!("{formula}");
        };
        let Some(Node::FunctionKind {
            args: call_args, ..
        }) = args.first_mut()
        else {
            panic!("{formula}");
        };
        let constant = std::mem::replace(&mut call_args[arm], Node::EmptyArgKind);
        call_args[arm] = Node::automatic_at(constant);
        node
    }

    /// Runs the marker step on the N call `node`, returning whether it was marked.
    fn mark(node: &mut Node) -> bool {
        let Node::FunctionKind { kind, args } = node else {
            panic!("{node:?}");
        };
        mark_legacy_reading(kind, args);
        *kind == Function::NLegacy
    }

    // The restore branch: a one-element `@` is taken off to decide, and put back when the call is
    // not marked. No formula text reaches it, because the rewrite puts the `@` on the call
    // itself, so the step returns early (`N(@CHOOSE(1,{2},Z1))`). The tree is built by hand.
    #[test]
    fn an_unmarked_call_keeps_its_one_element_at() {
        let mut node = with_automatic_at("N(CHOOSE(1,{2},Z1))", 1);
        let stored = to_rc_format(&node);
        assert_eq!(stored, "N(CHOOSE(1,@{2},R[0]C[25]))");
        assert!(!mark(&mut node));
        assert_eq!(to_rc_format(&node), stored);
        assert!(matches!(
            &node,
            Node::FunctionKind { args, .. } if matches!(
                &args[0],
                Node::FunctionKind { args: arms, .. }
                    if matches!(arms[1], Node::ImplicitIntersection { automatic: true, .. })
            )
        ));
    }

    // The marked branch drops the `@` for good (`=N(IF(1,{2}))`, probes/legacy-n-t-reference-args
    // N20).
    #[test]
    fn a_marked_call_drops_its_one_element_at() {
        let mut node = with_automatic_at("N(IF(1,{2}))", 1);
        assert!(mark(&mut node));
        assert_eq!(to_rc_format(&node), "_RS_LEGACY_N(IF(1,{2}))");
    }
}
