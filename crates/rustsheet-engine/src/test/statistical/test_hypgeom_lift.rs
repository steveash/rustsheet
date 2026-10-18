//! HYPGEOM.DIST and HYPGEOMDIST lift every argument over an array and a multi-cell range, each
//! element the scalar call, now that HYPGEOM.DIST's cdf sum has a per-call cap of its own (T-1873,
//! SPEC-0011; probes/stat-argument-modes H330-H342, probes/stat-quantile-lift H51-H54;
//! docs/excel-semantics/statistics.md "HYPGEOM.DIST: per-call cap and lifting").

use crate::test::util::new_empty_model;

/// One call per function, the lifted argument a two-row array in each position.
const ARRAY_CALLS: &[&str] = &[
    "HYPGEOM.DIST({2;2},3,5,10,TRUE)",
    "HYPGEOM.DIST(2,{3;3},5,10,TRUE)",
    "HYPGEOM.DIST(2,3,{5;5},10,TRUE)",
    "HYPGEOM.DIST(2,3,5,{10;10},TRUE)",
    "HYPGEOM.DIST(2,3,5,10,{TRUE;TRUE})",
    "HYPGEOMDIST({2;2},3,5,10)",
    "HYPGEOMDIST(2,{3;3},5,10)",
    "HYPGEOMDIST(2,3,{5;5},10)",
    "HYPGEOMDIST(2,3,5,{10;10})",
];

#[test]
fn every_argument_lifts_over_an_array() {
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
fn ranges_lift_on_their_own_rows() {
    // probes/stat-argument-modes H332, H339 (the first argument over a range) and
    // probes/stat-quantile-lift H51-J54 (the last one; each element the scalar call, rounded to
    // 12 places).
    let mut model = new_empty_model();
    for (cell, value) in [
        ("G1", "0.4"),
        ("G2", "1"),
        ("G51", "TRUE"),
        ("G52", "FALSE"),
        ("G53", "10"),
        ("G54", "12"),
    ] {
        model._set(cell, value);
    }
    model._set("A1", "=ROWS(HYPGEOM.DIST(G1:G2,3,5,10,TRUE))");
    model._set("A2", "=ROWS(HYPGEOMDIST(G1:G2,3,5,10))");
    model._set("A3", "=ROWS(HYPGEOM.DIST(2,3,5,10,G51:G52))");
    model._set("A4", "=ROUND(INDEX(HYPGEOM.DIST(2,3,5,10,G51:G52),1),12)");
    model._set("A5", "=ROUND(INDEX(HYPGEOM.DIST(2,3,5,10,G51:G52),2),12)");
    model._set("A6", "=ROWS(HYPGEOMDIST(2,3,5,G53:G54))");
    model._set("A7", "=ROUND(INDEX(HYPGEOMDIST(2,3,5,G53:G54),1),12)");
    model._set("A8", "=ROUND(INDEX(HYPGEOMDIST(2,3,5,G53:G54),2),12)");
    // A one-cell range is its value.
    model._set("A9", "=ROUND(HYPGEOMDIST(2,3,5,G53:G53),12)");
    model.evaluate();
    let expected = [
        ("A1", "2"),
        ("A2", "2"),
        ("A3", "2"),
        ("A4", "0.916666667"),
        ("A5", "0.416666667"),
        ("A6", "2"),
        ("A7", "0.416666667"),
        ("A8", "0.318181818"),
        ("A9", "0.416666667"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn each_element_is_the_scalar_call_with_its_own_error() {
    let mut model = new_empty_model();
    // A boolean count is #VALUE!, as in the scalar call; an error element is its error.
    model._set("A1", "=INDEX(HYPGEOMDIST({2;TRUE},3,5,10),2)");
    model._set("A2", "=INDEX(HYPGEOM.DIST({2;#N/A},3,5,10,TRUE),2)");
    // Each element is checked alone: sample_s above number_sample is #NUM!.
    model._set("A3", "=INDEX(HYPGEOM.DIST({2;4},3,5,10,TRUE),2)");
    model._set("A4", "=ROUND(INDEX(HYPGEOM.DIST({2;4},3,5,10,TRUE),1),12)");
    // `cumulative` per element, the text "TRUE" and "FALSE" too.
    model._set(
        "A5",
        "=ROUND(INDEX(HYPGEOM.DIST(2,3,5,10,{\"TRUE\";\"FALSE\"}),2),12)",
    );
    model._set("A6", "=INDEX(HYPGEOM.DIST(2,3,5,10,{TRUE;\"x\"}),2)");
    // Broadcasting is the operators'.
    model._set("A7", "=ROWS(HYPGEOMDIST({1;2;3},3,{5,6},10))");
    model._set("A8", "=COLUMNS(HYPGEOMDIST({1;2;3},3,{5,6},10))");
    model.evaluate();
    let expected = [
        ("A1", "#VALUE!"),
        ("A2", "#N/A"),
        ("A3", "#NUM!"),
        ("A4", "0.916666667"),
        ("A5", "0.416666667"),
        ("A6", "#VALUE!"),
        ("A7", "3"),
        ("A8", "2"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn a_cdf_sum_is_bounded() {
    // `HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)` summed 25,000,001 terms on main
    // (about 4.8 s in release); it sums the 95,942 that are not 0 now, to main's value. A sum
    // with more than 2^18 terms left is #NUM! (the widest distribution at a population of 2^32,
    // and a population above 2^52, which skips nothing): a DEVIATION.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)",
    );
    model._set(
        "A2",
        "=HYPGEOM.DIST(1074741824,2147483648,2147483648,4294967296,TRUE)",
    );
    model._set("A3", "=HYPGEOM.DIST(524288,1048576,2^59,2^60,TRUE)");
    // The pmf is closed form; its binomials overflow at a large population, #NUM! as on main.
    model._set("A4", "=HYPGEOM.DIST(524288,1048576,2^59,2^60,FALSE)");
    model.evaluate();
    let expected = [
        ("A1", "0.500079709"),
        ("A2", "#NUM!"),
        ("A3", "#NUM!"),
        ("A4", "#NUM!"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn hypgeom_dist_is_charged_its_weight() {
    // HYPGEOM.DIST is charged 2^22 an element (the bound of one capped cdf): 2^27 / 2^22 = 32
    // elements, so a lifted call of 33 is refused before it runs. HYPGEOMDIST is `Constant`.
    let mut model = new_empty_model();
    let calls = [
        "HYPGEOM.DIST(SEQUENCE(#)*0+2,3,5,10,TRUE)",
        "HYPGEOM.DIST(2,3,5,10,SEQUENCE(#)>0)",
        "HYPGEOM.DIST(1,4,8,SEQUENCE(#)+19,FALSE)",
    ];
    for (row, call) in calls.iter().enumerate() {
        let row = row + 1;
        model._set(
            &format!("A{row}"),
            &format!("=ROWS({})", call.replace('#', "32")),
        );
        model._set(
            &format!("B{row}"),
            &format!("=ROWS({})", call.replace('#', "33")),
        );
    }
    model._set("C1", "=ROWS(HYPGEOMDIST(1,4,8,SEQUENCE(100000)+19))");
    model.evaluate();
    for (row, call) in calls.iter().enumerate() {
        let row = row + 1;
        assert_eq!(model._get_text(&format!("A{row}")), *"32", "{call}");
        assert_eq!(model._get_text(&format!("B{row}")), *"#ERROR!", "{call}");
    }
    assert_eq!(model._get_text("C1"), *"100000");
}

#[test]
fn unlifted_calls_are_the_scalar_calls() {
    let mut model = new_empty_model();
    model._set("A1", "=HYPGEOM.DIST(2,3,5,10,TRUE)");
    model._set("A2", "=HYPGEOMDIST(2,3,5,10)");
    model._set("A3", "=HYPGEOMDIST(2,3,5)");
    model._set("A4", "=HYPGEOMDIST(2,3,5,10,FALSE)");
    model._set("A5", "=HYPGEOM.DIST(2,3,5,10)");
    model._set("A6", "=HYPGEOM.DIST(2.9,3.9,5.9,10.9,TRUE)");
    model._set("A7", "=HYPGEOM.DIST(0,3,5,10,TRUE)");
    model._set("A8", "=HYPGEOM.DIST(3,3,5,10,TRUE)");
    model._set("A9", "=HYPGEOM.DIST(0,6,5,10,TRUE)");
    model.evaluate();
    let expected = [
        ("A1", "0.916666667"),
        ("A2", "0.416666667"),
        ("A3", "#ERROR!"),
        ("A4", "#ERROR!"),
        ("A5", "#ERROR!"),
        ("A6", "0.916666667"),
        ("A7", "0.083333333"),
        ("A8", "1"),
        ("A9", "#NUM!"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}
