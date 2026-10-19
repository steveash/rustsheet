use crate::test::util::new_empty_model;

// T-1235: the weekend argument lifts, and each distinct mask is computed once however many
// cells the argument has (the day loop is millions of steps long).
#[test]
fn weekend_lift_over_a_large_array_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "=NETWORKDAYS.INTL(1,2958465,SEQUENCE(100000)*0+1)");
    model._set("A100001", "=WORKDAY.INTL(45000,5,SEQUENCE(100000)*0+1)");
    model._set("B1", "=NETWORKDAYS.INTL(1,2958465,1)");

    model.evaluate();

    let scalar = model._get_text("B1");
    assert_eq!(model._get_text("A1"), scalar);
    assert_eq!(model._get_text("A100000"), scalar);
    assert_eq!(model._get_text("A100001"), *"45007");
}

#[test]
fn weekend_lifts_per_element() {
    let mut model = new_empty_model();
    model._set("A1", "=WORKDAY.INTL(45000,5,{1;2;3})");

    model.evaluate();

    assert_eq!(model._get_text("A2"), *"45007");
}

// Review of T-1235: the leftmost error wins over the unit and method of DATEDIF and DAYS360,
// and a single-cell range is the value of its cell for TIMEVALUE.
#[test]
fn leftmost_error_and_single_cell_ranges() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEDIF(1/0,5,NA())");
    model._set("A2", "=DAYS360(1/0,5,NA())");
    model._set("B3", "10:30");
    model._set("A3", "=TIMEVALUE(B3:B3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"0.4375");
}

// T-1428: a 3-D weekend reads as `#REF!`, which WORKDAY.INTL turns into `#NUM!` after the
// holidays and NETWORKDAYS.INTL into `#VALUE!` before the dates; a 3-D holidays is `#VALUE!`
// (probes/date-text-weekend-3d K3, J3, P23, Q22, M9).
#[test]
fn three_d_weekend_and_holidays() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", "=NETWORKDAYS.INTL(1,30,Sheet1:Sheet2!B1:B3)");
    model._set("A2", "=WORKDAY.INTL(45000,5,Sheet1:Sheet2!B1:B3)");
    model._set("A3", "=WORKDAY.INTL(45000,5,Sheet1:Sheet2!B1,1/0)");
    model._set("A4", "=NETWORKDAYS.INTL(1/0,30,Sheet1:Sheet2!B1)");
    model._set("A5", "=WORKDAY.INTL(45000,5,1,Sheet1:Sheet2!B1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"#VALUE!");
}
