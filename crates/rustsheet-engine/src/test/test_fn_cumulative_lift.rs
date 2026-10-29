#![allow(clippy::unwrap_used)]

//! CUMIPMT and CUMPRINC on the SPEC-0011 driver (T-1845): their period sums take constant time
//! whatever the period count, every argument lifts over an array and refuses a multi-cell range
//! (`LiftArrays`), and a lifted call is charged its bounded per-period sum. Evidence:
//! probes/fin-cumipmt-period-bound, probes/fin-argument-modes H72-H87 and
//! probes/fin-remaining-later-argument-ranges H65-H84, K1-K3, K11 (docs/excel-semantics/
//! financial.md "CUMIPMT and CUMPRINC over many periods", "Financial function argument modes").

use std::time::{Duration, Instant};

use crate::test::util::{check_cells, new_empty_model, round_trip_value};

/// Evaluates `cells` on a sheet whose G1:G2 is 0.01 (a rate), G3:G4 is 360 (an nper) and
/// Z1:Z8 hold the extreme values of probes/fin-cumipmt-period-bound (1E308 as a product: the
/// literal is past Excel's 9.99E307), and checks each cell's
/// value (an expected number is compared bit for bit).
fn check(cells: &[(&str, &str, &str)]) {
    check_cells(
        &[
            ("G1", "0.01"),
            ("G2", "0.01"),
            ("G3", "360"),
            ("G4", "360"),
            ("Z1", "1E-300"),
            ("Z2", "1E300"),
            ("Z3", "1E300"),
            ("Z4", "1E300"),
            ("Z5", "=1E307*10"),
            ("Z6", "1E-12"),
            ("Z7", "1E12"),
            ("Z8", "1E307"),
        ],
        cells,
    );
}

#[test]
fn a_billion_periods_take_milliseconds_and_are_excels_values() {
    // fin-cumipmt-period-bound A1, A2, A4, A26, A28; main looped 10^9 times for A1.
    let started = Instant::now();
    check(&[
        ("A1", "=CUMIPMT(1E-9,1E9,1,1,1E9,0)", "-0.5819767073296636"),
        ("A2", "=CUMPRINC(1E-9,1E9,1,1,1E9,0)", "-1"),
        ("A3", "=CUMPRINC(1E-9,1E9,1,1,1E9,1)", "-1"),
        ("A4", "=CUMIPMT(0.01,Z5,1,1,Z5,0)", "-1E306"),
        ("A5", "=CUMIPMT(0.01,Z5,1,Z8,Z5,1)", "-8.91089108910891E305"),
    ]);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn overflowing_powers_still_give_excels_values() {
    // fin-cumipmt-period-bound A13-A17, fin-remaining-later-argument-ranges K1, K2: main was
    // #NUM! once (1+r)^nper overflowed.
    check(&[
        ("A1", "=CUMIPMT(0.01,1E6,100000,1,10,0)", "-10000"),
        ("A2", "=CUMPRINC(0.01,1E6,100000,1,10,0)", "0"),
        (
            "A3",
            "=CUMIPMT(0.01,1E6,100000,1,10,1)",
            "-8910.89108910891",
        ),
        ("A4", "=CUMIPMT(1E10,100,1,1,100,0)", "-999999999999"),
        ("A5", "=CUMPRINC(1E10,100,1,1,100,0)", "-1.0000000000000013"),
        (
            "A6",
            "=CUMIPMT(0.01,1000000,100000,1,1000000,0)",
            "-999900000",
        ),
        (
            "A7",
            "=CUMPRINC(0.01,1000000,100000,1,1000000,0)",
            "-100000",
        ),
    ]);
}

#[test]
fn extreme_values_from_cells_are_values_without_a_panic() {
    // fin-cumipmt-period-bound A24, A25, A27, with the values in Z1:Z8.
    check(&[
        ("A1", "=CUMIPMT(Z1,Z2,Z3,1,Z2,0)", "-5.819767068693264E299"),
        ("A2", "=CUMPRINC(Z4,Z2,Z1,1,Z2,1)", "-1E-300"),
        ("A3", "=CUMPRINC(Z6,Z7,Z3,1,Z7,0)", "-9.999999999999999E299"),
        // A payment that overflows: CUMIPMT is #NUM! as in Excel (probes/fin-cumipmt-overflow-and-forms
        // A1, A3), while type 1's CUMPRINC is still the loan, -1E300 (A2; T-2192).
        ("A4", "=CUMIPMT(Z2,Z5,Z3,1,Z5,0)", "#NUM!"),
        ("A5", "=CUMPRINC(Z2,10,Z3,1,10,1)", "-1E300"),
        (
            "A6",
            "=ROWS(CUMIPMT({1E300;-1E300},Z5,Z3,{1;1E300},Z2,{0;1}))",
            "2",
        ),
        (
            "A7",
            "=INDEX(CUMIPMT({1E300;-1E300},Z5,Z3,{1;1E300},Z2,{0;1}),2)",
            "#NUM!",
        ),
    ]);
}

#[test]
fn an_end_period_past_nper_is_num() {
    // fin-cumipmt-period-bound A20-A22: main computed A2, whose end is below nper + 1.
    check(&[
        ("A1", "=CUMIPMT(0.0001,5000.5,1000,1,5001,0)", "#NUM!"),
        ("A2", "=CUMIPMT(0.01,100.5,1000,1,101,0)", "#NUM!"),
        ("A3", "=CUMIPMT(0.01,1E6,100000,1,1E6+1,0)", "#NUM!"),
    ]);
}

#[test]
fn up_to_32_periods_are_the_per_period_sum() {
    // Type 1's interest is main's sum of IPMT up to `CUMULATIVE_LOOP_PERIODS` (32), bit for bit
    // (ironcalc-calc/FINANCIAL/CUMPRINC_CUMIPMT.xlsx G3, G9, G15, G26); past it, the closed form.
    // Type 0's short sums are their own forms (T-2192, below). The principal of type 1 is the
    // closed form at any length, close to the PPMT sum (T-1890). The per-period sums are rebuilt
    // here from IPMT and PPMT.
    let mut model = new_empty_model();
    let pairs = [
        (
            "=CUMIPMT(0.004,1024,1000,2,33,1)",
            "=SUM(IPMT(0.004,SEQUENCE(32,1,2),1024,1000,0,1))",
        ),
        (
            "=CUMPRINC(0.004,1024,1000,3,34,1)",
            "=SUM(PPMT(0.004,SEQUENCE(32,1,3),1024,1000,0,1))",
        ),
        (
            "=CUMIPMT(0.004,1024,1000,1,33,0)",
            "=SUM(IPMT(0.004,SEQUENCE(33),1024,1000))",
        ),
    ];
    for (row, (cumulative, sum)) in pairs.iter().enumerate() {
        model._set(&format!("A{}", row + 1), cumulative);
        model._set(&format!("B{}", row + 1), sum);
    }
    model.evaluate();
    let number = |cell: &str| round_trip_value(&model, cell).parse::<f64>().unwrap();
    assert_eq!(number("A1").to_bits(), number("B1").to_bits());
    // The PPMT sum cancels (each principal is a fiftieth of its payment). The geometric sum on
    // Excel's own `ln(1+rate)` is Excel's value, an ulp or two from the exact
    // -2.33419231896298640 (probes/fin-cumipmt-gate-edges J1; T-1890, T-2521).
    assert_eq!(number("A2"), -2.334_192_318_962_987_4);
    assert!((number("A2") / number("B2") - 1.0).abs() < 1E-12);
    // 33 periods: past the per-period sums, Excel's value (gate-edges J2), within an ulp of the
    // IPMT sum.
    assert_eq!(number("A3"), -131.849_721_045_626_13);
    assert_ne!(number("A3").to_bits(), number("B3").to_bits());
    assert!((number("A3") / number("B3") - 1.0).abs() < 1E-14);
    check(&[
        // CUMPRINC_CUMIPMT.xlsx G3 (one period, the sum) and fin-cumipmt-period-bound A9, A11
        // (1,024 periods, the closed form).
        ("A1", "=CUMIPMT(0.1,10,345,2,2,1)", "-29.395712615048502"),
        (
            "A2",
            "=CUMIPMT(0.004,1024,1000,1,1024,0)",
            "-3165.8852276668513",
        ),
        ("A3", "=CUMPRINC(0.004,1024,1000,1,1024,1)", "-1000"),
        // A large rate over a short range, where expm1 of the range overflows: the difference
        // of powers (probes/fin-cumipmt-overflow-and-forms A15-A19).
        ("A5", "=CUMPRINC(1E10,40,1,10,40,0)", "-1.0000000000000013"),
        ("A6", "=CUMIPMT(1E300,10,1,1,10,0)", "-1E301"),
        // CUMPRINC_CUMIPMT.xlsx H6, which the PPMT sum missed by 3 ulps.
        (
            "A4",
            "=CUMPRINC(0.01,36,8000,1.2,8.53,1)",
            "-1339.6579298169904",
        ),
    ]);
}

#[test]
fn a_subnormal_or_underflowed_leading_power_keeps_its_digits() {
    // A rate of 1E10 over a short range: `v^(a-1-nper)` is subnormal (A20-A22) or underflows
    // (A23, A24), where the closed form's product `v^(a-1-nper)·expm1(..)` loses digits
    // (-9.999888696826292E-71 for A20) and its difference of powers is up to 343 ulps off. Type
    // 0's short principal is one geometric sum from the last period back (T-2514; the per-period
    // sum of `pmt·exp((k-1-nper)·L)` before, T-2192), within an ulp of Excel's values,
    // probes/fin-cumipmt-overflow-and-forms A20-A24 (excel-graph).
    let cells = [
        (
            "A1",
            "=CUMPRINC(1E10,32,1,1,25,0)",
            -9.999_999_993_000_11E-71,
        ),
        (
            "A2",
            "=CUMPRINC(1E10,32,1,1,10,0)",
            -9.999_999_978_000_387E-221,
        ),
        (
            "A3",
            "=CUMPRINC(1E10,31.5,1,1,20,0)",
            -9.999_999_988_500_136E-116,
        ),
        (
            "A4",
            "=CUMPRINC(1E10,40,1,1,20,0)",
            -9.999_999_980_000_501E-201,
        ),
        (
            "A5",
            "=CUMPRINC(1E10,40,1,15,20,0)",
            -9.999_999_980_000_501E-201,
        ),
    ];
    let mut model = new_empty_model();
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, excel) in cells {
        let ours = round_trip_value(&model, cell).parse::<f64>().unwrap();
        assert!(
            (ours / excel - 1.0).abs() < 3E-16,
            "{formula}: ours {ours:e}, Excel {excel:e}"
        );
    }
}

#[test]
fn a_short_principal_at_a_large_rate_is_one_geometric_sum() {
    // Excel's principal over a short range is `pmt·((1-v^m)/r·(1+r))·v^(nper-end+1)` (over `1+r`
    // for type 1): to the bit on these gen-m9 cells, where the per-period powers (type 0) were
    // 11 to 15 ulps off and the closed form (type 1) 16 to 104 (excel-graph; T-2514). A rate of
    // 1E-10 keeps type 1's interest as the payments less that principal (b72d952c5f52cb4d A257).
    check(&[
        // 9a1fa5366bc50918 A361, 311 ulps from the exact sum.
        (
            "A1",
            "=CUMPRINC(10,261,484823,4,24,0)",
            "-7.507871520276804E-242",
        ),
        // 1cd3096e3e87c965 A147.
        (
            "A2",
            "=CUMPRINC(1,200,480533,5,18,0)",
            "-7.838581871529754E-50",
        ),
        // 0a957119475760c5 A383 and a91bad4e84f49739 A193 (type 1).
        (
            "A3",
            "=CUMPRINC(10,121,308352,2,16,1)",
            "-1.263054652346343E-105",
        ),
        (
            "A4",
            "=CUMPRINC(1,331,199079,3,14,1)",
            "-3.727183608915611E-91",
        ),
        (
            "A5",
            "=CUMIPMT(1E-10,296,197540,13,19,1)",
            "-0.00013127066813467536",
        ),
        // Below `GEOMETRIC_MIN_LEAD` the old forms keep these to the bit: the per-period
        // sum (gen-m9 d912671baa1364aa A300) and type 1's closed form (CUMPRINC_CUMIPMT.xlsx H25).
        (
            "A6",
            "=CUMPRINC(1E-9,192,2707,8,24,0)",
            "-239.68227237224258",
        ),
        (
            "A7",
            "=CUMPRINC(0.1234,36,8000,3,10,1)",
            "-212.62776661477534",
        ),
        // Past 32 periods a range that ends a deep discount before nper is the geometric sum
        // too (gen-m9 b73c6cc96e093f03 A62, probes/fin-cumprinc-form-boundaries A2; T-3067), and
        // one that ends near nper keeps the closed form (fin-cumipmt-period-bound A6).
        (
            "A8",
            "=CUMPRINC(2,151,319288,3,42,1)",
            "-1.049166891970377E-47",
        ),
        (
            "A9",
            "=CUMPRINC(2,600,319288,3,42,1)",
            "-6.214428680002485E-262",
        ),
        (
            "A10",
            "=CUMPRINC(0.005,100000,1000,2,99999,1)",
            "-990.0745031063589",
        ),
    ]);
}

#[test]
fn type_0_short_interest_is_the_balance_form() {
    // probes/fin-cumipmt-overflow-and-forms A12-A14 (excel-graph): the interest of period k is
    // `-(pv·q·r - pmt·(q-1))` with `q = exp((k-1)·L)`, not the exact IPMT (A12 was
    // -15.708656559792592, the exact value; T-2192). A rate of 1E10 overflows `q`, and the closed
    // form keeps A19.
    check(&[
        (
            "A1",
            "=CUMIPMT(0.01,100,100000,100,100,0)",
            "-15.708656559792416",
        ),
        (
            "A2",
            "=CUMIPMT(0.0083,36,8000,3,4,0)",
            "-124.79622431364345",
        ),
        ("A3", "=CUMIPMT(1E10,40,1,10,40,0)", "-309999999999"),
    ]);
}

#[test]
fn type_0_interest_is_the_payments_less_the_principal() {
    // T-2363 (excel-graph, gen-m9): outside the balance form's growth band [2^-10, 2^10] Excel's
    // interest is `-pmt·m - principal`, cancellation included (A1 is 4E-10 relative off the exact
    // sum; 5c6a23c4f6597f1c A392); at huge rates it is the payments (A2, A3: 1cd3096e3e87c965 A316,
    // e168be7f6d3f84c2 A330); where `1 + rate` is 1 the principal is the balance drop (A4, A5:
    // 81bcdc0bec0c8e6b A274, abb7d565af89b849 A84; A6: past 2^53, probes/fin-cumipmt-overflow-and-
    // forms A5); inside the band the balance form stays (A7, probes A12).
    check(&[
        (
            "A1",
            "=CUMIPMT(1E-9,55,157761,1,18,0)",
            "-0.002400835590378847",
        ),
        ("A2", "=CUMIPMT(10,89,267388,3,13,0)", "-29412680"),
        ("A3", "=CUMIPMT(100,167,7,6,16,0)", "-7700"),
        (
            "A4",
            "=CUMIPMT(1E-300,243,80923,12,22,0)",
            "-4.547473508864641E-13",
        ),
        ("A5", "=CUMIPMT(1E-300,83,156759,5,17,0)", "0"),
        (
            "A6",
            "=CUMIPMT(1E-15,2E16,1,1E16,1E16+8,0)",
            "-8.999591419181679E-15",
        ),
        (
            "A7",
            "=CUMIPMT(0.01,100,100000,100,100,0)",
            "-15.708656559792416",
        ),
    ]);
}

#[test]
fn every_argument_lifts_over_an_array() {
    // probes/fin-argument-modes H73, H75-H79, H81, H83-H87.
    let mut formulas = Vec::new();
    for function in ["CUMIPMT", "CUMPRINC"] {
        for index in 0..6 {
            let mut args = ["0.01", "360", "100000", "1", "12", "0"].map(String::from);
            args[index] = format!("{{{0};{0}}}", args[index]);
            formulas.push(format!("=ROWS({function}({}))", args.join(",")));
        }
    }
    let names: Vec<String> = (1..=formulas.len()).map(|row| format!("A{row}")).collect();
    let cells: Vec<(&str, &str, &str)> = names
        .iter()
        .zip(&formulas)
        .map(|(name, formula)| (name.as_str(), formula.as_str(), "2"))
        .collect();
    check(&cells);
}

#[test]
fn each_element_is_the_scalar_call() {
    // fin-remaining-later-argument-ranges K11; fin-cumipmt-period-bound A29, A30.
    check(&[
        (
            "A1",
            "=INDEX(CUMIPMT({0.01;0.02},360,100000,1,12,0),2)=CUMIPMT(0.02,360,100000,1,12,0)",
            "TRUE",
        ),
        (
            "A2",
            "=INDEX(CUMIPMT(1E-9,{1E9;1E8},1,1,1E8,0),2)",
            "-0.0508331949770886",
        ),
        (
            "A3",
            "=INDEX(CUMPRINC(0.005,100000,1000,{2;3},99999,1),2)",
            "-990.0745031063589",
        ),
        // An element error stays in its position: type 2 is #NUM!, a boolean #VALUE!.
        (
            "A4",
            "=INDEX(CUMIPMT(0.01,360,100000,1,12,{0;2}),2)",
            "#NUM!",
        ),
        (
            "A5",
            "=INDEX(CUMPRINC(0.01,360,100000,{1;TRUE},12,0),2)",
            "#VALUE!",
        ),
        (
            "A6",
            "=INDEX(CUMPRINC(0.01,360,100000,{1;TRUE},12,0),1)=CUMPRINC(0.01,360,100000,1,12,0)",
            "TRUE",
        ),
    ]);
}

#[test]
fn a_range_is_refused_in_every_argument() {
    // probes/fin-argument-modes H74, H82; fin-remaining-later-argument-ranges H65-H84 (on the
    // range's own rows here: G1:G4 beside rows 1-4), each with its intersected control.
    check(&[
        ("A1", "=ROWS(CUMIPMT(G1:G2,360,100000,1,12,0))", "#VALUE!"),
        ("A2", "=ROWS(CUMPRINC(G1:G2,360,100000,1,12,0))", "#VALUE!"),
        ("A3", "=ROWS(CUMIPMT(0.01,G3:G4,100000,1,12,0))", "#VALUE!"),
        ("A4", "=ROWS(CUMPRINC(0.01,G3:G4,100000,1,12,0))", "#VALUE!"),
        (
            "B1",
            "=CUMIPMT(@G1:G2,360,100000,1,12,0)=CUMIPMT(0.01,360,100000,1,12,0)",
            "TRUE",
        ),
        (
            "B3",
            "=CUMPRINC(0.01,@G3:G4,100000,1,12,0)=CUMPRINC(0.01,360,100000,1,12,0)",
            "TRUE",
        ),
    ]);
}

#[test]
fn a_lifted_call_is_charged_for_its_period_sum() {
    // SPEC-0011 "Security rule for kernel work": each element is charged `CUMULATIVE_WEIGHT`, 2^8
    // (at most 32 PPMT steps and the closed form, about 2.9 us in release), so a lifted call
    // runs at most 2^27 / 2^8 = 524,288 elements, whatever their period counts, and the next
    // element makes it #ERROR! before any runs (financial.md "CUMIPMT and CUMPRINC work cap", a
    // DEVIATION).
    check(&[
        (
            "A1",
            "=ROWS(CUMPRINC(SEQUENCE(524288)*0+0.004,1024,1000,1,1,1))",
            "524288",
        ),
        (
            "A2",
            "=ROWS(CUMPRINC(SEQUENCE(524289)*0+0.004,1024,1000,1,32,1))",
            "#ERROR!",
        ),
        (
            "A3",
            "=ROWS(CUMIPMT(SEQUENCE(1024,513)*0+1E-9,1E9,1,1,1E9,0))",
            "#ERROR!",
        ),
    ]);
}

#[test]
fn periods_past_two_to_the_53_take_the_closed_form_and_end() {
    // Past 2^53 `period + 1.0` is `period`, so a sum driven by it never ended (security review of
    // T-1845): a few periods at 1E17, rate 1E-15 so `(1+r)^nper` stays finite, now take the closed
    // form; NaN arguments fail the checks. Each finishes; its value is not asserted (unprobed).
    let mut model = crate::test::util::new_empty_model();
    model._set("A1", "=CUMIPMT(1E-15,2E17,1000,1E17,1E17,0)");
    model._set("A2", "=CUMPRINC(1E-15,2E17,1000,1E17,1E17+8,1)");
    model._set("A3", "=CUMIPMT(1E-15,2E16,1,1E16,1E16,0)");
    model._set("A4", "=CUMIPMT(1E-15,1E16,1,1E16,1E16,0)");
    model._set("A5", "=CUMPRINC(1E-15,1E16,1,1E16-3,1E16,0)");
    model._set("A6", "=CUMIPMT(1E-15,2E16,1,1E16,1E16+8,0)");
    model.evaluate();
    for cell in ["A4", "A5", "A6"] {
        let text = model._get_text(cell);
        assert!(!text.is_empty(), "{cell} {text}");
    }
    let a1 = model._get_text("A1");
    assert!(!a1.is_empty(), "A1 {a1}");
    let a2 = model._get_text("A2");
    assert!(!a2.is_empty(), "A2 {a2}");
    let a3 = model._get_text("A3");
    assert!(!a3.is_empty(), "A3 {a3}");
}
