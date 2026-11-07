//! The "engine under test" abstraction. The harness never calls engine internals
//! directly, so other engines (e.g. upstream IronCalc, a future rewritten core)
//! can be plugged in and compared side by side.

use std::collections::BTreeMap;

use rustsheet::{
    CellRef, Clock, DateSystem, ErrorKind, Input, LoadOptions, RangeRef, SaveOptions, SheetId,
    Value as ApiValue, Workbook, WorkbookOptions, XlError,
};

use rustsheet_engine::text::{lone_escape, Segment};
use rustsheet_engine::Text;

use crate::case::{split_qualified, ApiName, CaseSpec, CellInput, SheetOp, SheetSpec, TableSpec};
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
    /// The workbook's pivot tables and their locations (T-4124); empty when the engine has none
    /// or cannot list them.
    fn pivot_places(&self) -> Vec<rustsheet::PivotPlace> {
        Vec::new()
    }
    /// The cells the engine would write for the pivot `name` on `sheet` if it laid it out again
    /// from its saved definition and cache, changing nothing (T-4131): by cell, empty ones left
    /// out.
    ///
    /// # Errors
    /// Why the pivot cannot be laid out; by default, that the engine cannot lay out pivots.
    fn pivot_cells(&self, _sheet: &str, _name: &str) -> Result<Vec<(Addr, Value)>, String> {
        Err("this engine cannot lay out a pivot".into())
    }
    /// The workbook as the engine's xlsx export saves it now, after the evaluation: an
    /// engine-export case reads the `rowItems`/`colItems` the engine would save from it (T-4177).
    ///
    /// # Errors
    /// Why the workbook cannot be saved; by default, that the engine cannot save one.
    fn saved_package(&self) -> Result<Vec<u8>, String> {
        Err("this engine cannot save a workbook".into())
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
pub(crate) fn harness_options(name: Option<&str>) -> Result<WorkbookOptions, String> {
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

/// A facade value as the harness scores it; text as the facade gives it.
fn harness_value(value: ApiValue) -> Value {
    match value {
        ApiValue::Number(n) => Value::Number(n),
        ApiValue::Text(s) => Value::Text(s.to_string()),
        ApiValue::Bool(b) => Value::Bool(b),
        ApiValue::Error(e) => Value::Error(e.kind.stored_literal().to_string()),
        // `ApiValue` is non-exhaustive; `Empty` and future variants score as empty.
        _ => Value::Empty,
    }
}

impl Evaluated for RustSheetEvaluated {
    fn pivot_places(&self) -> Vec<rustsheet::PivotPlace> {
        self.workbook.pivot_places()
    }

    fn pivot_cells(&self, sheet: &str, name: &str) -> Result<Vec<(Addr, Value)>, String> {
        let id = self.workbook.sheet_id(sheet).map_err(|e| e.to_string())?;
        let cells = (self.workbook.pivot_table_cells(id, name)).map_err(|e| e.to_string())?;
        Ok(cells
            .into_iter()
            .map(|(cell, value)| {
                let at = Addr {
                    row: cell.row + 1,
                    col: cell.col + 1,
                };
                (at, harness_value(value))
            })
            .collect())
    }

    fn sheet_names(&self) -> Vec<String> {
        self.workbook.sheets().into_iter().map(|s| s.name).collect()
    }

    fn saved_package(&self) -> Result<Vec<u8>, String> {
        (self.workbook.to_xlsx_bytes(SaveOptions::default())).map_err(|e| e.to_string())
    }

    fn value(&self, sheet: &str, addr: Addr) -> Option<Value> {
        let cell = self.cell(sheet, addr)?;
        Some(match self.workbook.value(cell).ok()? {
            ApiValue::Text(s) => Value::Text(match self.workbook.text_utf16(cell) {
                Ok(Some(units)) => render_lone_units(&units),
                _ => s.to_string(),
            }),
            other => harness_value(other),
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
        Ok(Box::new(RustSheetEvaluated {
            workbook: build_spec(spec, name)?,
        }))
    }
}

/// The workbook a spec case describes, built and recalculated as [`Engine::evaluate_spec`]
/// evaluates it; the engine-export cases (T-4177) save it.
///
/// # Errors
/// A human-readable message when the spec cannot be materialized.
pub(crate) fn build_spec(spec: &CaseSpec, name: Option<&str>) -> Result<Workbook, String> {
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
    let (entered, _) = workbook
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
            define_names(tx, spec, &ids, false)?;
            // Values first, then tables (they read their headers), then stored formulas (a
            // structured reference parses only once its table exists). Entered text is kept
            // for after this edit, the workbook's load ([`type_entries`]).
            let mut entered = Vec::new();
            for phase in [Phase::Values, Phase::Stored] {
                if phase == Phase::Stored {
                    // Before the pivots too: a date-formatted source cell is a date to a
                    // pivot's refresh (probes/t4287-pivot-group-dates-as-numbers). Set again
                    // below, as a stored formula's own format replaces it.
                    for (sheet, &id) in spec.sheets.iter().zip(&ids) {
                        set_number_formats(tx, sheet, id)?;
                    }
                }
                for (sheet, &id) in spec.sheets.iter().zip(&ids) {
                    if phase == Phase::Stored {
                        spec_pivots::add_pivots(tx, spec, &ids, sheet, id)?;
                        for table in &sheet.tables {
                            add_table(tx, id, table, &sheet.hidden_rows)?;
                        }
                    }
                    if phase == Phase::Values {
                        spec_pivots::write_rendered(tx, sheet, id, spec_input)?;
                    }
                    for (addr, a1, input_spec) in row_major(&sheet.cells)? {
                        let Some((input, input_phase)) = cell_input(sheet, a1, input_spec) else {
                            continue;
                        };
                        let target = CellRef::new(id, addr.row - 1, addr.col - 1)?;
                        if input_phase == Phase::Entered {
                            if phase == Phase::Stored {
                                entered.push((target, input));
                            }
                            continue;
                        }
                        if input_phase == phase {
                            tx.set(target, input)?;
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
                set_number_formats(tx, sheet, id)?;
                for &row in &sheet.hidden_rows {
                    let row = row.checked_sub(1).ok_or(rustsheet::RefError::OutOfBounds)?;
                    tx.set_row_hidden(id, row, true)?;
                }
            }
            Ok(entered)
        })
        .map_err(|e| e.to_string())?;
    // Excel opens the built file, calculates and then refreshes its `refreshOnLoad` pivots
    // before the oracle types any entry.
    spec_pivots::load_defined(&mut workbook, spec)?;
    type_entries(&mut workbook, entered)?;
    apply_sheet_ops(&mut workbook, &spec.sheet_ops)?;
    if let Some(active) = &spec.active_sheet {
        let id = workbook
            .sheet_id(active)
            .map_err(|e| format!("active_sheet {active}: {e}"))?;
        workbook
            .set_active_sheet(id)
            .map_err(|e| format!("active_sheet {active}: {e}"))?;
    }
    define_api_names(&mut workbook, &spec.api_names)?;
    // The edit calculated once; an iterative case gets the oracle's other calculations.
    for _ in 1..calculations(spec) {
        workbook.recalculate();
    }
    Ok(workbook)
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

/// Types a case's entered cells after the workbook loads, each as its own edit, in row-major
/// order sheet by sheet, as the oracle types them: Excel recalculates after each, so a formula
/// that reads a cell typed later, or a circular one, keeps the value it had before that entry
/// (`ISBLANK` of a typed self-reference is TRUE, a cycle typed one cell at a time keeps the
/// first cell's value: probes/t3898-typed-circular-readers, T-3898), and a typed formula sees
/// every stored cell (probes/t2983-formulas-and-blank-enter-blank, T-3024). Excel refuses a typed
/// formula that does not parse and the cell stays blank (decision 0179, T-3213); a typed formula
/// past a resource limit still fails the case (decision 0168).
fn type_entries(workbook: &mut Workbook, entered: Vec<(CellRef, Input)>) -> Result<(), String> {
    for (target, input) in entered {
        match workbook.edit(|tx| tx.set(target, input)) {
            Err(rustsheet::EditError::InvalidInput { message, .. })
                if message.starts_with(rustsheet_engine::TYPED_FORMULA_DOES_NOT_PARSE) => {}
            result => {
                result.map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// Defines a case's `api_names` after its sheet edits, each as its own edit, as the oracle replays
/// them with Graph `names/add` (T-3264; a sheet-scoped one on its worksheet, T-4014). Excel refusing a name (400 `InvalidArgument`) leaves it
/// undefined, so a name the engine refuses the same way is skipped: a body nesting more than 65
/// calls ([`rustsheet_engine::expressions::parser::TOO_MANY_NESTED_CALLS_MESSAGE`]). Any other refusal fails the case.
fn define_api_names(workbook: &mut Workbook, names: &[ApiName]) -> Result<(), String> {
    for ApiName {
        name,
        formula,
        sheet,
    } in names
    {
        let scope = match sheet {
            Some(sheet) => Some(
                workbook
                    .sheet_id(sheet)
                    .map_err(|e| format!("api name {name} on {sheet}: {e}"))?,
            ),
            None => None,
        };
        let defined = workbook.edit(|tx| match scope {
            Some(id) => tx.define_sheet_name(id, name, formula),
            None => tx.define_name(name, formula),
        });
        match defined {
            Err(rustsheet::EditError::DefinedName(message))
                if message.contains(
                    rustsheet_engine::expressions::parser::TOO_MANY_NESTED_CALLS_MESSAGE,
                ) => {}
            result => {
                result.map_err(|e| format!("api name {name}: {e}"))?;
            }
        }
    }
    Ok(())
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
            SheetOp::AddTable {
                sheet,
                table,
                range,
            } => {
                let id = sheet_id_of(workbook, sheet)?;
                workbook
                    .edit(|tx| tx.add_table(range_ref(id, range)?, table, false))
                    .map(drop)
            }
            SheetOp::DeleteTable { table, .. } => workbook
                .delete_table(table)
                .map(|()| workbook.recalculate()),
            SheetOp::InsertRow { sheet, row } => {
                let id = sheet_id_of(workbook, sheet)?;
                let row = row
                    .checked_sub(1)
                    .ok_or_else(|| format!("sheet op {op:?}: rows are 1-based"))?;
                workbook
                    .insert_rows(id, row, 1)
                    .map(|()| workbook.recalculate())
            }
            SheetOp::DeleteRow { sheet, row, count } => {
                let id = sheet_id_of(workbook, sheet)?;
                let row = row
                    .checked_sub(1)
                    .ok_or_else(|| format!("sheet op {op:?}: rows are 1-based"))?;
                let count = count.unwrap_or(1);
                if count == 0 {
                    return Err(format!("sheet op {op:?}: count is at least 1"));
                }
                workbook
                    .delete_rows(id, row, count)
                    .map(|()| workbook.recalculate())
            }
            SheetOp::InsertColumn {
                sheet,
                column,
                count,
            }
            | SheetOp::DeleteColumn {
                sheet,
                column,
                count,
            } => {
                let count = count.unwrap_or(1);
                if count == 0 {
                    return Err(format!("sheet op {op:?}: count is at least 1"));
                }
                let id = sheet_id_of(workbook, sheet)?;
                let column = Addr::col_index(column)
                    .map(|c| c - 1)
                    .ok_or_else(|| format!("sheet op {op:?}: not a column letter"))?;
                let result = if matches!(op, SheetOp::InsertColumn { .. }) {
                    workbook.insert_columns(id, column, count)
                } else {
                    workbook.delete_columns(id, column, count)
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

/// Adds the spec table `table` on sheet `id`: its range and rows, each of its `criteria` as the
/// values filter spec2xlsx writes for it (so a column delete that takes one keeps the rows the
/// others reject, T-4803), and its calculated columns. `hidden_rows` are its sheet's (one-based).
fn add_table(
    tx: &mut rustsheet::Txn<'_>,
    id: SheetId,
    table: &TableSpec,
    hidden_rows: &[u32],
) -> Result<(), rustsheet::EditError> {
    let range = range_ref(id, &table.range)?;
    tx.add_table_with_rows(
        range,
        &table.name,
        filters_hidden_rows(table, range, hidden_rows),
        (table.header_row, table.totals_row),
    )?;
    for criterion in &table.criteria {
        let rejected: Vec<u32> = criterion
            .hides
            .iter()
            .map(|r| r.saturating_sub(1))
            .collect();
        let column = usize::try_from(criterion.column).unwrap_or(usize::MAX);
        tx.filter_table_column(&table.name, column, &rejected)?;
    }
    for (letter, formula) in &table.calculated {
        let (column, text) = calculated_column(&table.range, letter, formula)?;
        tx.set_table_column_formula(&table.name, column, text)?;
    }
    Ok(())
}

/// Whether the spec table's filter makes its sheet's hidden rows filtered: Excel calls them
/// filtered only for a filterColumn with a criterion inside the table, not for a criterion-less
/// one, a `<colorFilter/>` without a dxfId (probes/t4558-color-filter-subtotal), one whose `colId`
/// is past the table's width (probes/t4003-filter-column-criterion) or the empty `<filters>`
/// spec2xlsx writes for a values filter when no data row is visible (`hidden_rows`, one-based;
/// probes/t4749-empty-values-filter), and the facade's `filtered` flag is that "filtered"
/// (docs/excel-semantics/numbers.md, "SUBTOTAL and AGGREGATE over hidden and filtered rows").
fn filters_hidden_rows(table: &TableSpec, range: RangeRef, hidden_rows: &[u32]) -> bool {
    let width = (u64::from(range.right) + 1).saturating_sub(u64::from(range.left));
    let first = range.top + u32::from(table.header_row);
    let last = range.bottom.saturating_sub(u32::from(table.totals_row));
    let keeps_a_row = (first..=last).any(|row| !hidden_rows.contains(&(row + 1)));
    table.filter
        && table.filter_column.as_ref().map_or(keeps_a_row, |c| {
            c.criterion && !c.color && u64::from(c.col_id) < width && keeps_a_row
        })
}

/// Gives each cell of `sheet` (sheet `id`) its spec number format.
fn set_number_formats(
    tx: &mut rustsheet::Txn<'_>,
    sheet: &SheetSpec,
    id: SheetId,
) -> Result<(), rustsheet::EditError> {
    for (a1, format) in &sheet.number_formats {
        let addr = Addr::parse(a1)
            .ok_or_else(|| rustsheet::EditError::Engine(format!("bad address {a1}")))?;
        tx.set_number_format(CellRef::new(id, addr.row - 1, addr.col - 1)?, format)?;
    }
    Ok(())
}

/// A spec table's calculated column in column `letter` of table range `a1`: its 0-based index in
/// the table and its formula without `=`. The text is exported as is, so a `[@Col]` shorthand
/// (not file form) is refused.
fn calculated_column<'f>(
    a1: &str,
    letter: &str,
    formula: &'f str,
) -> Result<(usize, &'f str), rustsheet::EditError> {
    let bad = || rustsheet::EditError::Table(format!("calculated column {letter} outside {a1}"));
    let (first, last) = parse_range(a1).ok_or_else(bad)?;
    let col = Addr::col_index(letter).ok_or_else(bad)?;
    if !(first.col..=last.col).contains(&col) {
        return Err(bad());
    }
    if formula.contains("[@") {
        return Err(rustsheet::EditError::Table(format!(
            "calculated column {letter}: write {formula} in file form"
        )));
    }
    let index = usize::try_from(col - first.col).map_err(|_| bad())?;
    Ok((index, formula.trim_start_matches('=')))
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
fn cell_input(sheet: &SheetSpec, a1: &str, input_spec: &CellInput) -> Option<(Input, Phase)> {
    let mut input = spec_input(input_spec)?;
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
        Input::ParseR1C1(_) => Phase::Entered,
        _ => Phase::Values,
    };
    Some((input, phase))
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
        // Typed in R1C1 mode, read against its cell (`Model::set_typed_r1c1_user_input`).
        CellInput::EnteredR1c1 { entered_r1c1 } => Input::ParseR1C1(entered_r1c1.clone()),
        CellInput::Typed(Value::Empty) => return None,
    })
}

#[cfg(test)]
mod tests;
