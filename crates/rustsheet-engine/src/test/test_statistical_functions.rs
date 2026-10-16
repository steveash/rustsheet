#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── FREQUENCY ────────────────────────────────────────────────────────────────

#[test]
fn test_frequency_basic() {
    let mut model = new_empty_model();
    // data: 1,2,3,4,5  bins: 2,4
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("A5", "5");
    model._set("B1", "2");
    model._set("B2", "4");
    model._set("C1", "=FREQUENCY(A1:A5, B1:B2)");
    model.evaluate();
    // ≤2: {1,2}=2, (2,4]: {3,4}=2, >4: {5}=1
    assert_eq!(model._get_text("C1"), "2");
    assert_eq!(model._get_text("C2"), "2");
    assert_eq!(model._get_text("C3"), "1");
}

// Evidence: probes/mode-typed-arguments A21-A29 (excel-graph, 2026-10-16, T-1291), which supersedes
// the inference from poi EverythingTests row 936 (T-0947): typed and referenced text is skipped, a
// direct error propagates, and a blank single-cell reference or an empty argument is `#VALUE!`.
#[test]
fn test_mode_text_and_blank_reference_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "text");
    model._set("A4", "2");
    model._set("B1", "=MODE(A1,\"1\",2,3)");
    model._set("B2", "=MODE(1,\"1\",#DIV/0!,2,3)");
    model._set("B3", "=MODE(A1,2,A3)");
    model._set("B4", "=MODE(A1:A4,2)");
    model._set("B5", "=MODE(A1,A2,\"1\",2)");
    model._set("B6", "=MODE(1,1,)");
    model._set("B7", "=MODE(1,2,2,TRUE)");
    model._set("B8", "=MODE(\"2\",\"2\",5)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "#DIV/0!");
    assert_eq!(model._get_text("B3"), "#VALUE!");
    // Text and blanks inside a range are ignored.
    assert_eq!(model._get_text("B4"), "2");
    assert_eq!(model._get_text("B5"), "2");
    assert_eq!(model._get_text("B6"), "#VALUE!");
    // Typed booleans and numeric text are skipped, not read as numbers (A9, A6).
    assert_eq!(model._get_text("B7"), "2");
    assert_eq!(model._get_text("B8"), "#N/A");
}

// ── MODE.SNGL ────────────────────────────────────────────────────────────────

#[test]
fn test_mode_sngl_basic() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("A2", "1");
    model._set("A3", "3");
    model._set("A4", "2");
    model._set("B1", "=MODE.SNGL(A1:A4)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
}

#[test]
fn test_mode_sngl_no_mode_returns_na() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=MODE.SNGL(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#N/A");
}

// ── MODE.MULT ────────────────────────────────────────────────────────────────

#[test]
fn test_mode_mult_two_modes() {
    let mut model = new_empty_model();
    // 1,1,2,2,3 → modes are 1 and 2
    model._set("A1", "1");
    model._set("A2", "1");
    model._set("A3", "2");
    model._set("A4", "2");
    model._set("A5", "3");
    model._set("B1", "=MODE.MULT(A1:A5)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "2");
}

// ── PERCENTILE.INC ───────────────────────────────────────────────────────────

#[test]
fn test_percentile_inc_median() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("A5", "5");
    model._set("B1", "=PERCENTILE.INC(A1:A5, 0.5)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
}

#[test]
fn test_percentile_inc_q1() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("B1", "=PERCENTILE.INC(A1:A4, 0.25)");
    model.evaluate();
    // rank = 0.25 * 3 = 0.75 → 1 + 0.75*(2-1) = 1.75
    let v: f64 = model._get_text("B1").parse().unwrap();
    assert!((v - 1.75).abs() < 1e-10);
}

// ── PERCENTILE.EXC ───────────────────────────────────────────────────────────

#[test]
fn test_percentile_exc_basic() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("B1", "=PERCENTILE.EXC(A1:A4, 0.25)");
    model.evaluate();
    // rank = 0.25*(4+1)-1 = 0.25 → 1 + 0.25*(2-1) = 1.25
    let v: f64 = model._get_text("B1").parse().unwrap();
    assert!((v - 1.25).abs() < 1e-10);
}

// ── PERCENTRANK.INC ──────────────────────────────────────────────────────────

#[test]
fn test_percentrank_inc_exact_match() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("A5", "5");
    // rank of 3 in {1,2,3,4,5}: 2/4 = 0.5
    model._set("B1", "=PERCENTRANK.INC(A1:A5, 3)");
    model.evaluate();
    let v: f64 = model._get_text("B1").parse().unwrap();
    assert!((v - 0.5).abs() < 0.001);
}

#[test]
fn test_percentrank_inc_single_value() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("B1", "=PERCENTRANK.INC(A1, 2)");
    model._set("B2", "=PERCENTRANK(A1, 2)");
    model._set("B3", "=PERCENTRANK.INC(A1, 3)");
    model._set("B4", "=PERCENTRANK.EXC(A1, 2)");
    model._set("B5", "=PERCENTRANK.INC(A1, 2, 0)");
    model._set("B6", "=PERCENTRANK.INC(A1, 2, -400)");
    model.evaluate();
    assert_eq!(model._get_text("B4"), "1");
    assert_eq!(model._get_text("B5"), "1");
    assert_eq!(model._get_text("B6"), "1");
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "1");
    assert_eq!(model._get_text("B3"), "#N/A");
}

// ── PERCENTRANK.EXC ──────────────────────────────────────────────────────────

#[test]
fn test_percentrank_exc_basic() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("A5", "5");
    // rank of 3: position 2 (0-indexed) → (2+1)/(5+1) = 3/6 = 0.5
    model._set("B1", "=PERCENTRANK.EXC(A1:A5, 3)");
    model.evaluate();
    let v: f64 = model._get_text("B1").parse().unwrap();
    assert!((v - 0.5).abs() < 0.001);
}

// ── PERMUT ───────────────────────────────────────────────────────────────────

#[test]
fn test_permut_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=PERMUT(6, 2)");
    model.evaluate();
    // 6*5 = 30
    assert_eq!(model._get_text("A1"), "30");
}

#[test]
fn test_permut_invalid() {
    let mut model = new_empty_model();
    model._set("A1", "=PERMUT(2, 6)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NUM!");
}

// ── PERMUTATIONA ─────────────────────────────────────────────────────────────

#[test]
fn test_permutationa_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=PERMUTATIONA(3, 2)");
    model.evaluate();
    // 3^2 = 9
    assert_eq!(model._get_text("A1"), "9");
}

// ── PROB ─────────────────────────────────────────────────────────────────────

#[test]
fn test_prob_basic() {
    let mut model = new_empty_model();
    // x: 1,2,3,4  prob: 0.1, 0.2, 0.3, 0.4
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("B1", "0.1");
    model._set("B2", "0.2");
    model._set("B3", "0.3");
    model._set("B4", "0.4");
    model._set("C1", "=PROB(A1:A4, B1:B4, 2, 3)");
    model.evaluate();
    // P(2<=x<=3) = 0.2+0.3 = 0.5
    let v: f64 = model._get_text("C1").parse().unwrap();
    assert!((v - 0.5).abs() < 1e-10);
}

// ── QUARTILE.INC ─────────────────────────────────────────────────────────────

#[test]
fn test_quartile_inc_q2_is_median() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("A5", "5");
    model._set("B1", "=QUARTILE.INC(A1:A5, 2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
}

// ── QUARTILE.EXC ─────────────────────────────────────────────────────────────

#[test]
fn test_quartile_exc_q2() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("A5", "5");
    model._set("B1", "=QUARTILE.EXC(A1:A5, 2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
}

// ── TRIMMEAN ─────────────────────────────────────────────────────────────────

#[test]
fn test_trimmean_basic() {
    let mut model = new_empty_model();
    // data: 1,2,3,4,5,6,7,8,9,10  percent=0.2 → trim 1 from each end → mean(2..9)
    for i in 1..=10 {
        model._set(&format!("A{i}"), &i.to_string());
    }
    model._set("B1", "=TRIMMEAN(A1:A10, 0.2)");
    model.evaluate();
    // trim 1 from each end of [1..10]: mean(2,3,4,5,6,7,8,9) = 44/8 = 5.5
    let v: f64 = model._get_text("B1").parse().unwrap();
    assert!((v - 5.5).abs() < 1e-10);
}

// ── LINEST ───────────────────────────────────────────────────────────────────

#[test]
fn test_linest_slope_intercept() {
    let mut model = new_empty_model();
    // y = 2x + 1: data (1,3), (2,5), (3,7)
    model._set("A1", "3");
    model._set("A2", "5");
    model._set("A3", "7");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "=LINEST(A1:A3, B1:B3)");
    model.evaluate();
    // slope = 2, intercept = 1
    let slope: f64 = model._get_text("C1").parse().unwrap();
    let intercept: f64 = model._get_text("D1").parse().unwrap();
    assert!((slope - 2.0).abs() < 1e-10);
    assert!((intercept - 1.0).abs() < 1e-10);
}

#[test]
fn test_linest_stats_true() {
    let mut model = new_empty_model();
    // Perfect line y = 2x + 1 → R² = 1
    model._set("A1", "3");
    model._set("A2", "5");
    model._set("A3", "7");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "=LINEST(A1:A3, B1:B3, TRUE, TRUE)");
    model.evaluate();
    // LINEST stats=TRUE returns 5×2 array at C1:D5; R² is at C3
    let r_sq: f64 = model._get_text("C3").parse().unwrap();
    assert!(
        (r_sq - 1.0).abs() < 1e-10,
        "R² should be 1 for perfect fit, got {r_sq}"
    );
}

// ── TREND ────────────────────────────────────────────────────────────────────

#[test]
fn test_trend_basic() {
    let mut model = new_empty_model();
    // y = 2x + 1
    model._set("A1", "3");
    model._set("A2", "5");
    model._set("A3", "7");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("D1", "4");
    model._set("D2", "5");
    model._set("C1", "=TREND(A1:A3, B1:B3, D1:D2)");
    model.evaluate();
    // TREND(x=4) = 9, TREND(x=5) = 11
    let v1: f64 = model._get_text("C1").parse().unwrap();
    let v2: f64 = model._get_text("C2").parse().unwrap();
    assert!((v1 - 9.0).abs() < 1e-10);
    assert!((v2 - 11.0).abs() < 1e-10);
}

// ── GROWTH ───────────────────────────────────────────────────────────────────

#[test]
fn test_growth_basic() {
    let mut model = new_empty_model();
    // y = 2 * 3^x at x=1,2,3: y = 6, 18, 54
    model._set("A1", "6");
    model._set("A2", "18");
    model._set("A3", "54");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("D1", "4");
    model._set("C1", "=GROWTH(A1:A3, B1:B3, D1)");
    model.evaluate();
    // GROWTH(x=4) ≈ 2 * 3^4 = 162
    let v: f64 = model._get_text("C1").parse().unwrap();
    assert!(
        (v - 162.0).abs() < 0.01,
        "GROWTH(4) should be ~162, got {v}"
    );
}

// ── LOGEST ───────────────────────────────────────────────────────────────────

#[test]
fn test_logest_basic() {
    let mut model = new_empty_model();
    // y = 2 * 3^x: same data as GROWTH test
    model._set("A1", "6");
    model._set("A2", "18");
    model._set("A3", "54");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "=LOGEST(A1:A3, B1:B3)");
    model.evaluate();
    // Returns {m, b} where y = b * m^x; m ≈ 3, b ≈ 2
    let m: f64 = model._get_text("C1").parse().unwrap();
    let b: f64 = model._get_text("D1").parse().unwrap();
    assert!((m - 3.0).abs() < 0.01, "m should be ≈3, got {m}");
    assert!((b - 2.0).abs() < 0.01, "b should be ≈2, got {b}");
}

// Evidence: poi FormulaEvalTestData_Copy.xlsx EverythingTests J888, F912 (T-0949).
#[test]
fn test_maxa_mina_single_cell_references() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "y");
    model._set("A3", "TRUE");
    model._set("A4", "FALSE");
    model._set("A5", "534");
    model._set("B1", "=MAXA(A1,A2)");
    model._set("B2", "=MINA(A3,A5,A5)");
    model._set("B3", "=MAXA(A4,A1)");
    model._set("B4", "=MINA(A5,A4)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "0");
    assert_eq!(model._get_text("B2"), "1");
    assert_eq!(model._get_text("B3"), "0");
    assert_eq!(model._get_text("B4"), "0");
}

// ── RANK with a ref of unknown static shape ─────────────────────────────────

#[test]
fn rank_of_an_unknown_shape_ref_spills_or_ranks() {
    // The formula_eval fuzz reproducer (T-1303): `A1:<undefined name>` has no static shape, so
    // static analysis took the RANK call for a scalar while it returned a 1x3 array (a debug
    // assertion). Excel: probes/rank-unknown-shape-ref.
    let mut model = new_empty_model();
    model._set("C3", "=RANK(5,IF(A1:C1>0,A1:CWWWWWWWWWWWWWWWWWWWWWWWWW1))");
    model._set("A6", "7");
    model._set("B6", "1");
    model._set("C6", "5");
    model._set("F6", "=RANK(5,IF(A6:C6>0,A6:CWWWW6))");
    model._set("F8", "=RANK(5,A6:B6:C6)");
    model._set("F10", "=RANK(5,INDEX(IF(A6:C6>6,NA(),A6:C6),1,0))");
    model.evaluate();

    for cell in ["C3", "D3", "E3"] {
        assert_eq!(model._get_text(cell), *"#VALUE!");
    }
    for cell in ["F6", "G6", "H6"] {
        assert_eq!(model._get_text(cell), *"#NAME?");
    }
    assert_eq!(model._get_text("F8"), *"2");
    assert_eq!(model._get_text("G8"), *"");
    // An error element keeps its error; every other element is #VALUE!
    // (probes/rank-nonref-error-elements).
    assert_eq!(model._get_text("F10"), *"#N/A");
    assert_eq!(model._get_text("G10"), *"#VALUE!");
    assert_eq!(model._get_text("H10"), *"#VALUE!");
}

// ── COUNTBLANK of an error ──────────────────────────────────────────────────

#[test]
fn nested_countblank_of_an_error_reads_each_argument_once() {
    // T-1181: an error argument is the result, and deciding that must not evaluate the argument
    // a second time, or 40 nested calls cost 2^40 evaluations (directive 5).
    let mut model = new_empty_model();
    let mut formula = "NA()".to_string();
    for _ in 0..40 {
        formula = format!("COUNTBLANK(IF(TRUE,{formula}))");
    }
    model._set("A1", &format!("={formula}"));
    model._set("B1", "=NA()");
    model._set("B2", "=COUNTBLANK(CHOOSE(1,B1))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#N/A");
    assert_eq!(model._get_text("B2"), *"0");
}

#[test]
fn rank_of_at_prefixed_non_reference_array_is_one_cell() {
    // The formula_eval fuzz reproducer (T-1596): `@` of the 2x2 IF is its top-left value, so
    // RANK returns one cell instead of a 2x2 array static analysis took for a scalar (a debug
    // assertion). Excel: probes/rank-at-if-ref. In the fuzz model A1:B2 is empty, so every test
    // is FALSE, the undefined name is never picked, and RANK of FALSE is #VALUE!
    // (probes/rank-unknown-shape-ref); with numbers there it is #NAME? (below).
    let mut model = new_empty_model();
    model._set("C3", "=RANK(5,@IF(A1:B2>0,A1:ANK))");
    model.evaluate();
    assert_eq!(model._get_text("C3"), *"#VALUE!");
    assert_eq!(model._get_text("D3"), *"");

    let mut model = new_empty_model();
    for (cell, value) in [("A1", "7"), ("B1", "1"), ("A2", "5"), ("B2", "3")] {
        model._set(cell, value);
    }
    model._set("H1", "=RANK(5,@IF(A1:B2>0,A1:B2))");
    model._set("H4", "=RANK.EQ(5,@IF(A1:B2>0,A1:B2))");
    model._set("H7", "=RANK.AVG(5,@IF(A1:B2>0,A1:B2))");
    model._set("C3", "=RANK(5,@IF(A1:B2>0,A1:ANK))");
    model._set("H19", "=RANK.EQ(5,@IF(A1:B2>0,A1:ANK))");
    model._set("H22", "=RANK.AVG(5,@IF(A1:B2>0,A1:ANK))");
    model._set("H28", "=RANK(5,@IF(TRUE,A1:A30))");
    model._set("H31", "=RANK(5,@IF(A1:B2>0,1/0))");
    model._set("K1", "=SUM(@(A1:B2*1))");
    model._set("K7", "=COUNT(@IF(A1:B2>0,A1:B2))");
    model._set("K25", "=SUM(@{3,4})");
    model.evaluate();

    for cell in ["H1", "H4", "H7"] {
        assert_eq!(model._get_text(cell), *"#VALUE!");
    }
    for cell in ["C3", "H19", "H22"] {
        assert_eq!(model._get_text(cell), *"#NAME?");
    }
    for cell in ["D3", "C4", "I1", "H2", "I4", "I7", "I19", "I22"] {
        assert_eq!(model._get_text(cell), *"");
    }
    // A picked reference is still one: A28 is blank, so 5 is absent.
    assert_eq!(model._get_text("H28"), *"#N/A");
    assert_eq!(model._get_text("H31"), *"#DIV/0!");
    assert_eq!(model._get_text("K1"), *"7");
    assert_eq!(model._get_text("K7"), *"1");
    assert_eq!(model._get_text("K25"), *"3");
}
