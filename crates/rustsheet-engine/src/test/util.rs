#![allow(clippy::unwrap_used)]

use crate::expressions::types::{Area, CellReferenceIndex};
use crate::model::Model;
use crate::types::{Cell, FormulaValue};

pub fn new_empty_model() -> Model {
    Model::new_empty("model", "en", "UTC", "en").unwrap()
}

/// A union of `leaves` copies of `leaf`, nested as a balanced tree so that its depth, about
/// three levels per halving, stays far under the parser's `MAX_FORMULA_DEPTH`.
pub fn balanced_union(leaf: &str, leaves: usize) -> String {
    if leaves <= 1 {
        return leaf.to_string();
    }
    let half = leaves / 2;
    format!(
        "({},{})",
        balanced_union(leaf, half),
        balanced_union(leaf, leaves - half)
    )
}

impl Model {
    pub fn _parse_reference(&self, cell: &str) -> CellReferenceIndex {
        if cell.contains('!') {
            self.parse_reference(cell).unwrap()
        } else {
            let sheet_name = self.get_worksheets_properties()[0].name.clone();
            self.parse_reference(&format!("{sheet_name}!{cell}"))
                .unwrap()
        }
    }
    pub fn _set(&mut self, cell: &str, value: &str) {
        let cell_reference = self._parse_reference(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        self.set_user_input(cell_reference.sheet, row, column, value.to_string())
            .unwrap();
    }
    pub fn _has_formula(&self, cell: &str) -> bool {
        self._get_formula_opt(cell).is_some()
    }
    pub fn _get_formula(&self, cell: &str) -> String {
        self._get_formula_opt(cell).unwrap_or_default()
    }
    fn _get_formula_opt(&self, cell: &str) -> Option<String> {
        let cell_reference = self._parse_reference(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        self.get_cell_formula(cell_reference.sheet, row, column)
            .unwrap()
    }
    pub fn _get_text_at(&self, sheet: u32, row: i32, column: i32) -> String {
        self.get_formatted_cell_value(sheet, row, column).unwrap()
    }
    pub fn _get_text(&self, cell: &str) -> String {
        let CellReferenceIndex { sheet, row, column } = self._parse_reference(cell);
        self._get_text_at(sheet, row, column)
    }
    /// The message of the error in formula cell `cell`, which tells the engine's caps apart
    /// from a plain error of the same kind.
    pub fn _get_error_message(&self, cell: &str) -> String {
        match self._get_cell(cell) {
            Cell::CellFormula {
                v: FormulaValue::Error { m, .. },
                ..
            }
            | Cell::ArrayFormula {
                v: FormulaValue::Error { m, .. },
                ..
            } => m.clone(),
            other => format!("not an error: {other:?}"),
        }
    }
    pub fn _get_cell(&self, cell: &str) -> &Cell {
        let cell_reference = self._parse_reference(cell);
        let worksheet = self.workbook.worksheet(cell_reference.sheet).unwrap();
        worksheet
            .cell(cell_reference.row, cell_reference.column)
            .unwrap()
    }

    pub fn _cell_clear_contents(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<(), String> {
        let area = Area {
            sheet,
            row,
            column,
            width: 1,
            height: 1,
        };
        self.range_clear_contents(&area)
    }

    pub fn _cell_clear_all(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let area = Area {
            sheet,
            row,
            column,
            width: 1,
            height: 1,
        };
        self.range_clear_all(&area)
    }
}

/// Deterministic xorshift64 generator for seeded tests; the wrapped state must be non-zero.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}
