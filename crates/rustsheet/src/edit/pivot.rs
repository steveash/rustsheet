//! [`Txn::add_pivot_table`]: test support that registers a pivot over a source range (SPEC-0015).

use rustsheet_engine::cell::CellValue;
use rustsheet_engine::pivot::{
    Aggregate, Axis, AxisField, CacheBuilder, CacheItem, CacheSource, CellArea, DataField,
    PivotCacheError, PivotField, PivotTable, RecordValue, MAX_RECORD_VALUES,
};
use rustsheet_engine::Model;

use super::{Txn, Undo};
use crate::error::{EditError, RefError};
use crate::refs::RangeRef;
use crate::workbook::{engine_rc, sheet_index};

/// A data field's summary function.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum PivotFunction {
    /// `sum`.
    Sum,
    /// `count`.
    Count,
    /// `average`.
    Average,
    /// `max`.
    Max,
    /// `min`.
    Min,
}

impl PivotFunction {
    /// The function named `name` in a spec (`sum`, `count`, `average`, `max`, `min`).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "sum" => Self::Sum,
            "count" => Self::Count,
            "average" => Self::Average,
            "max" => Self::Max,
            "min" => Self::Min,
            _ => return None,
        })
    }

    /// The word a new data field's caption starts with (`Sum of Sales`).
    #[must_use]
    pub fn caption_name(self) -> &'static str {
        match self {
            Self::Sum => "Sum",
            Self::Count => "Count",
            Self::Average => "Average",
            Self::Max => "Max",
            Self::Min => "Min",
        }
    }
}

impl From<PivotFunction> for Aggregate {
    fn from(function: PivotFunction) -> Self {
        match function {
            PivotFunction::Sum => Aggregate::Sum,
            PivotFunction::Count => Aggregate::Count,
            PivotFunction::Average => Aggregate::Average,
            PivotFunction::Max => Aggregate::Max,
            PivotFunction::Min => Aggregate::Min,
        }
    }
}

/// A pivot table to register: one row field, an optional column field and one data field over a
/// source range. Test support (`Txn::add_pivot_table`), not public API.
#[derive(Clone, Debug)]
pub struct PivotTableInput {
    /// The pivot's name.
    pub name: String,
    /// The source range, field names (text) in its first row, constants below.
    pub source: RangeRef,
    /// The cells the rendered pivot covers; its sheet is the pivot's sheet.
    pub location: RangeRef,
    /// The row field's name.
    pub row: String,
    /// The column field's name, if any.
    pub column: Option<String>,
    /// The data field's source field name.
    pub value: String,
    /// How the data field summarizes.
    pub function: PivotFunction,
    /// The data field's caption (`Sum of Sales`).
    pub caption: String,
}

/// Why a pivot table could not be registered.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PivotError {
    /// A header cell of the source is not text.
    #[error("pivot source header cell {0} is not text")]
    Header(String),
    /// A field name is not in the source's header row.
    #[error("pivot field {0:?} is not in the source header row")]
    UnknownField(String),
    /// The row, column and data fields are not three distinct (or two distinct) fields.
    #[error("a pivot's row, column and data fields must differ")]
    RepeatedField,
    /// The source has no data row.
    #[error("the pivot source has no data row")]
    NoRecords,
    /// The cache exceeds a limit.
    #[error(transparent)]
    Cache(#[from] PivotCacheError),
}

impl Txn<'_> {
    /// Registers a pivot table over `input.source`, building its cache from the source cells as
    /// they hold now (the snapshot `refreshOnLoad` gives); the rendered cells are the caller's to
    /// write. Test support for the eval adapter, not public API, and not mirrored in the Python
    /// binding (SPEC-0015). A change to the workbook's pivots makes the transaction evaluate, so
    /// every GETPIVOTDATA cell is recalculated.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet or a range outside the sheet,
    /// [`EditError::Pivot`] for a malformed source or field, or a cache over a limit.
    #[doc(hidden)]
    pub fn add_pivot_table(&mut self, input: &PivotTableInput) -> Result<(), EditError> {
        let (cache, table) = self.build_pivot(input)?;
        let (caches, tables) = (
            self.model.workbook.pivot_caches.len(),
            self.model.workbook.pivot_tables.len(),
        );
        self.model.workbook.pivot_caches.push(cache);
        self.model.workbook.pivot_tables.push(table);
        self.sheet_op = true;
        self.undo.push(Undo::Pivot { caches, tables });
        Ok(())
    }

    fn build_pivot(
        &self,
        input: &PivotTableInput,
    ) -> Result<(rustsheet_engine::pivot::PivotCache, PivotTable), EditError> {
        let model: &Model = self.model;
        let source_sheet = sheet_index(model, input.source.sheet)?;
        let sheet_name = model
            .workbook
            .worksheet(source_sheet)
            .map_err(EditError::Engine)?
            .get_name();
        let pivot_sheet = sheet_index(model, input.location.sheet)?;
        let sheet_id = model
            .workbook
            .worksheet(pivot_sheet)
            .map_err(EditError::Engine)?
            .get_sheet_id();
        let value_at = |row: u32, col: u32| -> Result<CellValue, EditError> {
            let (r, c) = engine_rc(row, col).ok_or(RefError::OutOfBounds)?;
            model
                .get_cell_value_by_index(source_sheet, r, c)
                .map_err(EditError::Engine)
        };
        let used: usize = model
            .workbook
            .pivot_caches
            .iter()
            .map(|c| c.record_count as usize * c.fields.len())
            .sum();
        let mut builder = CacheBuilder::new(
            Some(CacheSource {
                sheet_name,
                range: input.source.to_string(),
            }),
            MAX_RECORD_VALUES.saturating_sub(used),
        );
        let mut names = Vec::new();
        for col in input.source.left..=input.source.right {
            let CellValue::String(name) = value_at(input.source.top, col)? else {
                let at = crate::refs::CellRef::new(input.source.sheet, input.source.top, col)?;
                return Err(EditError::Pivot(PivotError::Header(at.to_string())));
            };
            builder.add_field(name.clone()).map_err(PivotError::from)?;
            names.push(name);
        }
        if input.source.bottom == input.source.top {
            return Err(PivotError::NoRecords.into());
        }
        for row in input.source.top + 1..=input.source.bottom {
            let mut record = Vec::with_capacity(names.len());
            for col in input.source.left..=input.source.right {
                record.push(RecordValue::Inline(match value_at(row, col)? {
                    CellValue::None => CacheItem::Missing,
                    CellValue::String(s) => CacheItem::Text(s),
                    CellValue::Number(n) => CacheItem::Number(n),
                    CellValue::Boolean(b) => CacheItem::Boolean(b),
                }));
            }
            builder.push_record(record).map_err(PivotError::from)?;
        }
        let cache = builder.finish();
        let field = |name: &str| {
            names
                .iter()
                .position(|n| n == name)
                .and_then(|i| u32::try_from(i).ok())
                .ok_or_else(|| PivotError::UnknownField(name.to_string()))
        };
        let row = field(&input.row)?;
        let column = input.column.as_deref().map(field).transpose()?;
        let value = field(&input.value)?;
        if Some(row) == column || row == value || column == Some(value) {
            return Err(PivotError::RepeatedField.into());
        }
        let fields = cache
            .fields
            .iter()
            .zip(0u32..)
            .map(|(f, i)| {
                let axis = if i == row {
                    Some(Axis::Row)
                } else if Some(i) == column {
                    Some(Axis::Column)
                } else {
                    None
                };
                PivotField::new(axis, f.items.len())
            })
            .collect();
        let table = PivotTable {
            name: input.name.clone(),
            sheet_id,
            location: CellArea {
                top: input.location.top,
                left: input.location.left,
                bottom: input.location.bottom,
                right: input.location.right,
            },
            cache: u32::try_from(model.workbook.pivot_caches.len())
                .map_err(|_| RefError::OutOfBounds)?,
            fields,
            row_fields: vec![AxisField::Field(row)],
            column_fields: column.map(AxisField::Field).into_iter().collect(),
            data_fields: vec![DataField {
                caption: input.caption.clone(),
                field: value,
                function: input.function.into(),
            }],
        };
        Ok((cache, table))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;
    use crate::{Input, Workbook, WorkbookOptions};

    /// Region/Sales over A1:B4 and a pivot rendered at D1:E4.
    fn book() -> (Workbook, PivotTableInput) {
        let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
        wb.edit(|tx| {
            for (a1, input) in [
                ("A1", Input::from("Region")),
                ("B1", Input::from("Sales")),
                ("A2", Input::from("East")),
                ("B2", Input::from(10.0)),
                ("A3", Input::from("West")),
                ("B3", Input::from(20.0)),
                ("A4", Input::from("East")),
                ("B4", Input::from(5.0)),
            ] {
                tx.set(a1, input)?;
            }
            Ok(())
        })
        .unwrap();
        let input = PivotTableInput {
            name: "Pv".into(),
            source: wb.range_ref("A1:B4").unwrap(),
            location: wb.range_ref("D1:E4").unwrap(),
            row: "Region".into(),
            column: None,
            value: "Sales".into(),
            function: PivotFunction::Sum,
            caption: "Sum of Sales".into(),
        };
        (wb, input)
    }

    #[test]
    fn the_cache_and_definition_are_built_from_the_source_cells() {
        let (mut wb, input) = book();
        wb.edit(|tx| tx.add_pivot_table(&input)).unwrap();
        let [cache] = &wb.model.workbook.pivot_caches[..] else {
            panic!("one cache");
        };
        assert_eq!(cache.record_count, 3);
        assert_eq!(cache.fields[0].name, "Region");
        assert_eq!(cache.fields[0].items.len(), 2);
        assert_eq!(cache.fields[0].records, vec![0, 1, 0]);
        let source = cache.source.as_ref().unwrap();
        assert_eq!(
            (source.sheet_name.as_str(), source.range.as_str()),
            ("Sheet1", "A1:B4")
        );
        let [pivot] = &wb.model.workbook.pivot_tables[..] else {
            panic!("one pivot");
        };
        assert_eq!(
            pivot.location,
            CellArea {
                top: 0,
                left: 3,
                bottom: 3,
                right: 4
            }
        );
        assert_eq!(pivot.row_fields, vec![AxisField::Field(0)]);
        assert_eq!(pivot.data_fields[0].field, 1);
        assert_eq!(pivot.fields[0].axis, Some(Axis::Row));
    }

    #[test]
    fn bad_pivots_are_typed_errors_and_leave_no_pivot() {
        let (mut wb, input) = book();
        for (change, expected) in [
            (
                PivotTableInput {
                    row: "Nope".into(),
                    ..input.clone()
                },
                PivotError::UnknownField("Nope".into()),
            ),
            (
                PivotTableInput {
                    value: "Region".into(),
                    ..input.clone()
                },
                PivotError::RepeatedField,
            ),
            (
                PivotTableInput {
                    source: wb.range_ref("A1:B1").unwrap(),
                    ..input.clone()
                },
                PivotError::NoRecords,
            ),
            (
                PivotTableInput {
                    source: wb.range_ref("B2:B4").unwrap(),
                    ..input.clone()
                },
                PivotError::Header("B2".into()),
            ),
        ] {
            let err = wb.edit(|tx| tx.add_pivot_table(&change)).unwrap_err();
            assert!(
                matches!(&err, EditError::Pivot(e) if *e == expected),
                "{err:?}"
            );
        }
        assert!(wb.model.workbook.pivot_tables.is_empty());
        assert!(wb.model.workbook.pivot_caches.is_empty());
    }

    #[test]
    fn a_failed_edit_removes_the_pivot() {
        let (mut wb, input) = book();
        let failed: Result<((), _), _> = wb.edit(|tx| {
            tx.add_pivot_table(&input)?;
            tx.add_pivot_table(&PivotTableInput {
                row: "Nope".into(),
                ..input.clone()
            })
        });
        assert!(failed.is_err());
        assert!(wb.model.workbook.pivot_tables.is_empty());
        assert!(wb.model.workbook.pivot_caches.is_empty());
    }
}
