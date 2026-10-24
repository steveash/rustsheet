use crate::constants::LAST_ROW;
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::near_integer_count;
use crate::functions::Function;
use crate::limits::check_array_size;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    /// `=SEQUENCE(rows, [cols], [start], [step])`
    ///
    /// Returns a 2-D array of sequential numbers.
    ///   * rows  – number of rows (required, ≥ 1)
    ///   * cols  – number of columns (default 1)
    ///   * start – first value (default 1)
    ///   * step  – increment between values (default 1)
    pub(crate) fn fn_sequence(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Every argument lifts over an array and a range, one SEQUENCE per element whose value
        // is that call's top-left (SPEC-0011 "Array-valued functions", T-1834).
        self.lift_call(Function::Sequence, args, cell, &mut |model, values| {
            model.sequence_kernel(values, cell)
        })
    }

    /// One SEQUENCE call over argument values. A missing or empty argument takes its default.
    fn sequence_kernel(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        // Every argument is read first, so an error value in any of them beats an invalid extent
        // (`SEQUENCE(0,1,<#NUM!>,..)` is `#NUM!`, gen-m7 36809e9faa59553f A334); then rows is
        // checked, then columns: negative or past the grid `#VALUE!`, 0 `#CALC!` (probes/
        // m7-argument-modes F53), so `SEQUENCE(0,<negative>)` is `#CALC!` (c36ff5f42a1a4d47 A359)
        // and a rows of 1E+15 beats a zero columns (611ce3bbfab27cef A1065); arrays.md "SEQUENCE
        // and TAKE: an argument error before zero rows"; T-2316.
        let number = |value: Option<&CalcResult>| match value {
            None | Some(CalcResult::EmptyArg) => Ok(1.0),
            Some(value) => self.cast_number_ref(value, cell),
        };
        let mut read = [1.0; 4];
        for (index, slot) in read.iter_mut().enumerate() {
            match number(values.get(index)) {
                Ok(n) => *slot = n,
                Err(error) => return error,
            }
        }
        let [rows, columns, start, step] = read;
        // A negative extent or one past 1048576 (rows and columns alike, though the grid has 16384
        // columns) is `#VALUE!` and beats the `#CALC!` of a zero one: `SEQUENCE(0.0123,
        // 123456789.123)` and `SEQUENCE(0,-1)` are `#VALUE!`, `SEQUENCE(0,1048576)` `#CALC!`
        // A negative extent within the count tolerance of 0 is 0 (`SEQUENCE(1,-1E-9)` is `#CALC!`,
        // `SEQUENCE(1,-1E-6)` `#VALUE!`; probes/t2580-sequence-tiny-negative). Evidence:
        // (probes/t2580-sequence-zero-extent, t2580-sequence-column-cap; arrays.md "An empty array
        // as a function argument", T-2580).
        let extent = |n: f64| -> Result<usize, Error> {
            let n = near_integer_count(n) as i64;
            match usize::try_from(n) {
                Ok(n) if n <= LAST_ROW as usize => Ok(n),
                _ => Err(Error::VALUE),
            }
        };
        let (rows, columns) = match (extent(rows), extent(columns)) {
            (Err(error), _) | (_, Err(error)) => {
                return CalcResult::new_error(
                    error,
                    cell,
                    format!("rows and columns must be between 1 and {LAST_ROW}"),
                );
            }
            (Ok(0), _) | (_, Ok(0)) => {
                return CalcResult::new_error(
                    Error::CALC,
                    cell,
                    "rows or columns cannot be 0".to_string(),
                );
            }
            (Ok(rows), Ok(columns)) => (rows, columns),
        };
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }

        // Excel adds `step` to the previous value in row-major order, not `start + i·step`:
        // `SEQUENCE(5,1,10,2.2)` ends 14.399999999999999, 16.599999999999998, …
        // (docs/excel-semantics/numbers.md "SEQUENCE accumulates its step").
        let mut result = Vec::with_capacity(rows);
        let mut value = start;
        for _ in 0..rows {
            let mut row = Vec::with_capacity(columns);
            for _ in 0..columns {
                row.push(ArrayNode::Number(value));
                value += step;
            }
            result.push(row);
        }

        CalcResult::Array(result)
    }
}
