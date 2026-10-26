#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel: gen-m9 (excel-graph, 2026-10-22/23, T-2255, T-2256, T-2436), statistics.md "Overflowing
// squares (T-2432)" and "T.TEST: error arguments before tails and type (T-2432)".
#[test]
fn overflowing_squares_and_t_test_error_order() {
    let cases = [
        // Data whose squares overflow: KURT, SKEW and SKEW.P #DIV/0!, Z.TEST and T.TEST #NUM!.
        ("=KURT({1E+200,1E+200,-1E+200},1,2)", "#DIV/0!"),
        ("=KURT(COS(3.14159),{1E+200,1E+200,-1E+200})", "#DIV/0!"),
        ("=SKEW.P({1E+200,1E+200,-1E+200})", "#DIV/0!"),
        ("=SKEW({1E+200,1E+200,-1E+200})", "#DIV/0!"),
        ("=ZTEST({1E+200,1E+200,-1E+200},4)", "#NUM!"),
        ("=T.TEST({1,2,3},{1E+200,1E+200,-1E+200},1,2)", "#NUM!"),
        ("=T.TEST({1,2,3},{1E+200,1E+200,-1E+200},2,3)", "#NUM!"),
        // WEIBULL's cdf where x^alpha and beta^-alpha are inf and 0 is still a number...
        ("=WEIBULL(1E+15,42,10000000001,TRUE)", "1"),
        ("=WEIBULL(0.2,1E+15,0.2,TRUE)", "0.632120559"),
        // ... while the density there stays #NUM!.
        ("=WEIBULL(3,3.14159E+15,1500000,FALSE)", "#NUM!"),
        // T.TEST: either sample's own error comes before tails and type, and both of those are
        // read before either is range-checked.
        ("=TTEST({1E+200,1E+200,-1E+200},\"a\"+1,NA(),3)", "#VALUE!"),
        ("=T.TEST({1,2,3},1/0,\"a\"+1,3)", "#DIV/0!"),
        ("=T.TEST(4,{1,2,3},7,\"a\"+1)", "#VALUE!"),
        ("=T.TEST(4,{1,2,3},7,2)", "#NUM!"),
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
