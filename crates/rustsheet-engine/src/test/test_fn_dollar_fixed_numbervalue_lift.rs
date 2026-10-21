use crate::test::util::new_empty_model;

/// DOLLAR, FIXED and NUMBERVALUE lift every argument over an array and a range, and mixed shapes
/// broadcast with `#N/A` fill (probes/t2198-m5-lift-argument-kinds rows 20-35;
/// docs/excel-semantics/arrays.md, T-2160).
#[test]
fn dollar_fixed_numbervalue_lift_over_arrays_and_ranges() {
    let mut model = new_empty_model();
    model._set("D1", "1.234");
    model._set("D2", "2.345");
    model._set("D3", "3.456");
    let cases = [
        ("=INDEX(DOLLAR({1.234;2.345;3.456},2),2)", "$2.35"),
        ("=INDEX(DOLLAR(1.234,{0;1;2}),2)", "$1.2"),
        ("=ROWS(DOLLAR(D1:D3,2))", "3"),
        ("=INDEX(FIXED({1234.5;2345.6;3456.7},2,TRUE),2)", "2345.60"),
        ("=INDEX(FIXED(1234.5,{0;1;2},TRUE),2)", "1234.5"),
        ("=INDEX(FIXED(1234.5,2,{TRUE;FALSE;TRUE}),2)", "1,234.50"),
        ("=ROWS(FIXED(D1:D3))", "3"),
        (
            "=INDEX(NUMBERVALUE({\"1\";\"2\";\"3\"},\".\",\",\"),2)",
            "2",
        ),
        (
            "=INDEX(NUMBERVALUE(\"1.5\",{\".\";\".\";\",\"},\",\"),2)",
            "1.5",
        ),
        (
            "=INDEX(NUMBERVALUE(\"1.5\",\".\",{\",\";\",\";\".\"}),2)",
            "1.5",
        ),
        ("=ROWS(NUMBERVALUE(D1:D3))", "3"),
        ("=INDEX(DOLLAR({1.234;2.345},{0,1,2}),2,3)", "$2.35"),
        ("=INDEX(FIXED({1234.5;2345.6},{0;1;2},TRUE),3)", "#N/A"),
        (
            "=INDEX(NUMBERVALUE({\"1\";\"2\"},{\".\",\",\",\".\"},\",\"),2,3)",
            "2",
        ),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn a_lifted_dollar_spills_or_is_spill_error() {
    let mut model = new_empty_model();
    model._set("A1", "=DOLLAR({1;2})");
    model._set("B1", "=FIXED({1;2})");
    model._set("B2", "x");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"$1.00");
    assert_eq!(model._get_text("A2"), *"$2.00");
    assert_eq!(model._get_text("B1"), *"#SPILL!");
}

/// The scalar rules the functions had before they moved onto the driver.
#[test]
fn dollar_fixed_numbervalue_keep_their_scalar_rules() {
    let mut model = new_empty_model();
    let cases = [
        ("=DOLLAR(-0.5,-1)", "$0"),
        ("=DOLLAR(-0.004)", "($0.00)"),
        ("=DOLLAR(1,128)", "#VALUE!"),
        ("=FIXED(-0.001,2)", "0.00"),
        ("=FIXED(1234.567,1,FALSE)", "1,234.6"),
        ("=NUMBERVALUE(\"1.5\",\",\",\",\")", "#VALUE!"),
        ("=NUMBERVALUE(\"1.5\",\"\")", "#VALUE!"),
        ("=NUMBERVALUE(\"\")", "0"),
        ("=NUMBERVALUE(\"50%%\")", "0.005"),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}
