#![allow(clippy::unwrap_used)]

//! The engineering functions lift their scalar arguments (T-1383, SPEC-0011): every argument
//! over an array, BIT* over a multi-cell range too, and the Analysis `ToolPak` ones refuse a range
//! with `#VALUE!` (probes/eng-argument-modes, probes/scalar-lift-shapes H7-H10,
//! probes/scalar-lift-open-a Y8-Y15, probes/eng-later-argument-lift;
//! docs/excel-semantics/engineering.md "Engineering function argument modes").

use crate::test::util::new_empty_model;

#[test]
fn every_argument_lifts_over_an_array() {
    let mut model = new_empty_model();
    // Every call has two rows: the lifted argument is a two-row array.
    let cases = [
        "BESSELI({2.5;2.5},1)",
        "BESSELJ(2.5,{1;2})",
        "BESSELK(2.5,{1;1})",
        "BESSELY({2.5;2.5},1)",
        "BIN2DEC({101;101})",
        "BIN2HEX(101,{8;4})",
        "DEC2BIN(9,{8;8})",
        "HEX2OCT({\"1F\";\"1F\"},8)",
        "OCT2HEX(17,{8;4})",
        "COMPLEX(3,4,{\"j\";\"i\"})",
        "CONVERT(2.5,\"m\",{\"ft\";\"in\"})",
        "DELTA(5,{5;2})",
        "GESTEP({5;5},4)",
        "ERF(0.5,{1.5;1})",
        "ERFC.PRECISE({0.75;0.75})",
        "IMABS({\"3+4i\";\"1\"})",
        "IMSUB(\"3+4i\",{\"1-2i\";\"1\"})",
        "IMPOWER(\"3+4i\",{2;3})",
        "BITAND(13,{25;2})",
        "BITRSHIFT({13;13},2)",
    ];
    for (row, formula) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), &format!("=ROWS({formula})"));
    }
    model._set("C1", "=INDEX(BIN2HEX(101,{8;4}),2)");
    model._set("C2", "=INDEX(COMPLEX(3,{4;2}),2)");
    model._set("C3", "=INDEX(DELTA(5,{5;2}),2)");
    model._set("C4", "=INDEX(BITOR(13,{25;2}),2)");
    model._set("C5", "=INDEX(IMSUB(\"3+4i\",{\"1-2i\";\"1\"}),2)");
    model._set("C6", "=INDEX(CONVERT(1,{\"m\";\"x\"},\"ft\"),2)");
    model.evaluate();
    for (row, formula) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *"2",
            "ROWS({formula})"
        );
    }
    // Each element is the scalar call (eng-later-argument-lift I4, I16, I17, I12).
    assert_eq!(model._get_text("C1"), *"0005");
    assert_eq!(model._get_text("C2"), *"3+2i");
    assert_eq!(model._get_text("C3"), *"0");
    assert_eq!(model._get_text("C4"), *"15");
    assert_eq!(model._get_text("C5"), *"2+4i");
    // A failing element is an error in its position only.
    assert_eq!(model._get_text("C6"), *"#N/A");
}

#[test]
fn bit_functions_lift_over_a_range_in_both_arguments() {
    let mut model = new_empty_model();
    for row in 1..=3 {
        model._set(&format!("A{row}"), "13");
        model._set(&format!("B{row}"), &format!("{row}"));
    }
    model._set("D1", "=BITAND(A1:A3,25)");
    model._set("E1", "=BITLSHIFT(13,B1:B3)");
    model._set("F1", "=BITXOR(A1:A3,B1:B3)");
    // Off the range's rows too: a lift, not an intersection.
    model._set("D10", "=ROWS(BITOR(13,B1:B3))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"9");
    assert_eq!(model._get_text("D3"), *"9");
    assert_eq!(model._get_text("E1"), *"26");
    assert_eq!(model._get_text("E3"), *"104");
    assert_eq!(model._get_text("F2"), *"15");
    assert_eq!(model._get_text("D10"), *"3");
}

#[test]
fn toolpak_functions_refuse_a_range_on_its_own_rows() {
    let mut model = new_empty_model();
    for row in 1..=3 {
        model._set(&format!("A{row}"), "2.5");
        model._set(&format!("B{row}"), "8");
        model._set(&format!("M{row}"), "3+4i");
    }
    // Each formula sits on the rows of its range, where an intersection would succeed.
    model._set("D1", "=BESSELI(A1:A3,1)");
    model._set("D2", "=BESSELJ(2.5,A1:A3)");
    model._set("D3", "=DEC2HEX(100,B1:B3)");
    model._set("E1", "=IMABS(M1:M3)");
    model._set("E2", "=DELTA(5,B1:B3)");
    model._set("E3", "=ERF(0.5,A1:A3)");
    // A one-cell range is its value; next to an array, the range is `#VALUE!` in every element.
    model._set("F1", "=BESSELJ(A1:A1,1)");
    model._set("F2", "=ROWS(DEC2BIN({9;9},B1:B3))");
    model._set("F3", "=INDEX(DEC2BIN({9;9},B1:B3),2)");
    model.evaluate();
    for cell in ["D1", "D2", "D3", "E1", "E2", "E3", "F3"] {
        assert_eq!(model._get_text(cell), *"#VALUE!", "{cell}");
    }
    assert_eq!(model._get_text("F1"), *"0.497094103");
    assert_eq!(model._get_text("F2"), *"2");
}

#[test]
fn scalar_calls_are_unchanged() {
    let mut model = new_empty_model();
    model._set("A1", "=BITAND(13,25)");
    model._set("A2", "=BESSELJ(2.5,1)");
    model._set("A3", "=DEC2BIN(9,8)");
    model._set("A4", "=COMPLEX(3,4,\"j\")");
    model._set("A5", "=IMDIV(\"3+4i\",\"1-2i\")");
    model._set("A6", "=DELTA(5)");
    model._set("A7", "=IMCOS(\"\")");
    // Excel: empty text is `#NUM!` in every IM* function (probes/eng-gen-sample-edges D5).
    model._set("A8", "=IMABS(\"\")");
    model._set("A9", "=BITLSHIFT(13,-2)");
    model._set("A10", "=CONVERT(1,\"m\",\"ft\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"9");
    assert_eq!(model._get_text("A2"), *"0.497094103");
    assert_eq!(model._get_text("A3"), *"00001001");
    assert_eq!(model._get_text("A4"), *"3+4j");
    assert_eq!(model._get_text("A5"), *"-1+2i");
    assert_eq!(model._get_text("A6"), *"0");
    assert_eq!(model._get_text("A7"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
    assert_eq!(model._get_text("A9"), *"3");
    assert_eq!(model._get_text("A10"), *"3.280839895");
}

/// The Bessel kernels loop once or twice per order: each caps the order at 100,000 (`#NUM!`
/// above, docs/excel-semantics/engineering.md "BESSEL order cap"), and a lifted call is charged
/// for that work by each element's order before its first element runs (SPEC-0011 "Security rule
/// for kernel work", T-1819): small orders lift past 2,048 elements, orders at the cap do not.
#[test]
fn bessel_work_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "=BESSELI(2.5,100001)");
    model._set("A2", "=BESSELK(2.5,100001)");
    model._set("A3", "=BESSELI(2.5,100000)");
    model._set("A4", "=ROWS(BESSELJ(2.5,SEQUENCE(2000)))");
    model._set("A5", "=ROWS(BESSELJ(2.5,SEQUENCE(2100)))");
    // 2,100 elements at the cap cost more than 2,048 at the cap: refused before any runs.
    model._set("A6", "=ROWS(BESSELI(2.5,SEQUENCE(2100,1,100000,0)))");
    // A text order may read as a number, so it is charged as at the cap.
    model._set("A7", "=ROWS(BESSELJ(2.5,IF(SEQUENCE(2100),\"3\")))");
    model._set("A8", "=ROWS(BESSELJ(2.5,IF(SEQUENCE(2000),\"3\")))");
    // Orders past the cap are charged as at it, though each element is a quick `#NUM!`.
    model._set("A9", "=ROWS(BESSELJ(2.5,SEQUENCE(2100,1,1E9,0)))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"0");
    assert_eq!(model._get_text("A4"), *"2000");
    assert_eq!(model._get_text("A5"), *"2100");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"2000");
    assert_eq!(model._get_text("A9"), *"#ERROR!");
}

/// A lift of many order-0 elements runs, each the scalar call's value (T-1819 Goal).
#[test]
fn many_small_bessel_orders_lift() {
    let mut model = new_empty_model();
    model._set("A1", "=ROWS(BESSELJ(SEQUENCE(5000)/100,0))");
    model._set("A2", "=INDEX(BESSELJ(SEQUENCE(5000)/100,0),250)");
    model._set("A3", "=BESSELJ(2.5,0)");
    model._set("A4", "=INDEX(BESSELJ(2.5,SEQUENCE(2100)),3)");
    model._set("A5", "=BESSELJ(2.5,3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"5000");
    assert_eq!(model._get_text("A2"), model._get_text("A3"));
    assert_eq!(model._get_text("A4"), model._get_text("A5"));
}

#[test]
fn a_lifted_argument_over_a_choose_of_values_spills() {
    // The formula_eval fuzz reproducer (T-1874): CHOOSE gives its array arms a range's shape, so
    // static analysis took the `LiftArrays` argument for a refused range and the call for a
    // scalar, while it lifted to a 2x3 array (a debug assertion). Excel:
    // probes/lift-arrays-choose-arms.
    let mut model = new_empty_model();
    model._set("C3", "=IMSECH(CHOOSE({1,2},{1,2,3},{4;5})");
    model.evaluate();
    assert_eq!(model._get_text("C3"), *"0.648054273663885");
    assert_eq!(model._get_text("D4"), *"0.0134752822213046");
    assert_eq!(model._get_text("E3"), *"#N/A");

    let mut model = new_empty_model();
    for (cell, value) in [
        ("H1", "3"),
        ("H2", "4"),
        ("H3", "5"),
        ("I1", "6"),
        ("I2", "8"),
    ] {
        model._set(cell, value);
    }
    // (top-left cell, formula, spill width, the spilled values row by row).
    let cases: [(&str, &str, i32, &[&str]); 9] = [
        (
            "A1",
            "=IMABS(CHOOSE({1,2},{1,2,3},{4;5}))",
            3,
            &["1", "4", "#N/A", "1", "5", "#N/A"],
        ),
        ("A5", "=IMABS(CHOOSE(1,{1,2,3}))", 3, &["1", "2", "3"]),
        ("A9", "=IMABS(CHOOSE(1,H1:H3))", 1, &["#VALUE!", ""]),
        (
            "A13",
            "=IMABS(CHOOSE({1,2},H1:H2,I1:I2))",
            2,
            &["3", "6", "4", "8"],
        ),
        ("A17", "=IMABS(CHOOSE(2,{3;4},H1:H3))", 1, &["#VALUE!", ""]),
        ("A21", "=EDATE(CHOOSE(1,{4;5}),1)", 1, &["35", "36"]),
        ("A25", "=ISEVEN(CHOOSE(1,{4;5}))", 1, &["TRUE", "FALSE"]),
        ("A29", "=IMABS(CHOOSE(1,H1))", 1, &["3", ""]),
        ("A37", "=IMABS((CHOOSE(1,{1,2,3})))", 3, &["1", "2", "3"]),
    ];
    for (cell, formula, _, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, width, expected) in cases {
        let row: i32 = cell[1..].parse().unwrap();
        for (index, value) in (0..).zip(expected.iter()) {
            let column = char::from(b'A' + u8::try_from(index % width).unwrap());
            let at = format!("{column}{}", row + index / width);
            assert_eq!(model._get_text(&at), **value, "{formula} at {at}");
        }
    }
}

#[test]
fn nested_lifted_calls_over_choose_are_analysed_in_linear_time() {
    // Each level used to analyse its CHOOSE index twice, so 60 levels took hours (T-1874 review).
    let mut formula = "{1,2}".to_string();
    for _ in 0..60 {
        formula = format!("IMABS(CHOOSE({formula},{{1,2}}))");
    }
    let mut model = new_empty_model();
    model._set("A1", &format!("={formula}"));
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1");
}
