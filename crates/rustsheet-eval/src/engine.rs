//! The "engine under test" abstraction. The harness never calls engine internals
//! directly, so other engines (e.g. upstream IronCalc, a future rewritten core)
//! can be plugged in and compared side by side.

use rustsheet::{CellRef, Input, LoadOptions, Value as ApiValue, Workbook, WorkbookOptions};

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
    /// Worksheet functions the engine implements, by en-US name (for the evidence report).
    fn implemented_functions(&self) -> Vec<String>;
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

    fn implemented_functions(&self) -> Vec<String> {
        rustsheet::implemented_functions()
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
        let mut workbook = Workbook::new(&WorkbookOptions::default()).map_err(|e| e.to_string())?;
        // The harness only reads final values: skip the per-edit value diff.
        workbook.set_track_changes(false);
        workbook
            .edit(|tx| {
                let mut ids = Vec::with_capacity(spec.sheets.len());
                for (i, sheet) in spec.sheets.iter().enumerate() {
                    ids.push(if i == 0 {
                        let first = tx.first_sheet()?;
                        tx.rename_sheet(first, &sheet.name)?;
                        first
                    } else {
                        tx.add_sheet(&sheet.name)?
                    });
                }
                // Names need their sheets; each reparses the workbook, so define them before
                // any cell is set.
                for (name, formula) in &spec.names {
                    tx.define_name(name, formula)?;
                }
                for (sheet, &id) in spec.sheets.iter().zip(&ids) {
                    for (a1, input) in &sheet.cells {
                        let addr = Addr::parse(a1).ok_or_else(|| {
                            rustsheet::EditError::Engine(format!("bad address {a1}"))
                        })?;
                        let cell = CellRef::new(id, addr.row - 1, addr.col - 1)?;
                        if let Some(input) = spec_input(input) {
                            tx.set(cell, input)?;
                        }
                    }
                }
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        Ok(Box::new(RustSheetEvaluated { workbook }))
    }
}

/// The facade input for a spec cell; `None` leaves the cell empty.
fn spec_input(input: &CellInput) -> Option<Input> {
    Some(match input {
        CellInput::Number(n) | CellInput::Typed(Value::Number(n)) => Input::from(*n),
        CellInput::Bool(b) | CellInput::Typed(Value::Bool(b)) => Input::from(*b),
        CellInput::Str(s) if s.starts_with('=') => Input::Formula(s.clone()),
        // Mirrors tools/src/rustsheet_tools/spec2xlsx.py: "" is materialized as ="".
        CellInput::Str(s) | CellInput::Typed(Value::Text(s)) if s.is_empty() => {
            Input::Formula("=\"\"".to_string())
        }
        CellInput::Str(s) | CellInput::Typed(Value::Text(s)) => {
            Input::Value(ApiValue::Text(s.as_str().into()))
        }
        CellInput::Typed(Value::Error(e)) | CellInput::Entered { entered: e } => {
            Input::Parse(e.clone())
        }
        CellInput::Typed(Value::Empty) => return None,
    })
}

#[cfg(test)]
mod tests {
    use rustsheet_engine::Model;

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

    fn spec(json: &str) -> CaseSpec {
        serde_json::from_str(json).expect("spec")
    }

    #[test]
    fn spec_names_are_defined_before_cells() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","names":{"src":"=Data!$A$1"},
                "sheets":[{"name":"Calc","cells":{"A1":"=src*2"}},
                          {"name":"Data","cells":{"A1":5}}]}"#,
        );
        let evaluated = RustSheetEngine.evaluate_spec(&spec).expect("evaluates");
        assert_eq!(
            evaluated.value("Calc", Addr { row: 1, col: 1 }),
            Some(Value::Number(10.0))
        );
    }

    #[test]
    fn a_bad_spec_name_fails_the_case() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","names":{"A1":"=1"},
                "sheets":[{"name":"Sheet1","cells":{}}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&spec).is_err());
    }
}
