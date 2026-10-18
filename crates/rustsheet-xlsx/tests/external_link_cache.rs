#![allow(clippy::unwrap_used)]

use rustsheet_xlsx::import::load_from_xlsx;

/// The fixture's formulas over a cached externalLink range recalculate without an array
/// reaching a scalar cell (a debug assertion in `Model`), and read the cache as Excel does
/// (eval/suites/fixtures/cases/external-link-cached-ranges.xlsx, excel-graph; T-1460).
#[test]
fn cached_external_ranges_recalculate() {
    let mut model = load_from_xlsx(
        "../../eval/suites/fixtures/cases/external-link-cached-ranges.xlsx",
        "en",
        "UTC",
        "en",
    )
    .unwrap();
    model.evaluate();
    let value = |row| model.get_formatted_cell_value(0, row, 2).unwrap();
    // SUM, COUNTA, INDEX, TEXTJOIN keeps the blank, RANK.
    assert_eq!(value(1), "15");
    assert_eq!(value(3), "5");
    assert_eq!(value(16), "7");
    assert_eq!(value(76), "Hello!|10|5|7|TRUE|");
    assert_eq!(value(71), "2");
}
