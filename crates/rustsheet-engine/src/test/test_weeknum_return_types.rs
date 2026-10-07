use crate::test::util::new_empty_model;

#[test]
fn test_weeknum_return_types_11_to_17_and_21() {
    let mut model = new_empty_model();

    // Date 44561 -> 2021-12-31 (Friday). Previously verified as week 53 (Sunday/Monday start).
    // We verify that custom week-start codes 11-17 all map to week 53 and ISO variant (21) maps to 52.
    let formulas = [
        ("A1", "=WEEKNUM(44561,11)"),
        ("A2", "=WEEKNUM(44561,12)"),
        ("A3", "=WEEKNUM(44561,13)"),
        ("A4", "=WEEKNUM(44561,14)"),
        ("A5", "=WEEKNUM(44561,15)"),
        ("A6", "=WEEKNUM(44561,16)"),
        ("A7", "=WEEKNUM(44561,17)"),
        ("A8", "=WEEKNUM(44561,21)"), // ISO week numbering
    ];
    for (cell, formula) in formulas {
        model._set(cell, formula);
    }

    model.evaluate();

    // All 11-17 variations should yield 53
    for cell in ["A1", "A2", "A3", "A4", "A5", "A6", "A7"] {
        assert_eq!(model._get_text(cell), *"53", "{cell} should be 53");
    }
    // ISO week (return_type 21)
    assert_eq!(model._get_text("A8"), *"52");
}

/// Excel numbers 1900's weeks as if serial 1 were a Sunday
/// (probes/dates-early-1900-edges, probes/weeknum-1900-later; T-0995).
#[test]
fn test_weeknum_1900_anchors_on_a_sunday_serial_1() {
    let mut model = new_empty_model();
    let cases = [
        ("=WEEKNUM(0)", "0"),
        ("=WEEKNUM(0,2)", "1"),
        ("=WEEKNUM(0,12)", "1"),
        ("=WEEKNUM(0,17)", "0"),
        ("=WEEKNUM(2,2)", "2"),
        ("=WEEKNUM(61,2)", "10"),
        ("=WEEKNUM(61)", "9"),
        ("=WEEKNUM(100,2)", "16"),
        ("=WEEKNUM(100)", "15"),
        ("=WEEKNUM(366,2)", "54"),
        ("=WEEKNUM(366)", "53"),
        ("=WEEKNUM(366,16)", "53"),
        // 1901 uses the real calendar again.
        ("=WEEKNUM(367,2)", "1"),
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
