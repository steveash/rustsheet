#![allow(clippy::unwrap_used)]

use rustsheet_xlsx::import::load_from_xlsx;

/// Excel caches `#CALC!` and `#SPILL!` as `#VALUE!` plus a `vm` pointing at a rich value
/// (docs/excel-semantics/arrays.md); the importer restores them through the shared decoder
/// (T-0921). ERROR.TYPE.xlsx A8 is errorType 13 and A14 errorType 8.
#[test]
fn cached_rich_errors_are_restored_on_import() {
    let model = load_from_xlsx(
        "tests/calc_tests/INFORMATION/ERROR.TYPE.xlsx",
        "en",
        "UTC",
        "en",
    )
    .unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 8, 1).unwrap(), "#CALC!");
    assert_eq!(model.get_formatted_cell_value(0, 14, 1).unwrap(), "#SPILL!");
}
