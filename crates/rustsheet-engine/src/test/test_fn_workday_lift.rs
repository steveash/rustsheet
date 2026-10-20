#![allow(clippy::unwrap_used)]

//! WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL on the SPEC-0011 driver (T-1236): the
//! dates lift over arrays and refuse a multi-cell range, the weekend lifts over both, and the
//! holidays are read whole once. Evidence: probes/date-argument-modes F/G 35-42 and 57-64
//! (docs/excel-semantics/dates.md "Date function argument modes").

use crate::test::util::new_empty_model;

/// Evaluates `cells` on a sheet whose A1:A3 holds dates and B1:B3 holds 1, 2 and 3, and checks
/// each cell's text.
fn check(cells: &[(&str, &str, &str)]) {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "45000"),
        ("A2", "45031"),
        ("A3", "45100"),
        ("B1", "1"),
        ("B2", "2"),
        ("B3", "3"),
    ] {
        model._set(cell, value);
    }
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), *expected, "{cell}: {formula}");
    }
}

#[test]
fn dates_lift_over_an_array() {
    // probes/date-argument-modes F/G 36, 38, 40, 58, 60, 62.
    check(&[
        (
            "C1",
            "=ROWS(NETWORKDAYS({45000.25;45031.5;45100.75},46000))",
            "3",
        ),
        (
            "C2",
            "=INDEX(NETWORKDAYS({45000.25;45031.5;45100.75},46000),2)",
            "692",
        ),
        (
            "C3",
            "=INDEX(NETWORKDAYS(45000,{45000.25;45031.5;45100.75}),2)",
            "23",
        ),
        (
            "C4",
            "=INDEX(NETWORKDAYS.INTL({45000.25;45031.5;45100.75},46000),2)",
            "692",
        ),
        (
            "C5",
            "=INDEX(WORKDAY({45000.25;45031.5;45100.75},5),2)",
            "45037",
        ),
        ("C6", "=INDEX(WORKDAY(45000,{1;2;3}),2)", "45002"),
        (
            "C7",
            "=INDEX(WORKDAY.INTL({45000.25;45031.5;45100.75},5),2)",
            "45037",
        ),
        (
            "C8",
            "=ROWS(WORKDAY.INTL({45000.25;45031.5;45100.75},5))",
            "3",
        ),
    ]);
}

#[test]
fn dates_refuse_a_range_and_the_weekend_lifts_over_one() {
    // probes/date-argument-modes F/G 35, 37, 39, 41, 42, 57, 59, 61, 63, 64.
    check(&[
        ("C1", "=ROWS(NETWORKDAYS(A1:A3,46000))", "#VALUE!"),
        ("C2", "=INDEX(NETWORKDAYS(45000,A1:A3),2)", "#VALUE!"),
        ("C3", "=INDEX(NETWORKDAYS.INTL(A1:A3,46000),2)", "#VALUE!"),
        ("C4", "=INDEX(WORKDAY(A1:A3,5),2)", "#VALUE!"),
        ("C5", "=INDEX(WORKDAY(45000,B1:B3),2)", "#VALUE!"),
        ("C6", "=INDEX(WORKDAY.INTL(A1:A3,5),2)", "#VALUE!"),
        ("C7", "=INDEX(NETWORKDAYS.INTL(45000,46000,B1:B3),2)", "715"),
        (
            "C8",
            "=INDEX(NETWORKDAYS.INTL(45000,46000,{1;2;3}),2)",
            "715",
        ),
        ("C9", "=INDEX(WORKDAY.INTL(45000,5,B1:B3),2)", "45007"),
        ("C10", "=INDEX(WORKDAY.INTL(45000,5,{1;2;3}),2)", "45007"),
    ]);
}

#[test]
fn holidays_are_read_whole_and_their_error_is_each_elements() {
    check(&[
        // The holidays B1:B3 (serials 1-3) are before every date: no effect.
        ("C1", "=INDEX(WORKDAY({45000;45001},1,B1:B3),2)", "45002"),
        // 45002 (Friday) is a holiday: the next working day is Monday 45005.
        (
            "C2",
            "=INDEX(WORKDAY({45000;45001},1,{45002,45002}),2)",
            "45005",
        ),
        (
            "C3",
            "=INDEX(NETWORKDAYS({45000;45001},45004,45002),2)",
            "1",
        ),
        // An element's own argument error comes first, then the holidays' error
        // (probes/workday-lift-errors K1:K15).
        ("C4", "=INDEX(WORKDAY({1;\"x\"},1,\"y\"),1)", "#VALUE!"),
        ("C5", "=INDEX(NETWORKDAYS({1;45000},1,NA()),2)", "#N/A"),
        ("C6", "=INDEX(WORKDAY({1;#N/A},1,\"y\"),2)", "#N/A"),
        // All-weekend masks are `#VALUE!`, element by element.
        (
            "C7",
            "=INDEX(WORKDAY.INTL(45000,1,{1;\"1111111\"}),2)",
            "#VALUE!",
        ),
        (
            "C8",
            "=INDEX(WORKDAY.INTL(45000,1,{1;\"1111111\"}),1)",
            "45001",
        ),
    ]);
}

/// The search gives what stepping one day at a time does: NETWORKDAYS from the day after the
/// start to WORKDAY's result is the day count, and the result is a working day, for counts that
/// cross many holidays, both ways.
#[test]
fn workday_agrees_with_networkdays() {
    let mut model = new_empty_model();
    // Holidays: every third day from 45000, repeats and weekends among them.
    model._set("H1", "=SEQUENCE(300,1,45000,3)");
    model._set("H301", "45003");
    model._set("A1", "=WORKDAY(45001,SEQUENCE(400,1,-200),H1:H301)");
    model._set(
        "B1",
        "=AND(MAP(SEQUENCE(400,1,-200),A1#,LAMBDA(n,d,IF(n>0,NETWORKDAYS(45002,d,H1:H301)=n,\
         IF(n<0,NETWORKDAYS(d,45000,H1:H301)=-n,d=45001)))))",
    );
    model._set("C1", "=SUM(--(NETWORKDAYS(A1#+0,A1#+0,H1:H301)=1))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "TRUE");
    // Every result is a working day: the zero step's is the start date, a Thursday.
    assert_eq!(model._get_text("C1"), "400");
}

/// Large day counts and many holidays cost a binary search per element, not a day loop
/// (CLAUDE.md directive 5).
#[test]
fn large_steps_are_bounded() {
    check(&[
        ("C1", "=INDEX(WORKDAY(45000,{1E9;-1E9}),1)", "#NUM!"),
        ("C2", "=INDEX(WORKDAY(45000,{1E9;-1E9}),2)", "#NUM!"),
        // 2000 elements stepping up to two million working days over 20000 holidays.
        (
            "C3",
            "=ISNUMBER(SUM(WORKDAY.INTL(1,SEQUENCE(2000,1,1000,1000),SEQUENCE(2000,1,1,0),\
             SEQUENCE(20000,1,10,7))))",
            "TRUE",
        ),
        (
            "C4",
            "=MAX(WORKDAY(1,SEQUENCE(2000,1,-1000,-1000)))",
            "#NUM!",
        ),
        ("C5", "=WORKDAY(1,2000000)", "2799999"),
    ]);
}
