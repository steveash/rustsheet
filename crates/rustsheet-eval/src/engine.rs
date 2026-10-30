//! The "engine under test" abstraction. The harness never calls engine internals
//! directly, so other engines (e.g. upstream IronCalc, a future rewritten core)
//! can be plugged in and compared side by side.

use std::collections::BTreeMap;

use rustsheet::{
    CellRef, Clock, DateSystem, ErrorKind, Input, LoadOptions, RangeRef, SheetId,
    Value as ApiValue, Workbook, WorkbookOptions, XlError,
};

use rustsheet_engine::text::{lone_escape, Segment};
use rustsheet_engine::Text;

use crate::case::{split_qualified, CaseSpec, CellInput, SheetOp, SheetSpec};
use crate::cellref::{parse_range, Addr};
use crate::spec_pivots;
use crate::value::Value;

/// A workbook after recalculation, ready to be queried.
pub trait Evaluated {
    /// Returns `None` if the sheet does not exist.
    fn value(&self, sheet: &str, addr: Addr) -> Option<Value>;
    /// Every stored cell of `sheet` (spill members included); empty for an unknown sheet. The
    /// spill-extent check reads it (T-1660), so an empty list turns that check off.
    fn stored_cells(&self, sheet: &str) -> Vec<Addr>;
    /// The cell's text as displayed under its number format (T-1610); `None` if the sheet does
    /// not exist or the engine cannot format the cell.
    fn text(&self, _sheet: &str, _addr: Addr) -> Option<String> {
        None
    }
    /// The sheet names in tab order, which the argument-coverage census parses formulas against
    /// (T-0077); empty when the engine cannot list them.
    fn sheet_names(&self) -> Vec<String> {
        Vec::new()
    }
}

pub trait Engine: Sync {
    fn name(&self) -> &'static str;
    /// Loads xlsx bytes, performs a full recalculation, and returns the result. `name` is the
    /// workbook's file name in the oracle (sidecar `source.workbook_name`), if recorded.
    ///
    /// # Errors
    /// Returns a human-readable message if the workbook cannot be loaded.
    fn evaluate_xlsx(&self, bytes: &[u8], name: Option<&str>)
        -> Result<Box<dyn Evaluated>, String>;
    /// Builds a workbook from a spec, recalculates, and returns the result. `name` is as for
    /// [`Engine::evaluate_xlsx`].
    ///
    /// # Errors
    /// Returns a human-readable message if the spec cannot be materialized.
    fn evaluate_spec(
        &self,
        spec: &CaseSpec,
        name: Option<&str>,
    ) -> Result<Box<dyn Evaluated>, String>;
    /// Worksheet functions the engine implements, by en-US name (for the evidence report).
    fn implemented_functions(&self) -> Vec<String>;
}

/// The instant every evaluation reads as "now" unless a METADATA `NOW` row pins another one:
/// 2022-11-08T11:13:28.578Z, the instant the engine's former thread-local mock clock defaulted to,
/// so harness results did not change when the clock moved into the workbook (SPEC-0008).
const HARNESS_CLOCK_MS: i64 = 1_667_906_008_578;

/// Default workbook options on the harness clock, named as the oracle named the workbook: its
/// file name without the extension the engine adds back (T-2042).
fn harness_options(name: Option<&str>) -> Result<WorkbookOptions, String> {
    Ok(WorkbookOptions {
        clock: Clock::fixed_unix_millis(HARNESS_CLOCK_MS).map_err(|e| e.to_string())?,
        name: name.map(|n| n.strip_suffix(".xlsx").unwrap_or(n).to_string()),
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

impl RustSheetEvaluated {
    /// The facade cell at `addr` (1-based) on `sheet`, `None` for an unknown sheet.
    fn cell(&self, sheet: &str, addr: Addr) -> Option<CellRef> {
        CellRef::new(
            self.workbook.sheet_id(sheet).ok()?,
            addr.row.checked_sub(1)?,
            addr.col.checked_sub(1)?,
        )
        .ok()
    }
}

impl Evaluated for RustSheetEvaluated {
    fn sheet_names(&self) -> Vec<String> {
        self.workbook.sheets().into_iter().map(|s| s.name).collect()
    }

    fn value(&self, sheet: &str, addr: Addr) -> Option<Value> {
        let cell = self.cell(sheet, addr)?;
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

    fn text(&self, sheet: &str, addr: Addr) -> Option<String> {
        self.workbook.formatted(self.cell(sheet, addr)?).ok()
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

    fn evaluate_xlsx(
        &self,
        bytes: &[u8],
        name: Option<&str>,
    ) -> Result<Box<dyn Evaluated>, String> {
        let opts = LoadOptions {
            recalc: false,
            workbook: harness_options(name)?,
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

    fn evaluate_spec(
        &self,
        spec: &CaseSpec,
        name: Option<&str>,
    ) -> Result<Box<dyn Evaluated>, String> {
        let date_system = match spec.date_system {
            None | Some(1900) => DateSystem::Excel1900,
            Some(1904) => DateSystem::Excel1904,
            Some(other) => return Err(format!("date_system must be 1900 or 1904, not {other}")),
        };
        let mut workbook = Workbook::new(&WorkbookOptions {
            date_system,
            ..harness_options(name)?
        })
        .map_err(|e| e.to_string())?;
        // The harness only reads final values: skip the per-edit value diff.
        workbook.set_track_changes(false);
        if let Some(c) = spec.calculation.as_ref().filter(|c| c.iterate) {
            let default = rustsheet::Iteration::default();
            workbook.set_iteration(Some(rustsheet::Iteration {
                max_iterations: c.iterate_count.unwrap_or(default.max_iterations),
                max_change: c.iterate_delta.unwrap_or(default.max_change),
            }));
        }
        workbook
            .edit(|tx| {
                let mut ids = Vec::with_capacity(spec.sheets.len());
                let sheet_names: Vec<String> = spec.sheets.iter().map(|s| s.name.clone()).collect();
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
                define_names(tx, spec, &ids, false)?;
                // Values first, then tables (they read their headers), then stored formulas (a
                // structured reference parses only once its table exists), then entered text on
                // every sheet, as the oracle types it after the workbook loads: a typed formula
                // sees every stored cell (probes/t2983-formulas-and-blank-enter-blank, T-3024).
                for phase in [Phase::Values, Phase::Stored, Phase::Entered] {
                    for (sheet, &id) in spec.sheets.iter().zip(&ids) {
                        if phase == Phase::Stored {
                            spec_pivots::add_pivots(tx, spec, &ids, sheet, id)?;
                            for table in &sheet.tables {
                                tx.add_table_with_totals(
                                    range_ref(id, &table.range)?,
                                    &table.name,
                                    table.filter,
                                    table.totals_row,
                                )?;
                            }
                        }
                        if phase == Phase::Values {
                            spec_pivots::write_rendered(tx, sheet, id, spec_input)?;
                        }
                        for (addr, a1, input_spec) in row_major(&sheet.cells)? {
                            let Some((input, input_phase)) =
                                cell_input(sheet, &sheet_names, addr, a1, input_spec)
                            else {
                                continue;
                            };
                            if input_phase != phase {
                                continue;
                            }
                            match tx.set(CellRef::new(id, addr.row - 1, addr.col - 1)?, input) {
                                // Excel refuses a typed formula that does not parse and the cell
                                // stays blank (decision 0179, T-3213); a typed formula past a
                                // resource limit still fails the case (decision 0168).
                                Err(rustsheet::EditError::InvalidInput { message, .. })
                                    if phase == Phase::Entered
                                        && message.starts_with(
                                            rustsheet_engine::TYPED_FORMULA_DOES_NOT_PARSE,
                                        ) => {}
                                result => result?,
                            }
                        }
                    }
                }
                // A name with a structured reference parses only once its table exists.
                define_names(tx, spec, &ids, true)?;
                for (sheet, &id) in spec.sheets.iter().zip(&ids) {
                    for table in &sheet.data_tables {
                        let input = |a1: &Option<String>| -> Result<_, rustsheet::EditError> {
                            let Some(a1) = a1.as_deref() else {
                                return Ok(None);
                            };
                            let addr = Addr::parse(a1).ok_or_else(|| {
                                rustsheet::EditError::Engine(format!("bad address {a1}"))
                            })?;
                            Ok(Some(CellRef::new(id, addr.row - 1, addr.col - 1)?))
                        };
                        tx.add_data_table(
                            range_ref(id, &table.range)?,
                            input(&table.row_input)?,
                            input(&table.column_input)?,
                        )?;
                    }
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
        apply_sheet_ops(&mut workbook, &spec.sheet_ops)?;
        // The edit calculated once; an iterative case gets the oracle's other calculations.
        for _ in 1..calculations(spec) {
            workbook.recalculate();
        }
        Ok(Box::new(RustSheetEvaluated { workbook }))
    }
}

/// A spec sheet's cells row by row, left to right: the order an xlsx stores them in and Excel
/// loads them, which decides the spelling a called name that is no function keeps (the first met;
/// probes/unknown-function-name-spelling, T-1719).
fn row_major(
    cells: &BTreeMap<String, CellInput>,
) -> Result<Vec<(Addr, &String, &CellInput)>, rustsheet::EditError> {
    let mut ordered = cells
        .iter()
        .map(|(a1, input)| {
            Addr::parse(a1)
                .map(|addr| (addr, a1, input))
                .ok_or_else(|| rustsheet::EditError::Engine(format!("bad address {a1}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    ordered.sort_by_key(|(addr, ..)| (addr.row, addr.col));
    Ok(ordered)
}

/// Makes a case's sheet edits after it is built, as the oracle backend replays them in Excel after
/// the workbook loads (T-1608). Each op is its own edit, so it sees the names the ops before it
/// left.
fn apply_sheet_ops(workbook: &mut Workbook, ops: &[SheetOp]) -> Result<(), String> {
    for op in ops {
        let sheet_id_of = |workbook: &Workbook, sheet: &str| {
            workbook
                .sheet_id(sheet)
                .map_err(|e| format!("sheet op on {sheet}: {e}"))
        };
        let done = match op {
            SheetOp::Add { sheet } => workbook.edit(|tx| tx.add_sheet(sheet)).map(drop),
            SheetOp::Rename { sheet, to } => {
                let id = sheet_id_of(workbook, sheet)?;
                workbook.edit(|tx| tx.rename_sheet(id, to)).map(drop)
            }
            SheetOp::Move { sheet, to } => {
                let id = sheet_id_of(workbook, sheet)?;
                workbook.edit(|tx| tx.move_sheet(id, *to)).map(drop)
            }
            SheetOp::DeleteSheet { sheet } => {
                let id = sheet_id_of(workbook, sheet)?;
                workbook.edit(|tx| tx.delete_sheet(id)).map(drop)
            }
            // Not `Txn` edits: the facade has no structural edits yet, so the harness uses its
            // feature-gated ones (decisions 0076, 0098). Rows and columns are 1-based in the case
            // file. A `Txn` edit recalculates on commit; these do not.
            SheetOp::ConvertTable { table, .. } => workbook
                .convert_table_to_range(table)
                .map(|()| workbook.recalculate()),
            SheetOp::DeleteTable { table, .. } => workbook
                .delete_table(table)
                .map(|()| workbook.recalculate()),
            SheetOp::InsertRow { sheet, row } | SheetOp::DeleteRow { sheet, row } => {
                let id = sheet_id_of(workbook, sheet)?;
                let row = row
                    .checked_sub(1)
                    .ok_or_else(|| format!("sheet op {op:?}: rows are 1-based"))?;
                let result = if matches!(op, SheetOp::InsertRow { .. }) {
                    workbook.insert_rows(id, row, 1)
                } else {
                    workbook.delete_rows(id, row, 1)
                };
                result.map(|()| workbook.recalculate())
            }
            SheetOp::InsertColumn { sheet, column } | SheetOp::DeleteColumn { sheet, column } => {
                let id = sheet_id_of(workbook, sheet)?;
                let column = Addr::col_index(column)
                    .map(|c| c - 1)
                    .ok_or_else(|| format!("sheet op {op:?}: not a column letter"))?;
                let result = if matches!(op, SheetOp::InsertColumn { .. }) {
                    workbook.insert_columns(id, column, 1)
                } else {
                    workbook.delete_columns(id, column, 1)
                };
                result.map(|()| workbook.recalculate())
            }
        };
        done.map_err(|e| format!("sheet op {op:?}: {e}"))?;
    }
    Ok(())
}

/// How many times excel-graph calculates a case workbook: on load (`fullCalcOnLoad`), at the
/// `FullRebuild` and at the scratch-sheet edit (`tools/src/rustsheet_tools/oracle/backends.py`).
/// Iterative calculation resumes from the previous values each time, so its oracle values hold
/// three calculations' worth of iterations (docs/excel-semantics/recalc.md "How excel-graph
/// calculates a probe workbook"; probes/circular-iteration-on, T-0428).
const ORACLE_CALCULATIONS: usize = 3;

/// The calculations the adapter runs for `spec` so that its values compare with the oracle's:
/// [`ORACLE_CALCULATIONS`] for an iterative case, one otherwise. Without iteration every
/// calculation gives the same values, so the extra ones would only cost time (T-0429).
fn calculations(spec: &CaseSpec) -> usize {
    if spec.calculation.as_ref().is_some_and(|c| c.iterate) {
        ORACLE_CALCULATIONS
    } else {
        1
    }
}

/// Defines the workbook and sheet-scoped names of `spec` whose formula has a structured
/// reference (`structured`) or has none. A `[` anywhere in the formula counts, an approximation:
/// a name whose `[` is in a string is only defined later, after the tables, which is harmless.
fn define_names(
    tx: &mut rustsheet::Txn<'_>,
    spec: &CaseSpec,
    ids: &[SheetId],
    structured: bool,
) -> Result<(), rustsheet::EditError> {
    for (name, formula) in spec
        .names
        .iter()
        .filter(|(_, f)| f.contains('[') == structured)
    {
        match split_qualified(name) {
            Some((sheet, local)) => {
                let id = spec
                    .sheets
                    .iter()
                    .position(|s| s.name.eq_ignore_ascii_case(&sheet))
                    .and_then(|i| ids.get(i).copied())
                    .ok_or_else(|| {
                        rustsheet::EditError::Engine(format!("name {name}: no sheet {sheet}"))
                    })?;
                tx.load_defined_name(Some(id), &local, formula)?;
            }
            None => tx.load_defined_name(None, name, formula)?,
        }
    }
    Ok(())
}

/// An A1 range (`A1:B4`, one-based) of sheet `id` as a facade range.
pub(crate) fn range_ref(id: SheetId, a1: &str) -> Result<RangeRef, rustsheet::EditError> {
    let (first, last) =
        parse_range(a1).ok_or_else(|| rustsheet::EditError::Engine(format!("bad range {a1}")))?;
    Ok(RangeRef::new(
        id,
        first.row - 1,
        first.col - 1,
        last.row - 1,
        last.col - 1,
    )?)
}

/// When a spec cell goes in ([`spec_input`]): its values, then the tables and stored formulas,
/// then what the oracle types after load.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Values,
    Stored,
    Entered,
}

/// The facade input for spec cell `a1` of `sheet` and the phase it goes in at; `None` leaves the
/// cell empty.
fn cell_input(
    sheet: &SheetSpec,
    sheet_names: &[String],
    addr: Addr,
    a1: &str,
    input_spec: &CellInput,
) -> Option<(Input, Phase)> {
    let mut input = match input_spec {
        CellInput::EnteredR1c1 { entered_r1c1 } => {
            r1c1_entry(entered_r1c1, sheet_names, &sheet.name, addr)
        }
        _ => spec_input(input_spec),
    }?;
    // spec2xlsx writes these as plain `<f>` formulas (T-1077).
    if let Input::StoredFormula(text) = &input {
        if sheet.legacy_formulas.iter().any(|f| f == a1) {
            input = Input::LegacyFormula(text.clone());
        }
    }
    // Entered text goes after the tables, as the oracle types it after the workbook loads: a
    // structured reference in a typed formula parses only once its table exists
    // (probes/t2786-whitespace-structured-refs-runs, T-2801), and a constant typed into a table
    // column after a typed formula is an exception to its calculated column
    // (probes/t2957-constant-exception, T-2958).
    let typed_entry = matches!(
        input_spec,
        CellInput::Entered { .. } | CellInput::EnteredR1c1 { .. }
    );
    let phase = match &input {
        Input::StoredFormula(_) | Input::LegacyFormula(_) => Phase::Stored,
        Input::Parse(text) if text.starts_with('=') || typed_entry => Phase::Entered,
        _ => Phase::Values,
    };
    Some((input, phase))
}

/// The facade input for `text` typed in R1C1 mode into `addr` of `sheet`: the A1 text it reads as
/// (`rustsheet_engine::r1c1_formula_to_a1`), typed. `None`, a blank cell, when it does not parse,
/// as Excel refuses it (decision 0179).
fn r1c1_entry(text: &str, sheet_names: &[String], sheet: &str, addr: Addr) -> Option<Input> {
    let row = i32::try_from(addr.row).ok()?;
    let column = i32::try_from(addr.col).ok()?;
    rustsheet_engine::r1c1_formula_to_a1(text, sheet_names, sheet, row, column).map(Input::Parse)
}

/// The facade input for a spec cell; `None` leaves the cell empty.
fn spec_input(input: &CellInput) -> Option<Input> {
    Some(match input {
        // spec2xlsx writes a number as an xlsx `<v>`, and Excel reads a subnormal one as 0, as xlsx
        // import does (numbers.md "A subnormal cell constant reads as 0").
        CellInput::Number(n) | CellInput::Typed(Value::Number(n)) => {
            Input::from(rustsheet_engine::flush_subnormal(*n))
        }
        CellInput::Bool(b) | CellInput::Typed(Value::Bool(b)) => Input::from(*b),
        // The formula as spec2xlsx stores it (decision 0103: LET names spelled like a cell bind).
        CellInput::Str(s) if s.starts_with('=') => Input::StoredFormula(s.clone()),
        // Mirrors tools/src/rustsheet_tools/spec2xlsx.py: "" is materialized as ="".
        CellInput::Str(s) | CellInput::Typed(Value::Text(s)) if s.is_empty() => {
            Input::StoredFormula("=\"\"".to_string())
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
        // Needs its cell to read relative references: `r1c1_entry`.
        CellInput::Typed(Value::Empty) | CellInput::EnteredR1c1 { .. } => return None,
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
        let evaluated = RustSheetEngine
            .evaluate_xlsx(bytes, None)
            .expect("evaluate");
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
    fn spec_sheet_ops_apply_in_order_after_the_build() {
        // probes/sheet-ops (T-1608): references follow the renamed sheet, the moved sheet keeps
        // them, and an added sheet is empty.
        let spec = spec(
            r#"{"format":"rustsheet-case/v1",
                "sheets":[{"name":"Calc","cells":{"A1":"=Data!A1","A2":"=FORMULATEXT(A1)",
                                                 "A3":"=SHEET(Inputs!A1)","A4":"=COUNTA(New!A1:B2)"}},
                          {"name":"Data","cells":{"A1":5}}],
                "sheet_ops":[{"op":"rename","sheet":"Data","to":"Inputs"},
                             {"op":"move","sheet":"Inputs","to":0},
                             {"op":"add","sheet":"New"}]}"#,
        );
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
        let calc = |row| evaluated.value("Calc", Addr { row, col: 1 });
        assert_eq!(calc(1), Some(Value::Number(5.0)));
        assert_eq!(calc(2), Some(Value::Text("=Inputs!A1".into())));
        assert_eq!(calc(3), Some(Value::Number(1.0)));
        assert_eq!(
            evaluated.value("New", Addr { row: 1, col: 1 }),
            Some(Value::Empty)
        );
    }

    #[test]
    fn structural_sheet_ops_shift_cells_and_rewrite_references() {
        // T-2323: A3 refers to B2; a row delete above, a column insert left of it and a delete
        // of the sheet and the table named by the later ops all replay in order.
        let spec = spec(
            r#"{"format":"rustsheet-case/v1",
                "sheets":[{"name":"Calc","cells":{"A1":1,"B2":5,"A3":"=B2*2","B3":"=SUM(A1:A2)",
                                     "D1":"x","E1":"y","D2":1,"E2":2},
                           "tables":[{"name":"Sales","ref":"D1:E2"}]},
                          {"name":"Gone","cells":{"A1":1}}],
                "sheet_ops":[{"op":"delete_row","sheet":"Calc","row":1},
                             {"op":"insert_column","sheet":"Calc","column":"A"},
                             {"op":"delete_column","sheet":"Calc","column":"A"},
                             {"op":"delete_sheet","sheet":"Gone"},
                             {"op":"delete_table","sheet":"Calc","table":"Sales"}]}"#,
        );
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
        // The deleted row 1 took A1; B2 moved to B1 and A3 to A2, still pointing at it.
        assert_eq!(
            evaluated.value("Calc", Addr { row: 2, col: 1 }),
            Some(Value::Number(10.0))
        );
        assert_eq!(
            evaluated.value("Calc", Addr { row: 1, col: 2 }),
            Some(Value::Number(5.0))
        );
        // T-2341: the table's cells are cleared, so D1 is empty (table-delete-table F!A6).
        assert_eq!(
            evaluated.value("Calc", Addr { row: 1, col: 4 }),
            Some(Value::Empty)
        );
    }

    #[test]
    fn bad_structural_ops_are_adapter_errors() {
        for op in [
            r#"{"op":"convert_table","sheet":"Calc","table":"Nope"}"#,
            r#"{"op":"delete_table","sheet":"Calc","table":"Nope"}"#,
            r#"{"op":"delete_row","sheet":"Calc","row":0}"#,
            r#"{"op":"delete_row","sheet":"Nope","row":1}"#,
            r#"{"op":"insert_column","sheet":"Calc","column":"A1"}"#,
            r#"{"op":"delete_column","sheet":"Calc","column":"XFE"}"#,
            r#"{"op":"delete_sheet","sheet":"Calc"}"#,
        ] {
            let json = format!(
                r#"{{"format":"rustsheet-case/v1",
                    "sheets":[{{"name":"Calc","cells":{{"A1":"h","A2":1}},
                                "tables":[{{"name":"Sales","ref":"A1:A2"}}]}}],
                    "sheet_ops":[{op}]}}"#
            );
            assert!(
                RustSheetEngine.evaluate_spec(&spec(&json), None).is_err(),
                "{op}"
            );
        }
    }

    #[test]
    fn unknown_or_malformed_sheet_ops_are_refused() {
        for ops in [
            r#"[{"op":"delete","sheet":"Data"}]"#,
            r#"[{"op":"move","sheet":"Data","to":-1}]"#,
            r#"[{"op":"add","sheet":"New","to":1}]"#,
        ] {
            let json = format!(r#"{{"format":"rustsheet-case/v1","sheets":[],"sheet_ops":{ops}}}"#);
            assert!(serde_json::from_str::<CaseSpec>(&json).is_err(), "{ops}");
        }
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"Calc","cells":{}}],
                "sheet_ops":[{"op":"rename","sheet":"Nope","to":"X"}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&spec, None).is_err());
    }

    #[test]
    fn spec_names_are_defined_before_cells() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","names":{"src":"=Data!$A$1"},
                "sheets":[{"name":"Calc","cells":{"A1":"=src*2"}},
                          {"name":"Data","cells":{"A1":5}}]}"#,
        );
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
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
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
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
        assert!(RustSheetEngine.evaluate_spec(&spec, None).is_err());
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
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
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
            r#"{"format":"rustsheet-case/v1","names":{"2real":"=1"},
                "sheets":[{"name":"Sheet1","cells":{}}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&spec, None).is_err());
    }

    #[test]
    fn iterative_cases_run_the_oracle_calculations() {
        let parse = |json: &str| serde_json::from_str::<CaseSpec>(json).expect("spec");
        let plain = parse(r#"{"format":"rustsheet-case/v1","sheets":[]}"#);
        assert_eq!(calculations(&plain), 1);
        let off =
            parse(r#"{"format":"rustsheet-case/v1","sheets":[],"calculation":{"iterate":false}}"#);
        assert_eq!(calculations(&off), 1);
        let on =
            parse(r#"{"format":"rustsheet-case/v1","sheets":[],"calculation":{"iterate":true}}"#);
        assert_eq!(calculations(&on), ORACLE_CALCULATIONS);
    }

    #[test]
    fn spec_date_system_and_number_formats_build() {
        // No value reads either yet (SPEC-0005 step 3); the facade test covers the format.
        let ok = spec(
            r#"{"format":"rustsheet-case/v1","date_system":1904,"sheets":[{"name":"S",
                "cells":{"A1":-1.5,"B1":"=A1*2"},"number_formats":{"A1":"[h]:mm","C3":"0.0"}}]}"#,
        );
        let evaluated = RustSheetEngine.evaluate_spec(&ok, None).expect("evaluates");
        assert_eq!(
            evaluated.value("S", Addr { row: 1, col: 2 }),
            Some(Value::Number(-3.0))
        );
        let bad_system = spec(
            r#"{"format":"rustsheet-case/v1","date_system":1901,"sheets":[{"name":"S","cells":{}}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&bad_system, None).is_err());
        let bad_cell = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{},
                "number_formats":{"nope":"0"}}]}"#,
        );
        assert!(RustSheetEngine.evaluate_spec(&bad_cell, None).is_err());
    }

    #[test]
    fn spec_pivot_rendered_cells_reach_the_engine() {
        let pivot = |rendered: &str| {
            spec(&format!(
                r#"{{"format":"rustsheet-case/v1","sheets":[{{"name":"S","cells":{{
                    "A1":"Key","B1":"Val","A2":"a","B2":1,"A3":"b","B3":2,"H1":"=D2"}},
                    "pivot_tables":[{{"name":"Pv","source":"A1:B3","location":"D1","row":"Key",
                    "value":"Val","rendered":{rendered}}}]}}]}}"#
            ))
        };
        let ok = pivot(r#"{"D1":"Row Labels","E1":"Sum of Val","D2":"a","E2":1}"#);
        let evaluated = RustSheetEngine.evaluate_spec(&ok, None).expect("evaluates");
        // The rendered cell is a constant a formula reads.
        assert_eq!(
            evaluated.value("S", Addr { row: 1, col: 8 }),
            Some(Value::Text("a".into()))
        );
        // Without rendered cells the pivot has no location.
        let err = RustSheetEngine.evaluate_spec(&pivot("{}"), None).err();
        assert!(err.is_some_and(|e| e.contains("bake-pivots")));
    }

    #[test]
    fn spec_hidden_rows_and_tables_reach_the_engine() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
                "A1":"Key","B1":"Val","A2":"a","B2":1,"A3":"b","B3":2,"A4":"c","B4":4,
                "D1":"=SUBTOTAL(109,B2:B4)","D2":"=SUBTOTAL(9,B2:B4)","D3":"=SUM(Data[Val])"},
                "hidden_rows":[3],"tables":[{"name":"Data","ref":"A1:B4","filter":true}]}]}"#,
        );
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
        let at = |row| evaluated.value("S", Addr { row, col: 4 });
        // Row 3 is filtered out of the table: both SUBTOTAL forms skip it.
        assert_eq!(at(1), Some(Value::Number(5.0)));
        assert_eq!(at(2), Some(Value::Number(5.0)));
        assert_eq!(at(3), Some(Value::Number(7.0)));
    }

    #[test]
    fn spec_typed_formulas_go_in_after_the_tables() {
        // A typed `=...` reads a structured reference, so it goes in after the tables; typed text
        // that is not a formula stays a value (`(4)` is -4). T-2955, T-2801.
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
                "A1":"Col","A2":3,"A3":{"entered":"(4)"},
                "C1":{"entered":"=SUM(T[Col])"},"C2":{"entered":"=A3*2"}},
                "tables":[{"name":"T","ref":"A1:A3"}]}]}"#,
        );
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
        assert_eq!(
            evaluated.value("S", Addr { row: 1, col: 3 }),
            Some(Value::Number(-1.0))
        );
        assert_eq!(
            evaluated.value("S", Addr { row: 2, col: 3 }),
            Some(Value::Number(-8.0))
        );
    }

    #[test]
    fn spec_r1c1_entries_read_relative_to_their_cell_and_unparsable_ones_stay_blank() {
        // `R1C2` is $B$1 and `R[-2]C[1]` the cell two up and one right (T-3113); R1C1 text that
        // does not parse is left blank, as Excel's refused PATCH leaves it (decision 0179).
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
                "B1":5,"A2":{"entered_r1c1":"=R1C2+1"},"A3":{"entered_r1c1":"=R[-2]C[1]*2"},
                "A5":{"entered_r1c1":"=1+"},"C5":"=ISBLANK(A5)"}}]}"#,
        );
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
        let at = |row, col| evaluated.value("S", Addr { row, col });
        assert_eq!(at(2, 1), Some(Value::Number(6.0)));
        assert_eq!(at(3, 1), Some(Value::Number(10.0)));
        assert_eq!(at(5, 3), Some(Value::Bool(true)));
    }

    #[test]
    fn spec_totals_row_is_part_of_the_table() {
        // The facade takes the totals row (T-1671): `#All` covers it and `#Totals` reads it.
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
                "A1":"Key","B1":"Val","A2":"a","B2":1,"A3":"b","B3":2,"A4":"Total",
                "B4":"=SUBTOTAL(109,Data[Val])","D1":"=ROWS(Data[#All])",
                "D2":"=Data[[#Totals],[Val]]"},
                "tables":[{"name":"Data","ref":"A1:B4","totals_row":true}]}]}"#,
        );
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
        // B4 sums the two data rows (3); `#All` has four rows; `#Totals` reads B4.
        for (row, col, want) in [(4, 2, 3.0), (1, 4, 4.0), (2, 4, 3.0)] {
            assert_eq!(
                evaluated.value("S", Addr { row, col }),
                Some(Value::Number(want))
            );
        }
    }

    #[test]
    fn spec_legacy_formulas_are_implicitly_intersected() {
        let spec = spec(
            r#"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
                "A1":1,"A2":2,"H7":"=A1:A2","H8":"=FORMULATEXT(H7)","H9":"=A1:A2"},
                "legacy_formulas":["H7"]}]}"#,
        );
        let evaluated = RustSheetEngine
            .evaluate_spec(&spec, None)
            .expect("evaluates");
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
        assert!(RustSheetEngine.evaluate_spec(&spec, None).is_err());
    }
}
