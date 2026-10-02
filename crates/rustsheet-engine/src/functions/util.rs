use crate::{
    calc_result::CalcResult,
    expressions::{token::Error, types::CellReferenceIndex},
    model::Model,
    number_format::to_excel_precision,
};

fn error_sort_rank(e: &Error) -> u8 {
    match e {
        Error::NULL => 1,
        Error::DIV => 2,
        Error::VALUE => 3,
        Error::REF => 4,
        Error::NAME => 5,
        Error::NUM => 6,
        Error::NA => 7,
        _ => 8,
    }
}

/// This test for exact match (modulo case).
///   * strings are not cast into bools or numbers
///   * empty cell is not cast into empty string or zero
pub(crate) fn values_are_equal(left: &CalcResult, right: &CalcResult) -> bool {
    match (left, right) {
        (CalcResult::Number(value1), CalcResult::Number(value2)) => {
            if (value2 - value1).abs() < f64::EPSILON {
                return true;
            }
            false
        }
        (CalcResult::String(value1), CalcResult::String(value2)) => {
            let value1 = value1.to_uppercase();
            let value2 = value2.to_uppercase();
            value1 == value2
        }
        (CalcResult::Boolean(value1), CalcResult::Boolean(value2)) => value1 == value2,
        (CalcResult::EmptyCell, CalcResult::EmptyCell) => true,
        // NOTE: Errors and Ranges are not covered
        (_, _) => false,
    }
}

// In Excel there are two ways of comparing cell values.
// The old school comparison valid in formulas like D3 < D4 or HLOOKUP,... cast empty cells into empty strings or 0
// For the new formulas like XLOOKUP or SORT an empty cell is always larger than anything else.

// ..., -2, -1, 0, 1, 2, ..., A-Z, FALSE, TRUE;
pub(crate) fn compare_values(left: &CalcResult, right: &CalcResult) -> i32 {
    match (left, right) {
        (CalcResult::Number(value1), CalcResult::Number(value2)) => {
            let value1 = to_excel_precision(*value1, 15);
            let value2 = to_excel_precision(*value2, 15);
            if (value2 - value1).abs() < f64::EPSILON {
                return 0;
            }
            if value1 < value2 {
                return -1;
            }
            1
        }
        (CalcResult::Number(_value1), CalcResult::String(_value2)) => -1,
        (CalcResult::Number(_value1), CalcResult::Boolean(_value2)) => -1,
        (CalcResult::String(value1), CalcResult::String(value2)) => {
            let value1 = value1.to_uppercase();
            let value2 = value2.to_uppercase();
            match value1.cmp(&value2) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }
        (CalcResult::String(_value1), CalcResult::Boolean(_value2)) => -1,
        (CalcResult::Boolean(value1), CalcResult::Boolean(value2)) => {
            if value1 == value2 {
                return 0;
            }
            if *value1 {
                return 1;
            }
            -1
        }
        (CalcResult::EmptyCell, CalcResult::String(_value2)) => {
            compare_values(&CalcResult::String("".to_string()), right)
        }
        (CalcResult::String(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::String("".to_string()))
        }
        (CalcResult::EmptyCell, CalcResult::Number(_value2)) => {
            compare_values(&CalcResult::Number(0.0), right)
        }
        (CalcResult::Number(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::Number(0.0))
        }
        (CalcResult::EmptyCell, CalcResult::Boolean(_)) => {
            compare_values(&CalcResult::Boolean(false), right)
        }
        (CalcResult::Boolean(_), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::Boolean(false))
        }
        (CalcResult::EmptyCell, CalcResult::EmptyCell) => 0,
        // Errors sort after everything else, ordered by Excel's canonical rank
        (CalcResult::Error { error: e1, .. }, CalcResult::Error { error: e2, .. }) => {
            let r1 = error_sort_rank(e1);
            let r2 = error_sort_rank(e2);
            if r1 < r2 {
                -1
            } else if r1 > r2 {
                1
            } else {
                0
            }
        }
        (CalcResult::Error { .. }, _) => 1,
        (_, CalcResult::Error { .. }) => -1,
        (_, _) => 1,
    }
}

impl Model<'_> {
    /// Calls `f` with the value of every cell of the range `left..=right`, in
    /// row-major order.
    ///
    /// The one range walk behind every aggregate function. Ranges spanning two
    /// sheets are `#VALUE!`; whole-row and whole-column ranges are clipped to
    /// the used area so the walk stays proportional to the data. The first
    /// error cell aborts the walk and is returned as `Err`, as does an `Err`
    /// from `f`. `f` never sees `CalcResult::Error`.
    pub(crate) fn for_each_cell_in_range<F>(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CalcResult) -> Result<(), CalcResult>,
    {
        if left.sheet != right.sheet {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Ranges are in different sheets".to_string(),
            ));
        }
        let (row2, column2) = self
            .clip_to_used_area(left.sheet, left.row, left.column, right.row, right.column)
            .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
        for row in left.row..=row2 {
            for column in left.column..=column2 {
                match self.evaluate_cell(CellReferenceIndex {
                    sheet: left.sheet,
                    row,
                    column,
                }) {
                    error @ CalcResult::Error { .. } => return Err(error),
                    value => f(self, value)?,
                }
            }
        }
        Ok(())
    }

    /// Calls `f` with every number held by the cells of the range
    /// `left..=right`, in row-major order.
    ///
    /// This is how aggregate functions (SUM, NPV, VAR, ...) read a
    /// *reference*: numbers count, booleans, text and empty cells are skipped
    /// (unlike direct arguments, which the callers coerce themselves). See
    /// [`Self::for_each_cell_in_range`] for errors and range clipping.
    pub(crate) fn for_each_number_in_range<F>(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        self.for_each_cell_in_range(left, right, cell, |_, value| {
            // Booleans, text and empty cells in a reference are ignored.
            if let CalcResult::Number(value) = value {
                f(value);
            }
            Ok(())
        })
    }

    /// Like [`Self::for_each_number_in_range`] but with the coercion of the
    /// *A functions (VARA, STDEVA, ...): text counts as 0, TRUE/FALSE as 1/0,
    /// empty cells are skipped.
    pub(crate) fn for_each_coerced_number_in_range<F>(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        self.for_each_cell_in_range(left, right, cell, |_, value| {
            match value {
                CalcResult::Number(value) => f(value),
                CalcResult::String(_) => f(0.0),
                CalcResult::Boolean(value) => f(if value { 1.0 } else { 0.0 }),
                _ => {}
            }
            Ok(())
        })
    }
}
