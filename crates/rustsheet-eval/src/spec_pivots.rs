//! A spec sheet's pivot tables in the engine adapter (T-2084, SPEC-0015): the rendered cells are
//! written as constants, then the pivot is registered over its source.

use rustsheet::{
    CellRef, EditError, Input, PivotFunction, PivotTableInput, RangeRef, SheetId, Txn,
};

use crate::case::{split_qualified, CaseSpec, CellInput, PivotTableSpec, SheetSpec};
use crate::cellref::{parse_range, Addr};

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
        let input = pivot_input(spec, ids, sheet, id, pivot)?;
        tx.add_pivot_table(&input)?;
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
    let (first, last) = parse_range(&range)
        .ok_or_else(|| EditError::Engine(format!("pivot {}: bad source {range}", pivot.name)))?;
    let source = RangeRef::new(
        source_id,
        first.row - 1,
        first.col - 1,
        last.row - 1,
        last.col - 1,
    )?;
    // The rendered cells' bounding box is the pivot's `<location>` (it spans its labels).
    let mut cells = Vec::new();
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
    let function = pivot.function.as_deref().unwrap_or("sum");
    Ok(PivotTableInput {
        name: pivot.name.clone(),
        source,
        location: RangeRef::new(id, top - 1, left - 1, bottom - 1, right - 1)?,
        row: pivot.row.clone(),
        column: pivot.column.clone(),
        value: pivot.value.clone(),
        function: match function {
            "sum" => PivotFunction::Sum,
            "count" => PivotFunction::Count,
            "average" => PivotFunction::Average,
            "max" => PivotFunction::Max,
            "min" => PivotFunction::Min,
            other => {
                return Err(EditError::Engine(format!(
                    "pivot {}: function {other:?}",
                    pivot.name
                )))
            }
        },
        // The caption Excel gives a new data field: `Sum of Sales`.
        caption: format!("{} of {}", capitalized(function), pivot.value),
    })
}

fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn parse_addr(a1: &str) -> Result<Addr, EditError> {
    Addr::parse(a1).ok_or_else(|| EditError::Engine(format!("bad address {a1}")))
}
