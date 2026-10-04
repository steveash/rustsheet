//! Atomic edits (SPEC-0001 "Edits and change sets"): [`Workbook::edit`], [`Txn`], [`ChangeSet`].
//!
//! Every op applies to the engine immediately and records what it overwrote in an undo log;
//! an `Err` from the closure replays the log in reverse, so a failed edit never clones the
//! workbook. Nothing recalculates inside the closure: one `evaluate` runs after it returns
//! `Ok` (sheet-level engine ops reparse and so evaluate on their own, see [`Txn::add_sheet`]).
//!
//! The [`ChangeSet`] is computed by diffing (SPEC-0001): before the closure runs the facade
//! captures every formula and spill cell, ops add the cells they touch, and after the recalc
//! the captured values are compared with the current ones.

use std::collections::{HashMap, HashSet};

use rustsheet_engine::types::{Cell, Link, Worksheet};
use rustsheet_engine::Model;

use crate::error::{EditError, RefError};
use crate::refs::{CellRef, SheetId, Target};
use crate::value::{CellView, ErrorKind, Value};
use crate::workbook::{engine_rc, resolve_range, sheet_index, sheet_metas, Workbook};

/// What to store in a cell.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Input {
    /// A typed value, stored as is (text is never reinterpreted as a number or formula).
    Value(Value),
    /// A formula; must start with `=`.
    Formula(String),
    /// Text as a user would type it: `=` starts a formula, numbers, booleans, errors and
    /// percentages/currencies are recognised, a leading `'` forces text.
    Parse(String),
    /// Empties the cell (its style is kept).
    Clear,
}

impl From<f64> for Input {
    fn from(n: f64) -> Self {
        Self::Value(Value::Number(n))
    }
}

impl From<bool> for Input {
    fn from(b: bool) -> Self {
        Self::Value(Value::Bool(b))
    }
}

impl From<&str> for Input {
    fn from(text: &str) -> Self {
        Self::Parse(text.to_string())
    }
}

impl From<String> for Input {
    fn from(text: String) -> Self {
        Self::Parse(text)
    }
}

impl From<Value> for Input {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

/// A cell whose stored input (formula or literal text) changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputChange {
    /// The cell.
    pub cell: CellRef,
    /// The input before the edit, `None` for an empty cell.
    pub before: Option<String>,
    /// The input after the edit, `None` for an empty cell.
    pub after: Option<String>,
}

/// A cell whose computed value changed.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueChange {
    /// The cell.
    pub cell: CellRef,
    /// The value before the edit.
    pub before: Value,
    /// The value after the edit.
    pub after: Value,
}

/// A change to the sheet list.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SheetChange {
    /// A sheet was added.
    Added(SheetId),
    /// A sheet was renamed.
    Renamed {
        /// The sheet.
        id: SheetId,
        /// The old name.
        from: String,
        /// The new name.
        to: String,
    },
    /// A sheet was deleted.
    Deleted {
        /// The sheet.
        id: SheetId,
        /// Its name when it was deleted.
        name: String,
    },
}

/// Everything an edit changed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChangeSet {
    /// Cells whose input changed, sorted by (sheet order, row, column). Cells of a sheet deleted
    /// in the edit are reported by [`SheetChange::Deleted`] only.
    pub inputs: Vec<InputChange>,
    /// Cells whose value changed, sorted by (sheet order, row, column), cells of deleted sheets
    /// last. Empty when change tracking is off.
    pub values: Vec<ValueChange>,
    /// Sheet-list changes in the order they happened.
    pub sheets: Vec<SheetChange>,
}

/// `(sheet id, 1-based row, 1-based column)`.
type Key = (u32, i32, i32);

/// Cheap multiplicative hasher for [`Key`]s: the keys are small integers we produce ourselves, so
/// `SipHash`'s collision resistance buys nothing and costs a large share of the diff.
#[derive(Default, Clone, Copy)]
struct KeyHasher(u64);

impl std::hash::Hasher for KeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u64(u64::from(b));
        }
    }

    fn write_u32(&mut self, n: u32) {
        self.write_u64(u64::from(n));
    }

    fn write_i32(&mut self, n: i32) {
        self.write_u64(u64::from(n.cast_unsigned()));
    }

    fn write_u64(&mut self, n: u64) {
        self.0 = (self.0.rotate_left(5) ^ n).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
}

type KeyMap<V> = HashMap<Key, V, std::hash::BuildHasherDefault<KeyHasher>>;

/// Largest rectangle a single `set` snapshots for its spill footprint. Imported array ranges are
/// untrusted, so the loop must not follow them blindly.
const MAX_FOOTPRINT_CELLS: i64 = 1 << 22;

enum Undo {
    /// Cells (and the target's link) as they were before a `set`.
    Cells {
        sheet_id: u32,
        cells: Vec<(i32, i32, Option<Cell>)>,
        link: ((i32, i32), Option<Link>),
    },
    AddedSheet(u32),
    Renamed {
        sheet_id: u32,
        old_name: String,
    },
    DeletedSheet {
        index: u32,
        sheet: Box<Worksheet>,
    },
}

/// A captured cell value. Text lives in a shared arena so capturing a text cell allocates
/// nothing per cell.
enum Snap {
    Empty,
    Number(u64),
    Bool(bool),
    Text(usize, usize),
    Error(ErrorKind, Option<Box<str>>),
}

/// `(position of the sheet at capture time, row, column)`: the order cells are stored in.
type Rank = (usize, i32, i32);

/// The pre-edit values of every cell that can change without being written.
#[derive(Default)]
struct Before {
    /// Sheet ids in capture-time order; ranks refer to positions in this list.
    sheets: Vec<u32>,
    /// Every formula and spill cell, in storage order (sorted by rank), so the post-edit diff
    /// can merge against the engine's own iteration order instead of hashing every cell.
    derived: Vec<(Rank, Key, Snap)>,
    /// Cells a write touched that were not derived before; the flag marks the ones the diff
    /// already compared.
    extra: KeyMap<(Snap, bool)>,
    arena: String,
}

impl Before {
    fn snap(&mut self, view: CellView<'_>) -> Snap {
        match view {
            CellView::Empty => Snap::Empty,
            CellView::Number(n) => Snap::Number(n.to_bits()),
            CellView::Bool(b) => Snap::Bool(b),
            CellView::Text(s) => {
                let start = self.arena.len();
                self.arena.push_str(s);
                Snap::Text(start, s.len())
            }
            CellView::Error(kind, detail) => Snap::Error(kind, detail.map(Into::into)),
        }
    }

    fn rank(&self, key: Key) -> Rank {
        let sheet = self.sheets.iter().position(|&id| id == key.0);
        (sheet.unwrap_or(usize::MAX), key.1, key.2)
    }

    /// Captures the cell at `key` unless it already is.
    fn touch(&mut self, key: Key, ws: &Worksheet, model: &Model) {
        let rank = self.rank(key);
        if self.derived.binary_search_by_key(&rank, |e| e.0).is_ok()
            || self.extra.contains_key(&key)
        {
            return;
        }
        let snap = self.snap(view_at(ws, model, key.1, key.2));
        self.extra.insert(key, (snap, false));
    }

    /// Bitwise number equality (`-0.0 != 0.0`, `NaN == NaN`), exact text equality.
    fn unchanged(arena: &str, snap: &Snap, now: CellView<'_>) -> bool {
        match (snap, now) {
            (Snap::Empty, CellView::Empty) => true,
            (Snap::Number(bits), CellView::Number(n)) => *bits == n.to_bits(),
            (Snap::Bool(a), CellView::Bool(b)) => *a == b,
            (Snap::Text(start, len), CellView::Text(s)) => {
                arena.get(*start..start + len) == Some(s)
            }
            (Snap::Error(kind, detail), CellView::Error(k, d)) => {
                *kind == k && detail.as_deref() == d
            }
            _ => false,
        }
    }

    fn value(arena: &str, snap: &Snap) -> Value {
        match snap {
            Snap::Empty => Value::Empty,
            Snap::Number(bits) => Value::Number(f64::from_bits(*bits)),
            Snap::Bool(b) => Value::Bool(*b),
            Snap::Text(start, len) => {
                Value::Text(arena.get(*start..start + len).unwrap_or_default().into())
            }
            Snap::Error(kind, detail) => Value::Error(crate::value::XlError {
                kind: *kind,
                detail: detail.as_deref().map(Into::into),
            }),
        }
    }
}

fn view_at<'a>(ws: &'a Worksheet, model: &'a Model, row: i32, col: i32) -> CellView<'a> {
    ws.cell(row, col).map_or(CellView::Empty, |cell| {
        CellView::of(cell, &model.workbook.shared_strings)
    })
}

/// Cells whose value can change without being written: formulas and their spill members.
fn is_derived(cell: &Cell) -> bool {
    matches!(
        cell,
        Cell::CellFormula { .. } | Cell::ArrayFormula { .. } | Cell::SpillCell { .. }
    )
}

fn capture_derived(model: &Model) -> Before {
    let mut before = Before {
        sheets: model
            .workbook
            .worksheets
            .iter()
            .map(Worksheet::get_sheet_id)
            .collect(),
        ..Before::default()
    };
    for (position, ws) in model.workbook.worksheets.iter().enumerate() {
        let sheet = ws.get_sheet_id();
        for (row, col, cell) in ws.sheet_data.cells() {
            if is_derived(cell) {
                let snap = before.snap(CellView::of(cell, &model.workbook.shared_strings));
                before
                    .derived
                    .push(((position, row, col), (sheet, row, col), snap));
            }
        }
    }
    before
}

fn to_cell_ref(key: Key) -> CellRef {
    // Engine coordinates are 1-based and in range; the facade's are 0-based.
    CellRef {
        sheet: SheetId::new(key.0),
        row: u32::try_from(key.1 - 1).unwrap_or(0),
        col: u32::try_from(key.2 - 1).unwrap_or(0),
    }
}

/// The write-only handle passed to [`Workbook::edit`].
pub struct Txn<'a> {
    model: &'a mut Model,
    undo: Vec<Undo>,
    before: Option<Before>,
    /// First-seen input text of each written cell, in first-write order.
    inputs: Vec<(Key, Option<String>)>,
    input_seen: HashSet<Key>,
    sheets: Vec<SheetChange>,
    /// A sheet-level engine op reparses and evaluates, so rollback must re-evaluate.
    sheet_op: bool,
}

fn engine_edit_error(message: String, cell: Option<CellRef>) -> EditError {
    if message.starts_with("Invalid name for a sheet")
        || message.contains("already exists")
        || message.starts_with("Sheet already exists")
    {
        return EditError::SheetName(message);
    }
    if message.starts_with("Cannot delete only sheet") {
        return EditError::LastSheet;
    }
    let Some(cell) = cell else {
        return EditError::Engine(message);
    };
    match RefError::from_engine_message(&message, &cell.to_string()) {
        Some(e) => EditError::Ref(e),
        None => EditError::InvalidInput { cell, message },
    }
}

impl Txn<'_> {
    /// Stores `input` in a cell. A range target must be a single cell.
    ///
    /// # Errors
    /// [`EditError::Ref`] when the target does not resolve to one cell,
    /// [`EditError::InvalidInput`] when the engine rejects the input (a merged or array-formula
    /// cell, a formula without `=`, a non-finite number).
    pub fn set<'t>(
        &mut self,
        r: impl Into<Target<'t>>,
        input: impl Into<Input>,
    ) -> Result<(), EditError> {
        let range = resolve_range(self.model, r.into())?;
        if range.top != range.bottom || range.left != range.right {
            return Err(RefError::Unsupported.into());
        }
        let sheet = sheet_index(self.model, range.sheet)?;
        let (row, col) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        let cell = CellRef {
            sheet: range.sheet,
            row: range.top,
            col: range.left,
        };
        let key = (range.sheet.get(), row, col);
        self.log_cell(sheet, key, cell)?;
        self.apply(sheet, row, col, input.into(), cell)
    }

    /// Records the input text, the undo entry and the pre-edit values for a write to `key`.
    fn log_cell(&mut self, sheet: u32, key: Key, cell: CellRef) -> Result<(), EditError> {
        let (sheet_id, row, col) = key;
        if self.input_seen.insert(key) {
            let text = self.input_text(sheet, row, col);
            self.inputs.push((key, text));
        }
        let ws = self
            .model
            .workbook
            .worksheet(sheet)
            .map_err(EditError::Engine)?;
        // A spill member belongs to the anchor's footprint; an anchor owns its array range.
        let anchor = match ws.cell(row, col) {
            Some(Cell::SpillCell { a, .. }) => *a,
            _ => (row, col),
        };
        let mut positions = vec![(row, col)];
        if let Some(Cell::ArrayFormula {
            r: (width, height), ..
        }) = ws.cell(anchor.0, anchor.1)
        {
            if i64::from(*width) * i64::from(*height) > MAX_FOOTPRINT_CELLS {
                return Err(EditError::InvalidInput {
                    cell,
                    message: "spill extent too large to edit".to_string(),
                });
            }
            for r in anchor.0..anchor.0.saturating_add(*height) {
                for c in anchor.1..anchor.1.saturating_add(*width) {
                    if (r, c) != (row, col) {
                        positions.push((r, c));
                    }
                }
            }
        }
        let cells = positions
            .iter()
            .map(|&(r, c)| (r, c, ws.cell(r, c).cloned()))
            .collect();
        let link = ((row, col), ws.links.get(&(row, col)).cloned());
        if let Some(before) = &mut self.before {
            for &(r, c) in &positions {
                before.touch((sheet_id, r, c), ws, self.model);
            }
        }
        self.undo.push(Undo::Cells {
            sheet_id,
            cells,
            link,
        });
        Ok(())
    }

    fn input_text(&self, sheet: u32, row: i32, col: i32) -> Option<String> {
        self.model
            .get_localized_cell_content(sheet, row, col)
            .ok()
            .filter(|s| !s.is_empty())
    }

    fn apply(
        &mut self,
        sheet: u32,
        row: i32,
        col: i32,
        input: Input,
        cell: CellRef,
    ) -> Result<(), EditError> {
        let model = &mut *self.model;
        let result = match input {
            Input::Clear | Input::Value(Value::Empty) => {
                model.set_user_input(sheet, row, col, String::new())
            }
            Input::Parse(text) => model.set_user_input(sheet, row, col, text),
            Input::Formula(text) => {
                if !text.starts_with('=') {
                    return Err(EditError::InvalidInput {
                        cell,
                        message: format!("\"{text}\" is not a valid formula"),
                    });
                }
                model.set_user_input(sheet, row, col, text)
            }
            Input::Value(value) => {
                // Typed setters skip the spill/array/merge checks that `set_user_input` does.
                model
                    .prepare_cell_for_user_input(sheet, row, col)
                    .and_then(|()| match value {
                        Value::Number(n) if n.is_finite() => {
                            model.update_cell_with_number(sheet, row, col, n)
                        }
                        Value::Number(_) => Err("numbers must be finite".to_string()),
                        Value::Bool(b) => model.update_cell_with_bool(sheet, row, col, b),
                        Value::Text(s) => model.update_cell_with_text(sheet, row, col, &s),
                        Value::Error(e) => {
                            model.set_user_input(sheet, row, col, e.kind.literal().to_string())
                        }
                        Value::Empty => Ok(()),
                    })
            }
        };
        result.map_err(|m| engine_edit_error(m, Some(cell)))
    }

    /// The first sheet in workbook order.
    ///
    /// # Errors
    /// [`EditError::Engine`] if the workbook has no sheet (it never does).
    pub fn first_sheet(&self) -> Result<SheetId, EditError> {
        sheet_metas(self.model)
            .first()
            .map(|s| s.id)
            .ok_or_else(|| EditError::Engine("workbook has no sheet".to_string()))
    }

    /// Adds a sheet at the end.
    ///
    /// The engine reparses and re-evaluates the workbook on every sheet-level op, so these cost a
    /// recalculation each.
    ///
    /// # Errors
    /// [`EditError::SheetName`] when the name is empty, too long, duplicated or has forbidden
    /// characters.
    pub fn add_sheet(&mut self, name: &str) -> Result<SheetId, EditError> {
        self.model
            .add_sheet(name)
            .map_err(|m| engine_edit_error(m, None))?;
        self.sheet_op = true;
        let id = self
            .model
            .workbook
            .worksheets
            .last()
            .map_or(0, Worksheet::get_sheet_id);
        self.undo.push(Undo::AddedSheet(id));
        self.sheets.push(SheetChange::Added(SheetId::new(id)));
        Ok(SheetId::new(id))
    }

    /// Renames a sheet, rewriting formulas that refer to it.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, [`EditError::SheetName`] for a bad or duplicate
    /// name.
    pub fn rename_sheet(&mut self, id: SheetId, new_name: &str) -> Result<(), EditError> {
        let index = sheet_index(self.model, id)?;
        let old_name = sheet_name(self.model, index);
        self.model
            .rename_sheet_by_index(index, new_name)
            .map_err(|m| engine_edit_error(m, None))?;
        self.sheet_op = true;
        self.undo.push(Undo::Renamed {
            sheet_id: id.get(),
            old_name: old_name.clone(),
        });
        self.sheets.push(SheetChange::Renamed {
            id,
            from: old_name,
            to: new_name.to_string(),
        });
        Ok(())
    }

    /// Deletes a sheet. References to it become `#REF!`.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, [`EditError::LastSheet`] for the only sheet.
    pub fn delete_sheet(&mut self, id: SheetId) -> Result<(), EditError> {
        let index = sheet_index(self.model, id)?;
        let name = sheet_name(self.model, index);
        if let Some(before) = &mut self.before {
            if let Ok(ws) = self.model.workbook.worksheet(index) {
                for (row, col, _) in ws.sheet_data.cells() {
                    before.touch((id.get(), row, col), ws, self.model);
                }
            }
        }
        let sheet = self
            .model
            .take_sheet(index)
            .map_err(|m| engine_edit_error(m, None))?;
        self.sheet_op = true;
        self.undo.push(Undo::DeletedSheet {
            index,
            sheet: Box::new(sheet),
        });
        self.sheets.push(SheetChange::Deleted { id, name });
        Ok(())
    }
}

fn sheet_name(model: &Model, index: u32) -> String {
    model
        .workbook
        .worksheet(index)
        .map(Worksheet::get_name)
        .unwrap_or_default()
}

/// Replays `undo` in reverse. Failures cannot be reported past the original error and the
/// entries only restore state they themselves captured, so they are ignored.
fn roll_back(model: &mut Model, undo: Vec<Undo>, re_evaluate: bool) {
    for entry in undo.into_iter().rev() {
        match entry {
            Undo::Cells {
                sheet_id,
                cells,
                link: (link_key, link),
            } => {
                let Some(ws) = model
                    .workbook
                    .worksheets
                    .iter_mut()
                    .find(|ws| ws.get_sheet_id() == sheet_id)
                else {
                    continue;
                };
                for (row, col, cell) in cells {
                    match cell {
                        Some(cell) => {
                            ws.sheet_data.set_cell(row, col, cell);
                        }
                        None => {
                            ws.sheet_data.remove_cell(row, col);
                        }
                    }
                }
                match link {
                    Some(link) => {
                        ws.links.insert(link_key, link);
                    }
                    None => {
                        ws.links.remove(&link_key);
                    }
                }
            }
            Undo::AddedSheet(id) => {
                if let Some(index) = model
                    .workbook
                    .worksheets
                    .iter()
                    .position(|w| w.get_sheet_id() == id)
                {
                    model
                        .delete_sheet(u32::try_from(index).unwrap_or(u32::MAX))
                        .ok();
                }
            }
            Undo::Renamed { sheet_id, old_name } => {
                if let Some(index) = model
                    .workbook
                    .worksheets
                    .iter()
                    .position(|w| w.get_sheet_id() == sheet_id)
                {
                    model
                        .rename_sheet_by_index(u32::try_from(index).unwrap_or(u32::MAX), &old_name)
                        .ok();
                }
            }
            Undo::DeletedSheet { index, sheet } => {
                model.restore_sheet(index, *sheet).ok();
            }
        }
    }
    if re_evaluate {
        model.evaluate();
    }
}

impl Workbook {
    /// Turns the per-edit value diff on (default) or off. Off makes `edit`/`set` return a
    /// [`ChangeSet`] with empty `values`, for bulk loads that do not need the diff.
    pub fn set_track_changes(&mut self, on: bool) {
        self.track_changes = on;
    }

    /// Runs `f` as one transaction: all of its ops apply and the workbook recalculates once, or
    /// (when `f` returns `Err`) nothing changes.
    ///
    /// # Errors
    /// Whatever `f` returns; the workbook is then exactly as it was before the call.
    pub fn edit<T>(
        &mut self,
        f: impl FnOnce(&mut Txn<'_>) -> Result<T, EditError>,
    ) -> Result<(T, ChangeSet), EditError> {
        let before = self.track_changes.then(|| capture_derived(&self.model));
        let mut txn = Txn {
            model: &mut self.model,
            undo: Vec::new(),
            before,
            inputs: Vec::new(),
            input_seen: HashSet::new(),
            sheets: Vec::new(),
            sheet_op: false,
        };
        let result = f(&mut txn);
        let Txn {
            undo,
            before,
            inputs,
            sheets,
            sheet_op,
            ..
        } = txn;
        let out = match result {
            Ok(out) => out,
            Err(e) => {
                roll_back(&mut self.model, undo, sheet_op);
                return Err(e);
            }
        };
        self.model.evaluate();
        let changes = ChangeSet {
            inputs: input_changes(&self.model, inputs),
            values: before.map_or_else(Vec::new, |mut b| value_changes(&self.model, &mut b)),
            sheets,
        };
        Ok((out, changes))
    }

    /// Stores `input` in one cell; shorthand for a one-op [`Workbook::edit`].
    ///
    /// # Errors
    /// As [`Txn::set`].
    pub fn set<'a>(
        &mut self,
        r: impl Into<Target<'a>>,
        input: impl Into<Input>,
    ) -> Result<ChangeSet, EditError> {
        self.edit(|tx| tx.set(r, input))
            .map(|((), changes)| changes)
    }
}

/// The sort key of a sheet: its position now, deleted sheets last.
fn sheet_order(model: &Model) -> HashMap<u32, u32> {
    sheet_metas(model)
        .into_iter()
        .map(|s| (s.id.get(), s.index))
        .collect()
}

fn input_changes(model: &Model, inputs: Vec<(Key, Option<String>)>) -> Vec<InputChange> {
    let order = sheet_order(model);
    let mut changes: Vec<(u32, InputChange)> = Vec::new();
    for (key, before) in inputs {
        // A deleted sheet is reported by `SheetChange::Deleted`.
        let Some(&index) = order.get(&key.0) else {
            continue;
        };
        let after = model
            .get_localized_cell_content(index, key.1, key.2)
            .ok()
            .filter(|s| !s.is_empty());
        if after != before {
            changes.push((
                index,
                InputChange {
                    cell: to_cell_ref(key),
                    before,
                    after,
                },
            ));
        }
    }
    changes.sort_by_key(|(index, c)| (*index, c.cell.row, c.cell.col));
    changes.into_iter().map(|(_, c)| c).collect()
}

fn value_changes(model: &Model, before: &mut Before) -> Vec<ValueChange> {
    let order = sheet_order(model);
    let mut changes: Vec<(u32, ValueChange)> = Vec::new();
    let mut push = |key: Key, was: Value, now: CellView<'_>| {
        changes.push((
            order.get(&key.0).copied().unwrap_or(u32::MAX),
            ValueChange {
                cell: to_cell_ref(key),
                before: was,
                after: now.to_value(),
            },
        ));
    };
    let Before {
        sheets,
        derived,
        extra,
        arena,
    } = before;
    // Pass 1 merges the current derived cells with the captured ones; both are in storage order
    // (sheet, row, column). Entries the merge steps over no longer exist as derived cells.
    let mut gone: Vec<usize> = Vec::new();
    let mut next = 0;
    for ws in &model.workbook.worksheets {
        let id = ws.get_sheet_id();
        let position = sheets.iter().position(|&s| s == id).unwrap_or(usize::MAX);
        for (row, col, cell) in ws.sheet_data.cells() {
            if !is_derived(cell) {
                continue;
            }
            let rank = (position, row, col);
            while next < derived.len() && derived[next].0 < rank {
                gone.push(next);
                next += 1;
            }
            let now = CellView::of(cell, &model.workbook.shared_strings);
            let key = (id, row, col);
            if next < derived.len() && derived[next].0 == rank {
                let snap = &derived[next].2;
                if !Before::unchanged(arena, snap, now) {
                    push(key, Before::value(arena, snap), now);
                }
                next += 1;
            } else if let Some((snap, seen)) = extra.get_mut(&key) {
                // Written literal that is now a formula.
                *seen = true;
                if !Before::unchanged(arena, snap, now) {
                    push(key, Before::value(arena, snap), now);
                }
            } else if !matches!(now, CellView::Empty) {
                // New formula or grown spill.
                push(key, Value::Empty, now);
            }
        }
    }
    gone.extend(next..derived.len());
    // Pass 2: captured cells pass 1 did not reach (written literals, cleared cells, shrunk
    // spills, deleted sheets) are looked up directly.
    let sheet_map: HashMap<u32, &Worksheet> = model
        .workbook
        .worksheets
        .iter()
        .map(|ws| (ws.get_sheet_id(), ws))
        .collect();
    let now_at = |key: Key| {
        sheet_map
            .get(&key.0)
            .map_or(CellView::Empty, |ws| view_at(ws, model, key.1, key.2))
    };
    for index in gone {
        let (_, key, snap) = &derived[index];
        let now = now_at(*key);
        if !Before::unchanged(arena, snap, now) {
            push(*key, Before::value(arena, snap), now);
        }
    }
    for (&key, (snap, seen)) in extra.iter() {
        if !*seen {
            let now = now_at(key);
            if !Before::unchanged(arena, snap, now) {
                push(key, Before::value(arena, snap), now);
            }
        }
    }
    changes.sort_by_key(|(index, c)| (*index, c.cell.row, c.cell.col));
    changes.into_iter().map(|(_, c)| c).collect()
}

#[cfg(test)]
mod tests;
