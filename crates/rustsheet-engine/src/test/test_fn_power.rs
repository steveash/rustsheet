#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn numbers() {
    let mut model = new_empty_model();
    model._set("A1", "=POWER(2, 3)");
    model._set("A2", "=POWER(4, 0.5)");
    model._set("A3", "=POWER(5, 0)");
    model._set("A4", "=POWER(2, -1)");
    model._set("A5", "=POWER(-2, 3)");
    model._set("A6", "=POWER(\"3\", TRUE)");
    model._set("A7", "=POWER(B7, 2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"8");
    assert_eq!(model._get_text("A2"), *"2");
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A4"), *"0.5");
    assert_eq!(model._get_text("A5"), *"-8");
    assert_eq!(model._get_text("A6"), *"3");
    assert_eq!(model._get_text("A7"), *"0");
}

#[test]
fn errors() {
    let mut model = new_empty_model();
    model._set("A1", "=POWER(0, 0)");
    model._set("A2", "=POWER(0, -1)");
    model._set("A3", "=POWER(-8, 1/2)");
    model._set("A4", "=POWER(\"a\", 2)");
    model._set("A5", "=POWER(2)");
    model._set("A6", "=POWER(2, 3, 4)");
    model._set("A7", "=POWER(1/0, 2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#DIV/0!");
}

#[test]
fn odd_roots_of_negative_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "=POWER(-8, 1/3)");
    model._set("A2", "=POWER(-27, 1/3)");
    model._set("A3", "=POWER(-32, 1/5)");
    model._set("A4", "=POWER(-32, 0.2)");
    model._set("A5", "=POWER(-1, 1/7)");
    model._set("A6", "=POWER(-8, -1/3)");
    // One over 49 is not exactly 49 when turned over again
    model._set("A7", "=POWER(-2, 1/49)^49");
    model._set("A8", "=POWER({-8, -27, 8}, 1/3)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"-2");
    assert_eq!(model._get_text("A2"), *"-3");
    assert_eq!(model._get_text("A3"), *"-2");
    assert_eq!(model._get_text("A4"), *"-2");
    assert_eq!(model._get_text("A5"), *"-1");
    assert_eq!(model._get_text("A6"), *"-0.5");
    assert_eq!(model._get_text("A7"), *"-2");
    assert_eq!(model._get_text("A8"), *"-2");
    assert_eq!(model._get_text("B8"), *"-3");
    assert_eq!(model._get_text("C8"), *"2");
}

#[test]
fn other_roots_of_negative_numbers() {
    let mut model = new_empty_model();
    // Even roots, and powers that are not one over an integer
    model._set("A1", "=POWER(-8, 1/2)");
    model._set("A2", "=POWER(-8, 1/4)");
    model._set("A3", "=POWER(-8, 2/3)");
    model._set("A4", "=POWER(-8, 0.3333)");
    model._set("A5", "=POWER(-8, 1.5)");
    model._set("A6", "=POWER(-8, -1/2)");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "A5", "A6"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
}

#[test]
fn integer_powers_of_negative_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "=POWER(-8, 1)");
    model._set("A2", "=POWER(-8, 2)");
    model._set("A3", "=POWER(-8, 3)");
    model._set("A4", "=POWER(-8, -1)");
    model._set("A5", "=POWER(-2, -2)");
    model._set("A6", "=POWER(-8, 0)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"-8");
    assert_eq!(model._get_text("A2"), *"64");
    assert_eq!(model._get_text("A3"), *"-512");
    assert_eq!(model._get_text("A4"), *"-0.125");
    assert_eq!(model._get_text("A5"), *"0.25");
    assert_eq!(model._get_text("A6"), *"1");
}

#[test]
fn range_as_number() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A3", "=POWER(A1:C1, 3)");
    model.evaluate();

    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("B3"), *"8");
    assert_eq!(model._get_text("C3"), *"27");
    assert_eq!(model._get_text("D3"), *"");
}

#[test]
fn range_as_power() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("C1", "=POWER(2, A1:A3)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"2");
    assert_eq!(model._get_text("C2"), *"4");
    assert_eq!(model._get_text("C3"), *"8");
    assert_eq!(model._get_text("C4"), *"");
}

#[test]
fn both_arguments_are_ranges() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "2");
    model._set("A3", "3");
    // Element by element
    model._set("E1", "=POWER(A1:C1, A1:C1)");
    // A row and a column make a table
    model._set("E3", "=POWER(A1:C1, A2:A3)");
    model.evaluate();

    assert_eq!(model._get_text("E1"), *"1");
    assert_eq!(model._get_text("F1"), *"4");
    assert_eq!(model._get_text("G1"), *"27");

    assert_eq!(model._get_text("E3"), *"1");
    assert_eq!(model._get_text("F3"), *"4");
    assert_eq!(model._get_text("G3"), *"9");
    assert_eq!(model._get_text("E4"), *"1");
    assert_eq!(model._get_text("F4"), *"8");
    assert_eq!(model._get_text("G4"), *"27");
}

#[test]
fn errors_are_per_element() {
    let mut model = new_empty_model();
    model._set("A1", "0");
    model._set("B1", "2");
    model._set("C1", "text");
    model._set("D1", "=1/0");
    model._set("A3", "=POWER(A1:D1, 0)");
    model._set("A4", "=POWER(A1:D1, -1)");
    model.evaluate();

    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("B3"), *"1");
    assert_eq!(model._get_text("C3"), *"#VALUE!");
    assert_eq!(model._get_text("D3"), *"#DIV/0!");

    assert_eq!(model._get_text("A4"), *"#DIV/0!");
    assert_eq!(model._get_text("B4"), *"0.5");
}

#[test]
fn array_inside_other_functions() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A3", "=SUM(POWER(A1:C1, 2))");
    model._set("A4", "=SUM(POWER({1,2,3;4,5,6}, 2))");
    model._set("A5", "=SUM(POWER(SEQUENCE(4), 2))");
    model.evaluate();

    assert_eq!(model._get_text("A3"), *"14");
    assert_eq!(model._get_text("A4"), *"91");
    assert_eq!(model._get_text("A5"), *"30");
}

#[test]
fn implicit_intersection() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("C2", "=POWER(@A1:A3, 3)");
    model.evaluate();

    assert_eq!(model._get_text("C2"), *"8");
    assert_eq!(model._get_text("C3"), *"");
}

#[test]
fn inside_a_lambda() {
    let mut model = new_empty_model();
    model
        .new_defined_name(
            "gelu",
            None,
            "=LAMBDA(x, 0.5*x*(1 + TANH(SQRT(2/PI()) * (x + 0.044715 * POWER(x,3)))))",
        )
        .unwrap();
    model._set("A1", "1");
    model._set("B1", "-1");
    model._set("C1", "0.5");
    model._set("A3", "=gelu(A1:C1)");
    model._set("A4", "=gelu(OFFSET(A1,0,0,1,3))");
    model.evaluate();

    assert_eq!(model._get_text("A3"), *"0.841191991");
    assert_eq!(model._get_text("B3"), *"-0.158808009");
    assert_eq!(model._get_text("C3"), *"0.34571401");
    assert_eq!(model._get_text("A4"), *"0.841191991");
    assert_eq!(model._get_text("B4"), *"-0.158808009");
    assert_eq!(model._get_text("C4"), *"0.34571401");
}

// The operator `^` follows the same rules as the function.

#[test]
fn operator_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "=2^3");
    model._set("A2", "=4^0.5");
    model._set("A3", "=5^0");
    model._set("A4", "=2^(-1)");
    model._set("A5", "=(-8)^3");
    model._set("A6", "=(-2)^(-2)");
    model._set("A7", "=B7^2");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"8");
    assert_eq!(model._get_text("A2"), *"2");
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A4"), *"0.5");
    assert_eq!(model._get_text("A5"), *"-512");
    assert_eq!(model._get_text("A6"), *"0.25");
    assert_eq!(model._get_text("A7"), *"0");
}

#[test]
fn operator_roots_of_negative_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "=(-8)^(1/3)");
    model._set("A2", "=(-32)^0.2");
    model._set("A3", "=(-8)^(-1/3)");
    model._set("A4", "={-8, -27, 8}^(1/3)");
    // Even roots, and powers that are not one over an integer
    model._set("A5", "=(-8)^(1/2)");
    model._set("A6", "=(-8)^(2/3)");
    model._set("A7", "=(-8)^1.5");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"-2");
    assert_eq!(model._get_text("A2"), *"-2");
    assert_eq!(model._get_text("A3"), *"-0.5");
    assert_eq!(model._get_text("A4"), *"-2");
    assert_eq!(model._get_text("B4"), *"-3");
    assert_eq!(model._get_text("C4"), *"2");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
}

#[test]
fn operator_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=0^0");
    model._set("A2", "=0^(-1)");
    model._set("A3", "=\"a\"^2");
    model._set("A4", "=(1/0)^2");
    model._set("B1", "0");
    model._set("C1", "2");
    model._set("A6", "=B1:C1^0");
    model._set("A7", "=B1:C1^(-1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("B6"), *"1");
    assert_eq!(model._get_text("A7"), *"#DIV/0!");
    assert_eq!(model._get_text("B7"), *"0.5");
}

// Oracle: probes-numeric/power-large-integer-exponents*.case.json (excel-graph, T-1311).
#[test]
fn huge_integer_exponents_and_subnormal_results() {
    let mut model = new_empty_model();
    let cases = [
        ("=(-1)^4294967294", "1"),
        ("=(-1)^4294967295", "#NUM!"),
        ("=POWER(-0.5, -4294967296)", "#NUM!"),
        ("=0.1^310", "0"),
        ("=(-0.5)^1073", "0"),
        ("=10^-308", "0"),
        ("=10^33", "1E+33"),
        ("=10^-300", "1E-300"),
        ("=(-10)^33", "-1E+33"),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn powers_of_minus_ten_keep_the_square_and_multiply_digits() {
    // The 15-digit display of `(-10)^33` hides that it is -1.0000000000000001E+33, not -1E+33.
    assert_eq!(
        crate::arithmetic::power(-10.0, 33.0),
        Ok(-1.000_000_000_000_000_1e33)
    );
    assert_eq!(crate::arithmetic::power(10.0, 33.0), Ok(1e33));
}

#[test]
fn reciprocal_square_root_divides_by_the_twice_rounded_root() {
    // probes-numeric/power-neg-half-rounding A1, A9, A10 (excel-graph, T-1541): the root rounds
    // twice, as SQRT does, and so does the division; a subnormal radicand too.
    let x = 1.0 + 314_268_894_768_158.0 * 2f64.powi(-52);
    assert_eq!(
        crate::arithmetic::power(x, -0.5),
        Ok(0.966_835_114_610_424_7)
    );
    let subnormal = f64::from_bits(344_225_268_385_227);
    assert_eq!(
        crate::arithmetic::extended_sqrt(subnormal),
        4.123_952_952_442_882e-155
    );
    assert_eq!(
        crate::arithmetic::power(f64::from_bits(318_463_907_410_473), -0.5),
        Ok(2.521_027_376_716_698e154)
    );
}
