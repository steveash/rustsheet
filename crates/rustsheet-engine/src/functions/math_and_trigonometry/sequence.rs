use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
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
        let extent = |n: f64, last: usize, name: &str| -> Result<usize, CalcResult> {
            let n = n.floor() as i64;
            let error = match n {
                0 => Error::CALC,
                n if n < 0 => Error::VALUE,
                n => match usize::try_from(n) {
                    Ok(n) if n <= last => return Ok(n),
                    _ => Error::VALUE,
                },
            };
            Err(CalcResult::new_error(
                error,
                cell,
                format!("{name} must be between 1 and {last}"),
            ))
        };
        let rows = match extent(rows, LAST_ROW as usize, "rows") {
            Ok(rows) => rows,
            Err(error) => return error,
        };
        let columns = match extent(columns, LAST_COLUMN as usize, "columns") {
            Ok(columns) => columns,
            Err(error) => return error,
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
