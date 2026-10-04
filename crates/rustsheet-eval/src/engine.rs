//! The "engine under test" abstraction. The harness never calls engine internals
//! directly, so other engines (e.g. upstream IronCalc, a future rewritten core)
//! can be plugged in and compared side by side.

use rustsheet::{CellRef, LoadOptions, Value as ApiValue, Workbook, WorkbookOptions};
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
    workbook: Workbook,
}

impl Evaluated for RustSheetEvaluated {
    fn value(&self, sheet: &str, addr: Addr) -> Option<Value> {
        let cell = CellRef::new(
            self.workbook.sheet_id(sheet).ok()?,
            addr.row.checked_sub(1)?,
            addr.col.checked_sub(1)?,
        )
        .ok()?;
        Some(match self.workbook.value(cell).ok()? {
            ApiValue::Number(n) => Value::Number(n),
            ApiValue::Text(s) => Value::Text(s.to_string()),
            ApiValue::Bool(b) => Value::Bool(b),
            ApiValue::Error(e) => Value::Error(e.kind.literal().to_string()),
            // `ApiValue` is non-exhaustive; `Empty` and future variants score as empty.
            _ => Value::Empty,
        })
    }
}

impl Engine for RustSheetEngine {
    fn name(&self) -> &'static str {
        "rustsheet"
    }

    fn evaluate_xlsx(&self, bytes: &[u8]) -> Result<Box<dyn Evaluated>, String> {
        let opts = LoadOptions {
            recalc: false,
            ..LoadOptions::default()
        };
        let mut workbook =
            Workbook::from_xlsx_bytes(bytes, &opts).map_err(|e| format!("import failed: {e}"))?;
        // The mock clock is thread-local and defaults to a fixed instant; a METADATA `NOW` row
        // (IronCalc convention, mirrored by `volatile::pinned_clock_serial`) overrides it.
        workbook.pin_clock_from_metadata();
        workbook.recalculate();
        Ok(Box::new(RustSheetEvaluated { workbook }))
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
        let workbook = Workbook::from_engine_model(model, &WorkbookOptions::default());
        Ok(Box::new(RustSheetEvaluated { workbook }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Workbook with `Sheet1!A1 = TODAY()` and an optional `METADATA` NOW row.
    fn workbook(now_serial: Option<f64>) -> Vec<u8> {
        let mut model = Model::new_empty("case", "en", "UTC", "en").expect("model");
        model
            .update_cell_with_formula(0, 1, 1, "=TODAY()".into())
            .expect("formula");
        if let Some(serial) = now_serial {
            model.add_sheet("METADATA").expect("sheet");
            model.update_cell_with_text(1, 1, 1, "NOW").expect("label");
            model
                .update_cell_with_number(1, 1, 2, serial)
                .expect("serial");
        }
        model.evaluate();
        let cursor = std::io::Cursor::new(Vec::new());
        rustsheet_xlsx::export::save_xlsx_to_writer(&model, cursor)
            .expect("save")
            .into_inner()
    }

    fn today(bytes: &[u8]) -> Value {
        let evaluated = RustSheetEngine.evaluate_xlsx(bytes).expect("evaluate");
        evaluated
            .value("Sheet1", Addr { row: 1, col: 1 })
            .expect("sheet exists")
    }

    #[test]
    fn metadata_now_row_pins_the_clock() {
        assert_eq!(today(&workbook(Some(45000.75))), Value::Number(45000.0));
    }

    #[test]
    fn without_metadata_the_clock_is_a_fixed_default() {
        // 8 November 2022 (the engine's mock default).
        assert_eq!(today(&workbook(None)), Value::Number(44873.0));
    }
}
