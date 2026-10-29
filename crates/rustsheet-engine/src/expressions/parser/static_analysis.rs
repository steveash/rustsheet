use super::reference_operators::{at_outside, wrong_range_shape};
use super::{for_each_chain_operand_mut, Node};
use crate::functions::lift::lifting_table_functions;
use crate::functions::lookup_and_reference::lifts_into_references;
use crate::functions::Function;
use broadcast::{broadcast_shapes, static_analysis_operator_chain};
use if_arm::{returns_range_arm, returns_single_cell_index};
use legacy::{
    add_argument_intersection, add_array_at, add_in_array_context, is_array_context,
    legacy_array_call_at, legacy_array_context, legacy_call_reads_a_value, let_returns_range,
    mark_legacy_reading,
};
pub use redundant::remove_redundant_implicit_intersection;
use redundant::remove_redundant_legacy_intersection;
mod aggregate;
mod broadcast;
mod if_arm;
mod legacy;
mod lifted;
mod operand_array;
mod redundant;
mod reference_parameter;
mod reference_result;
mod row_column;
mod signature;
use signature::{
    args_signature_arraytotext, args_signature_bycol_byrow, args_signature_cell,
    args_signature_choose, args_signature_choosecols, args_signature_fieldvalue,
    args_signature_filter, args_signature_getpivotdata, args_signature_groupby,
    args_signature_hlookup, args_signature_ifs, args_signature_index, args_signature_lambda,
    args_signature_let, args_signature_lookup, args_signature_map, args_signature_match,
    args_signature_npv, args_signature_offset, args_signature_one_vector, args_signature_pivotby,
    args_signature_randarray, args_signature_reduce, args_signature_row, args_signature_scalars,
    args_signature_scan, args_signature_sumif, args_signature_textafter, args_signature_textsplit,
    args_signature_trimrange, args_signature_xlookup, args_signature_xmatch, Signature,
};

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
/// Assumes formula return a scalar. The operators are those of the formula with no whitespace
/// ([`Node::rewrite_as_unspaced`]).
pub fn add_implicit_intersection(node: &mut Node, add: bool) {
    node.rewrite_as_unspaced(|node| legacy::add_root_legacy_intersection(node, add));
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
        | Node::SheetRefErrorKind { .. }
        | Node::EmptyArgKind
        | Node::ParseErrorKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::ArrayKind(_)
        | Node::ReferenceKind { .. }
        | Node::ThisRowKind { .. }
        | Node::OpUnionKind { .. }
        | Node::LambdaDefKind { .. }
        | Node::LambdaCallKind { .. } => {}
        // A span of one sheet is that sheet's reference (`=@Data1:Data1!A1` is 1,
        // probes/3d-ref-span-edges B2), so its range gets the `@` a plain range gets (T-2351).
        // A wider span is `#REF!` as a value either way and gets none.
        Node::SheetSpanKind {
            first_index,
            last_index,
            reference,
            ..
        } => {
            if add
                && first_index == last_index
                && matches!(reference.as_ref(), Node::RangeKind { .. })
            {
                *node = Node::automatic_at(node.clone());
            }
        }
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
        // A structured reference is its area: a range gets the `@` a plain range gets, around
        // the reference as written (SPEC-0016), and so does a `#This Row` range, which its
        // lowering on a data row was (T-1699).
        Node::StructuredReferenceKind { lowered, .. } => {
            if add
                && matches!(
                    lowered.as_ref(),
                    Node::RangeKind { .. } | Node::ThisRowKind { cell: false, .. }
                )
            {
                *node = Node::automatic_at(node.clone());
            }
        }
        // A range of another workbook too (fixtures/external-link-cached-ranges Refs!B44, T-1526).
        Node::OpRangeKind { .. } | Node::OpIntersectKind { .. } | Node::WrongRangeKind { .. } => {
            if add {
                *node = Node::automatic_at(node.clone());
            }
        }

        // operations
        Node::UnaryKind { right: child, .. }
        | Node::Group { child, .. }
        | Node::UnaryPlus { child }
        | Node::Spaced { child, .. } => {
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

        // A defined name takes the `@` in a value position whatever its body: a range, a scalar,
        // a SWITCH, a LET or a call (`=ABS(swArr)` shows `=ABS(@swArr)`, `=ABS(Z30:Z32)+swOne`
        // `=ABS(@Z30:Z32)+@swOne`; probes/legacy-at-defined-name-dynamic-funcs A20-V20), and so
        // does a defined-name lambda's call (`=ABS(@Z30:Z32)+@myId(1)`, probes/legacy-at-dynamic-
        // funcs-2 AE; T-2440). The call of a LET-bound one removes every `@`
        // (`removes_automatic_at`).
        Node::DefinedNameKind(_) | Node::NamedFunctionKind { .. } => {
            if add {
                *node = Node::automatic_at(node.clone());
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
                let arrays = legacy_array_context(kind, signature.get(index), index, arg, arrays);
                add_array_at(kind, signature.get(index), index, arg, arrays);
                add_argument_intersection(kind, signature.get(index), index, arg, arrays);
            }
            if arrays {
                mark_legacy_reading(kind, args); // The legacy N/T marker (SPEC-0012).
            }
            // There are some function that will never add an automatic II:
            // A LET takes the `@` itself when it returns a range, none otherwise (T-2419).
            let new_function = match kind {
                Function::Lambda => true,
                Function::Let => !(add && let_returns_range(args)),
                _ => false,
            };
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
                        (arrays
                            && (rows > 1 || columns > 1)
                            && legacy_array_call_at(kind, &signature, args))
                            || (add && returns_range_arm(kind, args))
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
        | Node::SheetRefErrorKind { .. }
        | Node::ParseErrorKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::EmptyArgKind => StaticResult::Scalar,
        Node::WrongRangeKind { .. } => wrong_range_shape(node), // T-1454
        Node::UnaryKind { right: child, .. }
        | Node::Group { child, .. }
        | Node::UnaryPlus { child }
        | Node::Spaced { child, .. }
        | Node::StructuredReferenceKind { lowered: child, .. } => {
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
        Node::ReferenceKind { .. } | Node::ThisRowKind { cell: true, .. } => StaticResult::Scalar,
        // The shape of the area on a data row (T-1699); off them it is one `#VALUE!`.
        Node::ThisRowKind {
            column1, column2, ..
        } => StaticResult::Range(0, column2 - column1),
        // A span of one sheet evaluates as that sheet's reference, so it has its shape
        // (`Model::get_sheet_span_value`, T-2351); a wider 3-D reference is a value only as an
        // error (`#REF!`), so it never spills.
        Node::SheetSpanKind {
            first_index,
            last_index,
            reference,
            ..
        } => {
            if first_index == last_index {
                run_static_analysis_on_node(reference)
            } else {
                StaticResult::Scalar
            }
        }

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

/// COUNTIF, COUNTIFS, SUMIF, SUMIFS, AVERAGEIF and MAXIFS also call themselves once per element of
/// an array of references in a range argument (`Function::per_reference_element`; SPEC-0014 rule 6,
/// `COUNTIF(OFFSET(A1,{0;1},0),">0")` spills 1, 1, probes/reference-lift-consumers E57): a range
/// argument that is a lifted OFFSET or INDIRECT gives the shape, the criteria
/// (`args[first]`, `args[first + 2]`, ...) only without one. Anything else does not:
/// `SUMIF(IF(TRUE,A1:A6),"a",B1:B6)` stays a scalar call (probes/legacy-if-arm-range-argument U12).
fn criteria_and_grid_static_result(kind: &Function, args: &[Node], first: usize) -> StaticResult {
    let takes_reference = kind
        .per_reference_element(args.len())
        .map_or((|_| false) as fn(usize) -> bool, |per_element| {
            per_element.takes_reference
        });
    let grids = args.iter().enumerate().filter_map(|(index, arg)| {
        if !lifts_into_references(arg) || !takes_reference(index) {
            return None;
        }
        match run_static_analysis_on_node(arg) {
            shape @ StaticResult::Array(..) => Some(shape),
            _ => None,
        }
    });
    // Over a grid each call's own spill is cut to its top-left value (as the engine runs it),
    // so the grids alone give the shape.
    let grids: Vec<StaticResult> = grids.collect();
    if !grids.is_empty() {
        return broadcast_shapes(grids);
    }
    criteria_static_result(args, first)
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
        Function::Choose => args_signature_choose(arg_count),
        Function::Column => args_signature_row(arg_count),
        Function::Columns => args_signature_one_vector(arg_count),
        Function::Max => vec![Signature::Vector; arg_count],
        Function::Min => vec![Signature::Vector; arg_count],
        Function::Pi => args_signature_scalars(arg_count, 0, 0),
        Function::Power => args_signature_scalars(arg_count, 2, 0),
        Function::Product => vec![Signature::Vector; arg_count],
        Function::Sum => vec![Signature::Vector; arg_count],
        Function::Sumif => args_signature_sumif(arg_count),
        Function::Sumifs => args_signature_ifs(arg_count, 2),
        Function::Isformula => args_signature_scalars(arg_count, 1, 0),
        Function::Isomitted => args_signature_scalars(arg_count, 1, 0),
        Function::Isref => args_signature_one_vector(arg_count),
        Function::Na => args_signature_scalars(arg_count, 0, 0),
        // A reference argument (a range, a table's data area: `SHEET(Table1[])`) gives its sheet's
        // number and is not implicitly intersected (probes/info-logic-argument-modes A12; legacy formulas:
        // calc_test_tables/tables.xlsx sheet9 A5:A8).
        Function::Sheet if arg_count == 1 => args_signature_one_vector(arg_count),
        Function::Sheet => args_signature_scalars(arg_count, 0, 1),
        Function::Type => args_signature_one_vector(arg_count),
        Function::Areas | Function::Phonetic => args_signature_one_vector(arg_count),
        Function::Choosecols => args_signature_choosecols(arg_count),
        Function::Chooserows => args_signature_choosecols(arg_count),
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
        Function::Xlookup => args_signature_xlookup(arg_count),
        Function::Xmatch => args_signature_xmatch(arg_count),
        Function::Trimrange => args_signature_trimrange(arg_count),
        Function::Filter => args_signature_filter(arg_count),
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
        Function::Randarray => args_signature_randarray(arg_count),
        Function::Textsplit => args_signature_textsplit(arg_count),
        Function::Concat => vec![Signature::Vector; arg_count],
        Function::Lower => args_signature_scalars(arg_count, 1, 0),
        Function::Regexextract => args_signature_scalars(arg_count, 2, 2),
        Function::Regexreplace => args_signature_scalars(arg_count, 3, 2),
        Function::Regextest => args_signature_scalars(arg_count, 2, 1),
        Function::T | Function::TLegacy => args_signature_scalars(arg_count, 1, 0),
        Function::Textafter => args_signature_textafter(arg_count),
        Function::Textbefore => args_signature_textafter(arg_count),
        Function::Trim => args_signature_scalars(arg_count, 1, 0),
        Function::Upper => args_signature_scalars(arg_count, 1, 0),
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
        Function::Now => args_signature_scalars(arg_count, 0, 0),
        Function::Today => args_signature_scalars(arg_count, 0, 0),
        Function::Subtotal | Function::Aggregate => args_signature_npv(arg_count),
        Function::Rand => args_signature_scalars(arg_count, 0, 0),
        Function::Randbetween => args_signature_scalars(arg_count, 2, 0),
        Function::Formulatext => args_signature_scalars(arg_count, 1, 0),
        Function::Char => args_signature_scalars(arg_count, 1, 0),
        Function::Clean => args_signature_scalars(arg_count, 1, 0),
        Function::Asc | Function::Dbcs => args_signature_scalars(arg_count, 1, 0),
        Function::Arraytotext => args_signature_arraytotext(arg_count),
        Function::Image => args_signature_scalars(arg_count, 1, 4),
        Function::Stockhistory => args_signature_scalars(arg_count, 2, 9),
        Function::Fieldvalue => args_signature_fieldvalue(arg_count),
        Function::Proper => args_signature_scalars(arg_count, 1, 0),
        Function::Geomean => vec![Signature::Vector; arg_count],
        Function::Gcd => vec![Signature::Vector; arg_count],
        Function::Lcm => vec![Signature::Vector; arg_count],
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
        Function::N | Function::NLegacy => args_signature_scalars(arg_count, 1, 0),
        Function::Sheets => args_signature_scalars(arg_count, 0, 1),
        Function::Cell => args_signature_cell(arg_count),
        Function::Info => args_signature_scalars(arg_count, 1, 1),
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
        Function::Pearson => {
            if arg_count == 2 {
                vec![Signature::Vector, Signature::Vector]
            } else {
                vec![Signature::Error; arg_count]
            }
        }
        Function::Stdev => vec![Signature::Vector; arg_count],
        Function::StDevP => vec![Signature::Vector; arg_count],
        Function::StDevS => vec![Signature::Vector; arg_count],
        Function::Stdeva => vec![Signature::Vector; arg_count],
        Function::Stdevpa => vec![Signature::Vector; arg_count],
        Function::VarP => vec![Signature::Vector; arg_count],
        Function::VarS => vec![Signature::Vector; arg_count],
        Function::VarpA => vec![Signature::Vector; arg_count],
        Function::VarA => vec![Signature::Vector; arg_count],
        // Compatibility aliases
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
        Function::ModeCompat => vec![Signature::Vector; arg_count],
        Function::StDevPCompat => vec![Signature::Vector; arg_count],
        Function::VarCompat => vec![Signature::Vector; arg_count],
        Function::VarPCompat => vec![Signature::Vector; arg_count],
        Function::Sumx2my2 => vec![Signature::Vector; 2],
        Function::Sumx2py2 => vec![Signature::Vector; 2],
        Function::Sumxmy2 => vec![Signature::Vector; 2],
        Function::Correl => vec![Signature::Vector; 2],
        Function::Rsq => vec![Signature::Vector; 2],
        Function::Intercept => vec![Signature::Vector; 2],
        Function::Slope => vec![Signature::Vector; 2],
        Function::Steyx => vec![Signature::Vector; 2],
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
        Function::Harmean => vec![Signature::Vector; arg_count],
        Function::Kurt => vec![Signature::Vector; arg_count],
        Function::MaxA => vec![Signature::Vector; arg_count],
        Function::Median => vec![Signature::Vector; arg_count],
        Function::MinA => vec![Signature::Vector; arg_count],
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

/// The shape of the IF or CHOOSE `kind(args)`, from each argument's shape computed once (so nested
/// calls stay linear): a scalar when its value arms are one cell whatever the condition
/// ([`returns_single_cell_index`]), so a legacy formula shows no `@` on the call or a call over it
/// (T-3134), and the cell stores it as a plain formula, not a spill anchor; otherwise IF broadcasts
/// its arguments and CHOOSE is reference-shaped.
fn if_or_choose_shape(kind: &Function, args: &[Node]) -> StaticResult {
    let shapes: Vec<StaticResult> = args.iter().map(run_static_analysis_on_node).collect();
    if returns_single_cell_index(args, &shapes) {
        StaticResult::Scalar
    } else if matches!(kind, Function::Choose) {
        reference_result::choose_static_result(shapes)
    } else {
        broadcast_shapes(shapes)
    }
}

/// The hand-written shapes of the functions not in the lifting table; `None` for one in it.
fn fallback_shape(kind: &Function, args: &[Node]) -> Option<StaticResult> {
    Some(match kind {
        Function::And => StaticResult::Scalar,
        Function::False => StaticResult::Scalar,
        Function::If | Function::Choose => if_or_choose_shape(kind, args),
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
        Function::Column => row_column::static_analysis_row_column(args, false),
        Function::Columns => not_implemented(args),
        Function::Max => StaticResult::Scalar,
        Function::Min => StaticResult::Scalar,
        Function::Pi => StaticResult::Scalar,
        Function::Power => scalar_arguments(args),
        Function::Product => not_implemented(args),
        Function::Sum => StaticResult::Scalar,
        Function::Sumif => criteria_and_grid_static_result(kind, args, 1),
        Function::Sumifs => criteria_and_grid_static_result(kind, args, 2),
        Function::Isformula => scalar_arguments(args),
        Function::Isomitted => not_implemented(args),
        Function::Isref => not_implemented(args),
        Function::Na => StaticResult::Scalar,
        Function::Sheet => StaticResult::Scalar,
        Function::Type => not_implemented(args),
        Function::Areas => aggregate::areas_static_result(args),
        Function::Choosecols => StaticResult::Unknown,
        Function::Chooserows => StaticResult::Unknown,
        Function::Hlookup => aggregate::lookup_static_result(args), // T-0270, T-1269
        Function::Hstack => StaticResult::Unknown,
        // INDEX has two forms, but they are indistinguishable at parse time.
        Function::Index => StaticResult::Unknown,
        Function::Indirect => reference_result::static_analysis_indirect(args),
        // Spills by lookup_value; the vectors are consumed (T-1241).
        Function::Lookup => broadcast_static_result(args.first()),
        Function::Match => reference_result::match_static_result(args), // T-1866, T-1917
        Function::Offset => reference_result::static_analysis_offset(args),
        Function::Row => row_column::static_analysis_row_column(args, true),
        Function::Rows => not_implemented(args),
        Function::Vlookup => aggregate::lookup_static_result(args), // T-0270, T-1269
        Function::Vstack => StaticResult::Unknown,
        Function::Xlookup => StaticResult::Unknown,
        // Spills by lookup_value, match_mode and search_mode, broadcast; lookup_array is consumed
        // (T-1241, T-1958).
        Function::Xmatch => broadcast_static_result(
            args.iter()
                .enumerate()
                .filter(|(i, _)| *i != 1)
                .map(|(_, a)| a),
        ),
        Function::Trimrange => reference_result::static_analysis_trimrange(args),
        Function::Filter => StaticResult::Unknown,
        Function::Transpose => StaticResult::Unknown,
        Function::Mmult => StaticResult::Unknown,
        Function::Bycol | Function::Byrow => StaticResult::Unknown,
        Function::Groupby | Function::Pivotby => StaticResult::Unknown,
        // Spills by its items, broadcast; the other arguments never lift (SPEC-0015, T-2083).
        Function::Getpivotdata => broadcast_static_result(args.iter().skip(3).step_by(2)),
        Function::Map | Function::Reduce => StaticResult::Unknown,
        Function::Scan => StaticResult::Unknown,
        Function::Randarray => StaticResult::Unknown,
        Function::Textsplit => StaticResult::Unknown,
        Function::Concat => not_implemented(args),
        // UPPER/LOWER broadcast element-wise: an array/range argument yields an array.
        Function::Lower => scalar_arguments(args),
        // Their text and scalar arguments lift over a range or an array (T-1272), each lifted
        // element keeping one value, so the shape is not known statically.
        Function::Regexextract | Function::Regexreplace => StaticResult::Unknown,
        // Over scalars alone a single value: a legacy `=LEN(TEXTBEFORE("ab","b"))` and
        // `=N(REGEXTEST("a","a"))` show no `@` on the call (probes/legacy-at-dynamic-funcs-2 N, O,
        // Z, T-2440), where REGEXEXTRACT, whose return_mode can spill, takes one (Y).
        Function::Regextest | Function::Textafter | Function::Textbefore => {
            match scalar_arguments(args) {
                StaticResult::Scalar => StaticResult::Scalar,
                _ => StaticResult::Unknown,
            }
        }
        Function::T => scalar_arguments(args),
        Function::Trim => scalar_arguments(args),
        Function::Char => not_implemented(args),
        Function::Clean => scalar_arguments(args),
        Function::Asc | Function::Dbcs => scalar_arguments(args),
        Function::Phonetic => not_implemented(args),
        Function::Arraytotext => not_implemented(args),
        // All three lift over an array or range `source`/`stock`/`value` (T-1179).
        Function::Image | Function::Stockhistory | Function::Fieldvalue => StaticResult::Unknown,
        Function::Proper => scalar_arguments(args),
        Function::Upper => scalar_arguments(args),
        Function::Average => not_implemented(args),
        Function::Averagea => not_implemented(args),
        Function::Avedev => not_implemented(args),
        Function::Averageif => criteria_and_grid_static_result(kind, args, 1),
        Function::Averageifs => criteria_static_result(args, 2),
        Function::Count => not_implemented(args),
        Function::Counta => not_implemented(args),
        Function::Countblank => reference_parameter::countblank_static_result(args),
        Function::Countif => criteria_and_grid_static_result(kind, args, 1),
        Function::Countifs => criteria_and_grid_static_result(kind, args, 1),
        Function::Maxifs => criteria_and_grid_static_result(kind, args, 2),
        Function::Minifs => criteria_static_result(args, 2),
        Function::Now => not_implemented(args),
        Function::Today => not_implemented(args),
        Function::Subtotal => aggregate::subtotal_static_result(args),
        Function::Aggregate => aggregate::aggregate_static_result(args),
        Function::Rand => not_implemented(args),
        Function::Randbetween => scalar_arguments(args),
        // One result per cell of a multi-cell reference (T-1241).
        Function::Formulatext => broadcast_static_result(args),
        Function::Geomean => not_implemented(args),
        Function::Gcd => StaticResult::Scalar,
        Function::Lcm => StaticResult::Scalar,
        Function::Sumsq => StaticResult::Scalar,
        Function::Mdeterm => StaticResult::Scalar,
        Function::Minverse => StaticResult::Unknown,
        Function::Munit => StaticResult::Unknown,
        Function::Multinomial | Function::Sumproduct | Function::Percentof => StaticResult::Scalar,
        // A legacy N or T is one value: an array argument is #VALUE! (SPEC-0012).
        Function::NLegacy | Function::TLegacy => StaticResult::Scalar,
        Function::N => scalar_arguments(args),
        Function::Sheets => scalar_arguments(args),
        Function::Cell => StaticResult::Unknown,
        Function::Info => StaticResult::Unknown,
        Function::ChisqTest => StaticResult::Scalar,
        Function::CovarianceP => StaticResult::Scalar,
        Function::CovarianceS => StaticResult::Scalar,
        Function::Devsq => StaticResult::Scalar,
        Function::FTest => StaticResult::Scalar,
        Function::Pearson => StaticResult::Scalar,
        Function::Stdev => StaticResult::Scalar,
        Function::StDevP => StaticResult::Scalar,
        Function::StDevS => StaticResult::Scalar,
        Function::Stdeva => StaticResult::Scalar,
        Function::Stdevpa => StaticResult::Scalar,
        Function::VarP => StaticResult::Scalar,
        Function::VarS => StaticResult::Scalar,
        Function::VarpA => StaticResult::Scalar,
        Function::VarA => StaticResult::Scalar,
        Function::ChiTestCompat => StaticResult::Scalar,
        Function::CovarCompat => StaticResult::Scalar,
        Function::FTestCompat => StaticResult::Scalar,
        Function::ModeCompat => StaticResult::Scalar,
        Function::StDevPCompat => StaticResult::Scalar,
        Function::VarCompat => StaticResult::Scalar,
        Function::VarPCompat => StaticResult::Scalar,
        Function::Sumx2my2 => StaticResult::Scalar,
        Function::Sumx2py2 => StaticResult::Scalar,
        Function::Sumxmy2 => StaticResult::Scalar,
        Function::Correl => StaticResult::Scalar,
        Function::Rsq => StaticResult::Scalar,
        Function::Intercept => StaticResult::Scalar,
        Function::Slope => StaticResult::Scalar,
        Function::Steyx => StaticResult::Scalar,
        // Spill-returning functions
        Function::Frequency => StaticResult::Unknown,
        Function::Growth => StaticResult::Unknown,
        Function::Linest => StaticResult::Unknown,
        Function::Logest => StaticResult::Unknown,
        Function::ModeMult => StaticResult::Unknown,
        Function::Trend => StaticResult::Unknown,
        // Scalar-returning functions
        Function::ModeSingl => StaticResult::Scalar,
        Function::Harmean => StaticResult::Scalar,
        Function::Kurt => StaticResult::Scalar,
        Function::MaxA => StaticResult::Scalar,
        Function::Median => StaticResult::Scalar,
        Function::MinA => StaticResult::Scalar,
        Function::Skew => StaticResult::Scalar,
        Function::SkewP => StaticResult::Scalar,
        // In the lifting table (SPEC-0011): `lifted.rs` derives these.
        lifting_table_functions!() => return None,
    })
}
