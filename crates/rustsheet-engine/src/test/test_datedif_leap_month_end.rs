use crate::test::util::new_empty_model;

#[test]
fn test_datedif_yd_leap_year_edge_cases() {
    let mut model = new_empty_model();

    // 29 Feb 2020 → 28 Feb 2021: the end's month and day come first, so it moves into 2021 and
    // the count runs from the start; UNVERIFIED, no oracle cell starts on 29 February (dates.md "DATEDIF "MD" and "YD" day counts", T-2264)
    model._set("A1", "=DATEDIF(\"2/29/2020\", \"2/28/2021\", \"YD\")");

    // 29 Feb 2020 → 1 Mar 2021    (should be 1 day)
    model._set("A2", "=DATEDIF(\"2/29/2020\", \"2021-03-01\", \"YD\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"365");
    assert_eq!(model._get_text("A2"), *"1");
}

#[test]
fn test_datedif_md_month_end_edge_cases() {
    let mut model = new_empty_model();

    // 31 Jan 2021 → 28 Feb 2021 (non-leap) => 28
    model._set("B1", "=DATEDIF(\"1/31/2021\", \"2/28/2021\", \"MD\")");

    // 31 Jan 2020 → 29 Feb 2020 (leap) => 29
    model._set("B2", "=DATEDIF(\"1/31/2020\", \"2/29/2020\", \"MD\")");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"28");
    assert_eq!(model._get_text("B2"), *"29");
}

#[test]
fn test_datedif_excel_graph_day_counts() {
    let mut model = new_empty_model();
    // MD across a 31-to-30 month end; YD across a leap day and with the end's month-day first
    // (gen-m6 excel-graph cells, T-2264).
    model._set("A1", "=DATEDIF(45382,45944,\"MD\")");
    model._set("A2", "=DATEDIF(15974,43001,\"YD\")");
    model._set("A3", "=DATEDIF(29872,35271,\"YD\")");
    model._set("A4", "=DATEDIF(19510,55815,\"MD\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"13");
    assert_eq!(model._get_text("A2"), *"364");
    assert_eq!(model._get_text("A3"), *"285");
    assert_eq!(model._get_text("A4"), *"22");
}
