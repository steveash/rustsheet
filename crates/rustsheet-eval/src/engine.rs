//! The "engine under test" abstraction. The harness never calls engine internals
//! directly, so other engines (e.g. upstream IronCalc, a future rewritten core)
//! can be plugged in and compared side by side.

use rustsheet::{
    CellRef, Clock, DateSystem, ErrorKind, Input, LoadOptions, RangeRef, SheetId,
    Value as ApiValue, Workbook, WorkbookOptions, XlError,
};

use rustsheet_engine::text::{lone_escape, Segment};
use rustsheet_engine::Text;

use crate::case::{split_qualified, CaseSpec, CellInput};
use crate::cellref::{parse_range, Addr};
use crate::value::Value;

/// A workbook after recalculation, ready to be queried.
pub trait Evaluated {
    /// Returns `None` if the sheet does not exist.
    fn value(&self, sheet: &str, addr: Addr) -> Option<Value>;
    /// Every stored cell of `sheet` (spill members included); empty for an unknown sheet. The
    /// spill-extent check reads it (T-1660).
    fn stored_cells(&self, _sheet: &str) -> Vec<Addr> {
        Vec::new()
    }
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

/// The instant every evaluation reads as "now" unless a METADATA `NOW` row pins another one:
/// 2022-11-08T11:13:28.578Z, the instant the engine's former thread-local mock clock defaulted to,
/// so harness results did not change when the clock moved into the workbook (SPEC-0008).
const HARNESS_CLOCK_MS: i64 = 1_667_906_008_578;

/// Default workbook options on the harness clock.
fn harness_options() -> Result<WorkbookOptions, String> {
    Ok(WorkbookOptions {
        clock: Clock::fixed_unix_millis(HARNESS_CLOCK_MS).map_err(|e| e.to_string())?,
        ..WorkbookOptions::default()
    })
}

/// The RustSheet engine (currently the forked IronCalc `Model`).
pub struct RustSheetEngine;

struct RustSheetEvaluated {
    workbook: Workbook,
}

/// Text holding a lone surrogate as the snapshot reader sees Excel's value: each lone unit as
/// the `_xHHHH_` escape, which the reader keeps literally when it cannot decode it (SPEC-0010;
/// probes/text-surrogate-positions A6).
fn render_lone_units(units: &[u16]) -> String {
    let mut out = String::new();
    for segment in Text::from_utf16(units.to_vec()).segments() {
        match segment {
            Segment::Str(s) => out.push_str(s),
            Segment::Lone(unit) => out.push_str(&lone_escape(unit)),
        }
    }
    out
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
            ApiValue::Text(s) => Value::Text(match self.workbook.text_utf16(cell) {
                Ok(Some(units)) => render_lone_units(&units),
                _ => s.to_string(),
            }),
            ApiValue::Bool(b) => Value::Bool(b),
            ApiValue::Error(e) => Value::Error(e.kind.stored_literal().to_string()),
            // `ApiValue` is non-exhaustive; `Empty` and future variants score as empty.
            _ => Value::Empty,
        })
    }

    fn stored_cells(&self, sheet: &str) -> Vec<Addr> {
        let cells = (self.workbook.sheet_id(sheet))
            .and_then(|id| self.workbook.stored_cells(id))
            .unwrap_or_default();
        cells
            .into_iter()
            .map(|cell| Addr {
                row: cell.row + 1,
                col: cell.col + 1,
            })
            .collect()
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
            workbook: harness_options()?,
            ..LoadOptions::default()
        };
        let mut workbook =
            Workbook::from_xlsx_bytes(bytes, &opts).map_err(|e| format!("import failed: {e}"))?;
        // A METADATA `NOW` row (IronCalc convention, mirrored by `volatile::pinned_clock_serial`)
        // overrides the harness clock.
        workbook.pin_clock_from_metadata();
        workbook.recalculate();
        Ok(Box::new(RustSheetEvaluated { workbook }))
    }

    fn evaluate_spec(&self, spec: &CaseSpec) -> Result<Box<dyn Evaluated>, String> {
        let date_system = match spec.date_system {
            None | Some(1900) => DateSystem::Excel1900,
            Some(1904) => DateSystem::Excel1904,
            Some(other) => return Err(format!("date_system must be 1900 or 1904, not {other}")),
        };
        let mut workbook = Workbook::new(&WorkbookOptions {
            date_system,
            ..harness_options()?
        })
        .map_err(|e| e.to_string())?;
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
                // `Sheet1!Print_Area` is a name scoped to Sheet1, as xlsxwriter (spec2xlsx)
                // reads it.
                for (name, formula) in &spec.names {
                    match split_qualified(name) {
                        Some((sheet, local)) => {
                            let id = spec
                                .sheets
                                .iter()
                                .position(|s| s.name.eq_ignore_ascii_case(&sheet))
                                .and_then(|i| ids.get(i).copied())
                                .ok_or_else(|| {
                                    rustsheet::EditError::Engine(format!(
                                        "name {name}: no sheet {sheet}"
                                    ))
                                })?;
                            tx.define_sheet_name(id, &local, formula)?;
                        }
                        None => tx.define_name(name, formula)?,
                    }
                }
                // Values first, then tables (they read their headers), then formulas (a
                // structured reference parses only once its table exists).
                for formulas in [false, true] {
                    for (sheet, &id) in spec.sheets.iter().zip(&ids) {
                        if formulas {
                            for table in &sheet.tables {
                                tx.add_table(
                                    table_range(id, &table.range)?,
                                    &table.name,
                                    table.filter,
                                )?;
                            }
                        }
                        for (a1, input) in &sheet.cells {
                            let Some(mut input) = spec_input(input) else {
                                continue;
                            };
                            // spec2xlsx writes these as plain `<f>` formulas (T-1077).
                            if let Input::Formula(text) = &input {
                                if sheet.legacy_formulas.contains(a1) {
                                    input = Input::LegacyFormula(text.clone());
                                }
                            }
                            let is_formula =
                                matches!(input, Input::Formula(_) | Input::LegacyFormula(_));
                            if is_formula != formulas {
                                continue;
                            }
                            let addr = Addr::parse(a1).ok_or_else(|| {
                                rustsheet::EditError::Engine(format!("bad address {a1}"))
                            })?;
                            tx.set(CellRef::new(id, addr.row - 1, addr.col - 1)?, input)?;
                        }
                    }
                }
                for (sheet, &id) in spec.sheets.iter().zip(&ids) {
                    for (a1, format) in &sheet.number_formats {
                        let addr = Addr::parse(a1).ok_or_else(|| {
                            rustsheet::EditError::Engine(format!("bad address {a1}"))
                        })?;
                        tx.set_number_format(
                            CellRef::new(id, addr.row - 1, addr.col - 1)?,
                            format,
                        )?;
                    }
                    for &row in &sheet.hidden_rows {
                        let row = row.checked_sub(1).ok_or(rustsheet::RefError::OutOfBounds)?;
                        tx.set_row_hidden(id, row, true)?;
                    }
                }
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        Ok(Box::new(RustSheetEvaluated { workbook }))
    }
}

/// A spec table's `A1:B9` range on sheet `id`.
fn table_range(id: SheetId, a1: &str) -> Result<RangeRef, rustsheet::EditError> {
    let (first, last) = parse_range(a1)
        .ok_or_else(|| rustsheet::EditError::Engine(format!("bad table ref {a1}")))?;
    Ok(RangeRef::new(
        id,
        first.row - 1,
        first.col - 1,
        last.row - 1,
        last.col - 1,
    )?)
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
        // As xlsx import reads `<c t="e">`: the stored-name table, `#ERROR!` for an unknown name (the one
        // difference: `from_literal` folds case, the importer does not; no case stores a lowercase name).
        CellInput::StoredError(e) => Input::Value(ApiValue::Error(XlError::new(
            ErrorKind::from_literal(&e.v).unwrap_or(ErrorKind::Unknown),
        ))),
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

    #[test]
    fn lone_units_render_as_xhhhh_escapes_and_pairs_stay_text() {
        // "a😀" then the lone low surrogate of a cut pair, then "b" (text-surrogate-positions A6).
        let units = [0x61, 0xD83D, 0xDE00, 0xDE00, 0x62];
        assert_eq!(render_lone_units(&units), "a😀_xDE00_b");
        assert_eq!(render_lone_units(&[0xD83D]), "_xD83D_");
    }

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
    fn sheet_qualified_spec_names_are_sheet_scoped() {
        // probes/multiarea-printarea's spelling (T-1193): the name is local to Sheet1.
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","names":{"Sheet1!Print_Area":"=Sheet1!$A$1,Sheet1!$B$2"},
                "sheets":[{"name":"Sheet1","cells":{"A1":1,"B2":5,"H1":"=SUM(Print_Area)"}},
                          {"name":"Other","cells":{"A1":"=SUM(Print_Area)"}}]}"#,
        );
        let evaluated = RustSheetEngine.evaluate_spec(&spec).expect("evaluates");
        assert_eq!(
            evaluated.value("Sheet1", Addr { row: 1, col: 8 }),
            Some(Value::Number(6.0))
        );
        assert_eq!(
            evaluated.value("Other", Addr { row: 1, col: 1 }),
            Some(Value::Error("#NAME?".into()))
        );
    }

    #[test]
    fn a_spec_name_on_an_unknown_sheet_fails_the_case() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","names":{"Nope!x":"=1"},
                "sheets":[{"name":"Sheet1","cells":{}}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&spec).is_err());
    }

    #[test]
    fn stored_error_cells_load_as_constants_not_text() {
        let spec = spec(
            r##"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
                "A1":{"t":"e","v":"#N/A","stored":true},
                "A2":{"t":"e","v":"#GETTING_DATA","stored":true},
                "A3":{"t":"e","v":"#BUSY!","stored":true},
                "A4":{"t":"e","v":"#N/A"},
                "A5":{"t":"e","v":"#UNKNOWN!","stored":true},
                "B2":"=ERROR.TYPE(A2)",
                "B3":"=ERROR.TYPE(A3)"}}]}"##,
        );
        let evaluated = RustSheetEngine.evaluate_spec(&spec).expect("evaluates");
        let at = |row| evaluated.value("S", Addr { row, col: 1 });
        assert_eq!(at(1), Some(Value::Error("#N/A".into())));
        assert_eq!(at(2), Some(Value::Error("#N/A".into())));
        assert_eq!(
            evaluated.value("S", Addr { row: 2, col: 2 }),
            Some(Value::Number(8.0))
        );
        // Its own kind with ERROR.TYPE 0 (SPEC-0003 "Stored `#BUSY!`", T-1222).
        assert_eq!(at(3), Some(Value::Error("#BUSY!".into())));
        assert_eq!(
            evaluated.value("S", Addr { row: 3, col: 2 }),
            Some(Value::Number(0.0))
        );
        assert_eq!(at(4), Some(Value::Error("#N/A".into())));
        // Unknown to the stored-name table: `#ERROR!` (xlsx import's fallback), not the text.
        assert_eq!(at(5), Some(Value::Error("#ERROR!".into())));
    }

    #[test]
    fn a_bad_spec_name_fails_the_case() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","names":{"A1":"=1"},
                "sheets":[{"name":"Sheet1","cells":{}}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&spec).is_err());
    }

    #[test]
    fn spec_date_system_and_number_formats_build() {
        // No value reads either yet (SPEC-0005 step 3); the facade test covers the format.
        let ok = spec(
            r#"{"format":"rustsheet-case/v1","date_system":1904,"sheets":[{"name":"S",
                "cells":{"A1":-1.5,"B1":"=A1*2"},"number_formats":{"A1":"[h]:mm","C3":"0.0"}}]}"#,
        );
        let evaluated = RustSheetEngine.evaluate_spec(&ok).expect("evaluates");
        assert_eq!(
            evaluated.value("S", Addr { row: 1, col: 2 }),
            Some(Value::Number(-3.0))
        );
        let bad_system = spec(
            r#"{"format":"rustsheet-case/v1","date_system":1901,"sheets":[{"name":"S","cells":{}}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&bad_system).is_err());
        let bad_cell = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{},
                "number_formats":{"nope":"0"}}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&bad_cell).is_err());
    }

    #[test]
    fn spec_hidden_rows_and_tables_reach_the_engine() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
                "A1":"Key","B1":"Val","A2":"a","B2":1,"A3":"b","B3":2,"A4":"c","B4":4,
                "D1":"=SUBTOTAL(109,B2:B4)","D2":"=SUBTOTAL(9,B2:B4)","D3":"=SUM(Data[Val])"},
                "hidden_rows":[3],"tables":[{"name":"Data","ref":"A1:B4","filter":true}]}]}"#,
        );
        let evaluated = RustSheetEngine.evaluate_spec(&spec).expect("evaluates");
        let at = |row| evaluated.value("S", Addr { row, col: 4 });
        // Row 3 is filtered out of the table: both SUBTOTAL forms skip it.
        assert_eq!(at(1), Some(Value::Number(5.0)));
        assert_eq!(at(2), Some(Value::Number(5.0)));
        assert_eq!(at(3), Some(Value::Number(7.0)));
    }

    #[test]
    fn spec_legacy_formulas_are_implicitly_intersected() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
                "A1":1,"A2":2,"H7":"=A1:A2","H8":"=FORMULATEXT(H7)","H9":"=A1:A2"},
                "legacy_formulas":["H7"]}]}"#,
        );
        let evaluated = RustSheetEngine.evaluate_spec(&spec).expect("evaluates");
        let at = |row| evaluated.value("S", Addr { row, col: 8 });
        assert_eq!(at(7), Some(Value::Error("#VALUE!".into())));
        assert_eq!(at(8), Some(Value::Text("=@A1:A2".into())));
        // H9 is not listed: a dynamic-array formula that spills.
        assert_eq!(at(9), Some(Value::Number(1.0)));
    }

    #[test]
    fn a_zero_hidden_row_fails_the_case() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{},"hidden_rows":[0]}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&spec).is_err());
    }
}
