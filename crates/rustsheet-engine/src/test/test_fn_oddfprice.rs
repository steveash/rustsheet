#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Odd first long period (issue to settlement): Jan 15, 2020 => March 1, 2021
#[test]
fn test_oddfprice_basis1_2() {
    let mut model = new_empty_model();
    // Too few args
    model._set("A1", "=DATE(2020,3,1)"); // Settlement: 1/March/2020
    model._set("A2", "=DATE(2030,3,1)"); // Maturity: 1/March/2030
    model._set("A3", "=DATE(2020,1,15)"); // Issue: 15/Jan/2020
    model._set("A4", "=DATE(2021,3,1)"); // 1/March/2021
    model._set("A5", "7.5%"); // Rate: 7.5%
    model._set("A6", "7%"); // Yield: 7%
    model._set("A7", "120"); // Redemption
    model._set("A8", "1"); // Frequency: 1 (Annual)
    model._set("A9", "2"); // Basis: 2 (Actual/360)
    model._set("C3", "=ODDFPRICE(A1,A2,A3,A4,A5,A6,A7,A8,A9)");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "113.5084665");
    // 115.98854775223758 yield 6.7%
}

// =ODDFPRICE("3/1/2020","3/1/2030","1/15/2020","3/1/2021",7.5%,0%,120,1,2)

#[test]
fn test_oddfprice_basis_2_zero_yield() {
    let mut model = new_empty_model();
    // Too few args
    model._set("A1", "=DATE(2020,3,1)"); // Settlement: 1/March/2020
    model._set("A2", "=DATE(2030,3,1)"); // Maturity: 1/March/2030
    model._set("A3", "=DATE(2020,1,15)"); // Issue: 15/Jan/2020
    model._set("A4", "=DATE(2021,3,1)"); // 1/March/2021
    model._set("A5", "7.5%"); // Rate: 7.5%
    model._set("A6", "0%"); // Yield: 0%
    model._set("A7", "120"); // Redemption
    model._set("A8", "1"); // Frequency: 1 (Annual)
    model._set("A9", "2"); // Basis: 2 (Actual/360)
    model._set("C3", "=ODDFPRICE(A1,A2,A3,A4,A5,A6,A7,A8,A9)");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "195");
}

// =ODDFPRICE("3/1/2020","3/1/2030","1/15/2020","3/1/2021",0%,10%,100,1,2)

#[test]
fn test_oddfprice_basis_2_zero_rate() {
    let mut model = new_empty_model();
    // Too few args
    model._set("A1", "=DATE(2020,3,1)"); // Settlement: 1/March/2020
    model._set("A2", "=DATE(2030,3,1)"); // Maturity: 1/March/2030
    model._set("A3", "=DATE(2020,1,15)"); // Issue: 15/Jan/2020
    model._set("A4", "=DATE(2021,3,1)"); // 1/March/2021
    model._set("A5", "0%"); // Rate: 0%
    model._set("A6", "10%"); // Yield: 10%
    model._set("A7", "100"); // Redemption
    model._set("A8", "1"); // Frequency: 1 (Annual)
    model._set("A9", "2"); // Basis: 2 (Actual/360)
    model._set("C3", "=ODDFPRICE(A1,A2,A3,A4,A5,A6,A7,A8,A9)");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "38.50332632");
}

#[test]
fn test_oddfprice_basis2_long_first_period() {
    // Reverse-engineered against Excel to machine precision (131.48376704654200).
    // Long odd first period, semiannual, Actual/360. The stub (issue->first
    // quasi-coupon date) must be prorated over the nominal length 360/freq = 180,
    // not the period's actual length (182).
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2020,3,1)"); // settlement
    model._set("A2", "=DATE(2030,3,1)"); // maturity
    model._set("A3", "=DATE(2020,1,15)"); // issue
    model._set("A4", "=DATE(2021,3,1)"); // first coupon
    model._set("A5", "7.5%"); // rate
    model._set("A6", "5%"); // yield
    model._set("A7", "120"); // redemption
    model._set("A8", "2"); // frequency: semiannual
    model._set("A9", "2"); // basis: Actual/360
    model._set("C3", "=ODDFPRICE(A1,A2,A3,A4,A5,A6,A7,A8,A9)");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "131.483767");
}

#[test]
fn test_oddfprice_basis1_long_first_period_unchanged() {
    // Same bond, Actual/Actual. Stub prorated over its actual length (182).
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2020,3,1)");
    model._set("A2", "=DATE(2030,3,1)");
    model._set("A3", "=DATE(2020,1,15)");
    model._set("A4", "=DATE(2021,3,1)");
    model._set("A5", "7.5%");
    model._set("A6", "5%");
    model._set("A7", "120");
    model._set("A8", "2");
    model._set("A9", "1"); // basis: Actual/Actual
    model._set("C3", "=ODDFPRICE(A1,A2,A3,A4,A5,A6,A7,A8,A9)");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "131.5569687");
}

// The first coupon must be on the maturity's schedule and before the maturity, and a first period
// exactly one regular period long on the basis (DFC = E) is #NUM!; two periods is fine
// (probes/date-system-1900-oddf-schedule, date-system-1900-oddf-issue-boundary; T-1457, T-1490).
#[test]
fn oddfprice_first_coupon_must_fit_the_schedule() {
    let mut model = new_empty_model();
    let price = "0.05,0.06,100,2,0)";
    model._set(
        "A1",
        &format!("=ODDFPRICE(DATE(2024,2,1),DATE(2025,7,2),DATE(2024,1,1),DATE(2024,7,1),{price}"),
    );
    model._set(
        "A2",
        &format!("=ODDFPRICE(DATE(2024,2,1),DATE(2024,7,1),DATE(2024,1,1),DATE(2024,7,1),{price}"),
    );
    model._set(
        "A3",
        &format!("=ODDFPRICE(DATE(2024,2,1),DATE(2025,7,1),DATE(2024,1,1),DATE(2024,7,1),{price}"),
    );
    model._set(
        "A4",
        &format!("=ODDFPRICE(DATE(2024,2,1),DATE(2025,7,1),DATE(2023,7,1),DATE(2024,7,1),{price}"),
    );
    model._set(
        "A5",
        &format!(
            "=ODDFPRICE(DATE(2024,2,1),DATE(2025,2,28),DATE(2024,1,1),DATE(2024,8,31),{price}"
        ),
    );
    // Far-negative dates are #NUM! without walking the schedule past the calendar.
    model._set(
        "A6",
        "=ODDFPRICE(DATE(2024,2,1),DATE(2025,7,1),-1E+15,-1E+14,0.05,0.06,100,4,0)",
    );
    model.evaluate();
    assert_eq!(model._get_text("A6"), "#NUM!");
    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "#NUM!");
    assert_eq!(model._get_text("A4"), "98.59507655");
    assert_eq!(model._get_text("A5"), "98.93442798");
}

// A first period of exactly E days on the basis is #NUM!, even an issue a day after the quasi-coupon
// date (180 days 30/360), while the quasi-coupon date itself is a value on basis 2 (184 actual days)
// (probes/date-system-1900-oddf-issue-after-quasi A28, A36, A38, A41, A5; T-1490).
#[test]
fn oddfprice_first_period_of_one_period_on_the_basis_is_num() {
    let mut model = new_empty_model();
    let call = |issue: &str, basis: u32| {
        format!(
            "=ODDFPRICE(DATE(2023,8,1),DATE(2024,6,30),{issue},DATE(2023,12,31),0.05,0.06,100,2,{basis})"
        )
    };
    model._set("A1", &call("DATE(2023,7,1)", 0));
    model._set("A2", &call("DATE(2023,7,1)", 1));
    model._set("A3", &call("DATE(2023,7,1)", 4));
    model._set("A4", &call("DATE(2023,7,4)", 2));
    model._set("A5", &call("DATE(2023,6,30)", 2));
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "99.11926909");
    assert_eq!(model._get_text("A3"), "99.11852867");
    assert_eq!(model._get_text("A4"), "#NUM!");
    assert_eq!(model._get_text("A5"), "96.20983152");
}
