use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=DATEDIF()");
    model._set("A2", "=DATEDIF(\"1/1/2025\")");
    model._set("A3", "=DATEDIF(\"1/1/2025\", \"1/1/2026\")");
    model._set("A4", "=DATEDIF(\"1/1/2025\", \"1/1/2026\", \"D\")");
    model._set("A5", "=DATEDIF(\"1/1/2025\", \"1/1/2026\", \"D\", \"D\")");

    model._set("B1", "=YEARFRAC()");
    model._set("B2", "=YEARFRAC(\"1/1/2025\")");
    model._set("B3", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\")");
    model._set("B4", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\", 3)");
    model._set("B5", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\", 3, 3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"365");
    assert_eq!(model._get_text("A5"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"1");
    assert_eq!(model._get_text("B4"), *"1");
    assert_eq!(model._get_text("B5"), *"#ERROR!");
}

#[test]
fn yearfrac_out_of_calendar_start_beats_an_error_end() {
    // dates.md "YEARFRAC reads its start's range before the end's error" (T-2266): the start's
    // #NUM! comes before the end's error, but an error start keeps its own error.
    let mut model = new_empty_model();
    model._set("A1", "=YEARFRAC(-1,1/0)");
    model._set("A2", "=YEARFRAC(NA(),-1)");
    model._set("A3", "=YEARFRAC(123456790.12,1/0,0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#NUM!");
}
