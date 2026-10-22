#![allow(clippy::unwrap_used, reason = "tests")]

//! SPEC-0015 "xlsx import" (T-2082): a workbook's pivot caches and pivot tables load into the
//! model, and GETPIVOTDATA answers from them.
//!
//! `pivot_tables.xlsx` is probes/getpivotdata-real-pivot built by `rustsheet-spec2xlsx` (the case
//! builder's pivots, `tools/src/rustsheet_tools/pivot.py`), the workbook excel-graph computed.

use rustsheet_engine::pivot::{Aggregate, Axis, AxisField, CacheItem, CacheSource, CellArea};
use rustsheet_xlsx::import::load_from_xlsx;

#[test]
fn a_workbook_with_pivots_imports_its_caches_and_definitions() {
    let mut model = load_from_xlsx("tests/pivot_tables.xlsx", "en", "UTC", "en").unwrap();
    let workbook = &model.workbook;
    assert_eq!(workbook.pivot_caches.len(), 2);
    let cache = &workbook.pivot_caches[0];
    assert_eq!(
        cache.source,
        Some(CacheSource {
            sheet_name: "Data".to_string(),
            range: "A1:C7".to_string()
        })
    );
    assert_eq!(cache.record_count, 6);
    let region = &cache.fields[0];
    assert_eq!(region.name, "Region");
    assert_eq!(
        region.items,
        [
            CacheItem::Text("East".to_string()),
            CacheItem::Text("West".to_string())
        ]
    );
    assert_eq!(region.records, [0, 1, 0, 1, 0, 0]);

    assert_eq!(workbook.pivot_tables.len(), 2);
    let sum = &workbook.pivot_tables[0];
    assert_eq!(sum.name, "PivotSum");
    assert_eq!(sum.sheet_id, workbook.worksheets[1].sheet_id);
    assert_eq!(sum.cache, 0);
    assert_eq!(
        sum.location,
        CellArea {
            top: 2,
            left: 0,
            bottom: 6,
            right: 4
        }
    );
    assert_eq!(sum.fields[0].axis, Some(Axis::Row));
    assert_eq!(sum.fields[1].axis, Some(Axis::Column));
    assert_eq!(sum.row_fields, [AxisField::Field(0)]);
    assert_eq!(sum.column_fields, [AxisField::Field(1)]);
    assert_eq!(sum.data_fields[0].caption, "Sum of Sales");
    assert_eq!(sum.data_fields[0].function, Aggregate::Sum);
    assert_eq!(
        workbook.pivot_tables[1].data_fields[0].function,
        Aggregate::Average
    );

    // The expected values are excel-graph's (eval/suites/probes/cases/getpivotdata-real-pivot).
    model.evaluate();
    for (row, column, expected) in [
        (1, 8, "49"),    // H1: the grand total
        (3, 8, "22"),    // H3: East
        (5, 8, "14"),    // H5: East, A
        (7, 8, "#REF!"), // H7: no North
        (13, 8, "0"),    // H13: West, C has no records
        (1, 10, "5.5"),  // J1: the Average pivot, East
    ] {
        assert_eq!(
            model.get_formatted_cell_value(1, row, column).unwrap(),
            expected,
            "row {row}, column {column}"
        );
    }
}
