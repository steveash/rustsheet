#![allow(clippy::unwrap_used)]
#![allow(clippy::print_stdout)]
use crate::test::util::new_empty_model;

// These tests are grouped because in many cases XOR and OR have similar behaviour.

// Test specific to xor
#[test]
fn fn_xor() {
    let mut model = new_empty_model();

    model._set("A1", "=XOR(1, 1, 1, 0, 0)");
    model._set("A2", "=XOR(1, 1, 0, 0, 0)");
    model._set("A3", "=XOR(TRUE, TRUE, TRUE, FALSE, FALSE)");
    model._set("A4", "=XOR(TRUE, TRUE, FALSE, FALSE, FALSE)");
    model._set("A5", "=XOR(FALSE, FALSE, FALSE, FALSE, FALSE)");
    model._set("A6", "=XOR(TRUE, TRUE)");
    model._set("A7", "=XOR(0,0,0)");
    model._set("A8", "=XOR(0,0,1)");
    model._set("A9", "=XOR(0,1,0)");
    model._set("A10", "=XOR(0,1,1)");
    model._set("A11", "=XOR(1,0,0)");
    model._set("A12", "=XOR(1,0,1)");
    model._set("A13", "=XOR(1,1,0)");
    model._set("A14", "=XOR(1,1,1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"FALSE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"FALSE");
    assert_eq!(model._get_text("A5"), *"FALSE");
    assert_eq!(model._get_text("A6"), *"FALSE");
    assert_eq!(model._get_text("A7"), *"FALSE");
    assert_eq!(model._get_text("A8"), *"TRUE");
    assert_eq!(model._get_text("A9"), *"TRUE");
    assert_eq!(model._get_text("A10"), *"FALSE");
    assert_eq!(model._get_text("A11"), *"TRUE");
    assert_eq!(model._get_text("A12"), *"FALSE");
    assert_eq!(model._get_text("A13"), *"FALSE");
    assert_eq!(model._get_text("A14"), *"TRUE");
}

#[test]
fn fn_or() {
    let mut model = new_empty_model();

    model._set("A1", "=OR(1, 1, 1, 0, 0)");
    model._set("A2", "=OR(1, 1, 0, 0, 0)");
    model._set("A3", "=OR(TRUE, TRUE, TRUE, FALSE, FALSE)");
    model._set("A4", "=OR(TRUE, TRUE, FALSE, FALSE, FALSE)");
    model._set("A5", "=OR(FALSE, FALSE, FALSE, FALSE, FALSE)");
    model._set("A6", "=OR(TRUE, TRUE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"TRUE");
    assert_eq!(model._get_text("A5"), *"FALSE");
    assert_eq!(model._get_text("A6"), *"TRUE");
}

#[test]
fn fn_or_xor() {
    inner("or");
    inner("xor");

    fn inner(func: &str) {
        println!("Testing function: {func}");

        let mut model = new_empty_model();

        // Text args
        model._set("A1", &format!(r#"={func}("")"#));
        model._set("A2", &format!(r#"={func}("", "")"#));
        model._set("A3", &format!(r#"={func}("", TRUE)"#));
        model._set("A4", &format!(r#"={func}("", FALSE)"#));

        model._set("A5", &format!("={func}(FALSE, TRUE)"));
        model._set("A6", &format!("={func}(FALSE, FALSE)"));
        model._set("A7", &format!("={func}(TRUE, FALSE)"));

        // Reference to empty cell, plus true argument
        model._set("A8", &format!("={func}(Z99, 1)"));

        // Reference to empty cell/range
        model._set("A9", &format!("={func}(Z99)"));
        model._set("A10", &format!("={func}(X99:Z99"));

        // Reference to text cell
        model._set("B13", "some_text");
        model._set("A13", &format!("={func}(B13)"));
        model._set("A14", &format!("={func}(B13, 0)"));
        model._set("A15", &format!("={func}(B13, 1)"));

        // Reference to Implicit intersection
        model._set("X16", "1");
        model._set("B16", "=@X15:X16");
        model._set("A16", &format!("={func}(B16)"));

        // Non-empty range
        model._set("B17", "1");
        model._set("A17", &format!("={func}(B17:C17)"));

        // Non-empty range with text
        model._set("B18", "text");
        model._set("A18", &format!("={func}(B18:C18)"));

        // Non-empty range with text and number
        model._set("B19", "text");
        model._set("C19", "1");
        model._set("A19", &format!("={func}(B19:C19)"));

        // range with error
        model._set("B20", "=1/0");
        model._set("A20", &format!("={func}(B20:C20)"));

        model.evaluate();

        assert_eq!(model._get_text("A1"), *"#VALUE!");
        assert_eq!(model._get_text("A2"), *"#VALUE!");
        assert_eq!(model._get_text("A3"), *"TRUE");
        assert_eq!(model._get_text("A4"), *"FALSE");

        assert_eq!(model._get_text("A5"), *"TRUE");
        assert_eq!(model._get_text("A6"), *"FALSE");
        assert_eq!(model._get_text("A7"), *"TRUE");

        assert_eq!(model._get_text("A8"), *"TRUE");

        assert_eq!(model._get_text("A9"), *"#VALUE!");
        assert_eq!(model._get_text("A10"), *"#VALUE!");

        assert_eq!(model._get_text("A13"), *"#VALUE!");
        assert_eq!(model._get_text("A14"), *"FALSE");
        assert_eq!(model._get_text("A15"), *"TRUE");

        assert_eq!(model._get_text("A17"), *"TRUE");

        assert_eq!(model._get_text("A18"), *"#VALUE!");

        assert_eq!(model._get_text("A19"), *"TRUE");

        assert_eq!(model._get_text("A20"), *"#DIV/0!");
    }
}

#[test]
fn fn_or_xor_arrays() {
    inner("or");
    inner("xor");

    fn inner(func: &str) {
        println!("Testing function: {func}");

        let mut model = new_empty_model();

        // Boolean array literals
        model._set("A1", &format!("={func}({{TRUE, FALSE, FALSE}})"));
        model._set("A2", &format!("={func}({{FALSE, FALSE, FALSE}})"));

        // Number array literals (0 is FALSE, anything else TRUE)
        model._set("A3", &format!("={func}({{0, 0, 1}})"));
        model._set("A4", &format!("={func}({{0, 0, 0}})"));

        // Strings in arrays are ignored
        model._set("A5", &format!(r#"={func}({{"text", TRUE}})"#));
        model._set("A6", &format!(r#"={func}({{"text", FALSE}})"#));

        // Two-dimensional array combined with a scalar argument
        model._set("A7", &format!("={func}({{FALSE; FALSE}}, TRUE)"));

        // Errors inside an array are propagated
        model._set("A8", &format!(r#"={func}({{#DIV/0!, TRUE}})"#));

        model.evaluate();

        assert_eq!(model._get_text("A1"), *"TRUE");
        assert_eq!(model._get_text("A2"), *"FALSE");
        assert_eq!(model._get_text("A3"), *"TRUE");
        assert_eq!(model._get_text("A4"), *"FALSE");
        assert_eq!(model._get_text("A5"), *"TRUE");
        assert_eq!(model._get_text("A6"), *"FALSE");
        assert_eq!(model._get_text("A7"), *"TRUE");
        assert_eq!(model._get_text("A8"), *"#DIV/0!");
    }
}

#[test]
fn fn_and_arrays() {
    let mut model = new_empty_model();

    model._set("A1", "=AND({TRUE, TRUE, TRUE})");
    model._set("A2", "=AND({TRUE, FALSE, TRUE})");
    model._set("A3", "=AND({1, 2, 3})");
    model._set("A4", "=AND({1, 0, 3})");
    // Strings ignored
    model._set("A5", r#"=AND({"text", TRUE})"#);
    // Error propagated
    model._set("A6", "=AND({TRUE, #N/A})");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"FALSE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"FALSE");
    assert_eq!(model._get_text("A5"), *"TRUE");
    assert_eq!(model._get_text("A6"), *"#N/A");
}

#[test]
#[ignore = "not yet implemented"]
fn spill_behaviour() {
    let mut model = new_empty_model();

    // Reference to cell with reference to empty range
    model._set("B11", "=X99:Z99");
    model._set("A11", "=OR(B11)");

    // Reference to cell with non-empty range
    model._set("X12", "1");
    model._set("B12", "=X12:Z12");
    model._set("A12", "=OR(B12)");

    model.evaluate();

    assert_eq!(model._get_text("A11"), *"FALSE");

    assert_eq!(model._get_text("A12"), *"TRUE");
}

#[test]
fn fn_or_xor_no_arguments() {
    inner("or");
    inner("xor");

    fn inner(func: &str) {
        println!("Testing function: {func}");

        let mut model = new_empty_model();
        model._set("A1", &format!("={func}()"));
        model.evaluate();
        assert_eq!(model._get_text("A1"), *"#ERROR!");
    }
}

#[test]
fn fn_or_xor_missing_arguments() {
    inner("or");
    inner("xor");

    fn inner(func: &str) {
        println!("Testing function: {func}");

        let mut model = new_empty_model();
        model._set("A1", &format!("={func}(,)"));
        model._set("A2", &format!("={func}(,1)"));
        model._set("A3", &format!("={func}(1,)"));
        model._set("A4", &format!("={func}(,B1)"));
        model._set("A5", &format!("={func}(,B1:B4)"));
        model.evaluate();
        assert_eq!(model._get_text("A1"), *"FALSE");
        assert_eq!(model._get_text("A2"), *"TRUE");
        assert_eq!(model._get_text("A3"), *"TRUE");
        assert_eq!(model._get_text("A4"), *"FALSE");
        assert_eq!(model._get_text("A5"), *"FALSE");
    }
}

// AND/OR evaluate every argument: an error after a decisive value still wins
// (T-0215; docs/excel-semantics/coercion.md).
#[test]
fn test_and_or_do_not_short_circuit_past_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=OR(TRUE,1/0)");
    model._set("A2", "=AND(FALSE,1/0)");
    model._set("A3", "=OR(TRUE,NA())");
    model._set("A4", "=AND(FALSE,NA())");
    model._set("B1", "1");
    model._set("B2", "=1/0");
    // A5/A6 (an error inside a range argument) are UNVERIFIED: no oracle case, probe in T-0229.
    model._set("A5", "=OR(B1:B2)");
    model._set("A6", "=OR(TRUE,B1:B2)");
    model._set("A7", "=AND(FALSE,TRUE)");
    model._set("A8", "=OR(TRUE,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#N/A");
    assert_eq!(model._get_text("A4"), *"#N/A");
    assert_eq!(model._get_text("A5"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
    assert_eq!(model._get_text("A7"), *"FALSE");
    assert_eq!(model._get_text("A8"), *"TRUE");
}

// NOT lifts over arrays and ranges, one result per element (T-0225).
#[test]
fn test_not_lifts_over_arrays_and_ranges() {
    let mut model = new_empty_model();
    model._set("A1", "=NOT({TRUE,FALSE})");
    model._set("A3", "TRUE");
    model._set("A4", "0");
    model._set("C1", "=NOT(A3:A4)");
    model._set("E1", "=NOT(TRUE)");
    // Two whole columns exceed the shared array allocation bound (DEVIATION, decision 0014).
    model._set("G1", "=NOT(Z:AA)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
    assert_eq!(model._get_text("B1"), *"TRUE");
    assert_eq!(model._get_text("C1"), *"FALSE");
    assert_eq!(model._get_text("C2"), *"TRUE");
    assert_eq!(model._get_text("E1"), *"FALSE");
    assert_eq!(model._get_text("G1"), *"#ERROR!");
}

/// Nightly fuzz OOM (T-0230): `IFERROR` hands `OR` the reference `F:LSE` (8,606 whole
/// columns). The walk is clipped to the used area, so this is quick and bounded, and the
/// result is that of an empty range: no logical values, `#VALUE!`.
#[test]
fn fuzz_oom_or_over_whole_columns() {
    let mut model = new_empty_model();
    model._set("A1", "=OR(IFERROR(ERROR.TYPE(TRUNC($Z$1))=5,F:LSE))");
    model._set("A2", "=OR(F:LSE)");
    let start = std::time::Instant::now();
    model.evaluate();
    assert!(start.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
}

/// Several errors: the first one, arguments left to right and ranges row by row, wins for
/// AND, OR and XOR (probes/and-or-xor-error-order, excel-graph, T-0229).
#[test]
fn first_error_wins_row_by_row() {
    let mut model = new_empty_model();
    // D1:F2 = TRUE, #DIV/0!, FALSE / #N/A, TRUE, #NUM!
    model._set("D1", "TRUE");
    model._set("E1", "=1/0");
    model._set("F1", "FALSE");
    model._set("D2", "=NA()");
    model._set("E2", "TRUE");
    model._set("F2", "=SQRT(-1)");
    model._set("A1", "=OR(TRUE,NA(),1/0)");
    model._set("A2", "=AND(FALSE,1/0,NA())");
    model._set("A3", "=XOR(D1:F2)");
    model._set("A4", "=OR(TRANSPOSE(D1:F2))");
    model._set("A5", "=AND(D1:F2,NA())");
    model._set("A6", "=OR({TRUE,#DIV/0!,#N/A})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#N/A");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"#N/A");
    assert_eq!(model._get_text("A5"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
}

// AND, OR and XOR read through the SPEC-0009 walker with today's results kept (T-1677).
#[test]
fn logical_walker_keeps_text_empty_and_error_rules() {
    let mut model = new_empty_model();
    model._set("A1", "=\"TRUE\"");
    model._set("A2", "x");
    model.new_sheet();
    // Direct text counts in any case; referenced text, also "TRUE", is skipped.
    model._set("B1", "=AND(\"true\")");
    model._set("B2", "=OR(\"FALSE\",\"tRuE\")");
    model._set("B3", "=AND(A1)");
    model._set("B4", "=OR(A1:A2,FALSE)");
    model._set("B5", "=OR({\"TRUE\"},FALSE)");
    // An empty argument is FALSE only when nothing came before.
    model._set("B6", "=AND(TRUE,)");
    model._set("B7", "=AND(,TRUE)");
    model._set("B8", "=OR(,)");
    // The first error wins after a decisive value.
    model._set("B9", "=AND(FALSE,1/0)");
    model._set("B10", "=OR(TRUE,{1,#N/A},1/0)");
    // A whole column on an empty sheet has no logical value.
    model._set("B11", "=OR(Sheet2!C:C)");
    model._set("B13", "=AND(LAMBDA(x,x),1/0)");
    model._set("B14", "=XOR(A1:A2)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("B2"), "TRUE");
    assert_eq!(model._get_text("B3"), "#VALUE!");
    assert_eq!(model._get_text("B4"), "FALSE");
    assert_eq!(model._get_text("B5"), "FALSE");
    assert_eq!(model._get_text("B6"), "TRUE");
    assert_eq!(model._get_text("B7"), "FALSE");
    assert_eq!(model._get_text("B8"), "FALSE");
    assert_eq!(model._get_text("B9"), "#DIV/0!");
    assert_eq!(model._get_text("B10"), "#N/A");
    assert_eq!(model._get_text("B11"), "#VALUE!");
    assert_eq!(model._get_text("B13"), "#N/IMPL!");
    assert_eq!(model._get_text("B14"), "#VALUE!");
}
