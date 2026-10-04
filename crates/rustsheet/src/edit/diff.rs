//! The [`ChangeSet`] diff (SPEC-0001): capture every formula and spill cell before an edit, let
//! the ops add the cells they touch, and compare with the recalculated workbook afterwards.
//! M3 replaces this with dirty-set tracking.

use std::collections::{HashMap, HashSet};

use rustsheet_engine::types::{Cell, Worksheet};
use rustsheet_engine::{KeyBuild, Model};

use super::{InputChange, ValueChange};
use crate::refs::{CellRef, SheetId};
use crate::value::{CellView, ErrorKind, Value};
use crate::workbook::sheet_metas;

/// `(sheet id, 1-based row, 1-based column)`.
pub(super) type Key = (u32, i32, i32);

pub(super) type KeyMap<V> = HashMap<Key, V, KeyBuild>;
pub(super) type KeySet = HashSet<Key, KeyBuild>;

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
pub(super) struct Before {
    /// Sheet id to its position at capture time; ranks refer to these positions.
    positions: HashMap<u32, usize>,
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
        let sheet = self.positions.get(&key.0).copied();
        (sheet.unwrap_or(usize::MAX), key.1, key.2)
    }

    /// Captures the cell at `key` unless it already is.
    pub(super) fn touch(&mut self, key: Key, ws: &Worksheet, model: &Model) {
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

pub(super) fn capture_derived(model: &Model) -> Before {
    let sheets: Vec<u32> = model
        .workbook
        .worksheets
        .iter()
        .map(Worksheet::get_sheet_id)
        .collect();
    let mut before = Before {
        positions: sheets.iter().enumerate().map(|(i, &id)| (id, i)).collect(),
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

pub(super) fn to_cell_ref(key: Key) -> CellRef {
    // Engine coordinates are 1-based and in range; the facade's are 0-based.
    CellRef {
        sheet: SheetId::new(key.0),
        row: u32::try_from(key.1 - 1).unwrap_or(0),
        col: u32::try_from(key.2 - 1).unwrap_or(0),
    }
}

/// The sort key of a sheet: its position now, deleted sheets last.
fn sheet_order(model: &Model) -> HashMap<u32, u32> {
    sheet_metas(model)
        .into_iter()
        .map(|s| (s.id.get(), s.index))
        .collect()
}

pub(super) fn input_changes(model: &Model, inputs: Vec<(Key, Option<String>)>) -> Vec<InputChange> {
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
    sort_by_position(changes, |c| c.cell)
}

pub(super) fn value_changes(model: &Model, before: &mut Before) -> Vec<ValueChange> {
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
        positions,
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
        let position = positions.get(&id).copied().unwrap_or(usize::MAX);
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
    sort_by_position(changes, |c| c.cell)
}

/// Orders changes by (sheet position, row, column), the order cells are stored in.
fn sort_by_position<T>(mut changes: Vec<(u32, T)>, cell: impl Fn(&T) -> CellRef) -> Vec<T> {
    changes.sort_by_key(|(index, c)| {
        let cell = cell(c);
        (*index, cell.row, cell.col)
    });
    changes.into_iter().map(|(_, c)| c).collect()
}
