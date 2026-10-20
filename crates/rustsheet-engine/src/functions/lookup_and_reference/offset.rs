//! OFFSET: a reference moved and resized from another, lifted into an array of references when
//! rows, cols, height or width is an array or a multi-cell range (SPEC-0014; T-1956).

use super::ref_lift::lifts;
use crate::calc_result::{CalcResult, Range};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::Node;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::ArgBuf;
use crate::model::Model;
use crate::reference_operators::RefArg;

impl Model {
    // OFFSET(reference, rows, cols, [height], [width])
    // Returns a reference to a range that is a specified number of rows and columns from a cell or range of cells.
    // The reference that is returned can be a single cell or a range of cells.
    // You can specify the number of rows and the number of columns to be returned.
    pub(crate) fn fn_offset(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.offset_reference(args, cell).into_single_area(cell)
    }

    /// OFFSET in reference context: one reference, or, when rows, cols, height or width is an
    /// array or a multi-cell range, an array of references in their broadcast shape, each
    /// element the scalar OFFSET of its numbers or that call's error. A scalar argument's error
    /// does not stop the lift: `ROWS(OFFSET(A1,{0;1},NA()))` is 2 and each element `#N/A`
    /// (probes/reference-lift-elements F4, G4; docs/excel-semantics/lookup.md "OFFSET, INDIRECT
    /// and TRIMRANGE lift into arrays of references").
    pub(crate) fn offset_reference(&mut self, args: &[Node], cell: CellReferenceIndex) -> RefArg {
        if !(3..=5).contains(&args.len()) {
            return RefArg::Value(CalcResult::new_args_number_error(cell));
        }
        // OFFSET of a reference into another workbook is `#VALUE!`, not a reference
        // (`ISREF(OFFSET(…!A1,1,1))` is FALSE; probes/external-ref-no-link-shapes A5,
        // external-ref-no-link-index A16, A17, A32).
        // An error in `rows`, `cols` or `width` comes first: `OFFSET(…!A1,1/0,1)`,
        // `OFFSET(…!A1,1,1/0)` and `OFFSET(…!A1,1,1,1,1/0)` are `#DIV/0!`, but a height of 0 is
        // still `#VALUE!` (probes/external-ref-no-link-ifchoose A7, -ifchoose-b A2, A3, A6).
        if let Some(areas) = self.external_reference(&args[0], cell) {
            if let Err(error) = areas {
                return RefArg::Value(error);
            }
            for arg in &args[1..] {
                if let Err(error) = self.get_number(arg, cell) {
                    return RefArg::Value(error);
                }
            }
            return RefArg::Value(CalcResult::new_error(
                Error::VALUE,
                cell,
                "OFFSET of an external reference".to_string(),
            ));
        }
        let reference = match self.get_reference(&args[0], cell) {
            Ok(reference) => reference,
            Err(error) => return RefArg::Value(error),
        };
        let values = self.offset_values(args, cell);
        if !values.as_slice().iter().any(lifts) {
            return RefArg::from(self.offset_of(&reference, values.as_slice(), cell));
        }
        let lifted = match self.reference_lift_args(values, cell) {
            Ok(lifted) => lifted,
            Err(error) => return RefArg::Value(error),
        };
        self.lift_references(lifted, cell, &mut |model, values| {
            model.offset_of(&reference, values, cell)
        })
    }

    /// The values of rows, cols, height and width, held inline (no heap allocation for the
    /// scalar call, SPEC-0014 "Producers"); a written empty argument is `EmptyArg`, so a missing
    /// or empty height/width keeps the reference's size.
    fn offset_values(&mut self, args: &[Node], cell: CellReferenceIndex) -> ArgBuf<CalcResult> {
        self.reference_lift_values(args.get(1..).unwrap_or_default(), cell)
    }

    /// Whether an OFFSET call lifts into an array of references, its own arguments evaluated
    /// but no element computed: ISREF's question (probes/reference-lift-elements F15).
    pub(crate) fn offset_lifts(&mut self, args: &[Node], cell: CellReferenceIndex) -> bool {
        self.offset_values(args, cell).as_slice().iter().any(lifts)
    }

    /// One of OFFSET's numbers, read by reference: only a reference or an array is cast by value
    /// (dereferenced as `cast_to_number` does), so a text argument is never copied.
    fn offset_number(
        &mut self,
        value: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        match value {
            CalcResult::Range { .. } | CalcResult::Array(_) | CalcResult::Lambda(_) => {
                self.cast_to_number(value.clone(), cell)
            }
            value => self.cast_number_ref(value, cell),
        }
    }

    /// The scalar OFFSET of `reference` by the values of rows, cols and the optional height and
    /// width (`EmptyArg` when missing): the moved reference, or the first error in that order.
    fn offset_of(
        &mut self,
        reference: &Range,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // Rows and columns truncate toward zero.
        let count = |model: &mut Self, value: Option<&CalcResult>| {
            let number = model.offset_number(value.unwrap_or(&CalcResult::EmptyArg), cell)?;
            Ok::<i32, CalcResult>(number.trunc() as i32)
        };
        let rows = match count(self, values.first()) {
            Ok(rows) => rows,
            Err(error) => return error,
        };
        let cols = match count(self, values.get(1)) {
            Ok(cols) => cols,
            Err(error) => return error,
        };
        // `as i32` saturates, so sums saturate too (untrusted input must not overflow): any such
        // result is outside the sheet and falls through to #REF!.
        let row_start = reference.left.row.saturating_add(rows);
        let column_start = reference.left.column.saturating_add(cols);
        // A missing or empty height/width keeps the reference's size
        // (poi/FormulaEvalTestData_Copy.xlsx EverythingTests!I1008, J1008, M1008, N1008).
        let height = match self.offset_size(
            values.get(2),
            reference.right.row - reference.left.row,
            cell,
        ) {
            Ok(h) => h,
            Err(e) => return e,
        };
        let width = match self.offset_size(
            values.get(3),
            reference.right.column - reference.left.column,
            cell,
        ) {
            Ok(w) => w,
            Err(e) => return e,
        };
        let (Some(height), Some(width)) = (height, width) else {
            // A size of exactly 0 is #REF!, not an empty area.
            return CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message: "Invalid reference".to_string(),
            };
        };
        // A negative size extends the area up/left from the anchor cell.
        let column_end = column_start.saturating_add(width);
        let row_end = row_start.saturating_add(height);
        let (row_start, row_end) = (row_start.min(row_end), row_start.max(row_end));
        let (column_start, column_end) =
            (column_start.min(column_end), column_start.max(column_end));
        if row_start < 1 || row_end > LAST_ROW || column_start < 1 || column_end > LAST_COLUMN {
            return CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message: "Invalid reference".to_string(),
            };
        }
        let left = CellReferenceIndex {
            sheet: reference.left.sheet,
            row: row_start,
            column: column_start,
        };
        let right = CellReferenceIndex {
            sheet: reference.right.sheet,
            row: row_end,
            column: column_end,
        };
        CalcResult::Range { left, right }
    }

    /// Height or width argument of OFFSET as the signed distance from the anchor cell to the far
    /// edge of the area; `default` when the argument is missing or empty, `None` for exactly 0.
    ///
    /// Excel: see docs/excel-semantics/references.md "OFFSET negative and fractional height/width".
    /// A positive size `h` reaches `floor(h - 1)` cells on, a negative one `ceil(h + 1)` cells (so
    /// -3 reaches 2 back), and the fractions in (-1, 1) reach one cell back and on respectively.
    fn offset_size(
        &mut self,
        value: Option<&CalcResult>,
        default: i32,
        cell: CellReferenceIndex,
    ) -> Result<Option<i32>, CalcResult> {
        match value {
            None | Some(CalcResult::EmptyArg) => Ok(Some(default)),
            Some(value) => {
                let c = self.offset_number(value, cell)?;
                Ok(if c > 0.0 {
                    Some((c - 1.0).floor() as i32)
                } else if c < 0.0 {
                    Some((c + 1.0).ceil() as i32)
                } else {
                    None
                })
            }
        }
    }
}
