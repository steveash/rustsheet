use super::{
    left_chain,
    reference_operators::{at_text, group_text, intersection_operand, tight_operand, union_text},
    stringify::{sheet_span_text, stringify_reference, to_localized_string, DisplaceData},
    value_operands, ArrayNode, Node, Reference,
};
use crate::{
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::token::OpUnary,
    language::Language,
    locale::Locale,
};
use crate::{
    expressions::types::{Area, CellReferenceRC},
    number_format::to_excel_precision_str,
};

pub(crate) fn ref_is_in_area(sheet: u32, row: i32, column: i32, area: &Area) -> bool {
    if area.sheet != sheet {
        return false;
    }
    if row < area.row || row > area.row + area.height - 1 {
        return false;
    }
    if column < area.column || column > area.column + area.width - 1 {
        return false;
    }
    true
}

pub(crate) struct MoveContext<'a> {
    pub source_sheet_name: &'a str,
    pub row: i32,
    pub column: i32,
    pub area: &'a Area,
    pub target_sheet_name: &'a str,
    pub row_delta: i32,
    pub column_delta: i32,
}

/// This implements Excel's cut && paste
/// We are moving a formula in (row, column) to (row+row_delta, column + column_delta).
/// All references that do not point to a cell in area will be left untouched.
/// All references that point to a cell in area will be displaced
pub(crate) fn move_formula(
    node: &Node,
    move_context: &MoveContext,
    locale: &Locale,
    language: &Language,
) -> String {
    to_string_moved(node, move_context, locale, language)
}

fn move_function(
    name: &str,
    args: &Vec<Node>,
    move_context: &MoveContext,
    locale: &Locale,
    language: &Language,
) -> String {
    let mut first = true;
    let mut arguments = "".to_string();
    for el in args {
        if !first {
            arguments = format!(
                "{},{}",
                arguments,
                to_string_moved(el, move_context, locale, language)
            );
        } else {
            first = false;
            arguments = to_string_moved(el, move_context, locale, language);
        }
    }
    format!("{name}({arguments})")
}

fn format_number_locale(number: f64, locale: &Locale) -> String {
    let s = to_excel_precision_str(number);
    let decimal = &locale.numbers.symbols.decimal;
    if decimal == "." {
        s
    } else {
        s.replace('.', decimal)
    }
}

pub(crate) fn to_string_array_node(
    node: &ArrayNode,
    locale: &Locale,
    language: &Language,
) -> String {
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

fn to_string_moved(
    node: &Node,
    move_context: &MoveContext,
    locale: &Locale,
    language: &Language,
) -> String {
    use self::Node::*;
    match node {
        BooleanKind(value) => format!("{value}").to_uppercase(),
        NumberKind(number) => format_number_locale(*number, locale),
        StringKind(value) => format!("\"{value}\""),
        ReferenceKind {
            sheet_name,
            sheet_index,
            absolute_row,
            absolute_column,
            row,
            column,
        } => {
            let reference_row = if *absolute_row {
                *row
            } else {
                row + move_context.row
            };
            let reference_column = if *absolute_column {
                *column
            } else {
                column + move_context.column
            };

            let new_row;
            let new_column;
            let mut ref_sheet_name = sheet_name;
            let source_sheet_name = &Some(move_context.source_sheet_name.to_string());

            if ref_is_in_area(
                *sheet_index,
                reference_row,
                reference_column,
                move_context.area,
            ) {
                // if the reference is in the area we are moving we want to displace the reference
                new_row = row + move_context.row_delta;
                new_column = column + move_context.column_delta;
            } else {
                // If the reference is not in the area we are moving the reference remains unchanged
                new_row = *row;
                new_column = *column;
                if move_context.target_sheet_name != move_context.source_sheet_name
                    && sheet_name.is_none()
                {
                    ref_sheet_name = source_sheet_name;
                }
            };
            let context = CellReferenceRC {
                sheet: move_context.source_sheet_name.to_string(),
                column: move_context.column,
                row: move_context.row,
            };
            stringify_reference(
                Some(&context),
                &DisplaceData::None,
                &Reference {
                    sheet_name: ref_sheet_name,
                    sheet_index: *sheet_index,
                    absolute_row: *absolute_row,
                    absolute_column: *absolute_column,
                    row: new_row,
                    column: new_column,
                },
                false,
                false,
            )
        }
        // A cut-and-paste moves cells on one sheet, so it never moves a whole 3-D span: the
        // reference stays as written (edit semantics of 3-D references are not modelled yet).
        SheetSpanKind {
            first_sheet,
            last_sheet,
            reference,
            ..
        } => {
            let context = CellReferenceRC {
                sheet: move_context.source_sheet_name.to_string(),
                column: move_context.column,
                row: move_context.row,
            };
            format!(
                "{}!{}",
                sheet_span_text(first_sheet, last_sheet),
                to_localized_string(reference, &context, locale, language)
            )
        }
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
            let full_row = *absolute_row1 && *absolute_row2 && (*row1 == 1) && (*row2 == LAST_ROW);
            let full_column = *absolute_column1
                && *absolute_column2
                && (*column1 == 1)
                && (*column2 == LAST_COLUMN);

            let reference_row1 = if *absolute_row1 {
                *row1
            } else {
                row1 + move_context.row
            };
            let reference_column1 = if *absolute_column1 {
                *column1
            } else {
                column1 + move_context.column
            };

            let reference_row2 = if *absolute_row2 {
                *row2
            } else {
                row2 + move_context.row
            };
            let reference_column2 = if *absolute_column2 {
                *column2
            } else {
                column2 + move_context.column
            };

            let new_row1;
            let new_column1;
            let new_row2;
            let new_column2;
            let mut ref_sheet_name = sheet_name;
            let source_sheet_name = &Some(move_context.source_sheet_name.to_string());
            if ref_is_in_area(
                *sheet_index,
                reference_row1,
                reference_column1,
                move_context.area,
            ) && ref_is_in_area(
                *sheet_index,
                reference_row2,
                reference_column2,
                move_context.area,
            ) {
                // if the whole range is inside the area we are moving we want to displace the context
                new_row1 = row1 + move_context.row_delta;
                new_column1 = column1 + move_context.column_delta;
                new_row2 = row2 + move_context.row_delta;
                new_column2 = column2 + move_context.column_delta;
            } else {
                // If the reference is not in the area we are moving the context remains unchanged
                new_row1 = *row1;
                new_column1 = *column1;
                new_row2 = *row2;
                new_column2 = *column2;
                if move_context.target_sheet_name != move_context.source_sheet_name
                    && sheet_name.is_none()
                {
                    ref_sheet_name = source_sheet_name;
                }
            };
            let context = CellReferenceRC {
                sheet: move_context.source_sheet_name.to_string(),
                column: move_context.column,
                row: move_context.row,
            };
            let s1 = stringify_reference(
                Some(&context),
                &DisplaceData::None,
                &Reference {
                    sheet_name: ref_sheet_name,
                    sheet_index: *sheet_index,
                    absolute_row: *absolute_row1,
                    absolute_column: *absolute_column1,
                    row: new_row1,
                    column: new_column1,
                },
                full_row,
                full_column,
            );
            let s2 = stringify_reference(
                Some(&context),
                &DisplaceData::None,
                &Reference {
                    sheet_name: &None,
                    sheet_index: *sheet_index,
                    absolute_row: *absolute_row2,
                    absolute_column: *absolute_column2,
                    row: new_row2,
                    column: new_column2,
                },
                full_row,
                full_column,
            );
            format!("{s1}:{s2}")
        }
        WrongReferenceKind {
            sheet_name,
            absolute_row,
            absolute_column,
            row,
            column,
        } => {
            // NB: Excel does not displace wrong references but Google Docs does. We follow Excel
            let context = CellReferenceRC {
                sheet: move_context.source_sheet_name.to_string(),
                column: move_context.column,
                row: move_context.row,
            };
            // It's a wrong reference, so there is no valid `sheet_index`.
            // We don't need it, since the `sheet_index` is only used if `displace_data` is not `None`.
            // I should fix it, maybe putting the `sheet_index` inside the `displace_data`
            stringify_reference(
                Some(&context),
                &DisplaceData::None,
                &Reference {
                    sheet_name,
                    sheet_index: 0, // HACK
                    row: *row,
                    column: *column,
                    absolute_row: *absolute_row,
                    absolute_column: *absolute_column,
                },
                false,
                false,
            )
        }
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
            let full_row = *absolute_row1 && *absolute_row2 && (*row1 == 1) && (*row2 == LAST_ROW);
            let full_column = *absolute_column1
                && *absolute_column2
                && (*column1 == 1)
                && (*column2 == LAST_COLUMN);

            // NB: Excel does not displace wrong references but Google Docs does. We follow Excel
            let context = CellReferenceRC {
                sheet: move_context.source_sheet_name.to_string(),
                column: move_context.column,
                row: move_context.row,
            };
            let s1 = stringify_reference(
                Some(&context),
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
                Some(&context),
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
            let operand = |n| tight_operand(to_string_moved(n, move_context, locale, language), n);
            format!("{}:{}", operand(left), operand(right))
        }
        OpIntersectKind { left, right, .. } => {
            let operand = |n, is_left| {
                let text = to_string_moved(n, move_context, locale, language);
                intersection_operand(text, n, is_left)
            };
            format!("{} {}", operand(left, true), operand(right, false))
        }
        OpUnionKind { left, right, .. } => {
            let text = |n| to_string_moved(n, move_context, locale, language);
            union_text(node, &text(left), &text(right), locale)
        }
        OpConcatenateKind { .. }
        | OpSumKind { .. }
        | OpProductKind { .. }
        | OpPowerKind { .. }
        | CompareKind { .. } => {
            // A loop down the chain's left operands, not recursion (T-0128).
            let (links, first) = left_chain(node);
            let mut text = to_string_moved(first, move_context, locale, language);
            for link in links {
                text = moved_operator_text(link, text, move_context, locale, language);
            }
            text
        }
        NamedFunctionKind { name, args, id: _ } => {
            move_function(name, args, move_context, locale, language)
        }
        FunctionKind { kind, args } => {
            let name = &kind.to_localized_name(language);
            move_function(name, args, move_context, locale, language)
        }
        ArrayKind(args) => {
            let mut first_row = true;
            let mut matrix_string = String::new();

            // Each element in `args` is assumed to be one "row" (itself a `Vec<T>`).
            let row_separator = if locale.numbers.symbols.decimal == "." {
                ';'
            } else {
                '/'
            };
            let col_separator = if row_separator == ';' { ',' } else { ';' };
            for row in args {
                if !first_row {
                    matrix_string.push(col_separator);
                } else {
                    first_row = false;
                }

                // Build the string for the current row
                let mut first_col = true;
                let mut row_string = String::new();
                for el in row {
                    if !first_col {
                        row_string.push(row_separator);
                    } else {
                        first_col = false;
                    }

                    // Reuse your existing element-stringification function
                    row_string.push_str(&to_string_array_node(el, locale, language));
                }

                // Enclose the row in braces
                matrix_string.push('{');
                matrix_string.push_str(&row_string);
                matrix_string.push('}');
            }

            // Enclose the whole matrix in braces
            format!("{{{matrix_string}}}")
        }
        DefinedNameKind((name, ..)) => name.to_string(),
        TableNameKind(name) => name.to_string(),
        NamedVariableKind { name, id: _ } => name.to_string(),
        UnaryKind { kind, right } => match kind {
            OpUnary::Minus => {
                let x = to_string_moved(right, move_context, locale, language);
                // What binds less than the minus needs its parentheses:
                // `-(A1=B1)` is not `-A1=B1`
                let needs_parentheses = matches!(
                    **right,
                    OpPowerKind { .. }
                        | OpSumKind { .. }
                        | OpConcatenateKind { .. }
                        | CompareKind { .. }
                        | UnaryKind {
                            kind: OpUnary::Percentage,
                            ..
                        }
                );
                if needs_parentheses {
                    format!("-({x})")
                } else {
                    format!("-{x}")
                }
            }
            OpUnary::Percentage => format!(
                "{}%",
                to_string_moved(right, move_context, locale, language)
            ),
        },
        ErrorKind(kind) => format!("{kind}"),
        ParseErrorKind { formula, .. } => formula.to_string(),
        EmptyArgKind => "".to_string(),
        ImplicitIntersection {
            automatic: _,
            child,
            written,
            lead,
            grouped: _,
        } => {
            let sub = |n: &Node| to_string_moved(n, move_context, locale, language);
            // Same text as `stringify`: the space binds tighter than `@` (T-1098).
            at_text(child, *written, *lead, &sub)
        }
        SpillRangeOperator { child } => {
            let text = to_string_moved(child, move_context, locale, language);
            format!("{}#", tight_operand(text, child))
        }
        Group { levels, child } => group_text(
            *levels,
            &to_string_moved(child, move_context, locale, language),
        ),
        GroupedRootKind { child } => {
            format!(
                "({})",
                to_string_moved(child, move_context, locale, language)
            )
        }
        LambdaDefKind { parameters, body } => {
            let mut parts: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();
            parts.push(to_string_moved(body, move_context, locale, language));
            format!("LAMBDA({})", parts.join(","))
        }
        LambdaCallKind { lambda, args } => {
            let lambda_str = to_string_moved(lambda, move_context, locale, language);
            let call_args: Vec<String> = args
                .iter()
                .map(|a| to_string_moved(a, move_context, locale, language))
                .collect();
            format!("{}({})", lambda_str, call_args.join(","))
        }
    }
}

/// The text of moved value operator `node` whose left operand prints as `left_text`: one link
/// of the loop that prints a chain (T-0128).
fn moved_operator_text(
    node: &Node,
    left_text: String,
    move_context: &MoveContext,
    locale: &Locale,
    language: &Language,
) -> String {
    use self::Node::*;
    let Some((left, _)) = value_operands(node) else {
        return left_text;
    };
    match node {
        OpConcatenateKind { right, .. } => format!(
            "{left_text}&{}",
            to_string_moved(right, move_context, locale, language),
        ),
        OpSumKind { kind, right, .. } => format!(
            "{left_text}{}{}",
            kind,
            to_string_moved(right, move_context, locale, language),
        ),
        OpProductKind { kind, right, .. } => {
            let x = match left {
                OpSumKind { .. } | CompareKind { .. } => format!("({left_text})"),
                _ => left_text,
            };
            let y = match **right {
                OpSumKind { .. } => format!(
                    "({})",
                    to_string_moved(right, move_context, locale, language)
                ),
                CompareKind { .. } => format!(
                    "({})",
                    to_string_moved(right, move_context, locale, language)
                ),
                OpProductKind { .. } => format!(
                    "({})",
                    to_string_moved(right, move_context, locale, language)
                ),
                UnaryKind { .. } => {
                    format!(
                        "({})",
                        to_string_moved(right, move_context, locale, language)
                    )
                }
                _ => to_string_moved(right, move_context, locale, language),
            };
            format!("{x}{kind}{y}")
        }
        OpPowerKind { right, .. } => format!(
            "{left_text}^{}",
            to_string_moved(right, move_context, locale, language),
        ),
        CompareKind { kind, right, .. } => format!(
            "{left_text}{}{}",
            kind,
            to_string_moved(right, move_context, locale, language),
        ),
        _ => left_text,
    }
}
