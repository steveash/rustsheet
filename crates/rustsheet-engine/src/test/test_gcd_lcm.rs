#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM()");
    model._set("A2", "=LCM(2, 3)");

    model._set("A3", "=GCD()");
    model._set("A4", "=GCD(10, 25)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"6");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"5");
}

#[test]
fn arrays() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM({2, 3}, {4, 5, 6})");
    model._set("A2", "=GCD({10, 25}, {35, 40, 50})");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"60");
    assert_eq!(model._get_text("A2"), *"5");
}

#[test]
fn invalid_inputs() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM(-5, 6)");
    model._set("A2", "=LCM(\"-5\", 6)");
    model._set("A3", "=GCD(-5, 6)");
    model._set("A4", "=GCD(\"-5\", 6)");

    model._set("A5", "=LCM({1, 2, 3}, {4, -5, 6})");
    model._set("A6", "=LCM({1, 2, 3}, {4, \"-5\", 6})");
    model._set("A7", "=GCD({1, 2, 3}, {4, -5, 6})");
    model._set("A8", "=GCD({1, 2, 3}, {4, \"-5\", 6})");

    // Numbers are cut to 15 significant digits on parse (T-0143), so 2^53+2 would no longer be
    // past the limit; 1E+16 is.
    model._set("B1", "=LCM(1E+16, 6)");
    model._set("B2", "=LCM(\"1E+16\", 6)");
    model._set("B3", "=GCD(1E+16, 6)");
    model._set("B4", "=GCD(\"1E+16\", 6)");

    model._set("B5", "=LCM({1, 2, 3}, {4, 1E+16, 6})");
    model._set("B6", "=LCM({1, 2, 3}, {4, \"1E+16\", 6})");
    model._set("B7", "=GCD({1, 2, 3}, {4, 1E+16, 6})");
    model._set("B8", "=GCD({1, 2, 3}, {4, \"1E+16\", 6})");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");

    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
    assert_eq!(model._get_text("B3"), *"#NUM!");
    assert_eq!(model._get_text("B4"), *"#NUM!");

    assert_eq!(model._get_text("B5"), *"#NUM!");
    assert_eq!(model._get_text("B6"), *"#NUM!");
    assert_eq!(model._get_text("B7"), *"#NUM!");
    assert_eq!(model._get_text("B8"), *"#NUM!");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let cases = [("GCD(D:D)", "1"), ("LCM(D:D)", "60")];
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

// Excel accepts 2^53 as an argument and as an LCM result, rejects more
// (probes/gcd-lcm-bounds, lcm-result-bound, lcm-result-edge).
#[test]
fn bounds_at_2_pow_53() {
    let mut model = new_empty_model();
    model._set("A1", "=GCD(2^53,2)");
    model._set("A2", "=GCD(2^53+2,2)");
    model._set("A3", "=LCM(2^53,1)=2^53");
    model._set("A4", "=LCM(2^53,2)=2^53");
    model._set("A5", "=LCM(2^53,3)");
    model._set("A6", "=LCM(2^52+1,2)");
    model._set("A7", "=LCM(3,3002399751580330+1)=2^53");
    model._set("A8", "=LCM(1E+300,1)");
    model._set("A9", "=LCM(3037000500,3037000501)");
    model._set("A10", "=GCD(2^53+1,2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"TRUE");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"TRUE");
    assert_eq!(model._get_text("A8"), *"#NUM!");
    assert_eq!(model._get_text("A9"), *"#NUM!");
    assert_eq!(model._get_text("A10"), *"2");
}

// The type check over every argument wins over a negative number whatever the order
// (gen-m6/5cc04d6f42b23b51 Calc!A11, gen-m7/4629e931955aa4b8 Calc!A7).
#[test]
fn range_text_or_boolean_is_value_even_after_a_negative() {
    let mut model = new_empty_model();
    model._set("B1", "TRUE");
    model._set("B2", "4");
    model._set("C1", "-0.5");
    model._set("C2", "6");
    model._set("C3", "x");
    model._set("D1", "-1");
    model._set("D2", "6");
    model._set("A1", "=GCD(-1,B1:B2)");
    model._set("A2", "=GCD(C1:C3)");
    model._set("A3", "=GCD(D1:D2)");
    // Any other error beats the negative, typed or in a range
    // (probes/gcd-lcm-multinomial-negative-precedence E2, E6, E7).
    model._set("A4", "=GCD(-1,\"a\")");
    model._set("A5", "=GCD(-1,NA())");
    model._set("E1", "-1");
    model._set("E2", "=NA()");
    model._set("A6", "=GCD(E1:E2)");
    model._set("A7", "=GCD(-1,E2:E2)");
    model._set("A8", "=GCD(E2:E2)");
    model.evaluate();
    assert_eq!(model._get_text("A6"), *"#N/A");
    assert_eq!(model._get_text("A7"), *"#N/A");
    assert_eq!(model._get_text("A8"), *"#N/A");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"#N/A");
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
}
