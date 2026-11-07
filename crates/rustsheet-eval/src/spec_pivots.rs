//! A spec sheet's pivot tables in the engine adapter (T-2084, SPEC-0015): the rendered cells are
//! written as constants, then the pivot is registered over its source. A pivot with a
//! `definition` (case builder v2) is loaded from the parts the builder writes for it instead, and
//! rendered by the engine as Excel renders it on open (T-4152).

use rustsheet::{
    CellRef, EditError, Input, PivotDateGroupBy, PivotFunction, PivotParts, PivotRangeGroup,
    PivotTableInput, RangeRef, SheetId, Txn, Workbook,
};
use rustsheet_xlsx::import::MAX_PIVOTS;

use crate::case::{
    split_qualified, CaseSpec, CellInput, PivotGroupItem, PivotGroupSpec, PivotTableSpec, SheetSpec,
};
use crate::cellref::Addr;
use crate::engine::range_ref;

/// Writes the rendered cells of every pivot of `sheet` (sheet id `id`) as constants. They go in
/// with the sheet's other values, before any formula reads them.
///
/// # Errors
/// A bad cell address, or the engine refusing a cell.
pub fn write_rendered(
    tx: &mut Txn<'_>,
    sheet: &SheetSpec,
    id: SheetId,
    input: impl Fn(&CellInput) -> Option<Input>,
) -> Result<(), EditError> {
    // A grouped pivot has none: the engine lays it out (T-4235).
    for pivot in &sheet.pivot_tables {
        for (a1, cell) in &pivot.rendered {
            let addr = parse_addr(a1)?;
            if let Some(value) = input(cell) {
                tx.set(CellRef::new(id, addr.row - 1, addr.col - 1)?, value)?;
            }
        }
    }
    Ok(())
}

/// Registers every pivot of `sheet` over its source range. The cache is built from the source
/// cells written so far, the snapshot `refreshOnLoad` gives.
///
/// # Errors
/// A malformed pivot spec, or the engine refusing the pivot.
pub fn add_pivots(
    tx: &mut Txn<'_>,
    spec: &CaseSpec,
    ids: &[SheetId],
    sheet: &SheetSpec,
    id: SheetId,
) -> Result<(), EditError> {
    for pivot in &sheet.pivot_tables {
        // A defined pivot is loaded once the cells are in (`load_defined`).
        if pivot.definition.is_some() {
            continue;
        }
        tx.register_pivot_table(&pivot_input(spec, ids, sheet, id, pivot)?)?;
    }
    Ok(())
}

/// Loads every pivot of `spec` with a `definition` into `workbook`, whose cells are all in, from
/// the parts the case builder writes for it (`pivot_definition`), as the xlsx importer loads the
/// built file, then recalculates when one loaded: its `refreshOnLoad` cache is due, so the
/// recalculation refreshes and renders it after calculating, the order Excel opens the file in.
/// A pivot the importer drops is dropped here too, its cells left empty, as a load drops it: a
/// calculated field (t4128 K), or the builder's numeric group
/// labels past the rebuilt groups (t4128 G; pivots.md "Numeric and date group labels are
/// rewritten by Excel").
///
/// # Errors
/// A definition the builder would refuse, more pivots than a load keeps, or the engine refusing
/// a sheet.
pub fn load_defined(workbook: &mut Workbook, spec: &CaseSpec) -> Result<(), String> {
    let defined = (spec.sheets.iter())
        .flat_map(|sheet| sheet.pivot_tables.iter().map(move |pivot| (sheet, pivot)))
        .filter_map(|(sheet, pivot)| Some((sheet, pivot, pivot.definition.as_ref()?)));
    let mut parts = Vec::new();
    for (sheet, pivot, definition) in defined {
        if parts.len() == MAX_PIVOTS {
            return Err(format!("more than {MAX_PIVOTS} defined pivots"));
        }
        let cache_id = u32::try_from(parts.len() + 1).map_err(|e| e.to_string())?;
        let built = crate::pivot_definition::parts(spec, sheet, pivot, definition, cache_id)?;
        let id = workbook.sheet_id(&sheet.name).map_err(|e| e.to_string())?;
        parts.push(PivotParts {
            cache_definition: built.cache_definition,
            cache_records: built.cache_records,
            cache_id,
            tables: vec![(id, built.table)],
        });
    }
    if parts.is_empty() {
        return Ok(());
    }
    let skipped = workbook
        .load_pivot_parts(&parts)
        .map_err(|e| e.to_string())?;
    if skipped.iter().any(Option::is_none) {
        workbook.recalculate();
    }
    Ok(())
}

fn pivot_input(
    spec: &CaseSpec,
    ids: &[SheetId],
    sheet: &SheetSpec,
    id: SheetId,
    pivot: &PivotTableSpec,
) -> Result<PivotTableInput, EditError> {
    // `row` and `value` default to empty only so a pivot with a `definition` parses (T-4122); a
    // pivot without one needs both.
    if pivot.row.is_empty() || pivot.value.is_empty() {
        return Err(EditError::Engine(format!(
            "pivot {} needs `row` and `value`, or a `definition`",
            pivot.name
        )));
    }
    let (source_sheet, range) = split_qualified(&pivot.source)
        .unwrap_or_else(|| (sheet.name.clone(), pivot.source.clone()));
    let source_id = spec
        .sheets
        .iter()
        .position(|s| s.name.eq_ignore_ascii_case(&source_sheet))
        .and_then(|i| ids.get(i).copied())
        .ok_or_else(|| {
            EditError::Engine(format!("pivot {}: no sheet {source_sheet}", pivot.name))
        })?;
    let source = range_ref(source_id, &range)
        .map_err(|_| EditError::Engine(format!("pivot {}: bad source {range}", pivot.name)))?;
    // The rendered cells' bounding box is the pivot's `<location>` (it spans its labels). A
    // grouped pivot has none: the engine lays it out from its top-left cell, as Excel does on
    // load (T-4235).
    let mut cells = Vec::new();
    if pivot.group.is_some() {
        cells.push(parse_addr(&pivot.location.replace('$', ""))?);
    }
    for a1 in pivot.rendered.keys() {
        cells.push(parse_addr(a1)?);
    }
    let (Some(top), Some(bottom), Some(left), Some(right)) = (
        cells.iter().map(|a| a.row).min(),
        cells.iter().map(|a| a.row).max(),
        cells.iter().map(|a| a.col).min(),
        cells.iter().map(|a| a.col).max(),
    ) else {
        return Err(EditError::Engine(format!(
            "pivot {} has no rendered cells (rustsheet-spec bake-pivots)",
            pivot.name
        )));
    };
    // The names are those of tools/src/rustsheet_tools/pivot.py `_FUNCTIONS`, the source of truth.
    let name = pivot.function.as_deref().unwrap_or("sum");
    let function = PivotFunction::from_name(name)
        .ok_or_else(|| EditError::Engine(format!("pivot {}: function {name:?}", pivot.name)))?;
    Ok(PivotTableInput {
        name: pivot.name.clone(),
        source,
        location: RangeRef::new(id, top - 1, left - 1, bottom - 1, right - 1)?,
        row: pivot.row.clone(),
        column: pivot.column.clone(),
        value: pivot.value.clone(),
        function,
        // The caption Excel gives a new data field: `Sum of Sales`.
        caption: format!("{} of {}", function.caption_name(), pivot.value),
        group: pivot.group.as_ref().map(range_group).transpose()?,
        unused_items: pivot.unused_items.clone(),
        show_all: pivot.show_all,
        column_unused_items: pivot.column_unused_items.clone(),
        column_show_all: pivot.column_show_all,
    })
}

/// A spec pivot's grouping as the engine takes it: `by` names a date group (pivot.py
/// `_DATE_GROUPS`), and its absence a numeric range.
pub(crate) fn range_group(group: &PivotGroupSpec) -> Result<PivotRangeGroup, EditError> {
    let by = match group.by.as_deref() {
        None => None,
        Some(by) => Some(
            PivotDateGroupBy::parse(by)
                .ok_or_else(|| EditError::Engine(format!("pivot group by {by:?}")))?,
        ),
    };
    // A date group's item count is its layout's (`DateGroups::new`), so a number item there
    // cannot simply be left out; no probe has one.
    if by.is_some()
        && group
            .items
            .iter()
            .any(|i| matches!(i, PivotGroupItem::Number(_)))
    {
        return Err(EditError::Engine("a date group's items are texts".into()));
    }
    Ok(PivotRangeGroup {
        by,
        start: group.start,
        end: group.end,
        interval: group.interval,
        // A number item matches none of Excel's labels and takes no place among them, so it is
        // left out (probes/t4316-pivot-group-number-item; `NumberGroups::new`).
        items: group
            .items
            .iter()
            .filter_map(|item| match item {
                PivotGroupItem::Text(text) => Some(text.clone()),
                PivotGroupItem::Number(_) => None,
            })
            .collect(),
    })
}

fn parse_addr(a1: &str) -> Result<Addr, EditError> {
    Addr::parse(a1).ok_or_else(|| EditError::Engine(format!("bad address {a1}")))
}
