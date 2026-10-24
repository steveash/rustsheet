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

// T-2271: a number weekend truncates, a boolean is 1 or 0 and a blank cell is 0 (`#NUM!`)
// (probes/weekend-number-coercion A1:A26, excel-graph).
#[test]
fn weekend_number_truncates_and_boolean_and_blank_read_as_numbers() {
    let mut model = new_empty_model();
    model._set("B1", "TRUE");
    model._set("B2", "FALSE");
    let cases = [
        ("TRUE", "45007", "23"),
        ("FALSE", "#NUM!", "#NUM!"),
        ("B1", "45007", "23"),
        ("B2", "#NUM!", "#NUM!"),
        ("3.9", "45007", "23"),
        ("11.5", "45006", "27"),
        ("D1", "#NUM!", "#NUM!"),
        ("0.5", "#NUM!", "#NUM!"),
    ];
    for (row, (weekend, _, _)) in (1..).zip(cases) {
        model._set(
            &format!("A{row}"),
            &format!("=WORKDAY.INTL(45000,5,{weekend})"),
        );
        model._set(
            &format!("C{row}"),
            &format!("=NETWORKDAYS.INTL(45000,45030,{weekend})"),
        );
    }
    model.evaluate();
    for (row, (_, workday, networkdays)) in (1..).zip(cases) {
        assert_eq!(
            model._get_text(&format!("A{row}")),
            workday,
            "WORKDAY.INTL row {row}"
        );
        assert_eq!(
            model._get_text(&format!("C{row}")),
            networkdays,
            "NETWORKDAYS.INTL row {row}"
        );
    }
}
