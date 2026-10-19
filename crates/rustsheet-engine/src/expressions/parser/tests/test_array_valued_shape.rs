use std::collections::HashMap;

use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::types::CellReferenceRC;

fn shape(formula: &str) -> StaticResult {
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let context = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    run_static_analysis_on_node(&parser.parse(formula, &context))
}

/// An array-valued table entry is `Unknown` unlifted and the broadcast of its lifted arguments
/// otherwise (SPEC-0011 "Array-valued functions" rule 1; probes/m7-argument-modes F1, F2).
#[test]
fn an_array_valued_entry_is_unknown_unless_it_lifts() {
    for formula in [
        "SEQUENCE(3)",
        "SEQUENCE(2,3,1,1)",
        "MAKEARRAY(2,2,LAMBDA(r,c,r+c))",
    ] {
        assert!(matches!(shape(formula), StaticResult::Unknown), "{formula}");
    }
    assert!(matches!(
        shape("SEQUENCE({1;2;3})"),
        StaticResult::Array(3, 1)
    ));
    assert!(matches!(
        shape("MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c))"),
        StaticResult::Range(1, 0) | StaticResult::Array(2, 1)
    ));
}
