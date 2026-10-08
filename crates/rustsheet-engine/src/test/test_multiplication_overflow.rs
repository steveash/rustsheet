#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

/// probes-numeric/4f413cc257ce15c4 (Calc!A121:A124, excel-graph): a product beyond the double
/// range is `#NUM!` at once, so it stays an error through `&`, CONCAT, TEXT and LEN.
#[test]
fn overflowing_product_is_num_inside_formulas() {
    let mut model = new_empty_model();
    model._set("A1", "=CONCAT(1e307*100)");
    model._set("A2", "=(1e307*100)&\"\"");
    model._set("A3", "=TEXT(1e307*100,\"0.000000000000000000\")");
    model._set("A4", "=LEN((1e307*100)&\"\")");
    model._set("A5", "=10^308*10");
    model._set("A6", "=1E+307*10-10^308");
    model._set("A7", "=-1e307*100");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "A5", "A7"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
    assert_eq!(model._get_text("A6"), *"0");
}

/// The array path shares the closure: only the overflowing element is an error.
#[test]
fn overflow_in_an_array_product_is_per_element() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({1e307,1}*100,1,1)");
    model._set("B1", "=INDEX({1e307,1}*100,1,2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("B1"), *"100");
}

/// `+`, `-` and `/` overflow like `*`; a subnormal divisor is zero
/// (probes/arith-overflow, excel-graph; docs/excel-semantics/numbers.md).
#[test]
fn overflowing_sum_and_quotient_are_num_and_a_subnormal_divisor_is_zero() {
    let mut model = new_empty_model();
    let cases = [
        ("A1", "=(9E+307+9E+307)&\"\"", "#NUM!"),
        ("A2", "=(9E+307-(-9E+307))&\"\"", "#NUM!"),
        ("A3", "=(-9E+307-9E+307)&\"\"", "#NUM!"),
        ("A4", "=(9E+307+9E+307-9E+307)&\"\"", "#NUM!"),
        ("A5", "=(9E+307/0.1)&\"\"", "#NUM!"),
        ("A6", "=(-9E+307/0.1)&\"\"", "#NUM!"),
        ("A7", "=1/1E-308", "#DIV/0!"),
        ("A8", "=1/(2^-1023)", "#DIV/0!"),
        ("A9", "=1/(2^-1022)>1E+307", "TRUE"),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, _, expected) in cases {
        assert_eq!(model._get_text(cell), *expected, "{cell}");
    }
}

/// A result below the smallest normal double is 0 (probes/precision-and-rounding A14).
#[test]
fn subnormal_results_become_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=2^-1074");
    model._set("A2", "=1E-307/10");
    model._set("A3", "=1E-160*1E-160");
    model._set("A4", "=2^-1022*2^1022");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"0");
    // The smallest normal double itself is kept.
    assert_eq!(model._get_text("A4"), *"1");
}

/// SUM beyond the double range is `#NUM!` (ERROR.TYPE 6), not infinity
/// (probes/arith-overflow Calc!A4:D4, excel-graph).
#[test]
fn overflowing_sum_function_is_num() {
    let mut model = new_empty_model();
    model._set("A1", "=(SUM(9E+307,9E+307))&\"\"");
    model._set("A2", "=ISERROR(SUM(9E+307,9E+307))");
    model._set("A3", "=IFERROR(ERROR.TYPE(SUM(9E+307,9E+307)),0)");
    model._set("A4", "=SUM(9E+307,1E+307)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"6");
    assert_eq!(model._get_text("A4"), *"1E+308");
}

/// PRODUCT, SUMPRODUCT, SUMSQ and AVERAGE flush a subnormal result to a true 0, while SUM keeps
/// it (probes/subnormal-function-results A9, A10, A41, A63, A64, A66:A68, T-0189). The results
/// print as "0" either way, so they are scaled back up: a kept subnormal would show as 1E+..
#[test]
fn multiplying_functions_flush_subnormal_results() {
    let mut model = new_empty_model();
    let cases = [
        ("A1", "=PRODUCT(1E-200,1E-110)*1E+300*1E+300", "0"),
        ("A2", "=SUMPRODUCT({1E-200},{1E-110})*1E+300*1E+300", "0"),
        ("A3", "=PRODUCT(3E-308,0.5)*1E+300*1E+300", "0"),
        ("A4", "=SUMSQ(1E-160)*1E+300*1E+300", "0"),
        ("A5", "=AVERAGE(3E-308,-2.9E-308)*1E+300*1E+300", "0"),
        // Addition keeps the subnormal: 3E-308 - 2.5E-308 = 5E-309.
        ("A6", "=SUM(3E-308,-2.5E-308)*1E+300*1E+300", "5E+291"),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, _, expected) in cases {
        assert_eq!(model._get_text(cell), *expected, "{cell}");
    }
}

/// PRODUCT and SUMPRODUCT flush each running product, not only the total, and the other averaging
/// functions flush like AVERAGE (probes/subnormal-flush-edges, T-1007). Scaled back up as above.
#[test]
fn subnormal_flush_per_step_and_in_every_average() {
    let mut model = new_empty_model();
    model._set("F1", "=3E-308");
    model._set("F2", "=-2.9E-308");
    let cases = [
        ("A1", "=PRODUCT(1E-200,1E-110,1E+300)", "0"),
        ("A2", "=SUMPRODUCT({1E-200,1},{1E-110,1E-300})*1E+300", "1"),
        ("A3", "=AVERAGEA(F1:F2)*1E+300*1E+300", "0"),
        ("A4", "=AVERAGEIF(F1:F2,\">-1\")*1E+300*1E+300", "0"),
        ("A5", "=AVERAGEIFS(F1:F2,F1:F2,\">-1\")*1E+300*1E+300", "0"),
        ("A6", "=SUBTOTAL(1,F1:F2)*1E+300*1E+300", "0"),
        ("A7", "=SUBTOTAL(101,F1:F2)*1E+300*1E+300", "0"),
        ("A8", "=PRODUCT(1E-308,1E+300)", "0"),
        ("A9", "=AGGREGATE(1,6,F1:F2)*1E+300*1E+300", "0"),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, _, expected) in cases {
        assert_eq!(model._get_text(cell), *expected, "{cell}");
    }
}

/// SUMSQ flushes each square, not only the total, and SUMPRODUCT flushes left to right across three
/// arrays (probes/subnormal-sumsq-sumproduct-edges, T-1057).
#[test]
fn sumsq_flushes_each_square_and_sumproduct_each_factor() {
    let mut model = new_empty_model();
    let cases = [
        ("A1", "=SUMSQ(1E-155,1E-150)*1E+300", "1"),
        ("A2", "=SUMSQ(1E-155*ROW(1:200))*1E+300*1E+6", "268.5685"),
        ("A3", "=SUMPRODUCT({1E-200},{1E-110},{1E+300})", "0"),
        ("A4", "=SUMPRODUCT({1E-200},{1E+300},{1E-110})*1E+10", "1"),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, _, expected) in cases {
        assert_eq!(model._get_text(cell), *expected, "{cell}");
    }
}

/// SUMPRODUCT, SUMSQ, PRODUCT, AVERAGE and SUBTOTAL 1, 6, 9 overflow to `#NUM!` (ERROR.TYPE 6),
/// also with a later 0 or 1E-200 factor (probes/aggregate-overflow B11:B26, T-0990).
#[test]
fn overflowing_aggregates_are_num() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("F1", "9E+307"),
        ("F2", "9E+307"),
        ("F3", "1E+200"),
        ("F4", "1E+200"),
    ] {
        model._set(cell, value);
    }
    for (cell, formula) in [
        ("A1", "=SUMPRODUCT({1E+200},{1E+200})"),
        ("A2", "=SUMSQ(1E+200)"),
        ("A3", "=PRODUCT(1E+200,1E+200)"),
        ("A4", "=PRODUCT(1E+200,1E+200,0)"),
        ("A5", "=PRODUCT(1E+200,1E+200,1E-200)"),
        ("A6", "=AVERAGE(9E+307,9E+307)"),
        ("A7", "=AVERAGE(9E+307,9E+307,-9E+307)"),
        ("A8", "=SUBTOTAL(9,F1:F2)"),
        ("A9", "=SUBTOTAL(1,F1:F2)"),
        ("A10", "=SUBTOTAL(6,F3:F4)"),
    ] {
        model._set(cell, &format!("=IFERROR(ERROR.TYPE({}),0)", &formula[1..]));
    }
    model.evaluate();
    for row in 1..=10 {
        assert_eq!(model._get_text(&format!("A{row}")), *"6", "A{row}");
    }
}

/// SUMIF, SUMIFS, SUMX2PY2, SUMXMY2, DEVSQ, STDEV and VAR overflow to `#NUM!` (ERROR.TYPE 6)
/// (probes/aggregate-overflow B37:B38, B42:B46, T-0991), also per criterion in the spilling form.
#[test]
fn overflowing_conditional_and_dispersion_aggregates_are_num() {
    let mut model = new_empty_model();
    for (cell, value) in [("F1", "9E+307"), ("F2", "9E+307"), ("F3", "-9E+307")] {
        model._set(cell, value);
    }
    model._set("F5", "1E+200");
    model._set("F6", "1E+200");
    for (cell, formula) in [
        ("A1", "SUMIF(F1:F3,\">0\")"),
        ("A2", "SUMIFS(F1:F3,F1:F3,\">0\")"),
        ("A3", "SUMX2PY2({1E+154},{1E+154})"),
        ("A4", "SUMXMY2({9E+307},{-9E+307})"),
        ("A5", "DEVSQ(F1:F2)"),
        ("A6", "STDEV(F1:F2)"),
        ("A7", "VAR(F5:F6)"),
        ("A8", "INDEX(SUMIF(F1:F3,{\">0\",\"<0\"}),1,1)"),
    ] {
        model._set(cell, &format!("=IFERROR(ERROR.TYPE({formula}),0)"));
    }
    model.evaluate();
    for row in 1..=8 {
        assert_eq!(model._get_text(&format!("A{row}")), *"6", "A{row}");
    }
}

/// A tiny nonzero value is not equal to 0 (`EXP(-708)=0` is FALSE, probes/subnormal-function-results
/// C5), but a literal below the smallest normal reads as 0 (`1E-308=0` is TRUE, A70).
#[test]
fn tiny_values_compare_unequal_to_zero_but_subnormal_literals_are_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=EXP(-708)=0");
    model._set("A2", "=EXP(-708)<>0");
    model._set("A3", "=1E-308=0");
    model._set("A4", "=1E-300=0");
    model._set("A5", "=1E-20>0");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"FALSE");
    assert_eq!(model._get_text("A5"), *"TRUE");
}

/// probes/subnormal-function-results C69 and text-number-words H6 (excel-graph; T-1027):
/// `+` and `-` keep a subnormal result, while subnormal number text reads as 0.
#[test]
fn sums_keep_subnormals_and_number_text_flushes_them() {
    let mut model = new_empty_model();
    model._set("A1", "=(3E-308-2.5E-308)=0");
    model._set("A2", "=(3E-308+-2.5E-308)=0");
    model._set("A4", "=(\"1e-308\"+0)=0");
    model._set("A5", "=(3E-308*0.5)=0");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
    assert_eq!(model._get_text("A2"), *"FALSE");
    assert_eq!(model._get_text("A4"), *"TRUE");
    assert_eq!(model._get_text("A5"), *"TRUE");
}
