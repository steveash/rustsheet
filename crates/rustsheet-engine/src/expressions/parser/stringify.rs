mod bound_variables;
use super::reference_operators::{
    at_text, callee_operand, intersection_operand, spill_operand, tight_operand, union_text,
};
use super::{
    super::utils::{name_needs_quoting, quote_name},
    for_each_chain_operand_mut, left_chain, value_operands, Node, Reference,
};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::move_formula::to_string_array_node;
use crate::expressions::parser::static_analysis::remove_redundant_implicit_intersection;
use crate::expressions::token::{OpSum, OpUnary};
use crate::language::{get_language, Language};
use crate::locale::{get_locale, Locale};
use crate::{expressions::types::CellReferenceRC, number_format::to_excel_precision_str};
use bound_variables::prefix_bound_variables;

pub enum DisplaceData {
    Column {
        sheet: u32,
        column: i32,
        delta: i32,
    },
    Row {
        sheet: u32,
        row: i32,
        delta: i32,
    },
    CellHorizontal {
        sheet: u32,
        row: i32,
        column: i32,
        delta: i32,
    },
    CellVertical {
        sheet: u32,
        row: i32,
        column: i32,
        delta: i32,
    },
    RowMove {
        sheet: u32,
        row: i32,
        delta: i32,
    },
    ColumnMove {
        sheet: u32,
        column: i32,
        delta: i32,
    },
    None,
}

/// This is the internal mode in IronCalc
/// Formulas internally are stored in R1C1 format, the locale and language are always "en"
pub fn to_rc_format(node: &Node) -> String {
    #[allow(clippy::expect_used)]
    let locale = get_locale("en").expect("");
    #[allow(clippy::expect_used)]
    let language = get_language("en").expect("");
    stringify(node, None, &DisplaceData::None, false, locale, language)
}

pub fn to_english_string(node: &Node, context: &CellReferenceRC) -> String {
    #[allow(clippy::expect_used)]
    let locale = get_locale("en").expect("");
    #[allow(clippy::expect_used)]
    let language = get_language("en").expect("");
    stringify(
        node,
        Some(context),
        &DisplaceData::None,
        false,
        locale,
        language,
    )
}

/// This is the mode used to display the formula in the UI
pub fn to_localized_string(
    node: &Node,
    context: &CellReferenceRC,
    locale: &Locale,
    language: &Language,
) -> String {
    stringify(
        node,
        Some(context),
        &DisplaceData::None,
        false,
        locale,
        language,
    )
}

/// This is the mode used to export the formula to Excel
/// Internally the locale and language are always "en"
pub fn to_excel_string(node: &Node, context: &CellReferenceRC) -> String {
    excel_string(node, context, false)
}

/// Like [`to_excel_string`], for the formula of an array, CSE or dynamic.
///
/// No implicit intersection is added to the formula of an array when it is
/// read, by Excel or by the import, so none of its `@` is redundant: they are
/// all written, as `_xlfn.SINGLE`.
pub fn to_excel_array_formula_string(node: &Node, context: &CellReferenceRC) -> String {
    excel_string(node, context, true)
}

fn excel_string(node: &Node, context: &CellReferenceRC, is_array_formula: bool) -> String {
    #[allow(clippy::expect_used)]
    let locale = get_locale("en").expect("");
    #[allow(clippy::expect_used)]
    let language = get_language("en").expect("");
    // The internal representation stores every implicit intersection as `@`,
    // without the `automatic` flag. Drop the operators that Excel would re-insert
    // automatically on import; the rest are kept as `_xlfn.SINGLE` while
    // stringifying. See `remove_redundant_implicit_intersection`.
    let mut node = node.clone();
    if !is_array_formula {
        remove_redundant_implicit_intersection(&mut node, true);
    }
    prefix_bound_variables(&mut node, &mut Vec::new());
    stringify(
        &node,
        Some(context),
        &DisplaceData::None,
        true,
        locale,
        language,
    )
}

pub fn to_string_displaced(
    node: &Node,
    context: &CellReferenceRC,
    displace_data: &DisplaceData,
    locale: &Locale,
    language: &Language,
) -> String {
    stringify(node, Some(context), displace_data, false, locale, language)
}

/// The sheet prefix of a 3-D reference, `First:Last`, quoted as one name when either sheet
/// needs quotes (Excel writes `'Data2:Q 4'!A1`).
pub(crate) fn sheet_span_text(first: &str, last: &str) -> String {
    let span = format!("{first}:{last}");
    if !name_needs_quoting(first) && !name_needs_quoting(last) {
        span
    } else {
        format!("'{}'", span.replace('\'', "''"))
    }
}

/// Converts a local reference to a string applying some displacement if needed.
/// It uses A1 style if context is not None. If context is None it uses R1C1 style
/// If full_row is true then the row details will be omitted in the A1 case
/// If full_column is true then column details will be omitted.
pub(crate) fn stringify_reference(
    context: Option<&CellReferenceRC>,
    displace_data: &DisplaceData,
    reference: &Reference,
    full_row: bool,
    full_column: bool,
) -> String {
    let sheet_name = reference.sheet_name;
    let sheet_index = reference.sheet_index;
    let absolute_row = reference.absolute_row;
    let absolute_column = reference.absolute_column;
    let row = reference.row;
    let column = reference.column;
    match context {
        Some(context) => {
            let mut row = if absolute_row { row } else { row + context.row };
            let mut column = if absolute_column {
                column
            } else {
                column + context.column
            };
            match displace_data {
                DisplaceData::Row {
                    sheet,
                    row: displace_row,
                    delta,
                } => {
                    if sheet_index == *sheet && !full_row {
                        if *delta < 0 {
                            if &row >= displace_row {
                                if row < displace_row - *delta {
                                    return "#REF!".to_string();
                                }
                                row += *delta;
                            }
                        } else if &row >= displace_row {
                            row += *delta;
                        }
                    }
                }
                DisplaceData::Column {
                    sheet,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && !full_column {
                        if *delta < 0 {
                            if &column >= displace_column {
                                if column < displace_column - *delta {
                                    return "#REF!".to_string();
                                }
                                column += *delta;
                            }
                        } else if &column >= displace_column {
                            column += *delta;
                        }
                    }
                }
                DisplaceData::CellHorizontal {
                    sheet,
                    row: displace_row,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && displace_row == &row {
                        if *delta < 0 {
                            if &column >= displace_column {
                                if column < displace_column - *delta {
                                    return "#REF!".to_string();
                                }
                                column += *delta;
                            }
                        } else if &column >= displace_column {
                            column += *delta;
                        }
                    }
                }
                DisplaceData::CellVertical {
                    sheet,
                    row: displace_row,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && displace_column == &column {
                        if *delta < 0 {
                            if &row >= displace_row {
                                if row < displace_row - *delta {
                                    return "#REF!".to_string();
                                }
                                row += *delta;
                            }
                        } else if &row >= displace_row {
                            row += *delta;
                        }
                    }
                }
                DisplaceData::RowMove {
                    sheet,
                    row: move_row,
                    delta,
                } => {
                    if sheet_index == *sheet {
                        if row == *move_row {
                            row += *delta;
                        } else if *delta > 0 {
                            // Moving the row downwards
                            if row > *move_row && row <= *move_row + *delta {
                                // Intermediate rows move up by one position
                                row -= 1;
                            }
                        } else if *delta < 0 {
                            // Moving the row upwards
                            if row < *move_row && row >= *move_row + *delta {
                                // Intermediate rows move down by one position
                                row += 1;
                            }
                        }
                    }
                }
                DisplaceData::ColumnMove {
                    sheet,
                    column: move_column,
                    delta,
                } => {
                    if sheet_index == *sheet {
                        if column == *move_column {
                            column += *delta;
                        } else if *delta > 0 {
                            // Moving the column to the right
                            if column > *move_column && column <= *move_column + *delta {
                                // Intermediate columns move left by one position
                                column -= 1;
                            }
                        } else if *delta < 0 {
                            // Moving the column to the left
                            if column < *move_column && column >= *move_column + *delta {
                                // Intermediate columns move right by one position
                                column += 1;
                            }
                        }
                    }
                }
                DisplaceData::None => {}
            }
            if row < 1 {
                return "#REF!".to_string();
            }
            let mut row_abs = if absolute_row {
                format!("${row}")
            } else {
                format!("{row}")
            };
            let column = match crate::expressions::utils::number_to_column(column) {
                Some(s) => s,
                None => return "#REF!".to_string(),
            };
            let mut col_abs = if absolute_column {
                format!("${column}")
            } else {
                column
            };
            if full_row {
                row_abs = "".to_string()
            }
            if full_column {
                col_abs = "".to_string()
            }
            match &sheet_name {
                Some(name) => {
                    format!("{}!{}{}", quote_name(name), col_abs, row_abs)
                }
                None => {
                    format!("{col_abs}{row_abs}")
                }
            }
        }
        None => {
            let row_abs = if absolute_row {
                format!("R{row}")
            } else {
                format!("R[{row}]")
            };
            let col_abs = if absolute_column {
                format!("C{column}")
            } else {
                format!("C[{column}]")
            };
            match &sheet_name {
                Some(name) => {
                    format!("{}!{}{}", quote_name(name), row_abs, col_abs)
                }
                None => {
                    format!("{row_abs}{col_abs}")
                }
            }
        }
    }
}

/// The separator of function arguments, and of union operands, in `locale`: `,`, or `;`
/// where the decimal separator is not `.`.
pub(crate) fn argument_separator(locale: &Locale) -> &'static str {
    if locale.numbers.symbols.decimal == "." {
        ","
    } else {
        ";"
    }
}

fn format_function(
    name: &str,
    args: &Vec<Node>,
    context: Option<&CellReferenceRC>,
    displace_data: &DisplaceData,
    export_to_excel: bool,
    locale: &Locale,
    language: &Language,
) -> String {
    let mut first = true;
    let mut arguments = "".to_string();
    let arg_separator = argument_separator(locale);
    for el in args {
        if !first {
            arguments = format!(
                "{}{}{}",
                arguments,
                arg_separator,
                stringify(
                    el,
                    context,
                    displace_data,
                    export_to_excel,
                    locale,
                    language
                )
            );
        } else {
            first = false;
            arguments = stringify(
                el,
                context,
                displace_data,
                export_to_excel,
                locale,
                language,
            );
        }
    }
    format!("{name}({arguments})")
}

// There is just one representation in the AST (Abstract Syntax Tree) of a formula.
// But three different ways to convert it to a string.
//
// To stringify a formula we need a "context", that is in which cell are we doing the "stringifying"
//
// But there are three ways to stringify a formula:
//
// * To show it to the IronCalc user
// * To store internally
// * To export to Excel
//
// There are, of course correspondingly three "modes" when parsing a formula.
//
// The internal representation is the more different as references are stored in the RC representation.
// The the AST of the formula is kept close to this representation we don't need a context
//
// In the export to Excel representation certain things are different:
// * We add a _xlfn. in front of some (more modern) functions
// * We remove the Implicit Intersection operator when it is automatic and add _xlfn.SINGLE when it is not
//
// Examples:
// * =A1+B2
// * =RC+R1C1
// * =A1+B1

/// `text` in the `levels` pairs of parentheses a [`Node::Group`] was written in.
pub(crate) fn group_text(levels: u8, text: &str) -> String {
    let levels = usize::from(levels);
    format!("{}{text}{}", "(".repeat(levels), ")".repeat(levels))
}

fn stringify(
    node: &Node,
    context: Option<&CellReferenceRC>,
    displace_data: &DisplaceData,
    export_to_excel: bool,
    locale: &Locale,
    language: &Language,
) -> String {
    use self::Node::*;
    let sub = |n: &Node| stringify(n, context, displace_data, export_to_excel, locale, language);
    match node {
        BooleanKind(value) => {
            if *value {
                language.booleans.r#true.to_string()
            } else {
                language.booleans.r#false.to_string()
            }
        }
        NumberKind(number) => {
            let s = to_excel_precision_str(*number);
            if locale.numbers.symbols.decimal == "." {
                s
            } else {
                s.replace(".", &locale.numbers.symbols.decimal)
            }
        }
        StringKind(value) => format!("\"{value}\""),
        WrongReferenceKind {
            sheet_name,
            column,
            row,
            absolute_row,
            absolute_column,
        } => stringify_reference(
            context,
            &DisplaceData::None,
            &Reference {
                sheet_name,
                sheet_index: 0,
                row: *row,
                column: *column,
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
            },
            false,
            false,
        ),
        ReferenceKind {
            sheet_name,
            sheet_index,
            column,
            row,
            absolute_row,
            absolute_column,
        } => stringify_reference(
            context,
            displace_data,
            &Reference {
                sheet_name,
                sheet_index: *sheet_index,
                row: *row,
                column: *column,
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
            },
            false,
            false,
        ),
        RangeKind {
            sheet_name,
            sheet_index,
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
        } => {
            // Note that open ranges SUM(A:A) or SUM(1:1) will be treated as normal ranges in the R1C1 (internal) representation
            // A:A will be R1C[0]:R1048576C[0]
            // So when we are forming the A1 range we need to strip the irrelevant information
            let full_row = *absolute_row1 && *absolute_row2 && (*row1 == 1) && (*row2 == LAST_ROW);
            let full_column = *absolute_column1
                && *absolute_column2
                && (*column1 == 1)
                && (*column2 == LAST_COLUMN);
            let s1 = stringify_reference(
                context,
                displace_data,
                &Reference {
                    sheet_name,
                    sheet_index: *sheet_index,
                    row: *row1,
                    column: *column1,
                    absolute_row: *absolute_row1,
                    absolute_column: *absolute_column1,
                },
                full_row,
                full_column,
            );
            let s2 = stringify_reference(
                context,
                displace_data,
                &Reference {
                    sheet_name: &None,
                    sheet_index: *sheet_index,
                    row: *row2,
                    column: *column2,
                    absolute_row: *absolute_row2,
                    absolute_column: *absolute_column2,
                },
                full_row,
                full_column,
            );
            format!("{s1}:{s2}")
        }
        // Rows and columns inserted or moved on one sheet of the span do not shift the span's
        // cells (edit semantics are not modelled yet), so no displacement reaches `reference`.
        SheetSpanKind {
            first_sheet,
            last_sheet,
            reference,
            ..
        } => format!(
            "{}!{}",
            sheet_span_text(first_sheet, last_sheet),
            stringify(
                reference,
                context,
                &DisplaceData::None,
                export_to_excel,
                locale,
                language
            )
        ),
        WrongRangeKind {
            sheet_name,
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
        } => {
            // Note that open ranges SUM(A:A) or SUM(1:1) will be treated as normal ranges in the R1C1 (internal) representation
            // A:A will be R1C[0]:R1048576C[0]
            // So when we are forming the A1 range we need to strip the irrelevant information
            let full_row = *absolute_row1 && *absolute_row2 && (*row1 == 1) && (*row2 == LAST_ROW);
            let full_column = *absolute_column1
                && *absolute_column2
                && (*column1 == 1)
                && (*column2 == LAST_COLUMN);
            let s1 = stringify_reference(
                context,
                &DisplaceData::None,
                &Reference {
                    sheet_name,
                    sheet_index: 0, // HACK
                    row: *row1,
                    column: *column1,
                    absolute_row: *absolute_row1,
                    absolute_column: *absolute_column1,
                },
                full_row,
                full_column,
            );
            let s2 = stringify_reference(
                context,
                &DisplaceData::None,
                &Reference {
                    sheet_name: &None,
                    sheet_index: 0, // HACK
                    row: *row2,
                    column: *column2,
                    absolute_row: *absolute_row2,
                    absolute_column: *absolute_column2,
                },
                full_row,
                full_column,
            );
            format!("{s1}:{s2}")
        }
        OpRangeKind { left, right } => {
            let operand = |n: &Node| tight_operand(sub(n), n);
            format!("{}:{}", operand(left), operand(right))
        }
        OpIntersectKind {
            left,
            right,
            written,
        } => {
            let operand = |n: &Node, is_left| intersection_operand(sub(n), n, is_left);
            let cut = format!("{} {}", operand(left, true), operand(right, false));
            format!("{}{cut}{}", "(".repeat(*written), ")".repeat(*written))
        }
        OpUnionKind { left, right, .. } => union_text(node, &sub(left), &sub(right), locale),
        OpConcatenateKind { .. }
        | CompareKind { .. }
        | OpSumKind { .. }
        | OpProductKind { .. }
        | OpPowerKind { .. } => {
            // A loop down the chain's left operands, not recursion (T-0128).
            let (links, first) = left_chain(node);
            let mut text = sub(first);
            for link in links {
                text = operator_text(link, text, &sub);
            }
            text
        }
        NamedFunctionKind { name, args, id: _ } => format_function(
            &name.to_lowercase(),
            args,
            context,
            displace_data,
            export_to_excel,
            locale,
            language,
        ),
        FunctionKind { kind, args } => {
            let name = if export_to_excel {
                kind.to_xlsx_string()
            } else {
                kind.to_localized_name(language)
            };
            format_function(
                &name,
                args,
                context,
                displace_data,
                export_to_excel,
                locale,
                language,
            )
        }
        ArrayKind(args) => {
            let mut first_row = true;
            let mut matrix_string = String::new();
            let row_separator = if locale.numbers.symbols.decimal == "." {
                ';'
            } else {
                '/'
            };
            let col_separator = if row_separator == ';' { ',' } else { ';' };

            for row in args {
                if !first_row {
                    matrix_string.push(row_separator);
                } else {
                    first_row = false;
                }
                let mut first_column = true;
                let mut row_string = String::new();
                for el in row {
                    if !first_column {
                        row_string.push(col_separator);
                    } else {
                        first_column = false;
                    }
                    row_string.push_str(&to_string_array_node(el, locale, language));
                }
                matrix_string.push_str(&row_string);
            }
            format!("{{{matrix_string}}}")
        }
        TableNameKind(value) => value.to_string(),
        DefinedNameKind((name, ..)) => name.to_string(),
        NamedVariableKind { name, id: _ } => name.to_string(),
        UnaryKind { kind, right } => match kind {
            OpUnary::Minus => {
                let needs_parentheses = match **right {
                    BooleanKind(_)
                    | NumberKind(_)
                    | StringKind(_)
                    | ReferenceKind { .. }
                    | RangeKind { .. }
                    | SheetSpanKind { .. }
                    | WrongReferenceKind { .. }
                    | WrongRangeKind { .. }
                    | OpRangeKind { .. }
                    | OpIntersectKind { .. }
                    | OpUnionKind { .. }
                    | OpProductKind { .. }
                    | FunctionKind { .. }
                    | NamedFunctionKind { .. }
                    | LambdaDefKind { .. }
                    | LambdaCallKind { .. }
                    | ArrayKind(_)
                    | DefinedNameKind(_)
                    | TableNameKind(_)
                    | NamedVariableKind { .. }
                    | ImplicitIntersection { .. }
                    | SpillRangeOperator { .. }
                    | GroupedRootKind { .. }
                    | Group { .. }
                    | ErrorKind(_)
                    | ParseErrorKind { .. }
                    | EmptyArgKind => false,
                    // Looser than the minus: `-(A1=B1)` is not `-A1=B1`; `--x` needs none.
                    UnaryKind { ref kind, .. } => *kind == OpUnary::Percentage,
                    OpPowerKind { .. } | OpSumKind { .. } | OpConcatenateKind { .. } => true,
                    CompareKind { .. } => true,
                };
                if needs_parentheses {
                    format!("-({})", sub(right))
                } else {
                    format!("-{}", sub(right))
                }
            }
            OpUnary::Percentage => {
                format!("{}%", sub(right))
            }
        },
        ErrorKind(kind) => format!("{kind}"),
        ParseErrorKind { formula, .. } => formula.to_string(),
        EmptyArgKind => "".to_string(),
        GroupedRootKind { child } => format!("({})", sub(child)),
        Group { levels, child } => group_text(*levels, &sub(child)),
        SpillRangeOperator { child } => {
            if export_to_excel {
                return format!("_xlfn.ANCHORARRAY({})", sub(child));
            };
            format!("{}#", spill_operand(sub(child), child))
        }
        LambdaDefKind { parameters, body } => {
            let lambda_name = if export_to_excel {
                "_xlfn.LAMBDA"
            } else {
                "LAMBDA"
            };
            let arg_sep = argument_separator(locale);
            let mut parts: Vec<String> = parameters
                .iter()
                .map(|p| {
                    if export_to_excel {
                        if p.is_optional {
                            format!("_xlop.{}", p.name)
                        } else {
                            format!("_xlpm.{}", p.name)
                        }
                    } else if p.is_optional {
                        format!("[{}]", p.name)
                    } else {
                        p.name.clone()
                    }
                })
                .collect();
            parts.push(sub(body));
            format!("{}({})", lambda_name, parts.join(arg_sep))
        }
        LambdaCallKind { lambda, args } => {
            // When the callee is a plain named variable (unknown identifier used as a function),
            // use lowercase to distinguish from real function calls in R1C1 format and
            // to match the display behaviour of InvalidFunctionKind.
            let lambda_str = match lambda.as_ref() {
                Node::NamedVariableKind { name, id: _ } => name.to_lowercase(),
                other => callee_operand(sub(other), other),
            };
            let call_args: Vec<String> = args.iter().map(sub).collect();
            format!(
                "{}({})",
                lambda_str,
                call_args.join(argument_separator(locale))
            )
        }
        ImplicitIntersection {
            automatic: _,
            child,
            written,
            lead,
            grouped: _,
        } => {
            if export_to_excel {
                // Redundant operators have already been stripped by
                // `remove_redundant_implicit_intersection`; whatever remains is
                // meaningful and must be exported explicitly.
                return format!("_xlfn.SINGLE({})", sub(child));
            }
            // The space binds tighter than `@`, so `@A1:A3 A2:B2` is `@(A1:A3 A2:B2)`; Excel's
            // FORMULATEXT prints it without parentheses (T-1098).
            at_text(child, *written, *lead, &sub)
        }
    }
}

/// The text of value operator `node` (`+ - * / ^ &` or a comparison) whose left operand prints
/// as `left_text`: the operator and its right operand appended, with the parentheses either
/// operand needs to re-parse to the same tree. One link of the loop that prints a chain
/// (T-0128).
fn operator_text(node: &Node, left_text: String, sub: &dyn Fn(&Node) -> String) -> String {
    use self::Node::*;
    let wrap = |text: String, wrap: bool| if wrap { format!("({text})") } else { text };
    let paren_if = |n: &Node, parens: bool| wrap(sub(n), parens);
    // `&` and the comparisons bind looser than `+ - * /`, so as operands of those they need
    // parentheses to re-parse to the same tree (T-0157).
    let is_loose = |n: &Node| matches!(n, CompareKind { .. } | OpConcatenateKind { .. });
    let is_compare = |n: &Node| matches!(n, CompareKind { .. });
    let compare_in_parens = |n: &Node| paren_if(n, is_compare(n));
    let Some((left, _)) = value_operands(node) else {
        return left_text;
    };
    match node {
        OpConcatenateKind { right, .. } => {
            // `&` binds tighter than the comparisons: `"a"&(2<3)` printed bare re-parses as
            // `("a"&2)<3` (T-0151).
            format!(
                "{}&{}",
                wrap(left_text, is_compare(left)),
                compare_in_parens(right)
            )
        }
        CompareKind { kind, right, .. } => {
            // Comparisons chain left to right, so only a comparison on the right needs parentheses.
            format!("{left_text}{kind}{}", compare_in_parens(right))
        }
        OpSumKind { kind, right, .. } => {
            // CompareKind and `&` have lower precedence than +/-, so wrap them to preserve semantics;
            // `a-(b+c)` additionally needs its parentheses.
            let left_str = wrap(left_text, is_loose(left));
            let right_wrap = is_loose(right)
                || (matches!(kind, OpSum::Minus) && matches!(**right, OpSumKind { .. }));
            format!("{left_str}{kind}{}", paren_if(right, right_wrap))
        }
        OpProductKind { kind, right, .. } => {
            let x = wrap(
                left_text,
                is_loose(left) || matches!(left, OpSumKind { .. }),
            );
            let right_wrap =
                is_loose(right) || matches!(**right, OpSumKind { .. } | OpProductKind { .. });
            format!("{x}{kind}{}", paren_if(right, right_wrap))
        }
        OpPowerKind { right, .. } => {
            // Only these print bare, else parenthesised; unary: bare left, wrapped right (T-0190).
            let is_atom = |n: &Node| {
                matches!(
                    n,
                    BooleanKind(_)
                        | NumberKind(_)
                        | StringKind(_)
                        | ReferenceKind { .. }
                        | RangeKind { .. }
                        | OpIntersectKind { .. }
                        | OpUnionKind { .. }
                        | WrongReferenceKind { .. }
                        | DefinedNameKind(_)
                        | TableNameKind(_)
                        | NamedVariableKind { .. }
                        | WrongRangeKind { .. }
                        | Group { .. }
                        | ImplicitIntersection { .. } // never wrapped (probes/at-power-text)
                )
            };
            // `^` is left associative, so a `^` left operand prints bare: `=A1^2^3` stays as
            // written, `=(A1^2)^3` keeps its `Group` (probes/power-chain-text, T-1331).
            let x = wrap(
                left_text,
                !(is_atom(left) || matches!(left, UnaryKind { .. } | OpPowerKind { .. })),
            );
            let y = paren_if(right, !is_atom(right));
            format!("{x}^{y}")
        }
        _ => left_text,
    }
}

pub(crate) fn rename_sheet_in_node(node: &mut Node, sheet_index: u32, new_name: &str) {
    match node {
        // Rename
        Node::ReferenceKind {
            sheet_name,
            sheet_index: index,
            ..
        } => {
            if *index == sheet_index && sheet_name.is_some() {
                *sheet_name = Some(new_name.to_owned());
            }
        }
        Node::RangeKind {
            sheet_name,
            sheet_index: index,
            ..
        } => {
            if *index == sheet_index && sheet_name.is_some() {
                *sheet_name = Some(new_name.to_owned());
            }
        }
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
                    *sheet_name = Some(name.to_owned())
                }
            }
        }
        Node::WrongRangeKind { sheet_name, .. } => {
            if sheet_name.is_some() {
                *sheet_name = Some(new_name.to_owned());
            }
        }

        // Go next level
        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            rename_sheet_in_node(left, sheet_index, new_name);
            rename_sheet_in_node(right, sheet_index, new_name);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| {
                rename_sheet_in_node(operand, sheet_index, new_name);
            });
        }
        Node::FunctionKind { kind: _, args } => {
            for arg in args {
                rename_sheet_in_node(arg, sheet_index, new_name);
            }
        }
        Node::NamedFunctionKind {
            name: _,
            args,
            id: _,
        } => {
            for arg in args {
                rename_sheet_in_node(arg, sheet_index, new_name);
            }
        }
        Node::UnaryKind { kind: _, right } => {
            rename_sheet_in_node(right, sheet_index, new_name);
        }
        Node::ImplicitIntersection { child, .. } => {
            rename_sheet_in_node(child, sheet_index, new_name);
        }
        Node::SpillRangeOperator { child }
        | Node::GroupedRootKind { child }
        | Node::Group { child, .. } => {
            rename_sheet_in_node(child, sheet_index, new_name);
        }

        // Do nothing
        Node::BooleanKind(_) => {}
        Node::NumberKind(_) => {}
        Node::StringKind(_) => {}
        Node::ErrorKind(_) => {}
        Node::ParseErrorKind { .. } => {}
        Node::ArrayKind(_) => {}
        Node::DefinedNameKind(_) => {}
        Node::TableNameKind(_) => {}
        Node::NamedVariableKind { .. } => {}
        Node::EmptyArgKind => {}
        Node::LambdaDefKind {
            parameters: _,
            body,
        } => {
            rename_sheet_in_node(body, sheet_index, new_name);
        }
        Node::LambdaCallKind { lambda, args } => {
            rename_sheet_in_node(lambda, sheet_index, new_name);
            for arg in args {
                rename_sheet_in_node(arg, sheet_index, new_name);
            }
        }
    }
}

pub(crate) fn rename_defined_name_in_node(
    node: &mut Node,
    name: &str,
    scope: Option<u32>,
    new_name: &str,
) {
    match node {
        // Rename
        Node::DefinedNameKind((n, s, _)) => {
            if name.to_lowercase() == n.to_lowercase() && *s == scope {
                *n = new_name.to_string();
            }
        }
        // Go next level
        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            rename_defined_name_in_node(left, name, scope, new_name);
            rename_defined_name_in_node(right, name, scope, new_name);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| {
                rename_defined_name_in_node(operand, name, scope, new_name);
            });
        }
        Node::FunctionKind { kind: _, args } => {
            for arg in args {
                rename_defined_name_in_node(arg, name, scope, new_name);
            }
        }
        Node::NamedFunctionKind {
            name: _,
            args,
            id: _,
        } => {
            for arg in args {
                rename_defined_name_in_node(arg, name, scope, new_name);
            }
        }
        Node::UnaryKind { kind: _, right } => {
            rename_defined_name_in_node(right, name, scope, new_name);
        }
        Node::ImplicitIntersection { child, .. } => {
            rename_defined_name_in_node(child, name, scope, new_name);
        }
        Node::SpillRangeOperator { child }
        | Node::GroupedRootKind { child }
        | Node::Group { child, .. } => {
            rename_defined_name_in_node(child, name, scope, new_name);
        }
        // Do nothing
        Node::BooleanKind(_) => {}
        Node::NumberKind(_) => {}
        Node::StringKind(_) => {}
        Node::ErrorKind(_) => {}
        Node::ParseErrorKind { .. } => {}
        Node::ArrayKind(_) => {}
        Node::EmptyArgKind => {}
        Node::ReferenceKind { .. } => {}
        Node::RangeKind { .. } => {}
        Node::SheetSpanKind { .. } => {}
        Node::WrongReferenceKind { .. } => {}
        Node::WrongRangeKind { .. } => {}
        Node::TableNameKind(_) => {}
        Node::NamedVariableKind { .. } => {}
        Node::LambdaDefKind {
            parameters: _,
            body,
        } => {
            rename_defined_name_in_node(body, name, scope, new_name);
        }
        Node::LambdaCallKind { lambda, args } => {
            rename_defined_name_in_node(lambda, name, scope, new_name);
            for arg in args {
                rename_defined_name_in_node(arg, name, scope, new_name);
            }
        }
    }
}
