mod bound_variables;
use super::reference_operators::{
    at_text, callee_operand, group_text, intersection_operand, spill_operand, tight_operand,
    union_text,
};
use super::{
    super::utils::{name_needs_quoting, quote_name, rename_defined_name_sheet},
    for_each_chain_operand_mut, left_chain,
    move_formula::{moved_reference_text, MoveContext},
    value_operands, ArrayNode, Node, Reference,
};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::static_analysis::remove_redundant_implicit_intersection;
use crate::expressions::token::OpUnary;
use crate::language::{get_language, Language};
use crate::locale::{get_locale, Locale};
use crate::{expressions::types::CellReferenceRC, number_format::to_excel_precision_str};
use bound_variables::{prefix_bound_variables, variable_text};

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

/// How the printer rewrites the references of a formula: displaced by a structural edit, or
/// moved by a cut and paste. Everything else prints the same either way, so a moved formula
/// has the text FORMULATEXT shows (SPEC-0007 step 5).
enum Rewrite<'a> {
    Displace(&'a DisplaceData),
    Move(&'a MoveContext<'a>),
}

const NO_REWRITE: Rewrite<'static> = Rewrite::Displace(&DisplaceData::None);

/// This is the internal mode in IronCalc
/// Formulas internally are stored in R1C1 format, the locale and language are always "en"
pub fn to_rc_format(node: &Node) -> String {
    #[allow(clippy::expect_used)]
    let locale = get_locale("en").expect("");
    #[allow(clippy::expect_used)]
    let language = get_language("en").expect("");
    stringify(node, None, &NO_REWRITE, false, locale, language)
}

pub fn to_english_string(node: &Node, context: &CellReferenceRC) -> String {
    #[allow(clippy::expect_used)]
    let locale = get_locale("en").expect("");
    #[allow(clippy::expect_used)]
    let language = get_language("en").expect("");
    stringify(node, Some(context), &NO_REWRITE, false, locale, language)
}

/// This is the mode used to display the formula in the UI
pub fn to_localized_string(
    node: &Node,
    context: &CellReferenceRC,
    locale: &Locale,
    language: &Language,
) -> String {
    stringify(node, Some(context), &NO_REWRITE, false, locale, language)
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
    stringify(&node, Some(context), &NO_REWRITE, true, locale, language)
}

pub fn to_string_displaced(
    node: &Node,
    context: &CellReferenceRC,
    displace_data: &DisplaceData,
    locale: &Locale,
    language: &Language,
) -> String {
    let rewrite = Rewrite::Displace(displace_data);
    stringify(node, Some(context), &rewrite, false, locale, language)
}

/// The text of `node` cut from the cell `move_context` names and pasted by its deltas: the
/// references into the moved area follow it (see [`super::move_formula::move_formula`]).
pub(crate) fn to_string_moved(
    node: &Node,
    move_context: &MoveContext,
    locale: &Locale,
    language: &Language,
) -> String {
    let context = CellReferenceRC {
        sheet: move_context.source_sheet_name.to_string(),
        column: move_context.column,
        row: move_context.row,
    };
    let rewrite = Rewrite::Move(move_context);
    stringify(node, Some(&context), &rewrite, false, locale, language)
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
/// If `full_row` is true then the row details will be omitted in the A1 case
/// If `full_column` is true then column details will be omitted.
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
                row_abs = "".to_string();
            }
            if full_column {
                col_abs = "".to_string();
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

/// `number` as a formula prints it, with the decimal separator of `locale`.
fn format_number_locale(number: f64, locale: &Locale) -> String {
    let s = to_excel_precision_str(number);
    let decimal = &locale.numbers.symbols.decimal;
    if decimal == "." {
        s
    } else {
        s.replace('.', decimal)
    }
}

/// The printed element `node` of an array constant.
fn to_string_array_node(node: &ArrayNode, locale: &Locale, language: &Language) -> String {
    match node {
        ArrayNode::Boolean(value) => {
            if *value {
                language.booleans.r#true.to_uppercase()
            } else {
                language.booleans.r#false.to_uppercase()
            }
        }
        ArrayNode::Number(number) => format_number_locale(*number, locale),
        ArrayNode::String(value) => format!("\"{value}\""),
        ArrayNode::Error(kind) => format!("{kind}"),
        ArrayNode::Empty => "0".to_string(),
    }
}

fn format_function(
    name: &str,
    args: &Vec<Node>,
    context: Option<&CellReferenceRC>,
    rewrite: &Rewrite,
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
                stringify(el, context, rewrite, export_to_excel, locale, language)
            );
        } else {
            first = false;
            arguments = stringify(el, context, rewrite, export_to_excel, locale, language);
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

fn stringify(
    node: &Node,
    context: Option<&CellReferenceRC>,
    rewrite: &Rewrite,
    export_to_excel: bool,
    locale: &Locale,
    language: &Language,
) -> String {
    use self::Node::*;
    let displace_data = match rewrite {
        Rewrite::Displace(displace_data) => displace_data,
        Rewrite::Move(move_context) => {
            if let Some(text) = moved_reference_text(node, move_context) {
                return text;
            }
            &DisplaceData::None
        }
    };
    let sub = |n: &Node| stringify(n, context, rewrite, export_to_excel, locale, language);
    match node {
        BooleanKind(value) => {
            if *value {
                language.booleans.r#true.to_string()
            } else {
                language.booleans.r#false.to_string()
            }
        }
        NumberKind(number) => format_number_locale(*number, locale),
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
                &NO_REWRITE,
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
        OpIntersectKind { left, right } => {
            let operand = |n: &Node, is_left| intersection_operand(sub(n), n, is_left);
            format!("{} {}", operand(left, true), operand(right, false))
        }
        OpUnionKind { left, right, .. } => union_text(&sub(left), &sub(right), locale),
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
            rewrite,
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
                rewrite,
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
        DefinedNameKind((name, ..)) => name.to_string(),
        NamedVariableKind { name, id: _ } => variable_text(name, context),
        UnaryKind { kind, right } => {
            let operand = wrap(sub(right), binding(right) < binding(node));
            match kind {
                OpUnary::Minus => format!("-{operand}"),
                OpUnary::Percentage => format!("{operand}%"),
            }
        }
        ErrorKind(kind) => format!("{kind}"),
        ParseErrorKind { formula, .. } => formula.to_string(),
        EmptyArgKind => "".to_string(),
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
                        format!("[{}]", variable_text(&p.name, context))
                    } else {
                        variable_text(&p.name, context)
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
        } => {
            if export_to_excel {
                // Redundant operators have already been stripped by
                // `remove_redundant_implicit_intersection`; whatever remains is
                // meaningful and must be exported explicitly. A written group prints inside
                // SINGLE's own parentheses, `_xlfn.SINGLE((Z1+(Z2)))`, as Excel stores `=@(Z1+(Z2))`
                // (docs/excel-semantics/xlsx-storage.md, SPEC-0007).
                return format!("_xlfn.SINGLE({})", sub(child));
            }
            // The space binds tighter than `@`, so `@A1:A3 A2:B2` is `@(A1:A3 A2:B2)`; Excel's
            // FORMULATEXT prints it without parentheses (T-1098).
            at_text(child, &sub)
        }
    }
}

/// How tightly the printed `node` binds as an operand of a value operator, loosest first: the
/// parser's chain levels (`ChainLevel`), then `%`, then `-` (`-A1%` is `(-A1)%`, `-A1^2` is
/// `(-A1)^2`), then everything else (constants, references, reference operators, `@`, calls,
/// written groups), which never needs parentheses.
#[derive(PartialEq, PartialOrd)]
enum Binding {
    Compare,
    Concat,
    Sum,
    Product,
    Power,
    Percent,
    Minus,
    Operand,
}

fn binding(node: &Node) -> Binding {
    match node {
        Node::CompareKind { .. } => Binding::Compare,
        Node::OpConcatenateKind { .. } => Binding::Concat,
        Node::OpSumKind { .. } => Binding::Sum,
        Node::OpProductKind { .. } => Binding::Product,
        Node::OpPowerKind { .. } => Binding::Power,
        Node::UnaryKind {
            kind: OpUnary::Percentage,
            ..
        } => Binding::Percent,
        Node::UnaryKind {
            kind: OpUnary::Minus,
            ..
        } => Binding::Minus,
        _ => Binding::Operand,
    }
}

/// `text` in parentheses when `parens`.
fn wrap(text: String, parens: bool) -> String {
    if parens {
        format!("({text})")
    } else {
        text
    }
}

/// The text of value operator `node` (`+ - * / ^ &` or a comparison) whose left operand prints
/// as `left_text`: the operator and its right operand appended. The one precedence rule: an
/// operand gets parentheses only where the bare text would parse back to a different tree, a
/// looser left operand, or a right operand no tighter than `node` (the operators are left
/// associative). A parsed tree carries its written parentheses as [`Node::Group`]s, so it
/// prints as written: `=SUM(A1)^2`, `=2^-A1`, `=A1^2^3` (probes/operand-wrap-text,
/// probes/power-chain-text). One link of the loop that prints a chain (T-0128).
fn operator_text(node: &Node, left_text: String, sub: &dyn Fn(&Node) -> String) -> String {
    use self::Node::*;
    let Some((left, right)) = value_operands(node) else {
        return left_text;
    };
    let operator: &dyn std::fmt::Display = match node {
        OpConcatenateKind { .. } => &"&",
        CompareKind { kind, .. } => kind,
        OpSumKind { kind, .. } => kind,
        OpProductKind { kind, .. } => kind,
        OpPowerKind { .. } => &"^",
        _ => return left_text,
    };
    let level = binding(node);
    format!(
        "{}{operator}{}",
        wrap(left_text, binding(left) < level),
        wrap(sub(right), binding(right) <= level)
    )
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
                    *sheet_name = Some(name.to_owned());
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
        Node::SpillRangeOperator { child } | Node::Group { child, .. } => {
            rename_sheet_in_node(child, sheet_index, new_name);
        }

        // Do nothing
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::ArrayKind(_)
        | Node::NamedVariableKind { .. }
        | Node::EmptyArgKind => {}
        Node::DefinedNameKind((n, s, _)) => rename_defined_name_sheet(n, *s, sheet_index, new_name),
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
        Node::DefinedNameKind((n, s, _)) => {
            let prefix_len = n.len() - crate::expressions::utils::bare_defined_name(n).len();
            if name.to_lowercase() == n[prefix_len..].to_lowercase() && *s == scope {
                n.replace_range(prefix_len.., new_name); // `Sheet!` stays as typed
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
        Node::SpillRangeOperator { child } | Node::Group { child, .. } => {
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
