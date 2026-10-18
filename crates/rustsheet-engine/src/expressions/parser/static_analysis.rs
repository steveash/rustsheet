use super::reference_operators::{at_outside, wrong_range_shape};
use super::{for_each_chain_operand_mut, left_chain, value_operands, Node};
use crate::functions::lift::lifting_table_functions;
use crate::functions::Function;
use broadcast::broadcast_shapes;
use legacy::{
    add_argument_intersection, add_array_at, add_in_array_context, is_array_context,
    legacy_array_call_at, legacy_array_context, legacy_call_reads_a_value,
};
pub use redundant::remove_redundant_implicit_intersection;
use redundant::remove_redundant_legacy_intersection;
mod aggregate;
mod broadcast;
mod legacy;
mod lifted;
mod redundant;
mod reference_parameter;
mod reference_result;
mod row_column;
mod signature;
use signature::{
    args_signature_arraytotext, args_signature_bycol_byrow, args_signature_cell,
    args_signature_choose, args_signature_choosecols, args_signature_drop, args_signature_expand,
    args_signature_fieldvalue, args_signature_filter, args_signature_getpivotdata,
    args_signature_groupby, args_signature_hlookup, args_signature_ifs, args_signature_index,
    args_signature_irr, args_signature_lambda, args_signature_let, args_signature_lookup,
    args_signature_map, args_signature_match, args_signature_mirr, args_signature_npv,
    args_signature_offset, args_signature_one_vector, args_signature_pivotby,
    args_signature_randarray, args_signature_reduce, args_signature_row, args_signature_scalars,
    args_signature_scan, args_signature_sequence, args_signature_sort, args_signature_sortby,
    args_signature_sumif, args_signature_take, args_signature_textafter, args_signature_textjoin,
    args_signature_textsplit, args_signature_tocol, args_signature_trimrange,
    args_signature_unique, args_signature_with_holidays, args_signature_wrapcols,
    args_signature_xirr, args_signature_xlookup, args_signature_xmatch, args_signature_xnpv,
    Signature,
};

#[cfg(target_arch = "wasm32")]
use regex_lite as regex;

use regex::Regex;
use std::sync::OnceLock;

static RANGE_REFERENCE_REGEX: OnceLock<Regex> = OnceLock::new();

#[allow(clippy::expect_used)]
fn get_re() -> &'static Regex {
    RANGE_REFERENCE_REGEX
        .get_or_init(|| Regex::new(r":[A-Z]*[0-9]*$").expect("Regex is known to be valid"))
}

fn is_range_reference(s: &str) -> bool {
    get_re().is_match(s)
}

/*

# NOTES on the Implicit Intersection operator: @

 Sometimes we obtain a range where we expected a single argument. This can happen:

 * As an argument of a function, eg: `SIN(A1:A5)`
 * As the result of a computation of a formula `=A1:A5`

 In previous versions of the Friendly Giant the spreadsheet engine would perform an operation called _implicit intersection_
 that tries to find a single cell within the range. It works by picking a cell in the range that is the same row or the same column
 as the cell. If there is just one we return that otherwise we return the `#REF!` error.

 Examples:

 * Siting on `C3` the formula `=D1:D5` will return `D3`
 * Sitting on `C3` the formula `=D:D` will return `D3`
 * Sitting on `C3` the formula `=A1:A7` will return `A3`
 * Sitting on `C3` the formula `=A5:A8` will return `#REF!`
 * Sitting on `C3` the formula `D1:G7` will return `#REF!`

 Today's version of the engine will result in a dynamic array spilling the result through several cells.
 To force the old behaviour we can use the _implicit intersection operator_: @

 * `=@A1:A7` or `=SIN(@A1:A7)

 When parsing formulas that come form old workbooks this is done automatically.
 We call this version of the II operator the _automatic_ II operator.

 We can also insert the II operator in places where before was impossible:

 * `=SUM(@A1:A7)`

 This formulas will not be compatible with old versions of the engine. The FG will stringify this as `=SUM(_xlfn.SIMPLE(A1:A7))`.
 */

/// Traverses the formula tree adding the implicit intersection operator in all arguments of functions that
/// expect a scalar but get a range.
///  * A:A => @A:A
///  * SIN(A1:D1) => SIN(@A1:D1)
///
/// Assumes formula return a scalar
pub fn add_implicit_intersection(node: &mut Node, add: bool) {
    add_legacy_intersection(node, add, add);
}

/// [`add_implicit_intersection`] with `arrays`: whether `node` is in a value position of the cell
/// (every enclosing argument a scalar one, [`legacy_array_context`]), where a legacy formula also
/// puts the `@` on an array-valued call or array constant: `=@IF(1,{2;3})`, `=N(@{2;3})`, while
/// `=SUM(N({1,2}))` is 3 with no `@` (docs/excel-semantics/references.md "Legacy formulas put `@`
/// on array-valued calls and array constants").
pub(super) fn add_legacy_intersection(node: &mut Node, add: bool, arrays: bool) {
    match node {
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::EmptyArgKind
        | Node::ParseErrorKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::NamedFunctionKind { .. }
        | Node::ArrayKind(_)
        | Node::ReferenceKind { .. }
        | Node::OpUnionKind { .. }
        | Node::LambdaDefKind { .. }
        | Node::LambdaCallKind { .. } => {}
        Node::ImplicitIntersection { child, .. } => {
            // We need to check whether the II can be automatic or not
            let mut new_node = child.as_ref().clone();
            add_legacy_intersection(&mut new_node, add, arrays);
            if matches!(&new_node, Node::ImplicitIntersection { .. }) {
                *node = new_node;
            }
        }
        Node::SpillRangeOperator { child } => {
            // A spill anchor is never intersected: `INDEX(D1:D2,1)#` (T-1109, probes K1).
            add_legacy_intersection(child, false, false);
        }
        Node::RangeKind {
            row1,
            column1,
            row2,
            column2,
            sheet_name,
            sheet_index,
            absolute_row1,
            absolute_column1,
            absolute_row2,
            absolute_column2,
        } => {
            if add {
                *node = Node::automatic_at(Node::RangeKind {
                    sheet_name: sheet_name.clone(),
                    sheet_index: *sheet_index,
                    absolute_row1: *absolute_row1,
                    absolute_column1: *absolute_column1,
                    row1: *row1,
                    column1: *column1,
                    absolute_row2: *absolute_row2,
                    absolute_column2: *absolute_column2,
                    row2: *row2,
                    column2: *column2,
                });
            }
        }
        Node::Group { .. } if add && at_outside(node) => *node = Node::automatic_at(node.clone()),
        // A range of another workbook too (fixtures/external-link-cached-ranges Refs!B44, T-1526).
        Node::OpRangeKind { .. } | Node::OpIntersectKind { .. } | Node::WrongRangeKind { .. } => {
            if add {
                *node = Node::automatic_at(node.clone());
            }
        }

        // operations
        Node::UnaryKind { right: child, .. } | Node::Group { child, .. } => {
            add_legacy_intersection(child, add, arrays);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| {
                add_legacy_intersection(operand, add, arrays);
            });
        }

        Node::DefinedNameKind(v) => {
            if add {
                // Not all defined names deserve the II operator
                // For instance =Sheet1!A1 doesn't need to be intersected
                if is_range_reference(&v.2) {
                    *node = Node::automatic_at(Node::DefinedNameKind(v.to_owned()));
                }
            }
        }
        Node::NamedVariableKind { name, id } => {
            if add {
                *node = Node::automatic_at(Node::NamedVariableKind {
                    name: name.clone(),
                    id: *id,
                });
            }
        }
        Node::FunctionKind { kind, args } if is_array_context(kind) => add_in_array_context(args),
        Node::FunctionKind { kind, args } => {
            let signature = get_function_args_signature(kind, args.len());
            for (index, arg) in args.iter_mut().enumerate() {
                // Fixed-length signatures (D-functions, SMALL, ...) can be shorter than `args`; an
                // extra argument is not scalar (indexing it panicked; found by fuzzing, T-0021).
                let arrays = legacy_array_context(kind, signature.get(index), arg, arrays);
                add_array_at(kind, signature.get(index), index, arg, arrays);
                add_argument_intersection(kind, signature.get(index), index, arg, arrays);
            }
            // There are some function that will never add an automatic II:
            let new_function = matches!(kind, Function::Let | Function::Lambda);
            if !new_function
                && (add || arrays)
                && !legacy_call_reads_a_value(kind, args)
                && match static_analysis_on_function(kind, args) {
                    StaticResult::Range(_, _) | StaticResult::Unknown => add,
                    // A one-cell array gives the same value with or without the `@`. Excel shows
                    // none on `=SUBTOTAL(9,CHOOSE(1,{5}))`, a 1x1 array shape here
                    // (probes/legacy-one-cell-shape-intersection J3); other one-cell calls are
                    // unprobed (T-1847). A one-cell CHOOSE is a range shape and keeps its `@`.
                    StaticResult::Array(rows, columns) => {
                        arrays
                            && (rows > 1 || columns > 1)
                            && legacy_array_call_at(kind, &signature, args)
                    }
                    StaticResult::Scalar => false,
                }
            {
                *node = Node::automatic_at(node.clone());
            }
        }
    };
}

/// The result of the static analysis of a node
pub enum StaticResult {
    // The result of the evaluation is a single value (number, string, boolean, error)
    Scalar,
    // The result of the evaluation is an array of (rows, columns), counts: `{1,2}` is Array(1, 2)
    Array(i32, i32),
    // The result is a range of (rows - 1, columns - 1), extents: a single cell is Range(0, 0) and
    // A1:B2 is Range(1, 1). Every producer keeps this; `StaticResult::counts` converts (T-1265).
    Range(i32, i32),
    // The result of the evaluation is unknown, we cannot guaranty it is a scalar, an array or a range
    Unknown,
    // TODO: What if one of the dimensions is known?
    // what if the dimensions are unknown but bounded?
}

impl StaticResult {
    /// The (rows, columns) count of an array or range result; `None` for a scalar or unknown one.
    fn counts(&self) -> Option<(i32, i32)> {
        match self {
            StaticResult::Array(rows, columns) => Some((*rows, *columns)),
            StaticResult::Range(rows, columns) => Some((rows + 1, columns + 1)),
            StaticResult::Scalar | StaticResult::Unknown => None,
        }
    }
}

/// The shape of a chain of value operators, broadcast link by link in a loop (T-0128).
fn static_analysis_operator_chain(node: &Node) -> StaticResult {
    let (links, first) = left_chain(node);
    let mut result = run_static_analysis_on_node(first);
    for link in links {
        if let Some((_, right)) = value_operands(link) {
            result = static_analysis_op_results(result, run_static_analysis_on_node(right));
        }
    }
    result
}

/// An operator over a range gives an array of the range's counts.
fn static_analysis_op_results(lhs: StaticResult, rhs: StaticResult) -> StaticResult {
    match (lhs, rhs) {
        (StaticResult::Scalar, StaticResult::Scalar) => StaticResult::Scalar,
        (_, StaticResult::Unknown) | (StaticResult::Unknown, _) => StaticResult::Unknown,
        (lhs, rhs) => {
            // A scalar operand counts as 1x1.
            let (a1, b1) = lhs.counts().unwrap_or((1, 1));
            let (a2, b2) = rhs.counts().unwrap_or((1, 1));
            StaticResult::Array(a1.max(a2), b1.max(b2))
        }
    }
}

// Returns:
//  * Scalar if we can proof the result of the evaluation is a scalar
//  * Array(a, b) if we know it will be an a x b array.
//  * Range(a, b) if we know it will be an (a + 1) x (b + 1) range (extents, see StaticResult).
//  * Unknown if we cannot guaranty either
pub(crate) fn run_static_analysis_on_node(node: &Node) -> StaticResult {
    match node {
        // A parse error and a wrong reference could be Unknown too (or Array for a range).
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::EmptyArgKind => StaticResult::Scalar,
        Node::WrongRangeKind { .. } => wrong_range_shape(node), // T-1454
        Node::UnaryKind { right: child, .. } | Node::Group { child, .. } => {
            run_static_analysis_on_node(child)
        }
        Node::NamedFunctionKind { .. } => {
            // A named-function call invokes a LAMBDA (defined-name or LET-bound) whose
            // result shape is unknown at parse time — it may well be an array that needs
            // to spill (e.g. a LAMBDA wrapping SEQUENCE). Treat it like LambdaCallKind.
            StaticResult::Unknown
        }
        Node::ArrayKind(array) => {
            let n = array.len() as i32;
            let m = array.first().map(|row| row.len() as i32).unwrap_or(0);
            StaticResult::Array(n, m)
        }
        Node::RangeKind {
            row1,
            column1,
            row2,
            column2,
            ..
        } => StaticResult::Range(row2 - row1, column2 - column1),
        Node::OpRangeKind { .. } | Node::OpIntersectKind { .. } | Node::OpUnionKind { .. } => {
            StaticResult::Unknown
        }
        // A 3-D reference is a value only as an error (`#REF!`), so it never spills.
        Node::ReferenceKind { .. } | Node::SheetSpanKind { .. } => StaticResult::Scalar,

        // binary operations
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => static_analysis_operator_chain(node),

        // Defined names: their shape is not tracked.
        Node::DefinedNameKind(_) => StaticResult::Unknown,
        Node::NamedVariableKind { .. } | Node::ImplicitIntersection { .. } => StaticResult::Scalar,
        Node::FunctionKind { kind, args } => static_analysis_on_function(kind, args),
        Node::SpillRangeOperator { .. } => StaticResult::Unknown,
        Node::LambdaDefKind { .. } | Node::LambdaCallKind { .. } => StaticResult::Unknown,
    }
}

// If all the arguments are scalars the function will return a scalar
// If any of the arguments is a range or an array it will return an array
fn scalar_arguments(args: &[Node]) -> StaticResult {
    broadcast_static_result(args)
}

/// The shape of broadcasting `args` element-wise: scalar if all are, else an array of the largest
/// counts. A one-cell range (`E1:E1`, or CHOOSE of a 1x1 array) is a 1x1 array, not a scalar:
/// Excel puts a legacy `@` on it as on any range, `=ABS(@E1:E1)`, `=ABS(@CHOOSE(1,{5}))` and
/// `=ABS(@E1:E1+1)` (probes/legacy-one-cell-shape-intersection, T-1265).
fn broadcast_static_result<'a>(args: impl IntoIterator<Item = &'a Node>) -> StaticResult {
    broadcast_shapes(args.into_iter().map(run_static_analysis_on_node))
}

// We only care if the function can return a range or not
fn not_implemented(_args: &[Node]) -> StaticResult {
    StaticResult::Scalar
}

/// The *IF(S) functions spill according to the shapes of their criteria arguments
/// (`args[first]`, `args[first + 2]`, ...), broadcast against each other; the criteria and
/// value ranges are consumed, not broadcast (probes/ifs-array-criteria).
fn criteria_static_result(args: &[Node], first: usize) -> StaticResult {
    broadcast_static_result(args.iter().skip(first).step_by(2))
}

// The signature of the functions should be defined only once

// Given a function and a number of arguments this returns the arguments at each position
// are expected to be scalars or vectors (array/ranges).
// Sets signature::Error to all arguments if the number of arguments is incorrect.
// The lifting table decides first (SPEC-0011 "The table wins").
fn get_function_args_signature(kind: &Function, arg_count: usize) -> Vec<Signature> {
    match kind.arg_kinds() {
        Some(kinds) => lifted::lifted_signature(kinds, arg_count),
        None => fallback_signature(kind, arg_count).unwrap_or_default(),
    }
}

/// The hand-written signatures of the functions not in the lifting table; `None` for one in it.
fn fallback_signature(kind: &Function, arg_count: usize) -> Option<Vec<Signature>> {
    Some(match kind {
        Function::And => vec![Signature::Vector; arg_count],
        Function::False => args_signature_scalars(arg_count, 0, 0),
        Function::If => args_signature_scalars(arg_count, 2, 1),
        Function::Iferror => args_signature_scalars(arg_count, 2, 0),
        Function::Ifna => args_signature_scalars(arg_count, 2, 0),
        Function::Ifs => vec![Signature::Scalar; arg_count],
        Function::Lambda => args_signature_lambda(arg_count),
        Function::Let => args_signature_let(arg_count),
        Function::Not => args_signature_scalars(arg_count, 1, 0),
        Function::Or => vec![Signature::Vector; arg_count],
        Function::Switch => vec![Signature::Scalar; arg_count],
        Function::True => args_signature_scalars(arg_count, 0, 0),
        Function::Xor => vec![Signature::Vector; arg_count],
        Function::Abs => args_signature_scalars(arg_count, 1, 0),
        Function::Acos => args_signature_scalars(arg_count, 1, 0),
        Function::Acosh => args_signature_scalars(arg_count, 1, 0),
        Function::Asin => args_signature_scalars(arg_count, 1, 0),
        Function::Asinh => args_signature_scalars(arg_count, 1, 0),
        Function::Atan => args_signature_scalars(arg_count, 1, 0),
        Function::Atan2 => args_signature_scalars(arg_count, 2, 0),
        Function::Atanh => args_signature_scalars(arg_count, 1, 0),
        Function::Choose => args_signature_choose(arg_count),
        Function::Column => args_signature_row(arg_count),
        Function::Columns => args_signature_one_vector(arg_count),
        Function::Ln => args_signature_scalars(arg_count, 1, 0),
        Function::Log => args_signature_scalars(arg_count, 1, 1),
        Function::Log10 => args_signature_scalars(arg_count, 1, 0),
        Function::Cos => args_signature_scalars(arg_count, 1, 0),
        Function::Cosh => args_signature_scalars(arg_count, 1, 0),
        Function::Max => vec![Signature::Vector; arg_count],
        Function::Min => vec![Signature::Vector; arg_count],
        Function::Pi => args_signature_scalars(arg_count, 0, 0),
        Function::Power => args_signature_scalars(arg_count, 2, 0),
        Function::Product => vec![Signature::Vector; arg_count],
        Function::Round => args_signature_scalars(arg_count, 2, 0),
        Function::Rounddown => args_signature_scalars(arg_count, 2, 0),
        Function::Roundup => args_signature_scalars(arg_count, 2, 0),
        Function::Sin => args_signature_scalars(arg_count, 1, 0),
        Function::Sinh => args_signature_scalars(arg_count, 1, 0),
        Function::Sqrt => args_signature_scalars(arg_count, 1, 0),
        Function::Sqrtpi => args_signature_scalars(arg_count, 1, 0),
        Function::Sum => vec![Signature::Vector; arg_count],
        Function::Sumif => args_signature_sumif(arg_count),
        Function::Sumifs => args_signature_ifs(arg_count, 2),
        Function::Tan => args_signature_scalars(arg_count, 1, 0),
        Function::Tanh => args_signature_scalars(arg_count, 1, 0),
        Function::Isformula => args_signature_scalars(arg_count, 1, 0),
        Function::Isomitted => args_signature_scalars(arg_count, 1, 0),
        Function::Isref => args_signature_one_vector(arg_count),
        Function::Na => args_signature_scalars(arg_count, 0, 0),
        // A reference argument (a range, a table's data area: `SHEET(Table1[])`) gives its sheet's
        // number and is not implicitly intersected (probes/info-logic-argument-modes A12; legacy formulas:
        // calc_test_no_export/tables.xlsx sheet9 A5:A8).
        Function::Sheet if arg_count == 1 => args_signature_one_vector(arg_count),
        Function::Sheet => args_signature_scalars(arg_count, 0, 1),
        Function::Type => args_signature_one_vector(arg_count),
        Function::Areas | Function::Phonetic => args_signature_one_vector(arg_count),
        Function::Choosecols => args_signature_choosecols(arg_count),
        Function::Chooserows => args_signature_choosecols(arg_count),
        Function::Expand => args_signature_expand(arg_count),
        Function::Hlookup => args_signature_hlookup(arg_count),
        Function::Hstack => vec![Signature::Vector; arg_count],
        Function::Index => args_signature_index(arg_count),
        Function::Indirect => args_signature_scalars(arg_count, 1, 1),
        Function::Lookup => args_signature_lookup(arg_count),
        Function::Match => args_signature_match(arg_count),
        Function::Offset => args_signature_offset(arg_count),
        Function::Row => args_signature_row(arg_count),
        Function::Rows => args_signature_one_vector(arg_count),
        Function::Vlookup => args_signature_hlookup(arg_count),
        Function::Vstack => vec![Signature::Vector; arg_count],
        Function::Wrapcols => args_signature_wrapcols(arg_count),
        Function::Wraprows => args_signature_wrapcols(arg_count),
        Function::Xlookup => args_signature_xlookup(arg_count),
        Function::Xmatch => args_signature_xmatch(arg_count),
        Function::Trimrange => args_signature_trimrange(arg_count),
        Function::Sort => args_signature_sort(arg_count),
        Function::Sortby => args_signature_sortby(arg_count),
        Function::Unique => args_signature_unique(arg_count),
        Function::Filter => args_signature_filter(arg_count),
        Function::Take => args_signature_take(arg_count),
        Function::Drop => args_signature_drop(arg_count),
        Function::Tocol => args_signature_tocol(arg_count),
        Function::Torow => args_signature_tocol(arg_count),
        Function::Transpose => args_signature_one_vector(arg_count),
        Function::Mmult => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::Bycol | Function::Byrow => args_signature_bycol_byrow(arg_count),
        Function::Groupby => args_signature_groupby(arg_count),
        Function::Pivotby => args_signature_pivotby(arg_count),
        Function::Getpivotdata => args_signature_getpivotdata(arg_count),
        Function::Map => args_signature_map(arg_count),
        Function::Reduce => args_signature_reduce(arg_count),
        Function::Scan => args_signature_scan(arg_count),
        Function::Makearray => vec![Signature::Scalar; 3],
        Function::Sequence => args_signature_sequence(arg_count),
        Function::Randarray => args_signature_randarray(arg_count),
        Function::Textsplit => args_signature_textsplit(arg_count),
        Function::Concat => vec![Signature::Vector; arg_count],
        Function::Concatenate => vec![Signature::Scalar; arg_count],
        Function::Exact => args_signature_scalars(arg_count, 2, 0),
        Function::Find => args_signature_scalars(arg_count, 2, 1),
        Function::Left => args_signature_scalars(arg_count, 1, 1),
        Function::Len => args_signature_scalars(arg_count, 1, 0),
        Function::Lower => args_signature_scalars(arg_count, 1, 0),
        Function::Mid => args_signature_scalars(arg_count, 3, 0),
        Function::Rept => args_signature_scalars(arg_count, 2, 0),
        Function::Right => args_signature_scalars(arg_count, 2, 1),
        Function::Search => args_signature_scalars(arg_count, 2, 1),
        Function::Substitute => args_signature_scalars(arg_count, 3, 1),
        Function::Regexextract => args_signature_scalars(arg_count, 2, 1),
        Function::Regexreplace => args_signature_scalars(arg_count, 3, 0),
        Function::Regextest => args_signature_scalars(arg_count, 2, 0),
        Function::T => args_signature_scalars(arg_count, 1, 0),
        Function::Text => args_signature_scalars(arg_count, 2, 0),
        Function::Textafter => args_signature_textafter(arg_count),
        Function::Textbefore => args_signature_textafter(arg_count),
        Function::Textjoin => args_signature_textjoin(arg_count),
        Function::Trim => args_signature_scalars(arg_count, 1, 0),
        Function::Upper => args_signature_scalars(arg_count, 1, 0),
        Function::Value => args_signature_scalars(arg_count, 1, 0),
        Function::Valuetotext => args_signature_scalars(arg_count, 1, 1),
        Function::Average => vec![Signature::Vector; arg_count],
        Function::Avedev => vec![Signature::Vector; arg_count],
        Function::Averagea => vec![Signature::Vector; arg_count],
        Function::Averageif => args_signature_sumif(arg_count),
        Function::Averageifs => args_signature_ifs(arg_count, 2),
        Function::Count => vec![Signature::Vector; arg_count],
        Function::Counta => vec![Signature::Vector; arg_count],
        Function::Countblank => vec![Signature::Vector; arg_count],
        Function::Countif => args_signature_sumif(arg_count),
        Function::Countifs => args_signature_ifs(arg_count, 1),
        Function::Maxifs => args_signature_ifs(arg_count, 2),
        Function::Minifs => args_signature_ifs(arg_count, 2),
        Function::Date => args_signature_scalars(arg_count, 3, 0),
        Function::Datedif => args_signature_scalars(arg_count, 3, 0),
        Function::Datevalue => args_signature_scalars(arg_count, 1, 0),
        Function::Day => args_signature_scalars(arg_count, 1, 0),
        Function::Month => args_signature_scalars(arg_count, 1, 0),
        Function::Time => args_signature_scalars(arg_count, 3, 0),
        Function::Timevalue => args_signature_scalars(arg_count, 1, 0),
        Function::Hour => args_signature_scalars(arg_count, 1, 0),
        Function::Minute => args_signature_scalars(arg_count, 1, 0),
        Function::Second => args_signature_scalars(arg_count, 1, 0),
        Function::Now => args_signature_scalars(arg_count, 0, 0),
        Function::Today => args_signature_scalars(arg_count, 0, 0),
        Function::Year => args_signature_scalars(arg_count, 1, 0),
        Function::Days => args_signature_scalars(arg_count, 2, 0),
        Function::Days360 => args_signature_scalars(arg_count, 2, 1),
        Function::Weekday => args_signature_scalars(arg_count, 1, 1),
        Function::Workday => args_signature_with_holidays(arg_count, 2, 0),
        Function::WorkdayIntl => args_signature_with_holidays(arg_count, 2, 1),
        Function::Isoweeknum => args_signature_scalars(arg_count, 1, 0),
        Function::Fvschedule => vec![Signature::Scalar, Signature::Vector],
        Function::Cumipmt => args_signature_scalars(arg_count, 6, 0),
        Function::Cumprinc => args_signature_scalars(arg_count, 6, 0),
        Function::Db => args_signature_scalars(arg_count, 4, 1),
        Function::Irr => args_signature_irr(arg_count),
        Function::Mirr => args_signature_mirr(arg_count),
        Function::Npv => args_signature_npv(arg_count),
        Function::Xirr => args_signature_xirr(arg_count),
        Function::Xnpv => args_signature_xnpv(arg_count),
        Function::Yield => args_signature_scalars(arg_count, 6, 1),
        Function::Oddfprice => args_signature_scalars(arg_count, 8, 1),
        Function::Oddfyield => args_signature_scalars(arg_count, 8, 1),
        Function::Oddlprice => args_signature_scalars(arg_count, 7, 1),
        Function::Oddlyield => args_signature_scalars(arg_count, 7, 1),
        Function::Amordegrc => args_signature_scalars(arg_count, 6, 1),
        Function::Amorlinc => args_signature_scalars(arg_count, 6, 1),
        Function::Vdb => args_signature_scalars(arg_count, 5, 2),
        Function::Subtotal | Function::Aggregate => args_signature_npv(arg_count),
        Function::Rand => args_signature_scalars(arg_count, 0, 0),
        Function::Randbetween => args_signature_scalars(arg_count, 2, 0),
        Function::Formulatext => args_signature_scalars(arg_count, 1, 0),
        Function::Unicode => args_signature_scalars(arg_count, 1, 0),
        Function::Unichar => args_signature_scalars(arg_count, 1, 0),
        Function::Char => args_signature_scalars(arg_count, 1, 0),
        Function::Clean => args_signature_scalars(arg_count, 1, 0),
        Function::Code | Function::Bahttext => args_signature_scalars(arg_count, 1, 0),
        Function::Asc | Function::Dbcs => args_signature_scalars(arg_count, 1, 0),
        Function::Arraytotext => args_signature_arraytotext(arg_count),
        Function::Dollar => args_signature_scalars(arg_count, 1, 1),
        Function::Image => args_signature_scalars(arg_count, 1, 4),
        Function::Stockhistory => args_signature_scalars(arg_count, 2, 9),
        Function::Fieldvalue => args_signature_fieldvalue(arg_count),
        Function::Filterxml => args_signature_scalars(arg_count, 2, 0),
        Function::Findb => args_signature_scalars(arg_count, 2, 1),
        Function::Fixed => args_signature_scalars(arg_count, 1, 2),
        Function::Leftb => args_signature_scalars(arg_count, 1, 1),
        Function::Lenb => args_signature_scalars(arg_count, 1, 0),
        Function::Midb => args_signature_scalars(arg_count, 3, 0),
        Function::Numbervalue => args_signature_scalars(arg_count, 1, 2),
        Function::Proper => args_signature_scalars(arg_count, 1, 0),
        Function::Replace => args_signature_scalars(arg_count, 4, 0),
        Function::Replaceb => args_signature_scalars(arg_count, 4, 0),
        Function::Rightb => args_signature_scalars(arg_count, 1, 1),
        Function::Searchb => args_signature_scalars(arg_count, 2, 1),
        Function::Geomean => vec![Signature::Vector; arg_count],
        Function::Networkdays => args_signature_with_holidays(arg_count, 2, 0),
        Function::NetworkdaysIntl => args_signature_with_holidays(arg_count, 2, 1),
        Function::Acot => args_signature_scalars(arg_count, 1, 0),
        Function::Acoth => args_signature_scalars(arg_count, 1, 0),
        Function::Cot => args_signature_scalars(arg_count, 1, 0),
        Function::Coth => args_signature_scalars(arg_count, 1, 0),
        Function::Csc => args_signature_scalars(arg_count, 1, 0),
        Function::Csch => args_signature_scalars(arg_count, 1, 0),
        Function::Sec => args_signature_scalars(arg_count, 1, 0),
        Function::Sech => args_signature_scalars(arg_count, 1, 0),
        Function::Exp => args_signature_scalars(arg_count, 1, 0),
        Function::Fact => args_signature_scalars(arg_count, 1, 0),
        Function::Sign => args_signature_scalars(arg_count, 1, 0),
        Function::Radians => args_signature_scalars(arg_count, 1, 0),
        Function::Degrees => args_signature_scalars(arg_count, 1, 0),
        Function::Int => args_signature_scalars(arg_count, 1, 0),
        Function::Even => args_signature_scalars(arg_count, 1, 0),
        Function::Odd => args_signature_scalars(arg_count, 1, 0),
        Function::Ceiling | Function::EcmaCeiling => args_signature_scalars(arg_count, 2, 0),
        Function::CeilingMath => args_signature_scalars(arg_count, 1, 2),
        Function::CeilingPrecise => args_signature_scalars(arg_count, 1, 1),
        Function::Floor => args_signature_scalars(arg_count, 2, 0),
        Function::FloorMath => args_signature_scalars(arg_count, 1, 2),
        Function::FloorPrecise => args_signature_scalars(arg_count, 1, 1),
        Function::IsoCeiling => args_signature_scalars(arg_count, 1, 1),
        Function::Mod => args_signature_scalars(arg_count, 2, 0),
        Function::Mround => args_signature_scalars(arg_count, 2, 0),
        Function::Trunc => args_signature_scalars(arg_count, 1, 1),
        Function::Gcd => vec![Signature::Vector; arg_count],
        Function::Lcm => vec![Signature::Vector; arg_count],
        Function::Base => args_signature_scalars(arg_count, 2, 1),
        Function::Decimal => args_signature_scalars(arg_count, 2, 0),
        Function::Roman => args_signature_scalars(arg_count, 1, 1),
        Function::Arabic => args_signature_scalars(arg_count, 1, 0),
        Function::Combin => args_signature_scalars(arg_count, 2, 0),
        Function::Combina => args_signature_scalars(arg_count, 2, 0),
        Function::Sumsq => vec![Signature::Vector; arg_count],
        Function::Mdeterm => args_signature_one_vector(arg_count),
        Function::Minverse => args_signature_one_vector(arg_count),
        Function::Munit => args_signature_scalars(arg_count, 1, 0),
        Function::Multinomial => vec![Signature::Vector; arg_count],
        Function::Sumproduct => vec![Signature::Vector; arg_count],
        Function::Percentof => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::N => args_signature_scalars(arg_count, 1, 0),
        Function::Sheets => args_signature_scalars(arg_count, 0, 1),
        Function::Cell => args_signature_cell(arg_count),
        Function::Info => args_signature_scalars(arg_count, 1, 1),
        Function::Daverage => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dcount => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dget => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dmax => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dmin => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dsum => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dcounta => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dproduct => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dstdev => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dvar => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dvarp => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::Dstdevp => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        Function::ChisqDist => args_signature_scalars(arg_count, 4, 0),
        Function::ChisqDistRT => args_signature_scalars(arg_count, 3, 0),
        Function::ChisqInv => args_signature_scalars(arg_count, 3, 0),
        Function::ChisqInvRT => args_signature_scalars(arg_count, 2, 0),
        Function::ChisqTest => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::CovarianceP => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::CovarianceS => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::Devsq => vec![Signature::Vector; arg_count],
        Function::FTest => vec![Signature::Vector; 2],
        Function::GammaDist => args_signature_scalars(arg_count, 4, 0),
        Function::GammaInv => args_signature_scalars(arg_count, 3, 0),
        Function::HypGeomDist => args_signature_scalars(arg_count, 5, 0),
        Function::Pearson => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::PoissonDist => args_signature_scalars(arg_count, 3, 0),
        Function::Stdev => vec![Signature::Vector; arg_count],
        Function::StDevP => vec![Signature::Vector; arg_count],
        Function::StDevS => vec![Signature::Vector; arg_count],
        Function::Stdeva => vec![Signature::Vector; arg_count],
        Function::Stdevpa => vec![Signature::Vector; arg_count],
        Function::TTest => {
            if arg_count == 4 {
                vec![
                    Signature::Vector,
                    Signature::Vector,
                    Signature::Scalar,
                    Signature::Scalar,
                ]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::VarP => vec![Signature::Vector; arg_count],
        Function::VarS => vec![Signature::Vector; arg_count],
        Function::VarpA => vec![Signature::Vector; arg_count],
        Function::VarA => vec![Signature::Vector; arg_count],
        Function::ZTest => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Scalar]
            } else if arg_count == 3 {
                vec![Signature::Vector, Signature::Scalar, Signature::Scalar]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        // Compatibility wrappers
        Function::HypGeomDistCompat => args_signature_scalars(arg_count, 4, 0),
        // Compatibility aliases
        Function::ChiDistCompat => args_signature_scalars(arg_count, 2, 0),
        Function::ChiInvCompat => args_signature_scalars(arg_count, 2, 0),
        Function::ChiTestCompat => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::CovarCompat => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::FTestCompat => vec![Signature::Vector; 2],
        Function::GammaDistCompat => args_signature_scalars(arg_count, 4, 0),
        Function::GammaInvCompat => args_signature_scalars(arg_count, 3, 0),
        Function::ModeCompat => vec![Signature::Vector; arg_count],
        Function::PoissonCompat => args_signature_scalars(arg_count, 3, 0),
        Function::RankCompat => vec![Signature::Scalar, Signature::Vector, Signature::Scalar],
        Function::StDevPCompat => vec![Signature::Vector; arg_count],
        Function::TTestCompat => {
            if arg_count == 4 {
                vec![
                    Signature::Vector,
                    Signature::Vector,
                    Signature::Scalar,
                    Signature::Scalar,
                ]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::VarCompat => vec![Signature::Vector; arg_count],
        Function::VarPCompat => vec![Signature::Vector; arg_count],
        Function::ZTestCompat => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Scalar]
            } else if arg_count == 3 {
                vec![Signature::Vector, Signature::Scalar, Signature::Scalar]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::Sumx2my2 => vec![Signature::Vector; 2],
        Function::Sumx2py2 => vec![Signature::Vector; 2],
        Function::Sumxmy2 => vec![Signature::Vector; 2],
        Function::Correl => vec![Signature::Vector; 2],
        Function::Rsq => vec![Signature::Vector; 2],
        Function::Intercept => vec![Signature::Vector; 2],
        Function::Slope => vec![Signature::Vector; 2],
        Function::Steyx => vec![Signature::Vector; 2],
        Function::Forecast | Function::ForecastLinear => {
            vec![Signature::Scalar, Signature::Vector, Signature::Vector]
        }
        Function::ForecastEts
        | Function::ForecastEtsConfint
        | Function::ForecastEtsSeasonality
        | Function::ForecastEtsStat => vec![Signature::Vector; arg_count],
        Function::Frequency => vec![Signature::Vector, Signature::Vector],
        Function::Growth | Function::Trend => match arg_count {
            1 => vec![Signature::Vector],
            2 => vec![Signature::Vector, Signature::Vector],
            3 => vec![Signature::Vector, Signature::Vector, Signature::Vector],
            4 => vec![
                Signature::Vector,
                Signature::Vector,
                Signature::Vector,
                Signature::Scalar,
            ],
            _ => vec![Signature::Error; arg_count],
        },
        Function::Linest | Function::Logest => match arg_count {
            1 => vec![Signature::Vector],
            2 => vec![Signature::Vector, Signature::Vector],
            3 => vec![Signature::Vector, Signature::Vector, Signature::Scalar],
            4 => vec![
                Signature::Vector,
                Signature::Vector,
                Signature::Scalar,
                Signature::Scalar,
            ],
            _ => vec![Signature::Error; arg_count],
        },
        Function::ModeMult | Function::ModeSingl => vec![Signature::Vector; arg_count],
        Function::Prob => match arg_count {
            3 => vec![Signature::Vector, Signature::Vector, Signature::Scalar],
            4 => vec![
                Signature::Vector,
                Signature::Vector,
                Signature::Scalar,
                Signature::Scalar,
            ],
            _ => vec![Signature::Error; arg_count],
        },
        Function::Harmean => vec![Signature::Vector; arg_count],
        Function::Kurt => vec![Signature::Vector; arg_count],
        Function::MaxA => vec![Signature::Vector; arg_count],
        Function::Median => vec![Signature::Vector; arg_count],
        Function::MinA => vec![Signature::Vector; arg_count],
        Function::RankAvg => vec![Signature::Scalar, Signature::Vector, Signature::Scalar],
        Function::RankEq => vec![Signature::Scalar, Signature::Vector, Signature::Scalar],
        Function::Skew => vec![Signature::Vector; arg_count],
        Function::SkewP => vec![Signature::Vector; arg_count],
        // In the lifting table (SPEC-0011): `lifted.rs` derives these.
        lifting_table_functions!() => return None,
    })
}

// Returns the type of the result (Scalar, Array or Range) depending on the arguments; the
// lifting table decides first (SPEC-0011 "The table wins").
fn static_analysis_on_function(kind: &Function, args: &[Node]) -> StaticResult {
    match kind.arg_kinds() {
        Some(kinds) => lifted::lifted_static_result(kinds, args),
        None => fallback_shape(kind, args).unwrap_or(StaticResult::Unknown),
    }
}

/// The hand-written shapes of the functions not in the lifting table; `None` for one in it.
fn fallback_shape(kind: &Function, args: &[Node]) -> Option<StaticResult> {
    Some(match kind {
        Function::And => StaticResult::Scalar,
        Function::False => StaticResult::Scalar,
        Function::If => scalar_arguments(args),
        Function::Iferror => scalar_arguments(args),
        Function::Ifna => scalar_arguments(args),
        Function::Ifs => StaticResult::Unknown, // chosen value spills: logic.md
        Function::Lambda => StaticResult::Unknown,
        Function::Let => StaticResult::Unknown,
        Function::Not => scalar_arguments(args),
        Function::Or => StaticResult::Scalar,
        Function::Switch => StaticResult::Unknown,
        Function::True => StaticResult::Scalar,
        Function::Xor => StaticResult::Scalar,
        Function::Abs => scalar_arguments(args),
        Function::Acos => scalar_arguments(args),
        Function::Acosh => scalar_arguments(args),
        Function::Asin => scalar_arguments(args),
        Function::Asinh => scalar_arguments(args),
        Function::Atan => scalar_arguments(args),
        Function::Atan2 => scalar_arguments(args),
        Function::Atanh => scalar_arguments(args),
        // CHOOSE is a reference function: with a range arm the call itself is
        // reference-shaped, so legacy import wraps the whole CALL in `@`
        // (Excel upgrades `=CHOOSE(2,A1:A10,B1:B10)` to `=@CHOOSE(...)`).
        // Its arms' counts become the extents of that range shape.
        Function::Choose => match scalar_arguments(args) {
            StaticResult::Array(n, m) => StaticResult::Range(n - 1, m - 1),
            other => other,
        },
        Function::Column => row_column::static_analysis_row_column(args, false),
        Function::Columns => not_implemented(args),
        Function::Cos => scalar_arguments(args),
        Function::Cosh => scalar_arguments(args),
        Function::Max => StaticResult::Scalar,
        Function::Min => StaticResult::Scalar,
        Function::Pi => StaticResult::Scalar,
        Function::Power => scalar_arguments(args),
        Function::Product => not_implemented(args),
        Function::Round => scalar_arguments(args),
        Function::Rounddown => scalar_arguments(args),
        Function::Roundup => scalar_arguments(args),
        Function::Ln => scalar_arguments(args),
        Function::Log => scalar_arguments(args),
        Function::Log10 => scalar_arguments(args),
        Function::Sin => scalar_arguments(args),
        Function::Sinh => scalar_arguments(args),
        Function::Sqrt => scalar_arguments(args),
        Function::Sqrtpi => scalar_arguments(args),
        Function::Sum => StaticResult::Scalar,
        Function::Sumif => criteria_static_result(args, 1),
        Function::Sumifs => criteria_static_result(args, 2),
        Function::Tan => scalar_arguments(args),
        Function::Tanh => scalar_arguments(args),
        Function::Isformula => scalar_arguments(args),
        Function::Isomitted => not_implemented(args),
        Function::Isref => not_implemented(args),
        Function::Na => StaticResult::Scalar,
        Function::Sheet => StaticResult::Scalar,
        Function::Type => not_implemented(args),
        Function::Areas => aggregate::areas_static_result(args),
        Function::Choosecols => StaticResult::Unknown,
        Function::Chooserows => StaticResult::Unknown,
        Function::Expand => StaticResult::Unknown,
        Function::Hlookup => aggregate::lookup_static_result(args), // T-0270, T-1269
        Function::Hstack => StaticResult::Unknown,
        // INDEX has two forms, but they are indistinguishable at parse time.
        Function::Index => StaticResult::Unknown,
        Function::Indirect => reference_result::static_analysis_indirect(args),
        // Spills by lookup_value; the vectors are consumed (T-1241).
        Function::Lookup => broadcast_static_result(args.first()),
        Function::Match => not_implemented(args),
        Function::Offset => reference_result::static_analysis_offset(args),
        Function::Row => row_column::static_analysis_row_column(args, true),
        Function::Rows => not_implemented(args),
        Function::Vlookup => aggregate::lookup_static_result(args), // T-0270, T-1269
        Function::Vstack => StaticResult::Unknown,
        Function::Wrapcols => StaticResult::Unknown,
        Function::Wraprows => StaticResult::Unknown,
        Function::Xlookup => StaticResult::Unknown,
        // Spills by lookup_value and match_mode, broadcast; lookup_array is consumed (T-1241).
        Function::Xmatch => broadcast_static_result(args.iter().step_by(2).take(2)),
        Function::Trimrange => StaticResult::Unknown,
        Function::Sort | Function::Sortby | Function::Unique => StaticResult::Unknown,
        Function::Filter | Function::Take | Function::Drop => StaticResult::Unknown,
        Function::Tocol | Function::Torow | Function::Transpose => StaticResult::Unknown,
        Function::Mmult => StaticResult::Unknown,
        Function::Bycol | Function::Byrow => StaticResult::Unknown,
        Function::Groupby | Function::Pivotby => StaticResult::Unknown,
        Function::Getpivotdata => StaticResult::Scalar,
        Function::Map | Function::Reduce => StaticResult::Unknown,
        Function::Scan | Function::Makearray => StaticResult::Unknown,
        Function::Sequence => StaticResult::Unknown,
        Function::Randarray => StaticResult::Unknown,
        Function::Textsplit => StaticResult::Unknown,
        Function::Concat => not_implemented(args),
        Function::Concatenate => scalar_arguments(args),
        Function::Exact => scalar_arguments(args),
        Function::Find => scalar_arguments(args),
        // LEFT/MID/RIGHT broadcast element-wise: an array/range argument yields an array.
        Function::Left => scalar_arguments(args),
        Function::Len => scalar_arguments(args),
        // UPPER/LOWER broadcast element-wise: an array/range argument yields an array.
        Function::Lower => scalar_arguments(args),
        Function::Mid => scalar_arguments(args),
        Function::Rept => scalar_arguments(args),
        Function::Right => scalar_arguments(args),
        Function::Search => scalar_arguments(args),
        Function::Substitute => scalar_arguments(args),
        // Their text and scalar arguments lift over a range or an array (T-1272); the table derives
        // their shape once they move onto `lift_call`. TODO(T-1775)
        Function::Regexextract | Function::Regexreplace | Function::Regextest => {
            StaticResult::Unknown
        }
        Function::T => scalar_arguments(args),
        Function::Text => scalar_arguments(args),
        Function::Textafter | Function::Textbefore => StaticResult::Unknown,
        Function::Textjoin => not_implemented(args),
        Function::Trim => scalar_arguments(args),
        Function::Unicode => not_implemented(args),
        Function::Unichar => scalar_arguments(args),
        Function::Char => not_implemented(args),
        Function::Clean => scalar_arguments(args),
        Function::Code | Function::Bahttext => scalar_arguments(args),
        Function::Asc | Function::Dbcs => scalar_arguments(args),
        Function::Phonetic => not_implemented(args),
        Function::Arraytotext => not_implemented(args),
        Function::Dollar => not_implemented(args),
        // All three lift over an array or range `source`/`stock`/`value` (T-1179).
        Function::Image | Function::Stockhistory | Function::Fieldvalue => StaticResult::Unknown,
        // Lifts over array arguments, `#VALUE!` per element (filterxml.rs).
        Function::Filterxml => scalar_arguments(args),
        // The *B functions delegate to their non-B counterparts (byte_functions.rs), so they lift
        // over ranges exactly as FIND/LEFT/LEN/... do (T-0271).
        Function::Findb => scalar_arguments(args),
        Function::Fixed => not_implemented(args),
        Function::Leftb => scalar_arguments(args),
        Function::Lenb => scalar_arguments(args),
        Function::Midb => scalar_arguments(args),
        Function::Numbervalue => not_implemented(args),
        Function::Proper => scalar_arguments(args),
        Function::Replace => scalar_arguments(args),
        Function::Replaceb => scalar_arguments(args),
        Function::Rightb => scalar_arguments(args),
        Function::Searchb => scalar_arguments(args),
        Function::Upper => scalar_arguments(args),
        // VALUE lifts element-wise (probes/value-argument-lift C1:C8, T-1791).
        Function::Value => scalar_arguments(args),
        Function::Valuetotext => scalar_arguments(args),
        Function::Average => not_implemented(args),
        Function::Averagea => not_implemented(args),
        Function::Avedev => not_implemented(args),
        Function::Averageif => criteria_static_result(args, 1),
        Function::Averageifs => criteria_static_result(args, 2),
        Function::Count => not_implemented(args),
        Function::Counta => not_implemented(args),
        Function::Countblank => reference_parameter::countblank_static_result(args),
        Function::Countif => criteria_static_result(args, 1),
        Function::Countifs => criteria_static_result(args, 1),
        Function::Maxifs => criteria_static_result(args, 2),
        Function::Minifs => criteria_static_result(args, 2),
        // DATE broadcasts element-wise: an array/range argument yields an array.
        Function::Date => scalar_arguments(args),
        Function::Datedif => scalar_arguments(&args[..args.len().min(2)]),
        Function::Datevalue => scalar_arguments(args),
        Function::Day => scalar_arguments(args),
        Function::Month => scalar_arguments(args),
        Function::Time => scalar_arguments(args),
        Function::Timevalue => scalar_arguments(args),
        Function::Hour => scalar_arguments(args),
        Function::Minute => scalar_arguments(args),
        Function::Second => scalar_arguments(args),
        Function::Now => not_implemented(args),
        Function::Today => not_implemented(args),
        Function::Year => scalar_arguments(args),
        Function::Days => scalar_arguments(args),
        Function::Days360 => scalar_arguments(&args[..args.len().min(2)]),
        Function::Weekday => scalar_arguments(args),
        Function::Workday => not_implemented(args),
        Function::WorkdayIntl => scalar_arguments(args.get(2..3).unwrap_or_default()),
        Function::Isoweeknum => scalar_arguments(args),
        Function::Fvschedule => StaticResult::Scalar,
        Function::Cumipmt => not_implemented(args),
        Function::Cumprinc => not_implemented(args),
        Function::Db => not_implemented(args),
        Function::Irr => not_implemented(args),
        Function::Mirr => not_implemented(args),
        Function::Npv => not_implemented(args),
        Function::Xirr => not_implemented(args),
        Function::Xnpv => not_implemented(args),
        Function::Yield => StaticResult::Scalar,
        Function::Oddfprice => StaticResult::Scalar,
        Function::Oddfyield => StaticResult::Scalar,
        Function::Oddlprice => StaticResult::Scalar,
        Function::Oddlyield => StaticResult::Scalar,
        Function::Amordegrc => StaticResult::Scalar,
        Function::Amorlinc => StaticResult::Scalar,
        Function::Vdb => StaticResult::Scalar,
        Function::Subtotal => aggregate::subtotal_static_result(args),
        Function::Aggregate => aggregate::aggregate_static_result(args),
        Function::Rand => not_implemented(args),
        Function::Randbetween => scalar_arguments(args),
        // One result per cell of a multi-cell reference (T-1241).
        Function::Formulatext => broadcast_static_result(args),
        Function::Geomean => not_implemented(args),
        Function::Networkdays => not_implemented(args),
        Function::NetworkdaysIntl => scalar_arguments(args.get(2..3).unwrap_or_default()),
        Function::Acot => scalar_arguments(args),
        Function::Acoth => scalar_arguments(args),
        Function::Cot => scalar_arguments(args),
        Function::Coth => scalar_arguments(args),
        Function::Csc => scalar_arguments(args),
        Function::Csch => scalar_arguments(args),
        Function::Sec => scalar_arguments(args),
        Function::Sech => scalar_arguments(args),
        Function::Exp => scalar_arguments(args),
        Function::Fact => scalar_arguments(args),
        Function::Sign => scalar_arguments(args),
        Function::Radians => scalar_arguments(args),
        Function::Degrees => scalar_arguments(args),
        Function::Int => scalar_arguments(args),
        Function::Even => scalar_arguments(args),
        Function::Odd => scalar_arguments(args),
        Function::Ceiling | Function::EcmaCeiling => scalar_arguments(args),
        Function::CeilingMath => scalar_arguments(args),
        Function::CeilingPrecise => scalar_arguments(args),
        Function::Floor => scalar_arguments(args),
        Function::FloorMath => scalar_arguments(args),
        Function::FloorPrecise => scalar_arguments(args),
        Function::IsoCeiling => scalar_arguments(args),
        Function::Mod => scalar_arguments(args),
        Function::Mround => scalar_arguments(args),
        Function::Trunc => scalar_arguments(args),
        Function::Gcd => StaticResult::Scalar,
        Function::Lcm => StaticResult::Scalar,
        Function::Base => scalar_arguments(args),
        Function::Decimal => scalar_arguments(args),
        Function::Roman => scalar_arguments(args),
        Function::Arabic => scalar_arguments(args),
        Function::Combin => scalar_arguments(args),
        Function::Combina => scalar_arguments(args),
        Function::Sumsq => StaticResult::Scalar,
        Function::Mdeterm => StaticResult::Scalar,
        Function::Minverse => StaticResult::Unknown,
        Function::Munit => StaticResult::Unknown,
        Function::Multinomial => StaticResult::Scalar,
        Function::Sumproduct => StaticResult::Scalar,
        Function::Percentof => StaticResult::Scalar,
        Function::N => scalar_arguments(args),
        Function::Sheets => scalar_arguments(args),
        Function::Cell => StaticResult::Unknown,
        Function::Info => StaticResult::Unknown,
        Function::Dget => not_implemented(args),
        Function::Dmax => not_implemented(args),
        Function::Dmin => not_implemented(args),
        Function::Dcount => not_implemented(args),
        Function::Daverage => not_implemented(args),
        Function::Dsum => not_implemented(args),
        Function::Dcounta => not_implemented(args),
        Function::Dproduct => not_implemented(args),
        Function::Dstdev => not_implemented(args),
        Function::Dvar => not_implemented(args),
        Function::Dvarp => not_implemented(args),
        Function::Dstdevp => not_implemented(args),
        Function::ChisqDist => StaticResult::Scalar,
        Function::ChisqDistRT => StaticResult::Scalar,
        Function::ChisqInv => StaticResult::Scalar,
        Function::ChisqInvRT => StaticResult::Scalar,
        Function::ChisqTest => StaticResult::Scalar,
        Function::CovarianceP => StaticResult::Scalar,
        Function::CovarianceS => StaticResult::Scalar,
        Function::Devsq => StaticResult::Scalar,
        Function::FTest => StaticResult::Scalar,
        Function::GammaDist => StaticResult::Scalar,
        Function::GammaInv => StaticResult::Scalar,
        Function::HypGeomDist => StaticResult::Scalar,
        Function::Pearson => StaticResult::Scalar,
        Function::PoissonDist => StaticResult::Scalar,
        Function::Stdev => StaticResult::Scalar,
        Function::StDevP => StaticResult::Scalar,
        Function::StDevS => StaticResult::Scalar,
        Function::Stdeva => StaticResult::Scalar,
        Function::Stdevpa => StaticResult::Scalar,
        Function::TTest => StaticResult::Scalar,
        Function::VarP => StaticResult::Scalar,
        Function::VarS => StaticResult::Scalar,
        Function::VarpA => StaticResult::Scalar,
        Function::VarA => StaticResult::Scalar,
        Function::ZTest => StaticResult::Scalar,
        Function::HypGeomDistCompat => StaticResult::Scalar,
        Function::ChiDistCompat => StaticResult::Scalar,
        Function::ChiInvCompat => StaticResult::Scalar,
        Function::ChiTestCompat => StaticResult::Scalar,
        Function::CovarCompat => StaticResult::Scalar,
        Function::FTestCompat => StaticResult::Scalar,
        Function::GammaDistCompat => StaticResult::Scalar,
        Function::GammaInvCompat => StaticResult::Scalar,
        Function::ModeCompat => StaticResult::Scalar,
        Function::PoissonCompat => StaticResult::Scalar,
        Function::RankCompat => reference_parameter::rank_static_result(args),
        Function::StDevPCompat => StaticResult::Scalar,
        Function::TTestCompat => StaticResult::Scalar,
        Function::VarCompat => StaticResult::Scalar,
        Function::VarPCompat => StaticResult::Scalar,
        Function::ZTestCompat => StaticResult::Scalar,
        Function::Sumx2my2 => StaticResult::Scalar,
        Function::Sumx2py2 => StaticResult::Scalar,
        Function::Sumxmy2 => StaticResult::Scalar,
        Function::Correl => StaticResult::Scalar,
        Function::Rsq => StaticResult::Scalar,
        Function::Intercept => StaticResult::Scalar,
        Function::Slope => StaticResult::Scalar,
        Function::Steyx => StaticResult::Scalar,
        Function::Forecast => StaticResult::Scalar,
        Function::ForecastLinear => StaticResult::Scalar,
        Function::ForecastEts => StaticResult::Scalar,
        Function::ForecastEtsConfint => StaticResult::Scalar,
        Function::ForecastEtsSeasonality => StaticResult::Scalar,
        Function::ForecastEtsStat => StaticResult::Scalar,
        // Spill-returning functions
        Function::Frequency => StaticResult::Unknown,
        Function::Growth => StaticResult::Unknown,
        Function::Linest => StaticResult::Unknown,
        Function::Logest => StaticResult::Unknown,
        Function::ModeMult => StaticResult::Unknown,
        Function::Trend => StaticResult::Unknown,
        // Scalar-returning functions
        Function::ModeSingl => StaticResult::Scalar,
        Function::Prob => StaticResult::Scalar,
        Function::Harmean => StaticResult::Scalar,
        Function::Kurt => StaticResult::Scalar,
        Function::MaxA => StaticResult::Scalar,
        Function::Median => StaticResult::Scalar,
        Function::MinA => StaticResult::Scalar,
        Function::RankAvg => reference_parameter::rank_static_result(args),
        Function::RankEq => reference_parameter::rank_static_result(args),
        Function::Skew => StaticResult::Scalar,
        Function::SkewP => StaticResult::Scalar,
        // In the lifting table (SPEC-0011): `lifted.rs` derives these.
        lifting_table_functions!() => return None,
    })
}
