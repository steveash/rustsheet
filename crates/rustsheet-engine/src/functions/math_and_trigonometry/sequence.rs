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
        // An extent: 0 is `#CALC!` (probes/m7-argument-modes F53), a negative one `#VALUE!`.
        let extent = |value: Option<&CalcResult>, name: &str| -> Result<usize, CalcResult> {
            let n = match value {
                None | Some(CalcResult::EmptyArg) => return Ok(1),
                Some(value) => self.cast_number_ref(value, cell)?.floor() as i64,
            };
            match n {
                0 => Err(CalcResult::new_error(
                    Error::CALC,
                    cell,
                    format!("{name} must be >= 1"),
                )),
                n if n < 0 => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    format!("{name} must be >= 1"),
                )),
                n => Ok(usize::try_from(n).unwrap_or(usize::MAX)),
            }
        };
        let number = |value: Option<&CalcResult>| match value {
            None | Some(CalcResult::EmptyArg) => Ok(1.0),
            Some(value) => self.cast_number_ref(value, cell),
        };
        let rows = match extent(values.first(), "rows") {
            Ok(rows) => rows,
            Err(error) => return error,
        };
        let columns = match extent(values.get(1), "columns") {
            Ok(columns) => columns,
            Err(error) => return error,
        };
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        let start = match number(values.get(2)) {
            Ok(start) => start,
            Err(error) => return error,
        };
        let step = match number(values.get(3)) {
            Ok(step) => step,
            Err(error) => return error,
        };

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
