use crate::test::util::new_empty_model;

// T-2580: an empty array (`#CALC!`) as a value argument is `#VALUE!` in SWITCH's expression and
// cases, IMSUM, IMPRODUCT and the holidays of WORKDAY(.INTL) and NETWORKDAYS(.INTL), and stays
// `#CALC!` elsewhere (probes/t2436/computed-scalar A43:A50, probes/t2580-empty-array-args).
#[test]
fn empty_array_argument_is_value_in_switch_imsum_and_holidays() {
    let cases = [
        ("=SWITCH(1,TAKE(0.2,0,3),5,6)", "#VALUE!"),
        ("=SWITCH(DROP({1},1),1,2,3)", "#VALUE!"),
        ("=SWITCH(1,1,DROP({1},1))", "#CALC!"),
        ("=SWITCH(2,1,5,DROP({1},1))", "#CALC!"),
        ("=SWITCH(1,1,2,DROP({1},1),3)", "2"),
        ("=IMSUM(DROP({1},1))", "#VALUE!"),
        ("=IMPRODUCT(DROP({1},1))", "#VALUE!"),
        ("=WORKDAY(1,1,DROP({1},1))", "#VALUE!"),
        ("=WORKDAY.INTL(1,1,1,DROP({1},1))", "#VALUE!"),
        ("=NETWORKDAYS(1,10,DROP({1},1))", "#VALUE!"),
        ("=NETWORKDAYS.INTL(1,10,1,DROP({1},1))", "#VALUE!"),
        ("=ABS(DROP({1},1))", "#CALC!"),
        ("=SUM(DROP({1},1))", "#CALC!"),
        ("=CHOOSE(1,DROP({1},1))", "#CALC!"),
        ("=IFS(TRUE,DROP({1},1))", "#CALC!"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in (1..).zip(cases) {
        model._set(&format!("A{row}"), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in (1..).zip(cases) {
        assert_eq!(model._get_text(&format!("A{row}")), expected, "{formula}");
    }
}

// T-2580: SEQUENCE's rows and columns both stop at 1048576; an extent past that is `#VALUE!` even
// beside a zero one (probes/t2580-sequence-zero-extent, t2580-sequence-column-cap).
#[test]
fn sequence_zero_extent_beside_an_oversized_or_negative_one() {
    let cases = [
        ("=SEQUENCE(0.0123,1)", "#CALC!"),
        ("=SEQUENCE(0.0123,123456789.123)", "#VALUE!"),
        ("=SEQUENCE(0,1048576)", "#CALC!"),
        ("=SEQUENCE(0,1048577)", "#VALUE!"),
        ("=SEQUENCE(1048576,0)", "#CALC!"),
        ("=SEQUENCE(1048577,0)", "#VALUE!"),
        ("=SEQUENCE(0,-1)", "#VALUE!"),
        ("=SEQUENCE(-1,0)", "#VALUE!"),
        ("=SEQUENCE(0,0)", "#CALC!"),
        ("=SEQUENCE(1,-1E-9)", "#CALC!"),
        ("=SEQUENCE(0,-8.1E-10)", "#CALC!"),
        ("=SEQUENCE(1,-1E-6)", "#VALUE!"),
        ("=SEQUENCE(0,-1.19E-7)", "#CALC!"),
        ("=SEQUENCE(0,-1.2E-7)", "#VALUE!"),
        ("=SEQUENCE(0,-2E-7)", "#VALUE!"),
        ("=COLUMNS(SEQUENCE(1,3-2E-7))", "3"),
        ("=COLUMNS(SEQUENCE(1,3-3E-7))", "2"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in (1..).zip(cases) {
        model._set(&format!("A{row}"), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in (1..).zip(cases) {
        assert_eq!(model._get_text(&format!("A{row}")), expected, "{formula}");
    }
}
