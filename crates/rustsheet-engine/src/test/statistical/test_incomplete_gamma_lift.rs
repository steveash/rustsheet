#![allow(clippy::unwrap_used)]

//! GAMMA.DIST, GAMMA.INV, CHISQ.DIST, CHISQ.DIST.RT, CHISQ.INV, CHISQ.INV.RT and POISSON.DIST (and
//! GAMMADIST, GAMMAINV, CHIDIST, CHIINV, POISSON) lift every argument over an array and a
//! multi-cell range, each element the scalar call, now that their incomplete gamma has a per-call
//! cap of its own (T-1872, SPEC-0011; probes/stat-argument-modes, probes/stat-quantile-lift
//! H27-H50, probes/incomplete-gamma-cap; docs/excel-semantics/statistics.md "Incomplete gamma:
//! per-call cap and lifting").

use crate::test::util::new_empty_model;

/// One call per entered function, the lifted argument a two-row array in each position.
const ARRAY_CALLS: &[&str] = &[
    "CHISQ.DIST({1.5;2},5,TRUE)",
    "CHISQ.DIST(1.5,{5;6},TRUE)",
    "CHISQ.DIST(1.5,5,{TRUE;FALSE})",
    "CHISQ.DIST.RT({1.5;2},5)",
    "CHIDIST(1.5,{5;6})",
    "CHISQ.INV({0.4;0.5},5)",
    "CHISQ.INV.RT(0.4,{5;6})",
    "CHIINV({0.4;0.5},5)",
    "GAMMA.DIST({1.5;2},2,3,TRUE)",
    "GAMMA.DIST(1.5,{2;3},3,TRUE)",
    "GAMMA.DIST(1.5,2,{3;4},TRUE)",
    "GAMMADIST(1.5,2,3,{TRUE;FALSE})",
    "GAMMA.INV({0.4;0.5},2,3)",
    "GAMMA.INV(0.4,{2;3},3)",
    "GAMMAINV(0.4,2,{3;4})",
    "POISSON({5;6},1,TRUE)",
    "POISSON.DIST(5,{1;2},TRUE)",
    "POISSON.DIST(5,1,{TRUE;FALSE})",
];

#[test]
fn every_entered_function_lifts_over_an_array() {
    let mut model = new_empty_model();
    for (row, formula) in ARRAY_CALLS.iter().enumerate() {
        model._set(&format!("A{}", row + 1), &format!("=ROWS({formula})"));
    }
    model.evaluate();
    for (row, formula) in ARRAY_CALLS.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *"2",
            "ROWS({formula})"
        );
    }
}

#[test]
fn the_last_argument_lifts_over_a_range_on_its_own_rows() {
    // probes/stat-quantile-lift H27-J50: each element is the scalar call (rounded to 12 places).
    let mut model = new_empty_model();
    for (cell, value) in [
        ("G1", "TRUE"),
        ("G2", "FALSE"),
        ("G3", "5"),
        ("G4", "10"),
        ("G5", "3"),
        ("G6", "4"),
    ] {
        model._set(cell, value);
    }
    model._set("A1", "=ROWS(CHISQ.DIST(1.5,5,G1:G2))");
    model._set("A2", "=ROUND(INDEX(CHISQ.DIST(1.5,5,G1:G2),2),12)");
    model._set("A3", "=ROUND(INDEX(CHIDIST(1.5,G3:G4),2),12)");
    model._set("A4", "=ROUND(INDEX(CHISQ.INV(0.4,G3:G4),2),12)");
    model._set("A5", "=ROUND(INDEX(CHIINV(0.4,G3:G4),2),12)");
    model._set("A6", "=ROUND(INDEX(GAMMADIST(1.5,2,3,G1:G2),2),12)");
    model._set("A7", "=ROUND(INDEX(GAMMA.INV(0.4,2,G5:G6),2),12)");
    model._set("A8", "=ROUND(INDEX(POISSON(5,1,G1:G2),2),12)");
    // Off the range's rows too: a lift, not an intersection.
    model._set("A20", "=ROWS(POISSON.DIST(5,1,G1:G2))");
    // A one-cell range is its value (probes/incomplete-gamma-cap L18).
    model._set("A21", "=ROUND(CHISQ.DIST.RT(1.5,G3:G3),12)");
    model.evaluate();
    let expected = [
        ("A1", "2"),
        ("A2", "0.115399742"),
        ("A3", "0.998935322"),
        ("A4", "8.295471761"),
        ("A5", "10.47323623"),
        ("A6", "0.101088443"),
        ("A7", "5.505685368"),
        ("A8", "0.003065662"),
        ("A20", "2"),
        ("A21", "0.913069815"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn each_element_is_the_scalar_call_with_its_own_error() {
    // probes/incomplete-gamma-cap L1-L17.
    let mut model = new_empty_model();
    model._set("A1", "=INDEX(GAMMA.DIST({1.5;\"x\"},2,3,TRUE),2)");
    model._set(
        "A2",
        "=ROUND(INDEX(GAMMA.DIST({1.5;\"1.5\"},2,3,TRUE),2),12)",
    );
    model._set("A3", "=ROUND(INDEX(CHISQ.DIST({1.5;TRUE},5,TRUE),2),12)");
    model._set("A4", "=INDEX(CHISQ.DIST(1.5,{5;0.5},TRUE),2)");
    model._set("A5", "=INDEX(POISSON({5;-1},1,TRUE),2)");
    model._set("A6", "=INDEX(CHISQ.INV.RT({0.4;#N/A},5),2)");
    // Every argument is read before any is checked: an error wins over a bad probability.
    model._set("A7", "=GAMMA.INV(2,NA(),1)");
    model._set("A8", "=CHISQ.INV(2,NA())");
    model._set("A9", "=POISSON(-1,NA(),TRUE)");
    model._set("A10", "=ROWS(GAMMA.INV({0.1;0.2;0.3},{2,3},1))");
    model._set("A11", "=COLUMNS(GAMMA.INV({0.1;0.2;0.3},{2,3},1))");
    model._set(
        "A12",
        "=ROUND(INDEX(GAMMA.DIST(1.5,2,3,{\"TRUE\";\"FALSE\"}),2),12)",
    );
    model._set("A13", "=INDEX(GAMMA.DIST(1.5,2,3,{TRUE;\"x\"}),2)");
    model._set(
        "A14",
        "=ROUND(INDEX(POISSON({5.9;5},1,{FALSE;FALSE}),1),12)",
    );
    model._set("A15", "=INDEX(CHIDIST({1.5;-1},5),2)");
    model._set("A16", "=INDEX(GAMMAINV({0.4;1},2,3),2)");
    model._set("A17", "=ROUND(INDEX(CHIINV({0.4;1},5),2),12)");
    model.evaluate();
    let expected = [
        ("A1", "#VALUE!"),
        ("A2", "0.09020401"),
        ("A3", "0.037434227"),
        ("A4", "#NUM!"),
        ("A5", "#NUM!"),
        ("A6", "#N/A"),
        ("A7", "#N/A"),
        ("A8", "#N/A"),
        ("A9", "#N/A"),
        ("A10", "3"),
        ("A11", "2"),
        ("A12", "0.101088443"),
        ("A13", "#VALUE!"),
        ("A14", "0.003065662"),
        ("A15", "#NUM!"),
        ("A16", "#NUM!"),
        ("A17", "0"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn incomplete_gammas_that_hung_or_panicked_are_bounded() {
    // statrs' series never finished for a shape of 2^53 or more (E1-E3) or a mean of 1 at a shape
    // of 1E300 (E5), its continued fraction never finished once its terms were NaN (E4), and it
    // panicked where x / beta overflows (E6, which Excel also makes #NUM!) or underflows to 0
    // (E7). Past the cap of 2^19 terms a call is #NUM! too (E8, E10; GAMMA.DIST(1E12,..) took
    // 17 ms on main). Excel gives numbers except at E6: probes/incomplete-gamma-cap, a DEVIATION;
    // TODO(T-1891).
    let mut model = new_empty_model();
    model._set("C1", "1E300");
    model._set("C2", "1.0000001E300");
    model._set("C3", "1E-300");
    model._set("A1", "=GAMMA.DIST(1E16,1E16,1,TRUE)");
    model._set("A2", "=POISSON(1E16,1E16,TRUE)");
    model._set("A3", "=GAMMA.DIST(C1,C1,1,TRUE)");
    model._set("A4", "=GAMMA.DIST(C2,C1,1,TRUE)");
    model._set("A5", "=GAMMA.INV(0.5,1E300,1E-300)");
    model._set("A6", "=GAMMA.DIST(C1,1,1E-10,TRUE)");
    model._set("A7", "=GAMMA.DIST(C3,2,1E100,TRUE)");
    model._set("A8", "=GAMMA.DIST(1E10,1E10,1,TRUE)");
    model._set("A9", "=POISSON(1E12,1E12,TRUE)");
    model._set("A10", "=CHISQ.DIST.RT(C2,2)");
    // The largest chi-squared shape keeps statrs' value (481,117 terms).
    model._set("A11", "=CHISQ.DIST(1E10,1E10,TRUE)");
    model._set("A12", "=INDEX(GAMMA.DIST({1.5;1E16},{2;1E16},1,TRUE),2)");
    model.evaluate();
    let expected = [
        ("A1", "#NUM!"),
        ("A2", "#NUM!"),
        ("A3", "#NUM!"),
        ("A4", "#NUM!"),
        ("A5", "#NUM!"),
        ("A6", "#NUM!"),
        ("A7", "#NUM!"),
        ("A8", "#NUM!"),
        ("A9", "#NUM!"),
        ("A10", "0"),
        ("A11", "0.499998213"),
        ("A12", "#NUM!"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn incomplete_gamma_kernels_are_charged_their_weight() {
    // Each entry is charged 2^18 an element (the bound of one cdf or inversion at its cap, about
    // 2.5 ms): 2^27 / 2^18 = 512 elements, so a lifted call of 513 is refused before it runs.
    let mut model = new_empty_model();
    let calls = [
        "CHISQ.DIST.RT(1.5,SEQUENCE(#))",
        "CHIINV(0.4,SEQUENCE(#))",
        "CHISQ.DIST(1.5,5,SEQUENCE(#))",
        "POISSON(SEQUENCE(#),1,TRUE)",
        "GAMMA.INV(SEQUENCE(#)/(#+1),2,3)",
        "GAMMA.DIST(1.5,2,SEQUENCE(#),TRUE)",
    ];
    for (row, call) in calls.iter().enumerate() {
        let row = row + 1;
        model._set(
            &format!("A{row}"),
            &format!("=ROWS({})", call.replace('#', "512")),
        );
        model._set(
            &format!("B{row}"),
            &format!("=ROWS({})", call.replace('#', "513")),
        );
    }
    model.evaluate();
    for (row, call) in calls.iter().enumerate() {
        let row = row + 1;
        assert_eq!(model._get_text(&format!("A{row}")), *"512", "{call}");
        assert_eq!(model._get_text(&format!("B{row}")), *"#ERROR!", "{call}");
    }
}

#[test]
fn unlifted_calls_are_the_scalar_calls() {
    let mut model = new_empty_model();
    model._set("A1", "=CHISQ.DIST(1.5,5,TRUE)");
    model._set("A2", "=CHISQ.DIST.RT(1.5,5)");
    model._set("A3", "=CHIDIST(1.5,5)");
    model._set("A4", "=CHISQ.INV(0.4,5)");
    model._set("A5", "=CHISQ.INV.RT(0.4,5)");
    model._set("A6", "=GAMMA.DIST(1.5,2,3,TRUE)");
    model._set("A7", "=GAMMAINV(0.4,2,3)");
    model._set("A8", "=POISSON.DIST(5,1,TRUE)");
    model._set("A9", "=POISSON(5,0,FALSE)");
    model._set("A10", "=GAMMA.INV(0.4,2)");
    model._set("A11", "=CHISQ.DIST(1.5,10000000001,TRUE)");
    model._set("A12", "=GAMMAINV(0,2,1)");
    model.evaluate();
    let expected = [
        ("A1", "0.086930185"),
        ("A2", "0.913069815"),
        ("A3", "0.913069815"),
        ("A4", "3.655499623"),
        ("A5", "5.131867074"),
        ("A6", "0.09020401"),
        ("A7", "4.129264026"),
        ("A8", "0.999405815"),
        ("A9", "0"),
        ("A10", "#ERROR!"),
        ("A11", "#NUM!"),
        ("A12", "0"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}
