#![allow(clippy::unwrap_used)]

//! A formula stored without the dynamic-array flag that Excel still spills (a spill reference, a
//! dynamic-array function) loads as a dynamic anchor, as typing it would make it (T-3467,
//! probes/legacy-formula-spill-ref).

use crate::types::{ArrayKind, Cell, FormulaValue};
use crate::Model;

/// A1 `=SEQUENCE(3)` and, in D1 and D5, `formula` stored as a plain formula cell, as a decoded
/// workbook or an xlsx `<f>` without `cm` can hold it.
fn loaded_with_plain(formula: &str) -> Model {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model._set("A1", "=SEQUENCE(3)");
    model._set("D1", formula);
    model._set("D5", formula);
    let mut workbook = model.workbook;
    for row in [1, 5] {
        let f = workbook.worksheets[0]
            .cell(row, 4)
            .and_then(Cell::get_formula)
            .unwrap();
        workbook.worksheets[0].sheet_data.set_cell(
            row,
            4,
            Cell::CellFormula {
                f,
                s: 0,
                v: FormulaValue::Unevaluated,
            },
        );
    }
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    model
}

/// The fuzz crash of T-3467: a plain cell holding `SIN(A1#)` reached evaluation as a scalar cell
/// with an array result (a `debug_assert!`). It spills on the spill's rows and off them, as Excel
/// does.
#[test]
fn a_plain_formula_cell_with_a_spill_reference_spills() {
    let model = loaded_with_plain("=SIN(A1#)");
    for row in [1, 5] {
        assert!(matches!(
            model.workbook.worksheets[0].cell(row, 4),
            Some(Cell::ArrayFormula {
                kind: ArrayKind::Dynamic,
                ..
            })
        ));
    }
    assert_eq!(model._get_text("D1"), "0.841470985");
    assert_eq!(model._get_text("D3"), "0.141120008");
    assert_eq!(model._get_text("D7"), "0.141120008");
}

/// A dynamic-array function a legacy formula leaves without `@` spills too.
#[test]
fn a_plain_formula_cell_with_a_dynamic_array_function_spills() {
    let model = loaded_with_plain("=SEQUENCE(2)*10");
    assert_eq!(model._get_text("D2"), "20");
    assert_eq!(model._get_text("D6"), "20");
}

/// A plain cell whose formula is a scalar, or that a legacy reading intersects, stays as it is.
#[test]
fn other_plain_formula_cells_stay_plain() {
    let model = loaded_with_plain("=SUM(A1:A3)");
    assert!(matches!(
        model.workbook.worksheets[0].cell(1, 4),
        Some(Cell::CellFormula { .. })
    ));
    assert_eq!(model._get_text("D1"), "6");
}
