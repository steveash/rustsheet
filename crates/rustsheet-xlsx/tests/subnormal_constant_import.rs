#![allow(clippy::unwrap_used, reason = "tests")]

use rustsheet_engine::cell::CellValue;
use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;

/// T-2228: a subnormal number constant reads as 0 when a workbook loads, as Excel does
/// (docs/excel-semantics/numbers.md "A subnormal cell constant reads as 0",
/// probes/subnormal-cell-constants): `=A1=0` is TRUE and LOG10 is `#NUM!`. The smallest normal
/// double is kept, and so is a subnormal formula result, which the reload recalculates.
#[test]
fn subnormal_constants_load_as_zero() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    for (row, value) in [(1, 5e-324), (2, 1e-310), (3, f64::MIN_POSITIVE)] {
        model.update_cell_with_number(0, row, 1, value).unwrap();
    }
    model
        .set_user_input(0, 4, 1, "=EXP(-745)".to_string())
        .unwrap();
    model.evaluate();
    let path = "temp_subnormal_constant_import.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let mut reloaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    for row in 1..=3 {
        reloaded
            .set_user_input(0, row, 2, format!("=A{row}=0"))
            .unwrap();
        reloaded
            .set_user_input(0, row, 3, format!("=LOG10(A{row})"))
            .unwrap();
    }
    reloaded.evaluate();
    let value = |row, column| reloaded.get_cell_value_by_index(0, row, column).unwrap();
    let text = |row, column| reloaded.get_formatted_cell_value(0, row, column).unwrap();
    assert_eq!(value(1, 1), CellValue::Number(0.0));
    assert_eq!(value(2, 1), CellValue::Number(0.0));
    assert_eq!(value(3, 1), CellValue::Number(f64::MIN_POSITIVE));
    assert_eq!(value(4, 1), CellValue::Number(5e-324));
    assert_eq!(
        [text(1, 2), text(2, 2), text(3, 2)],
        ["TRUE", "TRUE", "FALSE"]
    );
    assert_eq!(text(1, 3), "#NUM!");
    assert_eq!(text(2, 3), "#NUM!");
}
