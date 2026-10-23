use crate::test::util::new_empty_model;

/// LOG, DECIMAL, BASE, ATAN2 and ARABIC lift every argument over an array and a range
/// (probes/t2198-m5-lift-argument-kinds; docs/excel-semantics/arrays.md, T-2159).
#[test]
fn log_decimal_base_atan2_arabic_lift_over_arrays_and_ranges() {
    let mut model = new_empty_model();
    model._set("D1", "1");
    model._set("D2", "2");
    model._set("D3", "3");
    let cases = [
        ("=SUM(LOG({100,1000}))", "5"),
        ("=SUM(LOG(8,{2,8}))", "4"),
        ("=SUM(DECIMAL({\"ff\",\"10\"},16))", "271"),
        ("=SUM(DECIMAL(\"10\",{2,16}))", "18"),
        ("=CONCAT(BASE({5,6},2))", "101110"),
        ("=CONCAT(BASE(5,2,{4,1}))", "0101101"),
        ("=ROUND(SUM(ATAN2({1,-1},1)),6)", "3.141593"),
        ("=SUM(ARABIC({\"X\",\"V\"}))", "15"),
        ("=ROWS(LOG(D1:D3))", "3"),
        ("=CONCAT(BASE(D1:D3,2))", "11011"),
        ("=SUM(DECIMAL(D1:D3,10))", "6"),
        ("=ROWS(ATAN2(1,D1:D3))", "3"),
        // Two arrays broadcast: {1;2} with {1,2,3} is 2x3 (arrays.md, rows 29-35).
        (
            "=ROWS(BASE({1;2},{2,3,4}))&COLUMNS(BASE({1;2},{2,3,4}))",
            "23",
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
fn a_lifted_result_spills_or_is_spill_error() {
    let mut model = new_empty_model();
    model._set("A1", "=BASE({5;6},2)");
    model._set("B1", "=LOG({1;10})");
    model._set("B2", "x");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"101");
    assert_eq!(model._get_text("A2"), *"110");
    assert_eq!(model._get_text("B1"), *"#SPILL!");
}

/// The scalar rules the functions had before they moved onto the driver.
#[test]
fn log_decimal_base_atan2_arabic_keep_their_scalar_rules() {
    let mut model = new_empty_model();
    let cases = [
        ("=LOG(1000)", "3"),
        ("=LOG(8,1)", "#DIV/0!"),
        ("=LOG(8,0)", "#NUM!"),
        ("=LOG(0,1)", "#NUM!"),
        ("=LEN(BASE(1,2,255.9))", "255"),
        ("=BASE(1,2,256)", "#NUM!"),
        ("=BASE(-0.0001,5,4)", "#NUM!"),
        ("=BASE(10,37)", "#NUM!"),
        ("=BASE(0,2)", "0"),
        ("=LEN(BASE(0,2,0))", "0"),
        ("=LEN(BASE(0.2,29,0.3))", "0"),
        ("=BASE(5,2,0)", "101"),
        ("=BASE(255,16,4)", "00FF"),
        ("=DECIMAL(\"ff\",16)", "255"),
        ("=DECIMAL(\"12\",37)", "#NUM!"),
        ("=DECIMAL(REPT(\"1\",256),2)", "#VALUE!"),
        ("=ATAN2(0,0)", "#DIV/0!"),
        ("=ATAN2(0,-1)", "-1.570796327"),
        ("=ARABIC(\"XIV\")", "14"),
        ("=ARABIC(\"\")", "0"),
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

/// ROMAN lifts over an array number and an array form (numbers.md, T-2410).
#[test]
fn roman_lifts_over_arrays() {
    let mut model = new_empty_model();
    model._set("A1", "=CONCAT(ROMAN(561,{0,1,4}))");
    model._set(
        "A2",
        "=ROWS(ROMAN(561,{0,1,4,5}))&COLUMNS(ROMAN(561,{0,1,4,5}))",
    );
    model._set("A3", "=INDEX(ROMAN(561,{0,1,4,5}),1,4)");
    model._set("A4", "=CONCAT(ROMAN({1;4},0))");
    model._set("A5", "=ROWS(ROMAN({1;4},0))");
    model._set("A6", "=ROMAN(4,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "DLXIDLXIDLXI");
    assert_eq!(model._get_text("A2"), "14");
    assert_eq!(model._get_text("A3"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "IIV");
    assert_eq!(model._get_text("A5"), "2");
    assert_eq!(model._get_text("A6"), "IV");
}
