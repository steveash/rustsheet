//! Whether a data table's results can have changed since it was last recomputed (T-0453).
//!
//! A recomputation evaluates the table's formula once per result cell (`data_tables.rs`), so a
//! table over a large model costs that model many times over at every evaluation. Its results
//! are a function of what the recomputation read: the cells (constants, formulas and their
//! styles), the shape of the sheets, and the workbook context (names, tables, settings, sheets,
//! hidden rows and columns). While a table is recomputed every sheet records what is read of it
//! (`sheet_reads.rs`); afterwards the contents of those reads are kept with the results. At the
//! next evaluation the table is recomputed only if any of that differs, or if its result cells no
//! longer hold its results.
//!
//! A table is always recomputed when its formula reaches a function whose value depends on
//! something else (the clock, a random draw, a cell's format or width, a pivot table), or when
//! iterative calculation has cycles to iterate (every evaluation runs another round of them). The spill
//! of a dynamic array it reads is left by the evaluation that last ran the array, and what a
//! recomputation reads there depends on it: a table is memoized only if the spills it read are
//! the ones it started from. The comparison is conservative: an
//! edit to a cell read only on the way to a value that does not matter still recomputes.

use std::collections::{BTreeMap, HashMap};

use crate::{
    constants::LAST_ROW,
    expressions::{token::Error, types::CellReferenceIndex},
    model::Model,
    sheet_reads::SheetReads,
    types::{ArrayKind, Cell, DataTable, DefinedName, SheetState, Table, WorkbookSettings},
};

/// Cells the memo of one table may hold (CLAUDE.md directive 5); a table that read more is
/// recomputed every time.
const MAX_MEMO_CELLS: usize = 1 << 18;

/// Calls whose value depends on more than the cells and the workbook context. Matched in the
/// formula text, so a match inside a string only costs a recomputation.
const UNMEMOIZABLE_CALLS: [&str; 8] = [
    "RAND(",
    "RANDBETWEEN(",
    "RANDARRAY(",
    "NOW(",
    "TODAY(",
    "CELL(",
    "INFO(",
    "GETPIVOTDATA(",
];

/// What a cell holds, as far as a formula reading it can tell, without the values the
/// evaluation stores in it.
#[derive(Debug, Clone, PartialEq)]
enum Content {
    Blank,
    Boolean(bool),
    /// The bits, so that the comparison is exact.
    Number(u64),
    Error(Error),
    Text(String),
    Formula(String),
    /// An array formula's anchor with its spill (width, height).
    Array(String, ArrayKind, (i32, i32)),
    /// A spill cell, by its anchor.
    Spill((i32, i32)),
}

/// A cell's content and its style, `None` for no cell.
type Held = Option<(Content, i32)>;

/// The spill (width, height) of every dynamic anchor, by `(sheet, row, column)`.
pub(crate) type Spills = HashMap<(u32, i32, i32), (i32, i32)>;

/// The contents of every position a recomputation read, with the sheets' extents if they were
/// read.
#[derive(Debug, Clone, PartialEq)]
struct ReadState {
    cells: BTreeMap<(u32, i32, i32), Held>,
    extents: Vec<(u32, i32, i32)>,
}

/// The workbook context every formula may read besides cells.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Context {
    defined_names: Vec<DefinedName>,
    tables: HashMap<String, Table>,
    settings: WorkbookSettings,
    /// The locale and language, by address: they are `'static` tables.
    locale: usize,
    language: usize,
    sheets: Vec<SheetContext>,
}

/// What formulas may read of a sheet besides its cells.
#[derive(Debug, Clone, PartialEq)]
struct SheetContext {
    name: String,
    sheet_id: u32,
    state: SheetState,
    hidden_rows: Vec<i32>,
    /// `(min, max)` of each hidden column span.
    hidden_columns: Vec<(i32, i32)>,
}

/// What one table's last recomputation read, and the results it wrote.
#[derive(Debug, Clone)]
struct TableMemo {
    sheet: u32,
    table: DataTable,
    /// The formula evaluations it took, charged again when it is not recomputed, so that the
    /// tables after it find the budget as a recomputation would leave it.
    cost: usize,
    reads: Vec<(u32, SheetReads)>,
    state: ReadState,
    results: Vec<Held>,
}

/// The memos of the last evaluation's tables, with the context they were made in.
#[derive(Debug, Clone, Default)]
pub(crate) struct DataTableMemo {
    context: Option<Context>,
    /// By the table's index in the evaluation's order; `None` for one that is not memoized.
    tables: Vec<Option<TableMemo>>,
}

impl Model {
    /// The workbook context of the formulas now.
    pub(crate) fn data_table_context(&self) -> Context {
        let workbook = &self.workbook;
        Context {
            defined_names: workbook.defined_names.clone(),
            tables: workbook.tables.clone(),
            settings: workbook.settings.clone(),
            locale: std::ptr::from_ref(self.locale) as usize,
            language: std::ptr::from_ref(self.language) as usize,
            sheets: (workbook.worksheets.iter())
                .map(|ws| SheetContext {
                    name: ws.name.clone(),
                    sheet_id: ws.sheet_id,
                    state: ws.state.clone(),
                    hidden_rows: ws.rows.iter().filter(|r| r.hidden).map(|r| r.r).collect(),
                    hidden_columns: (ws.cols.iter().filter(|c| c.hidden))
                        .map(|c| (c.min, c.max))
                        .collect(),
                })
                .collect(),
        }
    }

    /// The cost of the last recomputation of the table at index `index`, if it is `table` on
    /// `sheet`, the context is `context`, nothing it read has changed and its results are still
    /// in place: then it need not be recomputed.
    pub(crate) fn data_table_unchanged(
        &self,
        context: &Context,
        index: usize,
        (sheet, table): (u32, &DataTable),
    ) -> Option<usize> {
        let memo = &self.evaluation.data_table_memo;
        if memo.context.as_ref() != Some(context) {
            return None;
        }
        let entry = memo.tables.get(index)?.as_ref()?;
        let unchanged = entry.sheet == sheet
            && entry.table == *table
            && self.result_contents(sheet, table) == entry.results
            && self.read_state(&entry.reads, None).as_ref() == Some(&entry.state);
        unchanged.then_some(entry.cost)
    }

    /// Starts recording what is read of every sheet (`sheet_reads.rs`). Returns the spills the
    /// recomputation starts from.
    pub(crate) fn start_data_table_reads(&self) -> Spills {
        for ws in &self.workbook.worksheets {
            ws.sheet_data.start_reads();
        }
        (self.evaluation.anchor_order.iter())
            .filter_map(|at| match self.fetch_cell(*at)? {
                Cell::ArrayFormula {
                    r,
                    kind: ArrayKind::Dynamic,
                    ..
                } => Some(((at.sheet, at.row, at.column), *r)),
                _ => None,
            })
            .collect()
    }

    /// Stops the record [`Self::start_data_table_reads`] started from `spills`, and keeps it as
    /// the memo of the table at index `index`, whose recomputation took `cost` evaluations,
    /// unless `memoizable` is false or what was read rules it out. Called before the results are
    /// written; [`Self::seal_data_table_memo`] keeps them.
    pub(crate) fn finish_data_table_reads(
        &mut self,
        context: &Context,
        (index, sheet, table): (usize, u32, &DataTable),
        (cost, spills): (usize, &Spills),
        memoizable: bool,
    ) {
        let reads: Vec<(u32, SheetReads)> = (self.workbook.worksheets.iter().enumerate())
            .filter_map(|(i, ws)| Some((i as u32, ws.sheet_data.take_reads()?)))
            .filter(|(_, reads)| !reads.cells.is_empty() || !reads.areas.is_empty() || reads.extent)
            .collect();
        let state = if memoizable {
            self.read_state(&reads, Some(spills))
        } else {
            None
        };
        let memo = &mut self.evaluation.data_table_memo;
        if memo.context.as_ref() != Some(context) {
            *memo = DataTableMemo {
                context: Some(context.clone()),
                tables: Vec::new(),
            };
        }
        if memo.tables.len() <= index {
            memo.tables.resize_with(index + 1, || None);
        }
        memo.tables[index] = state.map(|state| TableMemo {
            sheet,
            table: table.clone(),
            cost,
            reads,
            state,
            results: Vec::new(),
        });
    }

    /// Keeps the results the table at index `index` has just written in its memo.
    pub(crate) fn seal_data_table_memo(&mut self, index: usize, sheet: u32, table: &DataTable) {
        let results = self.result_contents(sheet, table);
        let tables = &mut self.evaluation.data_table_memo.tables;
        if let Some(Some(entry)) = tables.get_mut(index) {
            entry.results = results;
        }
    }

    /// Forgets the memo of the table at index `index`, which was left as it was.
    pub(crate) fn forget_data_table_memo(&mut self, index: usize) {
        if let Some(entry) = self.evaluation.data_table_memo.tables.get_mut(index) {
            *entry = None;
        }
    }

    /// Forgets the memos of the tables from index `count` on: there are no such tables.
    pub(crate) fn truncate_data_table_memo(&mut self, count: usize) {
        self.evaluation.data_table_memo.tables.truncate(count);
    }

    fn result_contents(&self, sheet: u32, table: &DataTable) -> Vec<Held> {
        let (top, left, bottom, right) = table.range;
        (top..=bottom)
            .flat_map(|row| (left..=right).map(move |column| (row, column)))
            .map(|(row, column)| self.held(CellReferenceIndex { sheet, row, column }, None))
            .map(|held| held.ok().flatten())
            .collect()
    }

    /// The contents of everything `reads` covers, `None` when that is too much or holds what
    /// rules a memo out: with `spills`, also a spill that is not the one in `spills`.
    fn read_state(
        &self,
        reads: &[(u32, SheetReads)],
        spills: Option<&Spills>,
    ) -> Option<ReadState> {
        let mut state = ReadState {
            cells: BTreeMap::new(),
            extents: Vec::new(),
        };
        for (sheet, sheet_reads) in reads {
            let sheet = *sheet;
            if sheet_reads.overflow {
                return None;
            }
            let data = &self.workbook.worksheet(sheet).ok()?.sheet_data;
            if sheet_reads.extent {
                state
                    .extents
                    .push((sheet, data.last_row(), data.last_column()));
            }
            for &(row, column) in &sheet_reads.cells {
                let at = CellReferenceIndex { sheet, row, column };
                state
                    .cells
                    .insert((sheet, row, column), self.held(at, spills).ok()?);
            }
            let mut columns = Vec::new();
            for &(row1, column1, row2, column2) in &sheet_reads.areas {
                let mut from = row1.max(1);
                while let Some(row) = data.next_row_between(from, row2.min(LAST_ROW)).row {
                    data.columns_in_row_between(row, column1, column2, &mut columns);
                    for &column in &columns {
                        let at = CellReferenceIndex { sheet, row, column };
                        state
                            .cells
                            .insert((sheet, row, column), self.held(at, spills).ok()?);
                        if state.cells.len() > MAX_MEMO_CELLS {
                            return None;
                        }
                    }
                    from = row + 1;
                }
            }
            if state.cells.len() > MAX_MEMO_CELLS {
                return None;
            }
        }
        Some(state)
    }

    /// What the cell at `at` holds; `Err` when that rules a memo out, as [`Self::read_state`]
    /// says.
    fn held(&self, at: CellReferenceIndex, spills: Option<&Spills>) -> Result<Held, ()> {
        let ws = self.workbook.worksheet(at.sheet).map_err(|_| ())?;
        let started_from = |(row, column): (i32, i32), r: (i32, i32)| match spills {
            Some(spills) if spills.get(&(at.sheet, row, column)) != Some(&r) => Err(()),
            _ => Ok(()),
        };
        let formula = |f: i32| {
            let text = usize::try_from(f)
                .ok()
                .and_then(|f| ws.shared_formulas.get(f))
                .ok_or(())?;
            let upper = text.to_ascii_uppercase();
            if UNMEMOIZABLE_CALLS.iter().any(|call| upper.contains(call)) {
                return Err(());
            }
            Ok(text.clone())
        };
        let Some(cell) = ws.sheet_data.cell(at.row, at.column) else {
            return Ok(None);
        };

        let content = match cell {
            Cell::EmptyCell { .. } => Content::Blank,
            Cell::BooleanCell { v, .. } => Content::Boolean(*v),
            Cell::NumberCell { v, .. } => Content::Number(v.to_bits()),
            Cell::ErrorCell { ei, .. } => Content::Error(ei.clone()),
            Cell::SharedString { si, .. } => {
                let text = usize::try_from(*si)
                    .ok()
                    .and_then(|si| self.workbook.shared_strings.get(si))
                    .ok_or(())?;
                Content::Text(text.clone())
            }
            Cell::CellFormula { f, .. } => Content::Formula(formula(*f)?),
            Cell::ArrayFormula { f, r, kind, .. } => {
                if *kind == ArrayKind::Dynamic {
                    started_from((at.row, at.column), *r)?;
                }
                Content::Array(formula(*f)?, kind.clone(), *r)
            }
            Cell::SpillCell { a, .. } => {
                match ws.sheet_data.cell(a.0, a.1) {
                    Some(Cell::ArrayFormula {
                        kind: ArrayKind::Cse,
                        ..
                    }) => {}
                    Some(Cell::ArrayFormula {
                        kind: ArrayKind::Dynamic,
                        r,
                        ..
                    }) => started_from(*a, *r)?,
                    _ => return Err(()),
                }
                Content::Spill(*a)
            }
        };
        Ok(Some((content, cell.get_style())))
    }
}

/// Whether no defined name's formula calls one of [`UNMEMOIZABLE_CALLS`].
pub(crate) fn names_memoizable(names: &[DefinedName]) -> bool {
    names.iter().all(|name| {
        let upper = name.formula.to_ascii_uppercase();
        !UNMEMOIZABLE_CALLS.iter().any(|call| upper.contains(call))
    })
}
