#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_count_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=COUNT()");
    model._set("A2", "=COUNTA()");
    model._set("A3", "=COUNTBLANK()");
    model._set("A4", "=COUNTBLANK(C1:D1, H3:H4)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_countif_date_string_criterion() {
    // 45131..=45137 -> 7/24/2023..7/30/2023 (seven dates strictly before 7/31)
    // 45138         -> 7/31/2023
    // 45139, 45200  -> after 7/31/2023
    let mut model = new_empty_model();
    for (idx, serial) in (45131..=45137).enumerate() {
        let cell = format!("B{}", idx + 2);
        model._set(&cell, &serial.to_string());
    }
    model._set("B9", "45138");
    model._set("B10", "45139");
    model._set("B11", "45200");

    model._set("A1", "=COUNTIF(B2:B11, \"<7/31/2023\")");
    model._set("A2", "=COUNTIF(B2:B11, \"<=7/31/2023\")");
    model._set("A3", "=COUNTIF(B2:B11, \"7/31/2023\")");
    model._set("A4", "=COUNTIF(B2:B11, \">7/31/2023\")");
    model._set("A5", "=COUNTIF(B2:B11, \">=7/31/2023\")");
    model._set("A6", "=COUNTIF(B2:B11, \"<>7/31/2023\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"7");
    assert_eq!(model._get_text("A2"), *"8");
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A4"), *"2");
    assert_eq!(model._get_text("A5"), *"3");
    assert_eq!(model._get_text("A6"), *"9");
}

#[test]
fn test_fn_count_minimal() {
    let mut model = new_empty_model();
    model._set("B1", "3.1415926");
    model._set("B2", "Tomorrow's the day my bride's gonna come");
    model._set("B3", "'");
    model._set("A1", "=COUNT(B1:B5)");
    model._set("A2", "=COUNTA(B1:B5)");
    model._set("A3", "=COUNTBLANK(B1:B5)");
    model.evaluate();

    // There is only one number
    assert_eq!(model._get_text("A1"), *"1");
    // There are three non-empty cells
    assert_eq!(model._get_text("A2"), *"3");
    // There are 3 blank cells B4, B5 and B3 that contains the empty string
    assert_eq!(model._get_text("A3"), *"3");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn test_fn_countif_whole_column_sees_a_spill_beyond_the_used_area() {
    let cases = [
        ("COUNTIF(D:D,\">2\")", "3"),
        ("COUNTIFS(D:D,\">2\",D:D,\"<5\")", "2"),
    ];
    for (formula, expected) in cases {
        let mut model = new_empty_model();
        model._set("A1", &format!("=SEQUENCE(1,1,{formula})"));
        model._set("C1", "=SEQUENCE(3,1,10)");
        model._set("D1", "=SEQUENCE(5)");
        model.evaluate();
        assert_eq!(model._get_text("A1"), expected, "{formula}");
        assert_eq!(model.evaluation.restarts_in_last_evaluation, 1, "{formula}");
    }
}

// A value that is not a reference is `#VALUE!` in COUNTBLANK, one per element for an
// array (probes/countblank-array-values, excel-graph 2026-10-13, T-1437).
#[test]
fn test_fn_countblank_of_a_value_is_value_error() {
    let mut model = new_empty_model();
    model._set("D1", "10");
    model._set("D3", "30");
    model._set("A1", "=COUNTBLANK(INDEX(D1:D5,{1,2,3}))");
    model._set("A2", "=COUNTBLANK(IF({1;1},D1:D2))");
    model._set("A5", "=COUNTBLANK(CHOOSE(1,\"\"))");
    model._set("A6", "=COUNTBLANK(INDEX(D1:D5,2))");
    model._set("D6", "=NA()");
    model._set("A8", "=COUNTBLANK(INDEX(D1:D6,{5,6}))");
    model._set("A10", "=COUNTBLANK(CHOOSE({1,2},D1:D2,D3:D4))");
    model.evaluate();

    for cell in ["A1", "B1", "C1", "A2", "A3", "A5"] {
        assert_eq!(model._get_text(cell), *"#VALUE!", "{cell}");
    }
    // A single picked cell is still a reference.
    assert_eq!(model._get_text("A6"), *"1");
    // An error element keeps its error.
    assert_eq!(model._get_text("A8"), *"#VALUE!");
    assert_eq!(model._get_text("B8"), *"#N/A");
    // CHOOSE with an array index spills (2x2 in Excel; the engine's CHOOSE grid is 1x2, T-1508).
    assert_eq!(model._get_text("A10"), *"#VALUE!");
    assert_eq!(model._get_text("B10"), *"#VALUE!");
}
