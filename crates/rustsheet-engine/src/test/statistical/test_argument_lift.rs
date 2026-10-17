#![allow(clippy::unwrap_used)]

//! The statistical functions whose every argument is scalar lift every argument over an array
//! and over a multi-cell range, each element the scalar call (T-1290, SPEC-0011;
//! probes/stat-argument-modes, probes/stat-later-argument-lift, docs/excel-semantics/statistics.md
//! "Statistical function argument modes").

use crate::test::util::new_empty_model;

/// One call per entered function, the lifted argument a two-row array in a different position
/// each time: every one has two rows.
const ARRAY_CALLS: &[&str] = &[
    "BETA.DIST(1.5,2,3,TRUE,{0;1},3)",
    "BETA.DIST(1.5,2,3,TRUE,1,{2;3})",
    "BETADIST(1.5,2,3,1,{2;3})",
    "BINOM.DIST(5,5,0.4,{TRUE;FALSE})",
    "BINOMDIST({5;5},5,0.4,TRUE)",
    "BINOM.DIST.RANGE(5,0.4,1,{3;4})",
    "BINOM.INV(5,{0.4;0.4},0.4)",
    "CRITBINOM(5,0.4,{0.4;0.9})",
    "NEGBINOM.DIST(5,{5;5},0.4,TRUE)",
    "NEGBINOMDIST(2,5,{0.4;0.5})",
    "CONFIDENCE(0.4,2,{15;20})",
    "CONFIDENCE.NORM({0.4;0.4},2,15)",
    "EXPON.DIST(1.5,{0.5;0.5},TRUE)",
    "EXPONDIST(1.5,0.5,{TRUE;FALSE})",
    "F.DIST(1.5,5,{5;5},TRUE)",
    "F.DIST.RT(1.5,{5;5},5)",
    "FDIST({1.5;1.5},5,5)",
    "FISHER({0.5;0.25})",
    "FISHERINV({0.5;0.5})",
    "GAMMA({1.5;2.5})",
    "GAMMALN({1.5;2.5})",
    "GAMMALN.PRECISE({1.5;2.5})",
    "GAUSS({0.5;1})",
    "PHI({1.5;1})",
    "LOGNORM.DIST(1.5,1,2,{TRUE;FALSE})",
    "LOGNORMDIST(1.5,{1;1},2)",
    "LOGNORM.INV(0.4,1,{2;3})",
    "LOGINV({0.4;0.4},1,2)",
    "NORM.DIST(1.5,1,{2;2},TRUE)",
    "NORMDIST({1.5;1.5},1,2,TRUE)",
    "NORM.INV(0.4,{1;1},2)",
    "NORMINV(0.4,1,{2;3})",
    "NORM.S.DIST(0.5,{TRUE;FALSE})",
    "NORMSDIST({0.5;1})",
    "NORM.S.INV({0.4;0.5})",
    "NORMSINV({0.4;0.5})",
    "PERMUT(5,{2;3})",
    "PERMUTATIONA({5;5},2)",
    "STANDARDIZE(1.5,1,{2;4})",
    "T.DIST(1.5,5,{TRUE;FALSE})",
    "T.DIST.2T({1.5;1.5},5)",
    "T.DIST.RT(1.5,{5;10})",
    "TDIST(1.5,5,{1;2})",
    "WEIBULL(1.5,2,3,{TRUE;FALSE})",
    "WEIBULL.DIST({1.5;1.5},2,3,TRUE)",
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
fn later_arguments_lift_over_a_range_on_their_own_rows() {
    let mut model = new_empty_model();
    // G1:G2 = TRUE, FALSE; G3:G4 = 2, 3 (probes/stat-later-argument-lift H, I, J).
    model._set("G1", "TRUE");
    model._set("G2", "FALSE");
    model._set("G3", "2");
    model._set("G4", "3");
    model._set("A1", "=ROWS(NORM.DIST(1.5,1,2,G1:G2))");
    model._set("A2", "=INDEX(NORM.DIST(1.5,1,2,G1:G2),2)");
    model._set("A3", "=INDEX(BETA.DIST(1.5,2,3,TRUE,1,G3:G4),2)");
    model._set("A4", "=INDEX(PERMUT(5,G3:G4),2)");
    model._set("A5", "=INDEX(TDIST(1.5,5,G3:G4),1)");
    // Off the range's rows too: a lift, not an intersection.
    model._set("A10", "=ROWS(LOGNORM.INV(0.4,1,G3:G4))");
    // A one-cell range is its value, also as BETA.DIST's cumulative (L11).
    model._set("A11", "=BETA.DIST(0.5,2,3,G1:G1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"0.193334058");
    assert_eq!(model._get_text("A3"), *"0.26171875");
    assert_eq!(model._get_text("A4"), *"60");
    assert_eq!(model._get_text("A5"), *"0.19390368");
    assert_eq!(model._get_text("A10"), *"2");
    assert_eq!(model._get_text("A11"), *"0.6875");
}

#[test]
fn each_element_is_the_scalar_call_with_its_own_error() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX(PERMUT({5;#N/A},2),2)");
    model._set("A2", "=INDEX(PERMUT({5;5},{2;6}),2)");
    model._set("A3", "=INDEX(NORM.DIST({1;\"x\"},0,1,TRUE),2)");
    model._set("A4", "=INDEX(NORM.DIST({1;\"1\"},0,1,TRUE),2)");
    model._set("A5", "=INDEX(GAMMA({1.5;-1}),2)");
    model._set("A6", "=INDEX(BINOM.INV({5;5},0.4,{0.4;0.9}),2)");
    model._set("A7", "=INDEX(TDIST({-1;1.5},5,1),1)");
    model._set("A8", "=INDEX(TDIST({1.5;1.5},5,{3;2}),1)");
    model._set("A9", "=INDEX(BETA.DIST(0.5,2,3,{\"TRUE\";1}),1)");
    model._set("A10", "=ROWS(NORM.DIST({1;2;3},{0,1},1,TRUE))");
    model._set("A11", "=COLUMNS(NORM.DIST({1;2;3},{0,1},1,TRUE))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#N/A");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"0.841344746");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"3");
    assert_eq!(model._get_text("A7"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
    assert_eq!(model._get_text("A9"), *"0.6875");
    assert_eq!(model._get_text("A10"), *"3");
    assert_eq!(model._get_text("A11"), *"2");
}

#[test]
fn each_kernel_keeps_its_own_error_order() {
    let mut model = new_empty_model();
    // T.DIST.2T reads both arguments before it checks x: a df error wins over x < 0.
    model._set("A1", "=T.DIST.2T(-1,NA())");
    model._set("A2", "=INDEX(T.DIST.2T({-1;1},{#N/A;5}),1)");
    // TDIST checks x before it reads tails and the degrees of freedom.
    model._set("A3", "=TDIST(-1,NA(),1)");
    model._set("A4", "=INDEX(TDIST({-1;1},{#N/A;5},1),1)");
    model._set("A5", "=INDEX(TDIST(1,NA(),{1;2}),2)");
    // Leftmost first for the rest.
    model._set("A6", "=INDEX(NORM.DIST({#DIV/0!;1},NA(),1,TRUE),1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#N/A");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"#N/A");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
}

#[test]
fn permut_stops_once_its_product_is_infinite() {
    // 10^15 factors before the cap; now at most 171. The value is the same infinite product.
    let mut model = new_empty_model();
    model._set("A1", "=PERMUT(1E15,1E15)");
    model._set("A2", "=PERMUT(200,180)");
    model._set("A3", "=PERMUT(20,5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), model._get_text("A2"));
    assert_eq!(model._get_text("A3"), *"1860480");
}

#[test]
fn weighted_kernels_are_charged_their_weight() {
    // PERMUT is charged 171 per element: 2^27 / 171 = 784,890 elements, so a lifted call of
    // 800,000 elements is refused before it runs; the same call of BINOM.INV (64 each) and of
    // PERMUTATIONA (1 each) runs.
    let mut model = new_empty_model();
    model._set("A1", "=ROWS(PERMUT(5,SEQUENCE(800000)))");
    model._set("A2", "=ROWS(PERMUT(5,SEQUENCE(700000)))");
    model._set("A3", "=ROWS(BINOM.INV(5,0.4,SEQUENCE(800000)/1000000))");
    model._set("A4", "=ROWS(PERMUTATIONA(5,SEQUENCE(800000)))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"700000");
    assert_eq!(model._get_text("A3"), *"800000");
    assert_eq!(model._get_text("A4"), *"800000");
}

#[test]
fn unlifted_calls_are_the_scalar_calls() {
    let mut model = new_empty_model();
    model._set("A1", "=NORM.DIST(1.5,1,2,TRUE)");
    model._set("A2", "=BETADIST(1.5,2,3,1,3)");
    model._set("A3", "=NORMSDIST(0.5)");
    model._set("A4", "=NEGBINOMDIST(2,5,0.4)");
    model._set("A5", "=LOGNORMDIST(1.5,1,2)");
    model._set("A6", "=BINOM.DIST.RANGE(5,0.4,1)");
    model._set("A7", "=NORM.DIST(1.5,1,2)");
    model._set("A8", "=STANDARDIZE(TRUE,1,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.598706326");
    assert_eq!(model._get_text("A2"), *"0.26171875");
    assert_eq!(model._get_text("A3"), *"0.691462461");
    assert_eq!(model._get_text("A4"), *"0.055296");
    assert_eq!(model._get_text("A5"), *"0.383131167");
    assert_eq!(model._get_text("A6"), *"0.2592");
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    // TODO(T-1755): Excel reads the TRUE as 1 (-0.5 here); unchanged by the lift.
    assert_eq!(model._get_text("A8"), *"#VALUE!");
}
