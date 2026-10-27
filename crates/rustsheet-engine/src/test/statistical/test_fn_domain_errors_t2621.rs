#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel: probes/t2475-stat-subexpressions (excel-graph, 2026-10-24) and the cells cited per row;
// statistics.md "Statistical argument-domain errors and extreme values (T-2621)".
#[test]
fn statistical_domain_errors_and_extreme_values() {
    let cases = [
        // PERMUTATIONA's number_chosen truncates toward zero; COMBINA's only with a number of 0 (P!A28,
        // A35; gen-m5 c688ca172fe533dc A118, gen-m6 44194b2fefa30de2 A222).
        ("=PERMUTATIONA(Z1,-0.0001)", "1"),
        ("=PERMUTATIONA(10,-0.75)", "1"),
        ("=COMBINA(-0.5,-0.0001)", "1"),
        ("=COMBINA(7,1-1.005)", "#NUM!"),
        ("=COMBINA(24,-0.0001)", "#NUM!"),
        // T.TEST: a first sample that is an error value or a blank cell beats a bad tails/type
        // (P!A58, A222; probes/tbill-error-order-inner-calls B1).
        ("=T.TEST(\"x\"+1,42,1,-1)", "#VALUE!"),
        ("=TTEST(\"x\"+1,1/0,2,1/0)", "#VALUE!"),
        ("=T.TEST(Z1,{1,2,3},2,123456789.123)", "#VALUE!"),
        // Student t quantile at p = 0 with df truncating to 1 (P!A124).
        ("=T.INV.2T(FALSE,1)", "#DIV/0!"),
        ("=T.INV(0,1.999)", "#DIV/0!"),
        ("=TINV(0,1)", "#DIV/0!"),
        ("=T.INV(0,2)", "#NUM!"),
        // WEIBULL at x = 0 is 0; an overflowing (x/beta)^alpha is #NUM! (P!A78, A407).
        ("=WEIBULL.DIST(0,1E+8,4.6E-9,TRUE)", "0"),
        ("=WEIBULL(1E+15,1E+15,1,1)", "#NUM!"),
        ("=WEIBULL.DIST(7,1E+8,0.3,TRUE)", "#NUM!"),
        // POISSON's cdf past a u64 count is 1 (P!A216).
        ("=POISSON.DIST(2E+29,7,-1)", "1"),
        // IFERROR's blank fallback is the empty value, a 0 for STDEV.P (P!A158).
        (
            "=STDEV.P(100,2,IFERROR(1/0,Z1),{1,2,3,4,5,6,7,8,9,10})",
            "25.55976868",
        ),
        // ... the same empty value a blank value argument gives (logic.md, T-0291): OR reads it as
        // FALSE where OR(Z1) is #VALUE!, and COUNT counts it.
        ("=OR(IFERROR(1/0,Z1))", "FALSE"),
        ("=COUNT(IFERROR(1/0,Z1))", "1"),
        // TRIMMEAN of a lone blank cell is #NUM! (P!A353).
        ("=TRIMMEAN(Z1,3.14159)", "#NUM!"),
        ("=TRIMMEAN(Z1,0.5)", "#NUM!"),
        // SKEW with an overflowing sum of squares is #DIV/0! (P!A245).
        ("=SKEW({1E+200,1E+200,-1E+200},2.5)", "#DIV/0!"),
        // NORMDIST's density where z² overflows is #NUM! (P!A572).
        ("=NORMDIST(1E+300,0.3,2.5,0)", "#NUM!"),
        ("=NORMDIST(1E+300,0.3,2.5,1)", "1"),
        // BETA.INV whose quantile underflows to 0 is #NUM! (P!A127).
        ("=BETA.INV(0.025,1E-15,2.5)", "#NUM!"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}
