use crate::r1c1_formula_to_a1;

fn a1(text: &str, row: i32, column: i32) -> Option<String> {
    let sheets = ["Sheet1".to_string(), "S".to_string()];
    r1c1_formula_to_a1(text, &sheets, "Sheet1", row, column)
}

#[test]
fn absolute_and_relative_references_resolve_against_the_cell() {
    assert_eq!(a1("=R1C2+1", 3, 1).as_deref(), Some("=$B$1+1"));
    assert_eq!(a1("=R[-1]C[1]", 3, 1).as_deref(), Some("=B2"));
    assert_eq!(
        a1("=SUM(R1C1:R[1]C[2])", 3, 1).as_deref(),
        Some("=SUM($A$1:C4)")
    );
    assert_eq!(a1("=S!R1C2", 1, 1).as_deref(), Some("=S!$B$1"));
}

#[test]
fn constants_pass_through_and_unparsable_formulas_are_refused() {
    assert_eq!(a1("(1)", 1, 1).as_deref(), Some("(1)"));
    assert_eq!(a1("=$A$1", 1, 1), None);
    assert_eq!(a1("=1+", 1, 1), None);
}
