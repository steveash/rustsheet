//! The "engine under test" abstraction. The harness never calls engine internals
//! directly, so other engines (e.g. upstream IronCalc, a future rewritten core)
//! can be plugged in and compared side by side.

use rustsheet_engine::expressions::token::Error as EngineError;
use rustsheet_engine::types::{Cell, FormulaValue, SpillValue};
use rustsheet_engine::Model;

use crate::case::{CaseSpec, CellInput};
use crate::cellref::Addr;
use crate::value::Value;

/// A workbook after recalculation, ready to be queried.
pub trait Evaluated {
    /// Returns `None` if the sheet does not exist.
    fn value(&self, sheet: &str, addr: Addr) -> Option<Value>;
}

pub trait Engine: Sync {
    fn name(&self) -> &'static str;
    /// Loads xlsx bytes, performs a full recalculation, and returns the result.
    ///
    /// # Errors
    /// Returns a human-readable message if the workbook cannot be loaded.
    fn evaluate_xlsx(&self, bytes: &[u8]) -> Result<Box<dyn Evaluated>, String>;
    /// Builds a workbook from a spec, recalculates, and returns the result.
    ///
    /// # Errors
    /// Returns a human-readable message if the spec cannot be materialized.
    fn evaluate_spec(&self, spec: &CaseSpec) -> Result<Box<dyn Evaluated>, String>;
}

/// The RustSheet engine (currently the forked IronCalc `Model`).
pub struct RustSheetEngine;

struct RustSheetEvaluated {
    model: Model<'static>,
    names: Vec<String>,
}

fn error_literal(e: &EngineError) -> String {
    e.to_string()
}

impl Evaluated for RustSheetEvaluated {
    fn value(&self, sheet: &str, addr: Addr) -> Option<Value> {
        let idx = self
            .names
            .iter()
            .position(|n| n.eq_ignore_ascii_case(sheet))?;
        let ws = self
            .model
            .workbook
            .worksheet(u32::try_from(idx).ok()?)
            .ok()?;
        let row = i32::try_from(addr.row).ok()?;
        let col = i32::try_from(addr.col).ok()?;
        let shared = &self.model.workbook.shared_strings;
        let text = |si: i32| {
            usize::try_from(si)
                .ok()
                .and_then(|i| shared.get(i))
                .map_or(Value::Empty, |s| Value::Text(s.clone()))
        };
        let from_formula = |v: &FormulaValue| match v {
            FormulaValue::Unevaluated => Value::Empty,
            FormulaValue::Boolean(b) => Value::Bool(*b),
            FormulaValue::Number(n) => Value::Number(*n),
            FormulaValue::Text(s) => Value::Text(s.clone()),
            FormulaValue::Error { ei, .. } => Value::Error(error_literal(ei)),
        };
        Some(match ws.cell(row, col) {
            None | Some(Cell::EmptyCell { .. }) => Value::Empty,
            Some(Cell::BooleanCell { v, .. }) => Value::Bool(*v),
            Some(Cell::NumberCell { v, .. }) => Value::Number(*v),
            Some(Cell::ErrorCell { ei, .. }) => Value::Error(error_literal(ei)),
            Some(Cell::SharedString { si, .. }) => text(*si),
            Some(Cell::CellFormula { v, .. } | Cell::ArrayFormula { v, .. }) => from_formula(v),
            Some(Cell::SpillCell { v, .. }) => match v {
                SpillValue::Boolean(b) => Value::Bool(*b),
                SpillValue::Number(n) => Value::Number(*n),
                SpillValue::Text(s) => Value::Text(s.clone()),
                SpillValue::Error(ei) => Value::Error(error_literal(ei)),
            },
        })
    }
}

impl Engine for RustSheetEngine {
    fn name(&self) -> &'static str {
        "rustsheet"
    }

    fn evaluate_xlsx(&self, bytes: &[u8]) -> Result<Box<dyn Evaluated>, String> {
        let workbook = rustsheet_xlsx::import::load_from_xlsx_bytes(bytes, "case", "en", "UTC")
            .map_err(|e| format!("import failed: {e}"))?;
        let mut model = Model::from_workbook(workbook, "en")?;
        model.evaluate();
        let names = model.workbook.get_worksheet_names();
        Ok(Box::new(RustSheetEvaluated { model, names }))
    }

    fn evaluate_spec(&self, spec: &CaseSpec) -> Result<Box<dyn Evaluated>, String> {
        let mut model = Model::new_empty("case", "en", "UTC", "en")?;
        for (i, sheet) in spec.sheets.iter().enumerate() {
            let idx = u32::try_from(i).map_err(|e| e.to_string())?;
            if i == 0 {
                model.rename_sheet_by_index(0, &sheet.name)?;
            } else {
                model.add_sheet(&sheet.name)?;
            }
            for (a1, input) in &sheet.cells {
                let addr = Addr::parse(a1).ok_or_else(|| format!("bad address {a1}"))?;
                let row = i32::try_from(addr.row).map_err(|e| e.to_string())?;
                let col = i32::try_from(addr.col).map_err(|e| e.to_string())?;
                match input {
                    CellInput::Number(n) | CellInput::Typed(Value::Number(n)) => {
                        model.update_cell_with_number(idx, row, col, *n)?;
                    }
                    CellInput::Bool(b) | CellInput::Typed(Value::Bool(b)) => {
                        model.update_cell_with_bool(idx, row, col, *b)?;
                    }
                    CellInput::Str(s) if s.starts_with('=') => {
                        model.update_cell_with_formula(idx, row, col, s.clone())?;
                    }
                    // Mirrors tools/src/rustsheet_tools/spec2xlsx.py: "" is materialized as ="".
                    CellInput::Str(s) | CellInput::Typed(Value::Text(s)) if s.is_empty() => {
                        model.update_cell_with_formula(idx, row, col, "=\"\"".to_string())?;
                    }
                    CellInput::Str(s) | CellInput::Typed(Value::Text(s)) => {
                        model.update_cell_with_text(idx, row, col, s)?;
                    }
                    CellInput::Typed(Value::Error(e)) => {
                        model.set_user_input(idx, row, col, e.clone())?;
                    }
                    CellInput::Typed(Value::Empty) => {}
                }
            }
        }
        model.evaluate();
        let names = model.workbook.get_worksheet_names();
        Ok(Box::new(RustSheetEvaluated { model, names }))
    }
}
