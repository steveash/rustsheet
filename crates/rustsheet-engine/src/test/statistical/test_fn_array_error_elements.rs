//! An error element inside an array argument is the result of LARGE, SMALL, QUARTILE.EXC,
//! PERCENTRANK, MODE.MULT, COVARIANCE.S, CORREL and the like, not `#VALUE!` (gen-m9, T-2255,
//! T-2256; docs/excel-semantics/statistics.md "An error element of an array argument is that
//! error").

use crate::test::util::new_empty_model;

const CASES: &[(&str, &str)] = &[
    ("LARGE({1,#N/A,3},3)", "#N/A"),
    ("LARGE({1,#N/A,3},1)", "#N/A"),
    ("SMALL({1,#N/A,3},3)", "#N/A"),
    ("QUARTILE({1,#N/A,3},0)", "#N/A"),
    ("QUARTILE.EXC({1,#N/A,3},1)", "#N/A"),
    ("PERCENTILE({1,#N/A,3},0.5)", "#N/A"),
    ("PERCENTRANK.INC({1,#N/A,3},2)", "#N/A"),
    ("PERCENTRANK.EXC({1,#N/A,3},2)", "#N/A"),
    ("MODE({1,#N/A,3})", "#N/A"),
    ("MODE.SNGL({1,#N/A,3})", "#N/A"),
    ("COVARIANCE.S({1,#N/A,3},{-1,0,1})", "#N/A"),
    ("COVAR({1,#N/A,3},{-1,0,1})", "#N/A"),
    ("CORREL({1,#N/A,3},{-1,0,1})", "#N/A"),
    // Another argument's error, or one that does not coerce, wins over the element's
    // (gen-m9/31f09e6c48ca245c A14, 165db58d1540f014 A353, 9735c5ab72fa5ecf A36).
    ("PERCENTILE({1,#N/A,3},(\"x\"*1))", "#VALUE!"),
    ("TRIMMEAN({1,#N/A,3},{\"a\"})", "#VALUE!"),
    ("COVARIANCE.S({1,#N/A,3},(\"x\"*1))", "#VALUE!"),
    ("PEARSON({1,#N/A,3},{-1,0,1})", "#N/A"),
    ("TRIMMEAN({1,#N/A,3},0.2)", "#N/A"),
    // An inner call's result is the outer call's own data error, which comes first, even when
    // the inner call's value was an element error (gen-m9/1b31092a0d79e5f1 A363, T-3080).
    ("SMALL((SMALL({1,#N/A,3},2)/1.005),\"a*\")", "#N/A"),
    ("SMALL(SMALL({1,#N/A,3},2),\"x\")", "#N/A"),
    ("SMALL(LARGE({1,#N/A,3},2),\"x\")", "#N/A"),
    ("SMALL(IF(TRUE,SMALL({1,#N/A,3},2)),\"x\")", "#N/A"),
    ("SMALL(LET(v,SMALL({1,#N/A,3},2),v),\"x\")", "#N/A"),
    // A plain error that was never an element's (a guard, unchanged by T-3080).
    ("SMALL(NA(),\"x\")", "#N/A"),
    // An element error inside the array argument itself still loses to a text k.
    ("SMALL({1,#N/A,3},\"x\")", "#VALUE!"),
    ("LARGE({1,#N/A,3},\"x\")", "#VALUE!"),
];

#[test]
fn error_element_of_an_array_argument_is_that_error() {
    for (formula, expected) in CASES {
        let mut model = new_empty_model();
        model._set("A1", &format!("={formula}"));
        model.evaluate();
        assert_eq!(model._get_text("A1"), *expected, "{formula}");
    }
}
