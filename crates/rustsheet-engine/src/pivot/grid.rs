//! The grid writer (SPEC-0032 "Rendering and the grid", T-4151): a pivot owns the cells of its
//! location and of the page area above it, and a refresh clears what it owned and writes its new
//! cells there as constants, so a formula that reads them needs no new dependency kind. A value
//! cell also gets its data field's number format (T-4169), the rest of its style kept.
//!
//! A plan ([`Model::plan_pivot_refresh`], [`Model::plan_pivot_render`]) reads the model and
//! changes nothing; [`Model::apply_pivot_refresh`] writes it and swaps the refreshed cache and
//! pivots in, returning what they replaced ([`PivotSwap`]) for an undo. The rules:
//! - a layout that would meet a cell the pivot does not own (a stored value or formula outside
//!   its area) writes `#SPILL!` at the anchor and saves the location as that one cell, as Excel
//!   does (pivots.md "A pivot that would meet a foreign cell collapses to #SPILL!",
//!   probes/t4128-pivot-spike S). So does one that would meet another pivot's area, a merged
//!   range, a Table, an array formula's cells or the sheet's edge (UNVERIFIED, probe pending:
//!   T-4664; Microsoft documents the merged cells, Tables and the edge, excel-docs.md PSPILL);
//!   one in the plan's earlier pivots' new areas collapses too. A merged range in the pivot's own
//!   area refuses the refresh instead ([`PivotRefreshError::MergedArea`]);
//! - a refresh rebuilds the cache from the source ([`refresh`]) and lays out again every pivot
//!   over it (UNVERIFIED that Excel refreshes the other pivots of a shared cache, T-4664); a
//!   pivot's field indices follow its fields by name ([`field_map`]);
//! - who refreshes and when is the caller's: the facade refreshes `refreshOnLoad` caches after a
//!   loaded workbook's first recalculation and on `Txn::refresh_pivot_table`, never on a source
//!   edit or a recalculation; a pivot without `refreshOnLoad` keeps its saved cells (pivots.md
//!   "Pivots without refreshOnLoad, and source edits", probes/t4586-pivot-refresh-on-load).
//!
//! One [`PivotBudget`] bounds a plan, or every plan of a load, together: the source cells read,
//! the result and layout work, the cells scanned for ownership and foreign cells, and the cells
//! written or cleared (SPEC-0032 "Untrusted-input limits"). Walks of an area cost the cells
//! stored there, never the area (an untrusted location can be the whole sheet).

use std::collections::HashSet;

use thiserror::Error as ThisError;

use super::layout::{lay_out, render, CellFormat, PivotCell, RenderedCell, MAX_PIVOT_CELLS};
use super::refresh::{field_map, refresh, SourceCells};
use super::result::{compute_on, PivotComputeError, MAX_PIVOT_WORK};
use super::{
    AxisField, CellArea, OpaqueEntry, PivotCache, PivotCacheError, PivotField, PivotFilter,
    PivotGeometry, PivotTable, MAX_RECORD_VALUES,
};
use crate::expressions::parser::parse_range;
use crate::expressions::token::Error;
use crate::model::Model;
use crate::number_format::get_num_fmt;
use crate::types::{sheet_names_match, Cell};

/// Why a pivot could not be refreshed or laid out; the grid and the pivot are then as they were.
#[derive(ThisError, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PivotRefreshError {
    /// The cache could not be read again from its source (no worksheet source, a limit).
    #[error(transparent)]
    Cache(#[from] PivotCacheError),
    /// The pivot could not be computed or laid out (a work or size limit, a malformed pivot).
    #[error(transparent)]
    Compute(#[from] PivotComputeError),
    /// A field the pivot shows is no longer in the source. What Excel does then is UNVERIFIED
    /// (probe pending: T-4664).
    #[error("the pivot field {0:?} is no longer in the source")]
    FieldGone(String),
    /// The refresh would renumber the fields of a pivot that keeps a part of its definition
    /// verbatim ([`PivotTable::opaque`]) holding a field index, which nothing can remap
    /// ([`OpaqueEntry::holds_field_index`]: `formats`, `filters`, ...).
    #[error(
        "a refresh would renumber the fields of a pivot whose unread parts hold field indices"
    )]
    FieldsMoved,
    /// The cache, the pivot or its sheet is not in the workbook.
    #[error("no such pivot, cache or sheet")]
    Missing,
    /// A planned cell is outside the sheet: the plan is not over this workbook.
    #[error("a pivot cell outside the sheet")]
    OutsideSheet,
    /// A merged range lies in the pivot's own area (Excel's "Merge and center cells with labels",
    /// `mergeItem`, makes such ranges) or crosses its edge: the cell writers cannot write through
    /// it, so the refresh is refused and the pivot keeps its cells (UNVERIFIED, T-4664).
    #[error("a merged range lies in the pivot's own area")]
    MergedArea,
    /// The engine refused to write a planned cell (a plan applied to a workbook it was not made
    /// for); the cells may be partly written.
    #[error("the engine refused a pivot cell: {0}")]
    Write(String),
}

/// What one plan, or every plan of a load, may spend together (SPEC-0032 "Untrusted-input
/// limits"): `work` units, as [`compute`] and [`lay_out`] charge them, plus one per source cell
/// read and per cell or merged range scanned; and `cells` written or cleared. A plan past it fails
/// with [`PivotComputeError::Limit`], and its pivots keep their cells; what the failed plan spent
/// stays spent, so a later plan drawing on the same budget has only what is left. The default is
/// [`MAX_PIVOT_WORK`] and [`MAX_PIVOT_CELLS`], one pivot's own limits (DEVIATION, pivots.md "A
/// pivot result is bounded in work and nodes").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PivotBudget {
    /// The work units left.
    pub work: u64,
    /// The cells left to write or clear.
    pub cells: usize,
}

impl Default for PivotBudget {
    fn default() -> Self {
        PivotBudget {
            work: MAX_PIVOT_WORK,
            cells: MAX_PIVOT_CELLS,
        }
    }
}

impl PivotBudget {
    fn charge_work(&mut self, units: usize) -> Result<(), PivotRefreshError> {
        match u64::try_from(units).ok().filter(|&u| u <= self.work) {
            Some(units) => {
                self.work -= units;
                Ok(())
            }
            None => {
                self.work = 0;
                Err(PivotComputeError::Limit.into())
            }
        }
    }

    fn charge_cells(&mut self, cells: usize) -> Result<(), PivotRefreshError> {
        if cells <= self.cells {
            self.cells -= cells;
            Ok(())
        } else {
            self.cells = 0;
            Err(PivotComputeError::Limit.into())
        }
    }
}

/// One pivot as a plan lays it out on its sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct PivotPlacement {
    /// The pivot's index in [`Workbook::pivot_tables`](crate::types::Workbook::pivot_tables).
    pub table: usize,
    /// The pivot's sheet index.
    pub sheet: u32,
    /// The pivot as it will be: field indices over the new cache, and the new location and
    /// geometry.
    pub pivot: PivotTable,
    /// The 0-based (row, column) of each cell the pivot owned that it no longer writes, emptied.
    pub cleared: Vec<(u32, u32)>,
    /// The cells it writes.
    pub cells: Vec<RenderedCell>,
    /// Whether the layout met a foreign cell, so the pivot collapsed to `#SPILL!`.
    pub spilled: bool,
}

/// What a refresh or render writes: [`Model::apply_pivot_refresh`]'s input.
#[derive(Debug, Clone, PartialEq)]
pub struct PivotRefresh {
    /// The cache's index.
    pub cache_index: u32,
    /// The refreshed cache; `None` when the plan only lays the pivot out again.
    pub cache: Option<PivotCache>,
    /// Each pivot laid out.
    pub placements: Vec<PivotPlacement>,
}

impl PivotRefresh {
    /// Every cell the plan writes or empties as 0-based (sheet index, row, column), for a
    /// caller's undo log.
    pub fn touched(&self) -> impl Iterator<Item = (u32, u32, u32)> + '_ {
        self.placements.iter().flat_map(|p| {
            let cleared = p
                .cleared
                .iter()
                .map(move |&(row, column)| (p.sheet, row, column));
            let written = p.cells.iter().map(move |c| (p.sheet, c.row, c.column));
            cleared.chain(written)
        })
    }
}

/// The cache and pivots a refresh replaced, as [`Model::apply_pivot_refresh`] returns them;
/// [`Model::restore_pivot_swap`] puts them back.
#[derive(Debug, Clone, PartialEq)]
pub struct PivotSwap {
    cache_index: u32,
    cache: Option<PivotCache>,
    tables: Vec<(usize, PivotTable)>,
}

/// What the pivots of one plan meet besides cells: the Tables of the workbook, read once, and
/// the areas the plan's pivots already take.
struct Blockers {
    /// Each Table's sheet name and 0-based area.
    tables: Vec<(String, CellArea)>,
    /// Each placed pivot's sheet id and new areas.
    placed: Vec<(u32, [Option<CellArea>; 2])>,
}

impl Blockers {
    fn new(model: &Model, budget: &mut PivotBudget) -> Result<Self, PivotRefreshError> {
        let tables = &model.workbook.tables;
        budget.charge_work(tables.len())?;
        let tables = (tables.values())
            .filter_map(|t| {
                let (c1, r1, c2, r2) = parse_range(&t.reference).ok()?;
                Some((t.sheet_name.clone(), zero_based_area(r1, c1, r2, c2)?))
            })
            .collect();
        Ok(Blockers {
            tables,
            placed: Vec::new(),
        })
    }
}

/// The cells a pivot owns: its location and its page area (pivots.md "Page fields render above
/// the location": `rowPageCount` rows ending one row above the location, three columns per page
/// column but the last one's gap).
fn owned_areas(location: CellArea, geometry: &PivotGeometry) -> [Option<CellArea>; 2] {
    let (rows, columns) = (geometry.row_page_count, geometry.column_page_count);
    let page = (rows > 0 && columns > 0)
        .then(|| {
            let top = location.top.checked_sub(rows.checked_add(1)?)?;
            let right = location
                .left
                .checked_add(columns.checked_mul(3)?.checked_sub(2)?)?;
            Some(CellArea {
                top,
                left: location.left,
                bottom: location.top.checked_sub(2)?,
                right,
            })
        })
        .flatten();
    [Some(location), page]
}

fn contains(area: &CellArea, row: u32, column: u32) -> bool {
    (area.top..=area.bottom).contains(&row) && (area.left..=area.right).contains(&column)
}

/// A stored cell's 0-based (row, column) and the cell.
type Stored<'a> = ((u32, u32), &'a Cell);

/// Whether `cell` belongs to an array formula, whose cells are never a pivot's.
fn is_array(cell: &Cell) -> bool {
    matches!(cell, Cell::ArrayFormula { .. } | Cell::SpillCell { .. })
}

/// The 0-based area of the 1-based rows `top..=bottom` and columns `left..=right`; `None` when it
/// is not within the sheet's numbering.
fn zero_based_area(top: i32, left: i32, bottom: i32, right: i32) -> Option<CellArea> {
    let line = |n: i32| u32::try_from(n.checked_sub(1)?).ok();
    Some(CellArea {
        top: line(top.min(bottom))?,
        left: line(left.min(right))?,
        bottom: line(top.max(bottom))?,
        right: line(left.max(right))?,
    })
}

fn overlaps(a: &CellArea, b: &CellArea) -> bool {
    a.top <= b.bottom && b.top <= a.bottom && a.left <= b.right && b.left <= a.right
}

/// `pivot` over `new`, a refresh of its cache `old`: each field index follows its field by name
/// ([`field_map`]), a field new to the source joins on no axis, and a field gone from it is
/// dropped unless the pivot shows it ([`PivotRefreshError::FieldGone`]). The verbatim parts a
/// field owns follow it; a renumbering under one that holds a field index is refused
/// ([`PivotRefreshError::FieldsMoved`]).
fn remap(
    pivot: &PivotTable,
    old: &PivotCache,
    new: &PivotCache,
) -> Result<PivotTable, PivotRefreshError> {
    let map = field_map(old, new);
    let identity = map.iter().zip(0u32..).all(|(&to, from)| to == Some(from));
    if identity {
        let mut pivot = pivot.clone();
        for field in new.fields.iter().skip(pivot.fields.len()) {
            pivot.fields.push(PivotField::new(None, field.items.len()));
        }
        return Ok(pivot);
    }
    if pivot.opaque.iter().any(OpaqueEntry::holds_field_index) {
        return Err(PivotRefreshError::FieldsMoved);
    }
    let to = |from: u32| {
        map.get(from as usize).copied().flatten().ok_or_else(|| {
            let name = old.fields.get(from as usize).map(|f| f.name.clone());
            PivotRefreshError::FieldGone(name.unwrap_or_default())
        })
    };
    let axis = |fields: &[AxisField]| -> Result<Vec<AxisField>, PivotRefreshError> {
        fields
            .iter()
            .map(|field| match *field {
                AxisField::Field(from) => to(from).map(AxisField::Field),
                AxisField::Values => Ok(AxisField::Values),
            })
            .collect()
    };
    let mut out = pivot.clone();
    out.row_fields = axis(&pivot.row_fields)?;
    out.column_fields = axis(&pivot.column_fields)?;
    for page in &mut out.page_fields {
        page.field = to(page.field)?;
    }
    // A filter follows its field; one on a field gone from the source goes with it (UNVERIFIED,
    // T-4664). A pivot read from a file also keeps its `<filters>` verbatim, whose `fld` is a
    // field index (`OpaqueEntry::holds_field_index`), so a renumbering refuses it above with
    // `FieldsMoved`, and an identity map returns before this; only a pivot whose filters are not
    // kept verbatim (made through the API) gets here with filters.
    out.filters = (pivot.filters.iter())
        .filter_map(|filter| {
            let field = map.get(filter.field as usize).copied().flatten()?;
            Some(PivotFilter {
                field,
                ..filter.clone()
            })
        })
        .collect();
    for data in &mut out.data_fields {
        data.field = to(data.field)?;
        // A base field gone from the source falls back to the first field, the schema's default
        // (UNVERIFIED, T-4664).
        data.base_field = u32::try_from(data.base_field)
            .ok()
            .and_then(|from| to(from).ok())
            .and_then(|to| i32::try_from(to).ok())
            .unwrap_or(0);
    }
    // Which old field each new one refreshes, if any.
    let mut from = vec![None; new.fields.len()];
    for (old_index, to) in map.iter().enumerate() {
        if let Some(slot) = to.and_then(|to| from.get_mut(to as usize)) {
            *slot = Some(old_index);
        }
    }
    out.fields = (new.fields.iter().zip(from))
        .map(|(field, from)| {
            from.and_then(|from| pivot.fields.get(from).cloned())
                .unwrap_or_else(|| PivotField::new(None, field.items.len()))
        })
        .collect();
    // What a field owns follows it; what a dropped field owned goes with it.
    out.opaque = (pivot.opaque.iter())
        .filter_map(|entry| match entry.owner.field() {
            None => Some(entry.clone()),
            Some(from) => map
                .get(from as usize)
                .copied()
                .flatten()
                .map(|to| OpaqueEntry {
                    owner: entry.owner.with_field(to),
                    node: entry.node.clone(),
                }),
        })
        .collect();
    Ok(out)
}

impl Model {
    /// Today as the pivots' relative date filters read it (`compute_on`): the serial day TODAY
    /// gives, from the workbook's clock in its time zone, never the host's directly (CLAUDE.md
    /// directive 5); `None` when the clock's instant has no serial.
    #[must_use]
    pub fn pivot_today(&self) -> Option<f64> {
        crate::tz::excel_serial_for_now(&self.tz, self.now_unix_millis()).map(f64::floor)
    }

    /// The plan that refreshes the cache at `cache_index` from its source and lays out again
    /// every pivot over it, within `budget`. Changes nothing.
    ///
    /// # Errors
    /// [`PivotRefreshError`]: the cache does not refresh, a pivot over it does not lay out, or
    /// the plan passes `budget`.
    pub fn plan_pivot_refresh(
        &self,
        cache_index: u32,
        budget: &mut PivotBudget,
    ) -> Result<PivotRefresh, PivotRefreshError> {
        let caches = &self.workbook.pivot_caches;
        let old = caches
            .get(cache_index as usize)
            .ok_or(PivotRefreshError::Missing)?;
        // The source cells are charged before they are read.
        let source = old.source.as_ref().and_then(|source| self.resolve(source));
        let area = source.ok_or(PivotCacheError::Source)?.area;
        let rows = (area.bottom - area.top) as usize + 1;
        budget.charge_work(rows.saturating_mul((area.right - area.left) as usize + 1))?;
        // A field Excel added to group another is derived again over every record (T-4171).
        budget.charge_work(rows.saturating_mul(old.added_group_count()))?;
        // The record values the other caches hold count against the workbook's limit.
        let others: usize = (0u32..)
            .zip(caches)
            .filter(|&(i, _)| i != cache_index)
            .map(|(_, c)| (c.record_count as usize).saturating_mul(c.fields.len()))
            .fold(0, usize::saturating_add);
        let new = refresh(old, self, MAX_RECORD_VALUES.saturating_sub(others))?;
        let mut blockers = Blockers::new(self, budget)?;
        let mut placements = Vec::new();
        for (table, pivot) in self.workbook.pivot_tables.iter().enumerate() {
            if pivot.cache == cache_index {
                let remapped = remap(pivot, old, &new)?;
                placements.push(self.place(table, remapped, &new, budget, &mut blockers)?);
            }
        }
        Ok(PivotRefresh {
            cache_index,
            cache: Some(new),
            placements,
        })
    }

    /// The plan that lays out again the pivot at index `table` over its cache as it is, without
    /// refreshing it, within `budget`. Changes nothing.
    ///
    /// # Errors
    /// [`PivotRefreshError`]: the pivot does not lay out, or passes `budget`.
    pub fn plan_pivot_render(
        &self,
        table: usize,
        budget: &mut PivotBudget,
    ) -> Result<PivotRefresh, PivotRefreshError> {
        let pivot = self
            .workbook
            .pivot_tables
            .get(table)
            .ok_or(PivotRefreshError::Missing)?;
        let cache = self
            .workbook
            .pivot_caches
            .get(pivot.cache as usize)
            .ok_or(PivotRefreshError::Missing)?;
        let mut blockers = Blockers::new(self, budget)?;
        let placement = self.place(table, pivot.clone(), cache, budget, &mut blockers)?;
        Ok(PivotRefresh {
            cache_index: pivot.cache,
            cache: None,
            placements: vec![placement],
        })
    }

    /// `pivot` (the pivot at index `table`, over `cache`) laid out from its anchor, the top-left
    /// cell of its location: its cells, or `#SPILL!` there when they would meet a foreign cell
    /// or pass the sheet's edge.
    fn place(
        &self,
        table: usize,
        mut pivot: PivotTable,
        cache: &PivotCache,
        budget: &mut PivotBudget,
        blockers: &mut Blockers,
    ) -> Result<PivotPlacement, PivotRefreshError> {
        let sheet = self
            .workbook
            .worksheets
            .iter()
            .position(|w| w.sheet_id == pivot.sheet_id)
            .and_then(|i| u32::try_from(i).ok())
            .ok_or(PivotRefreshError::Missing)?;
        let result = compute_on(&pivot, cache, self.pivot_today(), &mut budget.work)?;
        let layout = match lay_out(&pivot, cache, &result, &mut budget.work) {
            Ok(layout) => Some(layout),
            Err(PivotComputeError::PastSheet) => None,
            Err(error) => return Err(error.into()),
        };
        let owned = owned_areas(pivot.location, &pivot.geometry);
        let mut fits = None;
        if let Some(layout) = layout {
            let cells = render(&pivot, cache, &result, &layout)?;
            budget.charge_cells(cells.len())?;
            let new = owned_areas(layout.location, &layout.geometry);
            let areas = [&owned, &new];
            if !self.meets_foreign(sheet, table, &pivot, areas, budget, blockers)? {
                fits = Some((layout, cells));
            }
        }
        let spilled = fits.is_none();
        // Laid out again, the pivot's saved layout lines no longer describe its cells; the
        // export writes the engine's in their place (T-4176).
        pivot.opaque.retain(|entry| !entry.is_layout_lines());
        let cells = match fits {
            Some((layout, cells)) => {
                pivot.location = layout.location;
                pivot.geometry = layout.geometry;
                cells
            }
            None => {
                // The page counts are zeroed with the rest: evidenced only for a pivot with no
                // page field (t4128 S; UNVERIFIED with page fields, T-4664).
                pivot.location.bottom = pivot.location.top;
                pivot.location.right = pivot.location.left;
                pivot.geometry = PivotGeometry::default();
                budget.charge_cells(1)?;
                vec![RenderedCell {
                    row: pivot.location.top,
                    column: pivot.location.left,
                    value: PivotCell::Error(Error::SPILL),
                    number_format: None,
                }]
            }
        };
        let written: HashSet<(u32, u32)> = cells.iter().map(|c| (c.row, c.column)).collect();
        // The two owned areas are disjoint, so no place is cleared twice. An array formula's
        // cells are never the pivot's, so they are left alone.
        let mut cleared = Vec::new();
        for area in owned.iter().flatten() {
            for (place, cell) in self.stored_in(sheet, area, budget)? {
                if !written.contains(&place) && !is_array(cell) {
                    cleared.push(place);
                }
            }
        }
        budget.charge_cells(cleared.len())?;
        let taken = owned_areas(pivot.location, &pivot.geometry);
        blockers.placed.push((pivot.sheet_id, taken));
        Ok(PivotPlacement {
            table,
            sheet,
            pivot,
            cleared,
            cells,
            spilled,
        })
    }

    /// Whether the areas `new` of the pivot at index `table` meet what it does not own: a stored
    /// value or formula outside `owned`, an array formula's cell anywhere in them, another
    /// pivot's saved area on the sheet or the new area of one placed earlier in the plan, a
    /// merged range or a Table.
    ///
    /// # Errors
    /// [`PivotRefreshError::MergedArea`] for a merged range in or across `owned`, and the
    /// budget's.
    fn meets_foreign(
        &self,
        sheet: u32,
        table: usize,
        pivot: &PivotTable,
        [owned, new]: [&[Option<CellArea>; 2]; 2],
        budget: &mut PivotBudget,
        blockers: &Blockers,
    ) -> Result<bool, PivotRefreshError> {
        let worksheet = self
            .workbook
            .worksheet(sheet)
            .map_err(|_| PivotRefreshError::Missing)?;
        let meets = |area: &CellArea| new.iter().flatten().any(|n| overlaps(n, area));
        let pivots = &self.workbook.pivot_tables;
        budget.charge_work(pivots.len().saturating_add(blockers.placed.len()))?;
        let other_pivot = (pivots.iter().enumerate())
            .filter(|&(i, other)| i != table && other.sheet_id == pivot.sheet_id)
            .flat_map(|(_, other)| owned_areas(other.location, &other.geometry))
            .chain(
                (blockers.placed.iter())
                    .filter(|(id, _)| *id == pivot.sheet_id)
                    .flat_map(|(_, areas)| *areas),
            )
            .flatten()
            .any(|area| meets(&area));
        budget.charge_work(worksheet.merged_cells.len())?;
        let mut merged = false;
        for m in &worksheet.merged_cells {
            let Some(area) = zero_based_area(m.row, m.column, m.last_row(), m.last_column()) else {
                continue;
            };
            if owned.iter().flatten().any(|o| overlaps(o, &area)) {
                // The pivot's own merge (`mergeItem`), or one across its edge: not writable.
                return Err(PivotRefreshError::MergedArea);
            }
            merged |= meets(&area);
        }
        let name = worksheet.get_name();
        let table_met = (blockers.tables.iter())
            .filter(|(sheet_name, _)| sheet_names_match(sheet_name, &name))
            .any(|(_, area)| meets(area));
        if other_pivot || merged || table_met {
            return Ok(true);
        }
        let is_owned = |(row, column): (u32, u32)| {
            (owned.iter().flatten()).any(|area| contains(area, row, column))
        };
        for area in new.iter().flatten() {
            for (place, cell) in self.stored_in(sheet, area, budget)? {
                if is_array(cell) || !is_owned(place) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// The 0-based places and cells of `area` that hold a value or formula (a cell that only
    /// holds a style is empty), walking the cells stored there, never the area, each charged to
    /// `budget`.
    fn stored_in(
        &self,
        sheet: u32,
        area: &CellArea,
        budget: &mut PivotBudget,
    ) -> Result<Vec<Stored<'_>>, PivotRefreshError> {
        let Ok(worksheet) = self.workbook.worksheet(sheet) else {
            return Ok(Vec::new());
        };
        let one_based = |n: u32| i32::try_from(n).ok().and_then(|n| n.checked_add(1));
        let (Some(top), Some(left), Some(bottom), Some(right)) = (
            one_based(area.top),
            one_based(area.left),
            one_based(area.bottom),
            one_based(area.right),
        ) else {
            return Ok(Vec::new());
        };
        let data = &worksheet.sheet_data;
        let places = data.places_in_block((top, left), (right - left + 1, bottom - top + 1));
        budget.charge_work(places.len().saturating_add(1))?;
        Ok(places
            .into_iter()
            .filter_map(|(row, column)| {
                let cell = data.cell(row, column)?;
                let place = (
                    u32::try_from(row - 1).ok()?,
                    u32::try_from(column - 1).ok()?,
                );
                (!matches!(cell, Cell::EmptyCell { .. })).then_some((place, cell))
            })
            .collect())
    }

    /// Writes `refresh`: empties each pivot's cleared cells, writes its cells as constants, and
    /// swaps in the refreshed cache and pivots, returning what they replaced. GETPIVOTDATA's
    /// indices of those pivots are dropped, so the next recalculation reads them anew.
    ///
    /// # Errors
    /// [`PivotRefreshError::OutsideSheet`] or [`PivotRefreshError::Write`] for a plan that is not
    /// over this workbook as it is: a plan's areas hold no merged range or array formula. Cells
    /// may then be partly written and the pivots are as they were: the caller restores the cells
    /// from its undo log (the facade's `Txn` logs [`PivotRefresh::touched`] first).
    pub fn apply_pivot_refresh(
        &mut self,
        refresh: PivotRefresh,
    ) -> Result<PivotSwap, PivotRefreshError> {
        let one_based = |n: u32| {
            i32::try_from(n)
                .ok()
                .and_then(|n| n.checked_add(1))
                .ok_or(PivotRefreshError::OutsideSheet)
        };
        for placement in &refresh.placements {
            let sheet = placement.sheet;
            for &(row, column) in &placement.cleared {
                let (row, column) = (one_based(row)?, one_based(column)?);
                self.prepare_cell_for_user_input(sheet, row, column)
                    .and_then(|()| {
                        self.workbook
                            .worksheet_mut(sheet)?
                            .cell_clear_contents(row, column)
                    })
                    .map_err(PivotRefreshError::Write)?;
            }
            for cell in &placement.cells {
                let (row, column) = (one_based(cell.row)?, one_based(cell.column)?);
                self.prepare_cell_for_user_input(sheet, row, column)
                    .and_then(|()| match &cell.value {
                        PivotCell::Text(text) => {
                            self.update_cell_with_text(sheet, row, column, text)
                        }
                        PivotCell::Number(n) => {
                            self.update_cell_with_number(sheet, row, column, *n)
                        }
                        PivotCell::Error(error) => {
                            self.update_cell_with_error(sheet, row, column, error.clone())
                        }
                    })
                    .map_err(PivotRefreshError::Write)?;
                if let Some(format) = cell.number_format {
                    self.set_pivot_number_format(sheet, row, column, format)
                        .map_err(PivotRefreshError::Write)?;
                }
            }
        }
        let tables = refresh
            .placements
            .into_iter()
            .map(|p| (p.table, p.pivot))
            .collect();
        Ok(self.restore_pivot_swap(PivotSwap {
            cache_index: refresh.cache_index,
            cache: refresh.cache,
            tables,
        }))
    }

    /// Gives the cell `format`, keeping the rest of the cell's style: a `numFmtId` is the one it
    /// names in the workbook's styles (a built-in one, or one of its `numFmts`; an id it does not
    /// hold is General, as `get_num_fmt` reads it), so a value cell shows its data field's format
    /// (pivots.md "Value cells show their data field's number format"); a code is a date item's.
    fn set_pivot_number_format(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        format: CellFormat,
    ) -> Result<(), String> {
        let code = match format {
            CellFormat::Id(num_fmt_id) => {
                let id = i32::try_from(num_fmt_id).unwrap_or(0);
                get_num_fmt(id, &self.workbook.styles.num_fmts)
            }
            CellFormat::Code(code) => code.to_string(),
        };
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        if style.num_fmt == code {
            return Ok(());
        }
        style.num_fmt = code;
        self.set_cell_style(sheet, row, column, &style)
    }

    /// Puts the cache and pivots of `swap` in place, returning the ones they replace; a swap of
    /// a pivot or cache the workbook no longer has is dropped.
    pub fn restore_pivot_swap(&mut self, swap: PivotSwap) -> PivotSwap {
        let workbook = &mut self.workbook;
        let cache = swap.cache.and_then(|cache| {
            let slot = workbook.pivot_caches.get_mut(swap.cache_index as usize)?;
            Some(std::mem::replace(slot, cache))
        });
        let tables = swap
            .tables
            .into_iter()
            .filter_map(|(index, table)| {
                let slot = workbook.pivot_tables.get_mut(index)?;
                Some((index, std::mem::replace(slot, table)))
            })
            .collect::<Vec<_>>();
        for (index, _) in &tables {
            self.evaluation.pivot_indexes.forget(*index);
        }
        PivotSwap {
            cache_index: swap.cache_index,
            cache,
            tables,
        }
    }
}

#[cfg(test)]
mod tests;
