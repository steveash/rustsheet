#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::test::util::new_empty_model;

/// Fuzz finding (nightly `formula_eval`, run 38344232074, T-1467): FIXED with about 1.2e8
/// decimals built a string of over 100 MB. Decimals of 128 or more are `#VALUE!`
/// (docs/excel-semantics/text.md, T-1281).
#[test]
fn fixed_and_dollar_with_huge_decimals_are_value_errors() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet2", "Data").expect("rename");
    model._set("Data!E9", "1.5");
    model._set("Data!D3", "a");
    model._set("Data!C2", "b");
    model._set(
        "A1",
        "=ENCODEURL(CONCATENATE(FIXED(Data!E9,123456789.123,TRUE),(Data!D3&Data!$C2)))",
    );
    model._set("A2", "=DOLLAR(0,123456789)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
}
