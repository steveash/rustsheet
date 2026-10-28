#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel: gen-m9 (excel-graph, 2026-10-23, T-2256; T-2436 probe for T.DIST.RT), statistics.md
// "Scalar distributions at extreme arguments, part 2" (T-2474).
#[test]
fn scalar_distributions_at_extreme_arguments_part2() {
    let cases = [
        // The cumulative Student t of 1E+300 and 1E+307 is #NUM!, of 1E+15 a number.
        ("=TDIST(1E+300,8,1)", "#NUM!"),
        ("=TDIST(1E+307,9,2)", "#NUM!"),
        ("=T.DIST.2T(1E+300,1)", "#NUM!"),
        ("=T.DIST.2T(1E+307,3)", "#NUM!"),
        ("=T.DIST.RT(1E+300,7)", "#NUM!"),
        ("=T.DIST(1E+307,9,TRUE)", "#NUM!"),
        // 46ac3ef5e61583ca A236: 6.366197723675813E-16; only "a number" is pinned, its last digits
        // are the Student t accuracy tasks' (out of scope).
        ("=ISNUMBER(T.DIST.2T(1E+15,1))", "TRUE"),
        // A negative mean with x truncating to 0 is EXP(-mean); x of 1 or more stays #NUM!.
        ("=POISSON(0,-1,0)", "2.718281828"),
        ("=POISSON(0.3,-0.5,TRUE)", "1.648721271"),
        ("=POISSON.DIST(0,-42,FALSE)", "1.73927E+18"),
        ("=POISSON(1,-1,TRUE)", "#NUM!"),
        // UNVERIFIED (ours, unprobed): EXP(-mean) overflowing is #NUM!.
        ("=POISSON(0,-1000,TRUE)", "#NUM!"),
        // A probability that underflows the result is 0.
        ("=NEGBINOMDIST(7,123456789.123,1E-200)", "0"),
        ("=NEGBINOM.DIST(9,4,1E-300,TRUE)", "0"),
        // A huge number is #NUM!, also with number_chosen 0.
        ("=PERMUTATIONA(1000000000000000,0)", "#NUM!"),
        ("=PERMUTATIONA(FACT(18),2)", "#NUM!"),
        ("=PERMUTATIONA(123456789.123,1)", "123456789"),
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
