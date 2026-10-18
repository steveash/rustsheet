#![allow(clippy::unwrap_used)]

//! QUOTIENT, FACTDOUBLE and SERIESSUM's x, n and m lift over an array and refuse a multi-cell
//! range; SERIESSUM's coefficients and MULTINOMIAL, GCD and LCM are read whole (T-1802,
//! SPEC-0011): probes/math-atp-array-lift, probes/math-atp-array-lift-b,
//! probes/scalar-lift-open-a Y19-Y30 (docs/excel-semantics/numbers.md "QUOTIENT, FACTDOUBLE and
//! SERIESSUM argument modes").

use crate::test::util::new_empty_model;

/// Sets `formulas` in column C from C1 on, evaluates, and checks each one's text.
fn check(setup: &[(&str, &str)], cases: &[(&str, &str)]) {
    let mut model = new_empty_model();
    for (cell, value) in setup {
        model._set(cell, value);
    }
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("C{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("C{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn every_scalar_argument_lifts_over_an_array() {
    check(
        &[("B1", "1"), ("B2", "2"), ("B3", "3")],
        &[
            // probes/math-atp-array-lift C1-C16.
            ("=INDEX(QUOTIENT({7;9},2),2)", "4"),
            ("=ROWS(QUOTIENT({7;9},2))", "2"),
            ("=INDEX(FACTDOUBLE({5;6}),2)", "48"),
            ("=ROWS(FACTDOUBLE({5;6}))", "2"),
            ("=INDEX(SERIESSUM({1;2},0,1,{1,2}),2)", "5"),
            ("=ROWS(SERIESSUM({1;2},0,1,{1,2}))", "2"),
            ("=INDEX(QUOTIENT(10,{3;4}),2)", "2"),
            ("=ROWS(QUOTIENT(10,{3;4}))", "2"),
            ("=INDEX(SERIESSUM(2,{0;1},1,{1,2}),2)", "10"),
            ("=ROWS(SERIESSUM(2,{0;1},1,{1,2}))", "2"),
            ("=INDEX(SERIESSUM(2,0,{1;2},{1,2}),2)", "9"),
            ("=ROWS(SERIESSUM(2,0,{1;2},{1,2}))", "2"),
            // The coefficients are whole beside a lifted x, as an array or a range (C13-C14,
            // C40, C44); x and n make a grid (C15-C16).
            ("=INDEX(SERIESSUM({1;2},0,1,B1:B3),2)", "17"),
            ("=ROWS(SERIESSUM({1;2},0,1,B1:B3))", "2"),
            ("=ROWS(SERIESSUM(2,0,1,{1;2;3}))", "1"),
            ("=INDEX(SERIESSUM({1;2},0,1,B1:B4),2)", "17"),
            ("=COLUMNS(SERIESSUM({1;2},{0,1},1,{1,2}))", "2"),
            ("=INDEX(SERIESSUM({1;2},{0,1},1,{1,2}),2,2)", "10"),
            // Padding past a shorter array (C33).
            ("=INDEX(QUOTIENT({7;9;11},{1;2}),3)", "#N/A"),
        ],
    );
}

#[test]
fn elements_are_their_own_scalar_calls() {
    check(
        &[],
        &[
            // An element error stays in its position (C17-C20).
            ("=INDEX(QUOTIENT({7;9},{0;2}),1)", "#DIV/0!"),
            ("=INDEX(QUOTIENT({7;9},{0;2}),2)", "4"),
            ("=INDEX(FACTDOUBLE({5;-2}),2)", "#NUM!"),
            ("=INDEX(FACTDOUBLE({5;-2}),1)", "15"),
            // A typed boolean element is `#VALUE!`; numeric text is read (C21-C24, -b E8-E10).
            ("=INDEX(QUOTIENT({TRUE;9},2),1)", "#VALUE!"),
            ("=INDEX(FACTDOUBLE({TRUE;5}),1)", "#VALUE!"),
            ("=INDEX(FACTDOUBLE({FALSE;5}),1)", "#VALUE!"),
            ("=INDEX(QUOTIENT({\"8\";9},2),1)", "4"),
            ("=INDEX(FACTDOUBLE({\"5\";5}),1)", "15"),
            ("=INDEX(FACTDOUBLE({\"x\";5}),1)", "#VALUE!"),
            ("=INDEX(SERIESSUM({TRUE;2},0,1,{1,2}),1)", "#VALUE!"),
            // A scalar error beside an array is broadcast (C25-C26).
            ("=INDEX(QUOTIENT(NA(),{1;2}),2)", "#N/A"),
            ("=ROWS(QUOTIENT(NA(),{1;2}))", "2"),
            // The kernel's own order inside one element: the leftmost (C29-C32).
            ("=INDEX(QUOTIENT({#N/A},{#DIV/0!}),1)", "#N/A"),
            ("=INDEX(QUOTIENT({#DIV/0!},{#N/A}),1)", "#DIV/0!"),
            ("=INDEX(SERIESSUM({#N/A;1},0,1,{1,#DIV/0!}),1)", "#N/A"),
            ("=INDEX(SERIESSUM({\"x\";1},#N/A,1,{1,2}),1)", "#VALUE!"),
        ],
    );
}

#[test]
fn a_multi_cell_range_is_refused() {
    let mut setup = Vec::new();
    let mut cells = Vec::new();
    for row in 1..=3 {
        cells.push((format!("A{row}"), format!("{}", 5 + 2 * row)));
        cells.push((format!("B{row}"), "1".to_string()));
        cells.push((format!("D{row}"), "13".to_string()));
    }
    for (cell, value) in &cells {
        setup.push((cell.as_str(), value.as_str()));
    }
    check(
        &setup,
        &[
            // On the ranges' own rows too (probes/scalar-lift-open-a Y19-Y21, Y24-Y26).
            ("=ROWS(QUOTIENT(A1:A3,2))", "#VALUE!"),
            ("=ROWS(QUOTIENT(10,B1:B3))", "#VALUE!"),
            ("=ROWS(FACTDOUBLE(D1:D3))", "#VALUE!"),
            ("=ROWS(SERIESSUM(B1:B3,0,1,{1,2}))", "#VALUE!"),
            ("=ROWS(SERIESSUM(2,B1:B3,1,{1,2}))", "#VALUE!"),
            ("=ROWS(SERIESSUM(2,0,B1:B3,{1,2}))", "#VALUE!"),
            // A range beside an array: the array's shape, every element `#VALUE!` (C27-C28).
            ("=ROWS(QUOTIENT(A1:A3,{1;2}))", "2"),
            ("=INDEX(QUOTIENT(A1:A3,{1;2}),1)", "#VALUE!"),
            // An operation on a range is an array, and lifts (C41-C42); a one-cell range is its
            // value (C43).
            ("=ROWS(QUOTIENT(A1:A3+0,2))", "3"),
            ("=INDEX(FACTDOUBLE(A1:A3+0),3)", "10395"),
            ("=FACTDOUBLE(A2)", "945"),
            // The scalar controls (-a Z19-Z21, Z24-Z27).
            ("=QUOTIENT(45000,2)", "22500"),
            ("=FACTDOUBLE(13)", "135135"),
            ("=SERIESSUM(2,0,1,{1,2})", "5"),
            ("=SERIESSUM(2,0,1,B1:B3)", "7"),
        ],
    );
}

#[test]
fn a_boolean_scalar_is_refused() {
    let mut model = new_empty_model();
    model._set("D1", "TRUE");
    model._set("D2", "'5");
    let cases = [
        // probes/math-atp-array-lift-b E1-E7, E11-E15: FACTDOUBLE read TRUE as 1 before T-1802.
        ("=FACTDOUBLE(TRUE)", "#VALUE!"),
        ("=FACTDOUBLE(FALSE)", "#VALUE!"),
        ("=FACTDOUBLE(D1)", "#VALUE!"),
        ("=FACTDOUBLE(\"5\")", "15"),
        ("=FACTDOUBLE(D2)", "15"),
        ("=FACTDOUBLE(D3)", "1"),
        ("=FACTDOUBLE(\"x\")", "#VALUE!"),
        ("=QUOTIENT(TRUE,1)", "#VALUE!"),
        ("=QUOTIENT(D1,1)", "#VALUE!"),
        ("=QUOTIENT(5,TRUE)", "#VALUE!"),
        ("=SERIESSUM(TRUE,0,1,{1,2})", "#VALUE!"),
        ("=SERIESSUM(D1,0,1,{1,2})", "#VALUE!"),
        ("=INDEX(FACTDOUBLE(D1:D2+0),1)", "1"),
        ("=ROWS(FACTDOUBLE(D1:D2))", "#VALUE!"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("E{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("E{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn multinomial_gcd_and_lcm_give_one_value_over_an_array() {
    check(
        &[],
        &[
            // probes/math-atp-array-lift C34-C39.
            ("=ROWS(GCD({4;6},2))", "1"),
            ("=GCD({4;6},2)", "2"),
            ("=ROWS(LCM({4;6},2))", "1"),
            ("=LCM({4;6},2)", "12"),
            ("=ROWS(MULTINOMIAL({1;2},2))", "1"),
        ],
    );
}

/// Every lifted SERIESSUM element walks the coefficients: the call is charged elements × cells
/// and refused up front past `MAX_LIFTED_CELL_VISITS` (2^27), the existing DEVIATION (arrays.md
/// "Array results above 1,048,576 cells"); below it, it runs.
#[test]
fn a_lifted_seriessum_is_charged_its_coefficients_per_element() {
    let mut model = new_empty_model();
    for row in 1..=3 {
        model._set(&format!("A{row}"), "1");
    }
    // 1,000 elements × 200,000 coefficient cells = 2·10^8 > 2^27.
    model._set("C1", "=ROWS(SERIESSUM(SEQUENCE(1000),0,1,A1:A200000))");
    // 10 × 200,000 = 2·10^6 runs: x = 0.5 gives 1 + 0.5 + 0.25 (the blank cells add 0).
    model._set(
        "C2",
        "=INDEX(SERIESSUM(SEQUENCE(10,1,0.5,0),0,1,A1:A200000),2)",
    );
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"1.75");
}
