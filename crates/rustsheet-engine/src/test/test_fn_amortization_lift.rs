#![allow(clippy::unwrap_used)]

//! AMORLINC and AMORDEGRC on the SPEC-0011 driver (T-1841): their period walks are bounded
//! whatever the period, every argument lifts over an array and refuses a multi-cell range
//! (`LiftArrays`), and a lifted call is charged its bounded walk. Evidence:
//! probes/fin-amor-period-bound, fin-amor-period-bound-b, probes/fin-argument-modes H18-H35 and
//! probes/fin-remaining-later-argument-ranges H11-H34, K4, K5 (docs/excel-semantics/financial.md
//! "AMORLINC and AMORDEGRC over many periods", "Financial function argument modes").

use std::time::{Duration, Instant};

use crate::test::util::check_cells;

/// Evaluates `cells` on a sheet whose G1:G4 hold 2400, 2400, 0.15, 0.15 (a cost, a rate) and
/// Z1:Z7 hold extreme values (1E308 as a product: the literal is past Excel's 9.99E307; Z6 is
/// 2.5E-308, near the smallest normal double, since a subnormal reads as 0).
fn check(cells: &[(&str, &str, &str)]) {
    check_cells(
        &[
            ("G1", "2400"),
            ("G2", "2400"),
            ("G3", "0.15"),
            ("G4", "0.15"),
            ("Z1", "1E-300"),
            ("Z2", "1E15"),
            ("Z3", "1E17"),
            ("Z4", "1E300"),
            ("Z5", "=1E307*10"),
            ("Z6", "=1E-300*2.5E-8"),
            ("Z7", "1E-16"),
        ],
        cells,
    );
}

#[test]
fn a_period_of_1e15_at_a_tiny_rate_takes_milliseconds() {
    // T-1841 acceptance: main walked 10^15 periods for A1 and never finished past 2^53 (A2, A3:
    // `counted += 1.0` stopped moving). Excel does not finish either (an excel-graph fill of A1
    // alone timed out; financial.md "AMORLINC and AMORDEGRC over many periods", a DEVIATION):
    // the charge 2.4E-297 no longer moves what is left, so every period charges it.
    let started = Instant::now();
    check(&[
        ("A1", "=AMORLINC(2400,39679,39813,300,Z2,Z1,1)", "2.4E-297"),
        ("A2", "=AMORLINC(2400,39679,39813,300,Z3,Z1,1)", "2.4E-297"),
        // Past the life, `ceil(1/1E-300)`, a period of 1E300 charges 0, as in main.
        ("A3", "=AMORLINC(2400,39679,39813,300,Z4,Z1,1)", "0"),
        // Each charge rounds to 0.
        ("B1", "=AMORDEGRC(2400,39679,39813,300,Z2,Z1,1)", "0"),
        ("B2", "=AMORDEGRC(2400,39679,39813,300,Z3,Z1,1)", "0"),
        // The charge 2.5 never moves 1E300 and rounds to 3, period after period.
        ("B3", "=AMORDEGRC(Z4,39679,39813,300,Z2,Z1,1)", "3"),
        ("B4", "=AMORDEGRC(Z4,39679,39813,0,Z3,Z1,1)", "3"),
        // A rate of 1E-16: about 2^53 periods of real progress, jumped in exact steps. Below
        // 2,048 the subtraction rounds the charge 2.4E-13 down to 2.27E-13, so the 2,100 last
        // past 2100/2.4E-13 = 8.75E15 periods.
        ("C1", "=AMORLINC(2400,39679,39813,300,1E16,Z7,1)", "0"),
        ("C2", "=AMORLINC(2400,39679,39813,300,9E15,Z7,1)", "2.4E-13"),
    ]);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn many_periods_are_excels_walk_bit_for_bit() {
    // fin-amor-period-bound A1-A13: AMORLINC's clamp period charges the remainder of 874,999
    // subtractions (A6, not the closed form's 0.0015213114757168815), then 0; a period between
    // two whole periods is the lower one, and one below 1 charges a full period (A8, A9).
    check(&[
        ("A1", "=AMORLINC(2400,39679,39813,300,874,0.001,1)", "2.4"),
        (
            "A2",
            "=AMORLINC(2400,39679,39813,300,875,0.001,1)",
            "1.5213114753762325",
        ),
        ("A3", "=AMORLINC(2400,39679,39813,300,876,0.001,1)", "0"),
        (
            "A4",
            "=AMORLINC(2400,39679,39813,300,875.5,0.001,1)",
            "1.5213114753762325",
        ),
        (
            "A5",
            "=AMORLINC(2400,39679,39813,300,874999,0.000001,1)",
            "0.0024",
        ),
        (
            "A6",
            "=AMORLINC(2400,39679,39813,300,875000,0.000001,1)",
            "0.0015212859222804571",
        ),
        (
            "A7",
            "=AMORLINC(2400,39679,39813,300,875001,0.000001,1)",
            "0",
        ),
        ("A8", "=AMORLINC(2400,39679,39813,300,0.5,0.15,1)", "360"),
        ("A9", "=AMORLINC(2400,39679,39813,300,0.5,0.001,1)", "2.4"),
        (
            "A10",
            "=AMORLINC(2400,39679,39813,2000,1,0.15,1)",
            "268.19672131147536",
        ),
        ("A11", "=AMORLINC(2400,39679,39813,2000,2,0.15,1)", "0"),
        // fin-amor-period-bound-b A1-A3: 10^6 to 10^9 periods.
        ("A12", "=AMORLINC(2400,39679,39813,300,1E9,1E-9,1)", "0"),
        (
            "A13",
            "=AMORLINC(Z5,39679,39813,300,1E6,1E-6,1)",
            "6.338839748453841E301",
        ),
        (
            "A14",
            "=AMORLINC(2400,39679,39813,300,875000000,1E-9,1)",
            "0",
        ),
    ]);
}

#[test]
fn degressive_walks_and_their_closed_form_are_excels_values() {
    // fin-amor-period-bound B1-B16, B18: up to 2^20 periods the walk (B6-B14: 10^6 periods,
    // the half period at 999,998 and the rest at 999,999), and fin-amor-period-bound-b B1, 5E8
    // periods, past it in closed form.
    check(&[
        ("B1", "=AMORDEGRC(2400,39679,39813,300,1,0.0001,1)", "1"),
        ("B2", "=AMORDEGRC(2400,39679,39813,300,100,0.0001,1)", "1"),
        ("B3", "=AMORDEGRC(2400,39679,39813,300,1000,0.0001,1)", "0"),
        ("B4", "=AMORDEGRC(2400,39679,39813,300,9998,0.0001,1)", "0"),
        ("B5", "=AMORDEGRC(2400,39679,39813,300,9999,0.0001,1)", "0"),
        (
            "B6",
            "=AMORDEGRC(1E15,39679,39813,1,1,0.000001,1)",
            "2499997712",
        ),
        (
            "B7",
            "=AMORDEGRC(1E15,39679,39813,1,1000,0.000001,1)",
            "2493761750",
        ),
        (
            "B8",
            "=AMORDEGRC(1E15,39679,39813,1,100000,0.000001,1)",
            "1947004435",
        ),
        (
            "B9",
            "=AMORDEGRC(1E15,39679,39813,1,500000,0.000001,1)",
            "716262008",
        ),
        (
            "B10",
            "=AMORDEGRC(1E15,39679,39813,1,999997,0.000001,1)",
            "205213720",
        ),
        (
            "B11",
            "=AMORDEGRC(1E15,39679,39813,1,999998,0.000001,1)",
            "41042641307062",
        ),
        (
            "B12",
            "=AMORDEGRC(1E15,39679,39813,1,999999,0.000001,1)",
            "41042641307062",
        ),
        (
            "B13",
            "=AMORDEGRC(1E15,39679,39813,1,1000000,0.000001,1)",
            "0",
        ),
        (
            "B14",
            "=AMORDEGRC(1E15,39679,39813,0,999999,0.000001,1)",
            "41042641307062",
        ),
        ("B15", "=AMORDEGRC(1E6,39679,39813,1,30000,0.00001,1)", "12"),
        ("B16", "=AMORDEGRC(1E6,39679,39813,1,50000,0.00001,1)", "7"),
        ("B18", "=AMORDEGRC(2400,39679,39813,300,2.5,0.15,1)", "485"),
        ("C1", "=AMORDEGRC(1E15,39679,39813,1,5E8,1E-9,1)", "716262"),
        // fin-remaining-later-argument-ranges K4, K5: past the asset life.
        ("D1", "=AMORLINC(2400,39679,39813,300,1E9,0.15,1)", "0"),
        ("D2", "=AMORDEGRC(2400,39679,39813,300,1E9,0.15,1)", "0"),
    ]);
}

#[test]
fn extreme_values_from_cells_are_values_without_a_panic() {
    // Unprobed. Where main finished (A3-A5, A7-A9) these are main's values: a cost of 1E308
    // overflows AMORDEGRC's rounding to 0 (fin-amor-period-bound-b B2 is 1.94700443465919E302 in
    // Excel; TODO(T-1895)), a rate of 1E300 is a life of one period, a date or basis out of range
    // is #NUM!. Where main walked past 2^53 and never finished (A1, A2, A6), the walk's value.
    check(&[
        // Rate 2.5E-308: a life of 4E307 periods, the charge 6E-305 never moves 2,100.
        (
            "A1",
            "=AMORLINC(2400,39679,39813,300,1E307,Z6,1)",
            "5.999999999999999E-305",
        ),
        ("A2", "=AMORDEGRC(2400,39679,39813,300,1E307,Z6,1)", "0"),
        ("A3", "=AMORDEGRC(Z5,39679,39813,300,1E5,1E-6,1)", "0"),
        ("A4", "=AMORLINC(Z5,39679,39813,300,1,Z4,1)", "0"),
        ("A5", "=AMORDEGRC(Z5,39679,39813,300,Z5,Z1,1)", "0"),
        ("A6", "=AMORLINC(Z5,39679,39813,0,Z3,Z1,1)", "1E8"),
        ("A7", "=AMORLINC(2400,-1E15,39813,300,1,0.15,1)", "#NUM!"),
        (
            "A8",
            "=AMORDEGRC(2400,39679,39813,300,1,0.15,-1E300)",
            "#NUM!",
        ),
        (
            "A9",
            "=AMORLINC(2400,39679,39813,300,1,0.15,1E300)",
            "#NUM!",
        ),
    ]);
}

#[test]
fn every_argument_lifts_over_an_array() {
    // probes/fin-argument-modes H19, H21-H26, H28, H30-H35.
    let mut formulas = Vec::new();
    for function in ["AMORDEGRC", "AMORLINC"] {
        for index in 0..7 {
            let mut args = ["2400", "39679", "39813", "300", "1", "0.15", "1"].map(String::from);
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
    // fin-argument-modes H18, H27 are the scalar values (776, 360).
    check(&[
        (
            "A1",
            "=INDEX(AMORDEGRC(2400,39679,39813,300,{0;1},0.15,1),2)",
            "776",
        ),
        (
            "A2",
            "=INDEX(AMORLINC(2400,39679,39813,300,{0;1},0.15,1),2)",
            "360",
        ),
        (
            "A3",
            "=INDEX(AMORLINC(2400,39679,39813,300,{1,875000},{0.15;0.000001},1),2,2)",
            "0.0015212859222804571",
        ),
        // An element error stays in its position: basis 2 is #NUM!, a boolean basis #VALUE!.
        (
            "A4",
            "=INDEX(AMORDEGRC(2400,39679,39813,300,1,0.15,{1;2}),2)",
            "#NUM!",
        ),
        (
            "A5",
            "=INDEX(AMORLINC(2400,39679,39813,300,1,0.15,{1;TRUE}),2)",
            "#VALUE!",
        ),
        (
            "A6",
            "=INDEX(AMORLINC(2400,39679,39813,300,1,0.15,{1;TRUE}),1)",
            "360",
        ),
    ]);
}

#[test]
fn a_range_is_refused_in_every_argument() {
    // probes/fin-argument-modes H20, H29; fin-remaining-later-argument-ranges H11-H34 (on the
    // range's own rows here: G1:G4 beside rows 1-4), each with its intersected control.
    check(&[
        (
            "A1",
            "=ROWS(AMORDEGRC(G1:G2,39679,39813,300,1,0.15,1))",
            "#VALUE!",
        ),
        (
            "A2",
            "=ROWS(AMORLINC(G1:G2,39679,39813,300,1,0.15,1))",
            "#VALUE!",
        ),
        (
            "A3",
            "=ROWS(AMORDEGRC(2400,39679,39813,300,1,G3:G4,1))",
            "#VALUE!",
        ),
        (
            "A4",
            "=ROWS(AMORLINC(2400,39679,39813,300,1,G3:G4,1))",
            "#VALUE!",
        ),
        ("B1", "=AMORDEGRC(@G1:G2,39679,39813,300,1,0.15,1)", "776"),
        ("B3", "=AMORLINC(2400,39679,39813,300,1,@G3:G4,1)", "360"),
    ]);
}

#[test]
fn a_lifted_call_is_charged_for_its_walk() {
    // SPEC-0011 "Security rule for kernel work": an AMORLINC element is charged 2^9 units (its
    // exact walk, at most a few thousand subtractions, about 4.7 us in release) and an AMORDEGRC
    // element 2^18 (2^20 periods walked, about 4 ms), so a lifted call runs at most
    // 2^27 / 2^9 = 262,144 and 2^27 / 2^18 = 512 elements, whatever their periods, and the next
    // element makes it #ERROR! before any runs (financial.md "AMORLINC and AMORDEGRC work cap",
    // a DEVIATION).
    check(&[
        (
            "A1",
            "=ROWS(AMORLINC(SEQUENCE(262144)*0+2400,39679,39813,300,1,0.15,1))",
            "262144",
        ),
        (
            "A2",
            "=ROWS(AMORLINC(SEQUENCE(262145)*0+2400,39679,39813,300,1,0.15,1))",
            "#ERROR!",
        ),
        (
            "A3",
            "=ROWS(AMORDEGRC(SEQUENCE(512)*0+2400,39679,39813,300,1,0.15,1))",
            "512",
        ),
        (
            "A4",
            "=ROWS(AMORDEGRC(SEQUENCE(513)*0+2400,39679,39813,300,1,0.15,1))",
            "#ERROR!",
        ),
        (
            "A5",
            "=ROWS(AMORDEGRC(SEQUENCE(1,513)*0+1E15,39679,39813,1,5E6,1E-7,1))",
            "#ERROR!",
        ),
    ]);
}
