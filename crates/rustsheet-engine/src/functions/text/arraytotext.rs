// DOLLAR, FIXED, NUMBERVALUE, PROPER, REPLACE, ARRAYTOTEXT

use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
    number_format::number_to_text,
};

impl Model {
    /// ARRAYTOTEXT(array, [format])
    /// format=0 (default, concise): values joined with "," (columns) and ";" (rows)
    /// format=1 (strict): same but wrapped in {} and strings are quoted
    pub(crate) fn fn_arraytotext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let format = if args.len() == 2 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f.floor() as i32,
                Err(e) => return e,
            }
        } else {
            0
        };
        if format != 0 && format != 1 {
            return CalcResult::new_error(Error::VALUE, cell, "format must be 0 or 1".to_string());
        }
        let strict = format == 1;

        let array = match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                // Excel: a 1x1 range holding an empty cell is as much an empty cell as a bare reference;
                // only a range, not an array (GROUPBY hands a one-blank group over as an array, which
                // gives "": probes/groupby-blank-values).
                Ok(array) if matches!(array.as_slice(), [row] if matches!(row.as_slice(), [ArrayNode::Empty])) =>
                {
                    return empty_cell_error(cell);
                }
                Ok(array) => array,
                Err(error) => return error,
            },
            CalcResult::Array(a) => a,
            // Excel: a scalar is a one-element array, so strict form braces it
            // (docs/excel-semantics/text.md#arraytotext-number-text, probes/arraytotext-number-text).
            CalcResult::Number(n) => {
                return scalar_text(&ArrayNode::Number(n), strict);
            }
            CalcResult::String(s) => {
                return scalar_text(&ArrayNode::String(s), strict);
            }
            CalcResult::Boolean(b) => {
                return scalar_text(&ArrayNode::Boolean(b), strict);
            }
            // Excel: a single empty cell (also a 1x1 range, INDEX/IF result or an omitted argument) is
            // #VALUE!, while an empty cell inside a larger range is empty text
            // (docs/excel-semantics/text.md#arraytotext-empty-cell, probes/arraytotext-empty-ref).
            CalcResult::EmptyCell | CalcResult::EmptyArg => {
                return empty_cell_error(cell);
            }
            error @ CalcResult::Error { .. } => return error,
            CalcResult::Lambda(_) => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Lambda not supported".to_string(),
                );
            }
        };

        let result = if strict {
            // Strict: {col1,col2;row2col1,row2col2}
            let rows: Vec<String> = array
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|v| format_array_value(v, true))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .collect();
            format!("{{{}}}", rows.join(";"))
        } else {
            // Concise: flatten all elements row-by-row, join with ", "
            let elements: Vec<String> = array
                .iter()
                .flat_map(|row| row.iter().map(|v| format_array_value(v, false)))
                .collect();
            elements.join(", ")
        };
        CalcResult::String(result.into())
    }
}

fn empty_cell_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "ARRAYTOTEXT of an empty cell".to_string(),
    )
}

fn scalar_text(node: &ArrayNode, strict: bool) -> CalcResult {
    let text = format_array_value(node, strict);
    CalcResult::String(if strict { format!("{{{text}}}") } else { text }.into())
}

fn format_array_value(node: &ArrayNode, strict: bool) -> String {
    match node {
        // Excel: General text, 15 significant digits, the text `&` and VALUETOTEXT give
        // (docs/excel-semantics/text.md#arraytotext-number-text).
        ArrayNode::Number(n) => number_to_text(*n),
        ArrayNode::Boolean(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        ArrayNode::String(s) => {
            if strict {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                (**s).to_owned()
            }
        }
        ArrayNode::Error(e) => format!("{e}"),
        ArrayNode::Empty => String::new(),
    }
}
