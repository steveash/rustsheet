//! Nightly fuzz crash (T-0278, decision 0035): loading the committed seed
//! `fixtures/ifs-legacy-range-criteria.xlsx` with `recalc: true` hit a debug assertion, because
//! a legacy `COUNTIFS(A1:A6,D1:D2)` gave a 1x2 array in a scalar cell. Every *IF(S) function now
//! intersects a multi-cell range criterion in a legacy formula, as Excel does.

#![allow(clippy::unwrap_used)]

use rustsheet::{LoadOptions, Value, Workbook};

#[test]
fn legacy_ifs_range_criteria_load_and_intersect() {
    let bytes =
        include_bytes!("../../../eval/suites/fixtures/cases/ifs-legacy-range-criteria.xlsx");
    let workbook = Workbook::from_xlsx_bytes(bytes, &LoadOptions::default()).unwrap();
    // Excel (oracle excel-graph): row 1 intersects the criterion D1, row 2 D2.
    let expected = [
        ("F1", 3.0),
        ("G1", 3.0),
        ("H1", 7.0),
        ("I1", 7.0),
        ("J1", 7.0),
        ("K1", 7.0 / 3.0),
        ("L1", 7.0 / 3.0),
        ("M1", 3.0),
        ("N1", 1.0),
        ("G2", 1.0),
        ("J2", 1.0),
        ("N2", 1.0),
    ];
    for (cell, value) in expected {
        assert_eq!(
            workbook.value(cell).unwrap(),
            Value::Number(value),
            "{cell}"
        );
    }
}
