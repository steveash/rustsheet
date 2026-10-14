#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "Monday");
    model._set("A2", "Tuesday");
    model._set("A3", "Wednesday");

    model._set("B1", "=TEXTJOIN(\", \", TRUE, A1:A3)");
    model._set("B2", "=TEXTJOIN(\" and \", TRUE, A1:A3)");
    // This formula might have the _xlfn. prefix
    model._set("B3", "=_xlfn.TEXTJOIN(\" or \", , A1:A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Monday, Tuesday, Wednesday");
    assert_eq!(model._get_text("B2"), *"Monday and Tuesday and Wednesday");
    assert_eq!(model._get_text("B3"), *"Monday or Tuesday or Wednesday");
    // Our text version removes the prefix, of course (and some white spaces)
    assert_eq!(model._get_formula("B3"), *"=TEXTJOIN(\" or \",,A1:A3)");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "Monday");
    model._set("A2", "Tuesday");
    model._set("A3", "Wednesday");

    model._set("B1", "=TEXTJOIN(\", \", TRUE)");
    model._set("B2", "=TEXTJOIN(\" and \", A1:A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,LEN(TEXTJOIN(\",\",TRUE,D:D)))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "9");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model._get_text("A1"), "9");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

// Excel values: probes/textjoin-array-arguments.
#[test]
fn array_literals_and_function_results() {
    let mut model = new_empty_model();
    model._set("B1", "pear");
    model._set("B3", "apple");
    model._set("A1", "=TEXTJOIN(\",\",FALSE,{1,2;3,4})");
    model._set("A2", "=TEXTJOIN(\",\",TRUE,{\"a\",\"\",\"c\"})");
    model._set("A3", "=TEXTJOIN(\",\",FALSE,{\"a\",\"\",\"c\"})");
    model._set("A4", "=TEXTJOIN(\",\",FALSE,{1,#N/A,3})");
    model._set("A5", "=TEXTJOIN(\",\",FALSE,{#DIV/0!,#N/A})");
    model._set("A6", "=TEXTJOIN(\",\",FALSE,SORT(B1:B3))");
    model._set("A7", "=TEXTJOIN(\",\",TRUE,SORT(B1:B3))");
    model._set("A8", "=TEXTJOIN(\",\",FALSE,SEQUENCE(2,2))");
    model._set("A9", "=TEXTJOIN(\",\",FALSE,{1,2},5,\"x\")");
    model._set("A10", "=TEXTJOIN(\",\",FALSE,{TRUE,FALSE,1.5})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1,2,3,4");
    assert_eq!(model._get_text("A2"), *"a,c");
    assert_eq!(model._get_text("A3"), *"a,,c");
    assert_eq!(model._get_text("A4"), *"#N/A");
    assert_eq!(model._get_text("A5"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"apple,pear,");
    assert_eq!(model._get_text("A7"), *"apple,pear");
    assert_eq!(model._get_text("A8"), *"1,2,3,4");
    assert_eq!(model._get_text("A9"), *"1,2,5,x");
    assert_eq!(model._get_text("A10"), *"TRUE,FALSE,1.5");
}

// Excel values: probes/textjoin-array-limit-empty-strings A7-A9.
#[test]
fn empty_strings_are_skipped_with_ignore_empty() {
    let mut model = new_empty_model();
    model._set("B1", "a");
    model._set("B2", "=\"\"");
    model._set("B3", "c");
    model._set("A1", "=TEXTJOIN(\",\",TRUE,\"a\",\"\",\"c\")");
    model._set("A2", "=TEXTJOIN(\",\",TRUE,B1:B3)");
    model._set("A3", "=TEXTJOIN(\",\",FALSE,B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"a,c");
    assert_eq!(model._get_text("A2"), *"a,c");
    assert_eq!(model._get_text("A3"), *"a,,c");
}
