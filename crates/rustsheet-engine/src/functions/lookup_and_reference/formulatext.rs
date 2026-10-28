//! FORMULATEXT: the formula text of the cell or cells a reference argument stands for, a pick
//! call read as the reference it picks (T-2726).

use crate::calc_result::Range;
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::text::util::within_text_budget;
use crate::limits::check_range_size;
use crate::reference_operators::{
    not_a_reference_array, several_areas, Ranges, RefArg, RefElement, RefGrid,
};
use crate::{calc_result::CalcResult, model::Model};

impl Model {
    // FORMULATEXT(reference)
    // Returns a formula as a string. Two differences with Excel:
    // - It returns the formula in English
    // - It formats the formula without spaces between elements
    pub(crate) fn fn_formulatext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // A call that can return a reference is read as SUBTOTAL reads it: the arm or cells it
        // picks. A bare union is read the same way: its first area is the reference.
        let picks_reference = match args[0].ungrouped() {
            Node::FunctionKind { kind, .. } => kind.may_be_reference(),
            node => matches!(node, Node::OpUnionKind { .. }),
        };
        let reference = match args[0].ungrouped() {
            _ if picks_reference => {
                match self.evaluate_picked_reference_arg(&args[0], cell) {
                    RefArg::Ranges(Ranges::One(Range { left, right })) => {
                        CalcResult::Range { left, right }
                    }
                    // A multi-area reference is FORMULATEXT of its first area, which spills when it
                    // covers several cells (probes/t2831-formulatext-indirect-multi-cell A33-A78;
                    // docs/excel-semantics/references.md "FORMULATEXT of a multi-area reference
                    // and of a multi-cell INDIRECT element").
                    RefArg::Ranges(Ranges::Areas(areas)) => {
                        let Some(Range { left, right }) = areas.first() else {
                            return several_areas(cell);
                        };
                        CalcResult::Range { left, right }
                    }
                    RefArg::Grid(grid) => return self.formula_texts_of_grid(&grid, cell),
                    // A pick that fails is its own error (`SWITCH(9,1,5)`, `XLOOKUP` not found:
                    // `#N/A`); one that gives values is `#VALUE!`, element by element over an
                    // array, an error element kept (`IF({TRUE,FALSE},NA(),Data!A2)` spills `#N/A`,
                    // `#VALUE!`): probes/formulatext-pick-scalars-and-arrays,
                    // probes/formulatext-erroring-references; docs/excel-semantics/references.md.
                    RefArg::Value(error @ CalcResult::Error { .. }) => return error,
                    RefArg::Value(CalcResult::Array(rows)) => return not_a_reference_array(rows),
                    RefArg::Value(_) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Argument must be a reference".to_string(),
                        )
                    }
                }
            }
            _ => self.evaluate_node_with_reference(&args[0], cell),
        };
        if let CalcResult::Range { left, right } = reference {
            if left.sheet != right.sheet {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "3D ranges not supported".to_string(),
                };
            }
            if left == right {
                return self.formula_text_of(left, cell);
            }
            // A multi-cell reference gives one result per cell, `#N/A` for a cell with no
            // formula (`ROWS(FORMULATEXT(A1:A3))` is 3; probes/lookup-argument-modes F38:H38,
            // docs/excel-semantics/lookup.md "Lookup and reference function argument modes").
            // The cells and the total text are bounded (CLAUDE.md directive 5).
            if let Some((error, message)) = check_range_size(&left, &right) {
                return CalcResult::new_error(error, cell, message);
            }
            let mut text_bytes = 0;
            let width = usize::try_from(right.column - left.column + 1).unwrap_or(0);
            let mut rows =
                Vec::with_capacity(usize::try_from(right.row - left.row + 1).unwrap_or(0));
            for row in left.row..=right.row {
                let mut values = Vec::with_capacity(width);
                for column in left.column..=right.column {
                    let at = CellReferenceIndex {
                        sheet: left.sheet,
                        row,
                        column,
                    };
                    match self.formula_text_element(at, cell, &mut text_bytes) {
                        Ok(value) => values.push(value),
                        Err(too_large) => return too_large,
                    }
                }
                rows.push(values);
            }
            CalcResult::Array(rows)
        } else {
            not_a_reference(cell)
        }
    }

    /// FORMULATEXT of each element of a lifted reference call (`INDIRECT({"Data!A1","Data!A2"})`
    /// spills `#N/A`, `=A1+1`; probes/formulatext-pick-scalars-and-arrays K1:T1): a one-cell
    /// element is its cell's formula text, an error element itself. A multi-cell element is
    /// its top-left cell's text. The total text is bounded (CLAUDE.md directive 5).
    fn formula_texts_of_grid(&self, grid: &RefGrid, cell: CellReferenceIndex) -> CalcResult {
        let (rows, columns) = grid.dims();
        let mut text_bytes = 0;
        let mut array = Vec::with_capacity(rows);
        for row in grid.elements().chunks(columns) {
            let mut values = Vec::with_capacity(columns);
            for element in row {
                values.push(match element {
                    RefElement::Error(error) => ArrayNode::Error(error.clone()),
                    // A multi-cell element is its top-left cell's text, with no spill of its own.
                    RefElement::Reference(Range { left, .. }) => {
                        match self.formula_text_element(*left, cell, &mut text_bytes) {
                            Ok(value) => value,
                            Err(too_large) => return too_large,
                        }
                    }
                });
            }
            array.push(values);
        }
        CalcResult::Array(array)
    }

    /// One element of a spilled FORMULATEXT: the formula text of `at`, `#N/A` without one, counted
    /// against the total text budget (`Err`: `#VALUE!` once the texts pass it).
    fn formula_text_element(
        &self,
        at: CellReferenceIndex,
        cell: CellReferenceIndex,
        text_bytes: &mut usize,
    ) -> Result<ArrayNode, CalcResult> {
        match self.formula_text_of(at, cell) {
            CalcResult::String(text) if within_text_budget(text_bytes, text.len()) => {
                Ok(ArrayNode::String(text))
            }
            CalcResult::String(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Total text result is too large".to_string(),
            )),
            _ => Ok(ArrayNode::Error(Error::NA)),
        }
    }

    /// FORMULATEXT of the one cell `at`: its formula, or `#N/A` when it holds none.
    fn formula_text_of(&self, at: CellReferenceIndex, cell: CellReferenceIndex) -> CalcResult {
        if let Some(table) = self.data_table_at(at) {
            return CalcResult::String(table.formula_text().into());
        }
        if let Ok(Some(f)) = self.get_english_cell_formula(at.sheet, at.row, at.column) {
            CalcResult::String(f.into())
        } else {
            CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Reference does not have a formula".to_string(),
            }
        }
    }
}

/// FORMULATEXT's `#ERROR!` for an argument that is not a reference.
fn not_a_reference(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::Error {
        error: Error::ERROR,
        origin: cell,
        message: "Argument must be a reference".to_string(),
    }
}
