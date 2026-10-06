use super::common::{broadcast_text, TextArg};
use crate::number_format::number_to_text;
use crate::{
    calc_result::CalcResult,
    cast::{array_node_to_string, exceeds_text_limit, truncate_text, MAX_TEXT_UNITS},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// Excel: CONCAT and TEXTJOIN text over 32767 characters is #CALC!, not truncated
/// (probes/text-result-limit C8, C9).
fn text_limit_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::CALC,
        cell,
        "Result is longer than 32767 characters".to_string(),
    )
}

/// Running UTF-16 size of a TEXTJOIN result, kept incrementally so each check is O(new values)
/// and an oversized join stops before it is built (CLAUDE.md directive 5).
#[derive(Default)]
struct JoinSize {
    counted: usize,
    units: usize,
}

impl JoinSize {
    fn exceeds(&mut self, values: &[String], delimiter: &str) -> bool {
        let delimiter_units = delimiter.encode_utf16().count();
        for value in &values[self.counted..] {
            let separator = if self.counted == 0 {
                0
            } else {
                delimiter_units
            };
            self.units = self
                .units
                .saturating_add(separator + value.encode_utf16().count());
            self.counted += 1;
        }
        self.units > MAX_TEXT_UNITS
    }
}

impl Model {
    /// CONCATENATE(text1, [text2], ...): joins text; array arguments lift element-wise.
    pub(crate) fn fn_concatenate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        // Array arguments lift element-wise (arrays.md "NOT and CONCATENATE lift over array
        // arguments"); with only scalars the result is the plain joined string.
        let mut operands = Vec::with_capacity(args.len());
        for arg in args {
            match self.text_arg(arg, cell) {
                Ok(operand) => operands.push(operand),
                Err(error) => return error,
            }
        }
        let operand_refs: Vec<&TextArg> = operands.iter().collect();
        broadcast_text(cell, &operand_refs, |i, j| {
            let mut joined = String::new();
            for operand in &operands {
                joined.push_str(&array_node_to_string(operand.elem(i, j))?);
                truncate_text(&mut joined);
            }
            Ok(ArrayNode::String(joined))
        })
    }

    pub(crate) fn fn_concat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut result = "".to_string();
        for arg in args {
            // Excel: joined text over 32767 characters is #CALC!, not truncated (probes/text-result-limit C8).
            if exceeds_text_limit(&result) {
                return text_limit_error(cell);
            }
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::String(value) => result = format!("{result}{value}"),
                CalcResult::Number(value) => result.push_str(&number_to_text(value)),
                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                CalcResult::Boolean(value) => {
                    if value {
                        result = format!("{result}TRUE");
                    } else {
                        result = format!("{result}FALSE");
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    for row in left.row..(right.row + 1) {
                        for column in left.column..(right.column + 1) {
                            if exceeds_text_limit(&result) {
                                return text_limit_error(cell);
                            }
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::String(value) => {
                                    result = format!("{result}{value}");
                                }
                                CalcResult::Number(value) => {
                                    result.push_str(&number_to_text(value))
                                }
                                CalcResult::Boolean(value) => {
                                    if value {
                                        result = format!("{result}TRUE");
                                    } else {
                                        result = format!("{result}FALSE");
                                    }
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                                CalcResult::Range { .. } => {}
                                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                                    return CalcResult::Error {
                                        error: Error::NIMPL,
                                        origin: cell,
                                        message: "Arrays not supported yet".to_string(),
                                    }
                                }
                            }
                        }
                    }
                }
                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };
        }
        if exceeds_text_limit(&result) {
            return text_limit_error(cell);
        }
        CalcResult::String(result)
    }

    // TEXTJOIN(delimiter, ignore_empty, text1, [text2], …)
    pub(crate) fn fn_textjoin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let delimiter = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let ignore_empty = match self.get_boolean(&args[1], cell) {
            Ok(b) => b,
            Err(error) => return error,
        };
        let mut values: Vec<String> = Vec::new();
        let mut size = JoinSize::default();
        for arg in &args[2..] {
            if size.exceeds(&values, &delimiter) {
                return text_limit_error(cell);
            }
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => values.push(number_to_text(value)),
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    let row1 = left.row;
                    let mut row2 = right.row;
                    let column1 = left.column;
                    let mut column2 = right.column;
                    match self.clip_to_used_area(left.sheet, row1, column1, row2, column2) {
                        Ok((r, c)) => {
                            row2 = r;
                            column2 = c;
                        }
                        Err(message) => return CalcResult::new_error(Error::ERROR, cell, message),
                    }
                    for row in row1..row2 + 1 {
                        for column in column1..(column2 + 1) {
                            if size.exceeds(&values, &delimiter) {
                                return text_limit_error(cell);
                            }
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(value) => {
                                    values.push(number_to_text(value));
                                }
                                CalcResult::String(value) => values.push(value),
                                CalcResult::Boolean(value) => {
                                    if value {
                                        values.push("TRUE".to_string())
                                    } else {
                                        values.push("FALSE".to_string())
                                    }
                                }
                                CalcResult::EmptyCell => {
                                    if !ignore_empty {
                                        values.push("".to_string())
                                    }
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::EmptyArg | CalcResult::Range { .. } => {}
                                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                                    return CalcResult::Error {
                                        error: Error::NIMPL,
                                        origin: cell,
                                        message: "Arrays not supported yet".to_string(),
                                    }
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::String(value) => values.push(value),
                CalcResult::Boolean(value) => {
                    if value {
                        values.push("TRUE".to_string())
                    } else {
                        values.push("FALSE".to_string())
                    }
                }
                CalcResult::EmptyCell => {
                    if !ignore_empty {
                        values.push("".to_string())
                    }
                }
                CalcResult::EmptyArg => {}
                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };
        }
        if size.exceeds(&values, &delimiter) {
            return text_limit_error(cell);
        }
        CalcResult::String(values.join(&delimiter))
    }
}
