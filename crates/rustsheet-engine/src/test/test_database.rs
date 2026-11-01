#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::Model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=DMIN()");
    model._set("A2", "=DMIN(2)");
    model._set("A3", "=DMIN(1, 2)");
    model._set("A4", "=DMIN(1, 2, 3, 4)");

    model._set("A5", "=DMAX()");
    model._set("A6", "=DMAX(2)");
    model._set("A7", "=DMAX(1, 2)");
    model._set("A8", "=DMAX(1, 2, 3, 4)");

    model._set("A9", "=DAVERAGE()");
    model._set("A10", "=DAVERAGE(2)");
    model._set("A11", "=DAVERAGE(1, 2)");
    model._set("A12", "=DAVERAGE(1, 2, 3, 4)");

    model._set("A13", "=DSUM()");
    model._set("A14", "=DSUM(2)");
    model._set("A15", "=DSUM(1, 2)");
    model._set("A16", "=DSUM(1, 2, 3, 4)");

    model._set("A17", "=DCOUNT()");
    model._set("A18", "=DCOUNT(2)");
    model._set("A19", "=DCOUNT(1, 2)");
    model._set("A20", "=DCOUNT(1, 2, 3, 4)");

    model._set("A21", "=DCOUNTA()");
    model._set("A22", "=DCOUNTA(2)");
    model._set("A23", "=DCOUNTA(1, 2)");
    model._set("A24", "=DCOUNTA(1, 2, 3, 4)");

    model._set("B1", "=DGET()");
    model._set("B2", "=DGET(2)");
    model._set("B3", "=DGET(1, 2)");
    model._set("B4", "=DGET(1, 2, 3, 4)");

    model._set("B5", "=DPRODUCT()");
    model._set("B6", "=DPRODUCT(2)");
    model._set("B7", "=DPRODUCT(1, 2)");
    model._set("B8", "=DPRODUCT(1, 2, 3, 4)");

    model._set("B9", "=DVAR()");
    model._set("B10", "=DVAR(2)");
    model._set("B11", "=DVAR(1, 2)");
    model._set("B12", "=DVAR(1, 2, 3, 4)");

    model._set("B13", "=DVARP()");
    model._set("B14", "=DVARP(2)");
    model._set("B15", "=DVARP(1, 2)");
    model._set("B16", "=DVARP(1, 2, 3, 4)");

    model._set("B17", "=DSTDEV()");
    model._set("B18", "=DSTDEV(2)");
    model._set("B19", "=DSTDEV(1, 2)");
    model._set("B20", "=DSTDEV(1, 2, 3, 4)");

    model._set("B21", "=DSTDEVP()");
    model._set("B22", "=DSTDEVP(2)");
    model._set("B23", "=DSTDEVP(1, 2)");
    model._set("B24", "=DSTDEVP(1, 2, 3, 4)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");

    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"#ERROR!");

    assert_eq!(model._get_text("A9"), *"#ERROR!");
    assert_eq!(model._get_text("A10"), *"#ERROR!");
    assert_eq!(model._get_text("A11"), *"#ERROR!");
    assert_eq!(model._get_text("A12"), *"#ERROR!");

    assert_eq!(model._get_text("A13"), *"#ERROR!");
    assert_eq!(model._get_text("A14"), *"#ERROR!");
    assert_eq!(model._get_text("A15"), *"#ERROR!");
    assert_eq!(model._get_text("A16"), *"#ERROR!");

    assert_eq!(model._get_text("A17"), *"#ERROR!");
    assert_eq!(model._get_text("A18"), *"#ERROR!");
    assert_eq!(model._get_text("A19"), *"#ERROR!");
    assert_eq!(model._get_text("A20"), *"#ERROR!");

    assert_eq!(model._get_text("A21"), *"#ERROR!");
    assert_eq!(model._get_text("A22"), *"#ERROR!");
    assert_eq!(model._get_text("A23"), *"#ERROR!");
    assert_eq!(model._get_text("A24"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
    assert_eq!(model._get_text("B4"), *"#ERROR!");

    assert_eq!(model._get_text("B5"), *"#ERROR!");
    assert_eq!(model._get_text("B6"), *"#ERROR!");
    assert_eq!(model._get_text("B7"), *"#ERROR!");
    assert_eq!(model._get_text("B8"), *"#ERROR!");

    assert_eq!(model._get_text("B9"), *"#ERROR!");
    assert_eq!(model._get_text("B10"), *"#ERROR!");
    assert_eq!(model._get_text("B11"), *"#ERROR!");
    assert_eq!(model._get_text("B12"), *"#ERROR!");

    assert_eq!(model._get_text("B13"), *"#ERROR!");
    assert_eq!(model._get_text("B14"), *"#ERROR!");
    assert_eq!(model._get_text("B15"), *"#ERROR!");
    assert_eq!(model._get_text("B16"), *"#ERROR!");

    assert_eq!(model._get_text("B17"), *"#ERROR!");
    assert_eq!(model._get_text("B18"), *"#ERROR!");
    assert_eq!(model._get_text("B19"), *"#ERROR!");
    assert_eq!(model._get_text("B20"), *"#ERROR!");

    assert_eq!(model._get_text("B21"), *"#ERROR!");
    assert_eq!(model._get_text("B22"), *"#ERROR!");
    assert_eq!(model._get_text("B23"), *"#ERROR!");
    assert_eq!(model._get_text("B24"), *"#ERROR!");
}

#[test]
fn locale_iso_format() {
    // ISO format with YYYY-MM-DD format. Works with any locale.
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();

    // Create database
    model._set("A1", "ID");
    model._set("B1", "Date");
    model._set("C1", "Amount");
    model._set("A2", "1");
    model._set("B2", "2026-01-15");
    model._set("C2", "1200");
    model._set("A3", "2");
    model._set("B3", "2026-03-15");
    model._set("C3", "900");
    model._set("A4", "3");
    model._set("B4", "2026-06-15");
    model._set("C4", "2100");

    // Define criteria
    model._set("A6", "Date");
    model._set("A7", ">=2026-03-01");
    model._set("B6", "Date");
    model._set("B7", "2026-01-15"); // DGET needs an exact match

    // Test functions
    model._set("A9", "=DMIN(A1:C4, C1, A6:A7)");
    model._set("A10", "=DMAX(A1:C4, C1, A6:A7)");
    model._set("A11", "=DAVERAGE(A1:C4, C1, A6:A7)");
    model._set("A12", "=DSUM(A1:C4, C1, A6:A7)");
    model._set("A13", "=DPRODUCT(A1:C4, C1, A6:A7)");
    model._set("A14", "=DGET(A1:C4, C1, B6:B7)");
    model._set("A15", "=DCOUNT(A1:C4, C1, A6:A7)");
    model._set("A16", "=DCOUNTA(A1:C4, C1, A6:A7)");
    model._set("A17", "=DVAR(A1:C4, C1, A6:A7)");
    model._set("A18", "=DVARP(A1:C4, C1, A6:A7)");
    model._set("A19", "=DSTDEV(A1:C4, C1, A6:A7)");
    model._set("A20", "=DSTDEVP(A1:C4, C1, A6:A7)");

    model.evaluate();

    assert_eq!(model._get_text("A9"), *"900");
    assert_eq!(model._get_text("A10"), *"2100");
    assert_eq!(model._get_text("A11"), *"1500");
    assert_eq!(model._get_text("A12"), *"3000");
    assert_eq!(model._get_text("A13"), *"1890000");
    assert_eq!(model._get_text("A14"), *"1200");
    assert_eq!(model._get_text("A15"), *"2");
    assert_eq!(model._get_text("A16"), *"2");
    assert_eq!(model._get_text("A17"), *"720000");
    assert_eq!(model._get_text("A18"), *"360000");
    assert_eq!(model._get_text("A19"), *"848.5281374");
    assert_eq!(model._get_text("A20"), *"600");
}

#[test]
fn locale_uk() {
    // en-GB locale with DD/MM/YYYY format
    let mut model = Model::new_empty("model", "en-GB", "UTC", "en").unwrap();

    // Create database
    model._set("A1", "ID");
    model._set("B1", "Date");
    model._set("C1", "Amount");
    model._set("A2", "1");
    model._set("B2", "15/01/2026");
    model._set("C2", "1200");
    model._set("A3", "2");
    model._set("B3", "15/03/2026");
    model._set("C3", "900");
    model._set("A4", "3");
    model._set("B4", "15/06/2026");
    model._set("C4", "2100");

    // Define criteria
    model._set("A6", "Date");
    model._set("A7", ">=01/03/2026");
    model._set("B6", "Date");
    model._set("B7", "15/01/2026"); // DGET needs an exact match

    // Test functions
    model._set("A9", "=DMIN(A1:C4, C1, A6:A7)");
    model._set("A10", "=DMAX(A1:C4, C1, A6:A7)");
    model._set("A11", "=DAVERAGE(A1:C4, C1, A6:A7)");
    model._set("A12", "=DSUM(A1:C4, C1, A6:A7)");
    model._set("A13", "=DPRODUCT(A1:C4, C1, A6:A7)");
    model._set("A14", "=DGET(A1:C4, C1, B6:B7)");
    model._set("A15", "=DCOUNT(A1:C4, C1, A6:A7)");
    model._set("A16", "=DCOUNTA(A1:C4, C1, A6:A7)");
    model._set("A17", "=DVAR(A1:C4, C1, A6:A7)");
    model._set("A18", "=DVARP(A1:C4, C1, A6:A7)");
    model._set("A19", "=DSTDEV(A1:C4, C1, A6:A7)");
    model._set("A20", "=DSTDEVP(A1:C4, C1, A6:A7)");

    model.evaluate();

    assert_eq!(model._get_text("A9"), *"900");
    assert_eq!(model._get_text("A10"), *"2100");
    assert_eq!(model._get_text("A11"), *"1500");
    assert_eq!(model._get_text("A12"), *"3000");
    assert_eq!(model._get_text("A13"), *"1890000");
    assert_eq!(model._get_text("A14"), *"1200");
    assert_eq!(model._get_text("A15"), *"2");
    assert_eq!(model._get_text("A16"), *"2");
    assert_eq!(model._get_text("A17"), *"720000");
    assert_eq!(model._get_text("A18"), *"360000");
    assert_eq!(model._get_text("A19"), *"848.5281374");
    assert_eq!(model._get_text("A20"), *"600");
}

#[test]
fn locale_us() {
    // en-US locale with MM/DD/YY format
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();

    // Create database
    model._set("A1", "ID");
    model._set("B1", "Date");
    model._set("C1", "Amount");
    model._set("A2", "1");
    model._set("B2", "1/15/26");
    model._set("C2", "1200");
    model._set("A3", "2");
    model._set("B3", "3/15/26");
    model._set("C3", "900");
    model._set("A4", "3");
    model._set("B4", "6/15/26");
    model._set("C4", "2100");

    // Define criteria
    model._set("A6", "Date");
    model._set("A7", ">=3/1/26");
    model._set("B6", "Date");
    model._set("B7", "1/15/26"); // DGET needs an exact match

    // Test functions
    model._set("A9", "=DMIN(A1:C4, C1, A6:A7)");
    model._set("A10", "=DMAX(A1:C4, C1, A6:A7)");
    model._set("A11", "=DAVERAGE(A1:C4, C1, A6:A7)");
    model._set("A12", "=DSUM(A1:C4, C1, A6:A7)");
    model._set("A13", "=DPRODUCT(A1:C4, C1, A6:A7)");
    model._set("A14", "=DGET(A1:C4, C1, B6:B7)");
    model._set("A15", "=DCOUNT(A1:C4, C1, A6:A7)");
    model._set("A16", "=DCOUNTA(A1:C4, C1, A6:A7)");
    model._set("A17", "=DVAR(A1:C4, C1, A6:A7)");
    model._set("A18", "=DVARP(A1:C4, C1, A6:A7)");
    model._set("A19", "=DSTDEV(A1:C4, C1, A6:A7)");
    model._set("A20", "=DSTDEVP(A1:C4, C1, A6:A7)");

    model.evaluate();

    assert_eq!(model._get_text("A9"), *"900");
    assert_eq!(model._get_text("A10"), *"2100");
    assert_eq!(model._get_text("A11"), *"1500");
    assert_eq!(model._get_text("A12"), *"3000");
    assert_eq!(model._get_text("A13"), *"1890000");
    assert_eq!(model._get_text("A14"), *"1200");
    assert_eq!(model._get_text("A15"), *"2");
    assert_eq!(model._get_text("A16"), *"2");
    assert_eq!(model._get_text("A17"), *"720000");
    assert_eq!(model._get_text("A18"), *"360000");
    assert_eq!(model._get_text("A19"), *"848.5281374");
    assert_eq!(model._get_text("A20"), *"600");
}

#[test]
fn locale_de() {
    // de-DE locale with D.M.YYYY format
    let mut model = Model::new_empty("model", "de", "UTC", "en").unwrap();

    // Create database
    model._set("A1", "ID");
    model._set("B1", "Date");
    model._set("C1", "Amount");
    model._set("A2", "1");
    model._set("B2", "15.1.2026");
    model._set("C2", "1200");
    model._set("A3", "2");
    model._set("B3", "15.3.2026");
    model._set("C3", "900");
    model._set("A4", "3");
    model._set("B4", "15.6.2026");
    model._set("C4", "2100");

    // Define criteria
    model._set("A6", "Date");
    model._set("A7", ">=1.3.2026");
    model._set("B6", "Date");
    model._set("B7", "15.1.2026"); // DGET needs an exact match

    // Test functions
    model._set("A9", "=DMIN(A1:C4; C1; A6:A7)");
    model._set("A10", "=DMAX(A1:C4; C1; A6:A7)");
    model._set("A11", "=DAVERAGE(A1:C4; C1; A6:A7)");
    model._set("A12", "=DSUM(A1:C4; C1; A6:A7)");
    model._set("A13", "=DPRODUCT(A1:C4; C1; A6:A7)");
    model._set("A14", "=DGET(A1:C4; C1; B6:B7)");
    model._set("A15", "=DCOUNT(A1:C4; C1; A6:A7)");
    model._set("A16", "=DCOUNTA(A1:C4; C1; A6:A7)");
    model._set("A17", "=DVAR(A1:C4; C1; A6:A7)");
    model._set("A18", "=DVARP(A1:C4; C1; A6:A7)");
    model._set("A19", "=DSTDEV(A1:C4; C1; A6:A7)");
    model._set("A20", "=DSTDEVP(A1:C4; C1; A6:A7)");

    model.evaluate();

    assert_eq!(model._get_text("A9"), *"900");
    assert_eq!(model._get_text("A10"), *"2100");
    assert_eq!(model._get_text("A11"), *"1500");
    assert_eq!(model._get_text("A12"), *"3000");
    assert_eq!(model._get_text("A13"), *"1890000");
    assert_eq!(model._get_text("A14"), *"1200");
    assert_eq!(model._get_text("A15"), *"2");
    assert_eq!(model._get_text("A16"), *"2");
    assert_eq!(model._get_text("A17"), *"720000");
    assert_eq!(model._get_text("A18"), *"360000");
    assert_eq!(model._get_text("A19"), *"848,5281374");
    assert_eq!(model._get_text("A20"), *"600");
}

#[test]
fn locale_wrong_format() {
    // en-US locale with incorrect D.M.YYYY format
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();

    // Create database
    model._set("A1", "ID");
    model._set("B1", "Date");
    model._set("C1", "Amount");
    model._set("A2", "1");
    model._set("B2", "10.1.2026");
    model._set("C2", "1200");
    model._set("A3", "2");
    model._set("B3", "10.3.2026");
    model._set("C3", "900");
    model._set("A4", "3");
    model._set("B4", "10.6.2026");
    model._set("C4", "2100");

    // Define criteria
    model._set("A6", "Date");
    model._set("A7", ">=1.3.2026");
    model._set("B6", "Date");
    model._set("B7", "10.1.2026"); // DGET needs an exact match

    // Test functions - results should be the same as using empty criteria
    model._set("A9", "=DMIN(A1:C4, C1, A6:A7)");
    model._set("A10", "=DMAX(A1:C4, C1, A6:A7)");
    model._set("A11", "=DAVERAGE(A1:C4, C1, A6:A7)");
    model._set("A12", "=DSUM(A1:C4, C1, A6:A7)");
    model._set("A13", "=DPRODUCT(A1:C4, C1, A6:A7)");
    model._set("A14", "=DGET(A1:C4, C1, B6:B7)");
    model._set("A15", "=DCOUNT(A1:C4, C1, A6:A7)");
    model._set("A16", "=DCOUNTA(A1:C4, C1, A6:A7)");
    model._set("A17", "=DVAR(A1:C4, C1, A6:A7)");
    model._set("A18", "=DVARP(A1:C4, C1, A6:A7)");
    model._set("A19", "=DSTDEV(A1:C4, C1, A6:A7)");
    model._set("A20", "=DSTDEVP(A1:C4, C1, A6:A7)");

    // Test functions with empty criteria - range C6:C7 is empty
    model._set("B9", "=DMIN(A1:C4, C1, C6:C7)");
    model._set("B10", "=DMAX(A1:C4, C1, C6:C7)");
    model._set("B11", "=DAVERAGE(A1:C4, C1, C6:C7)");
    model._set("B12", "=DSUM(A1:C4, C1, C6:C7)");
    model._set("B13", "=DPRODUCT(A1:C4, C1, C6:C7)");
    model._set("B14", "=DGET(A1:C4, C1, B6:B7)");
    model._set("B15", "=DCOUNT(A1:C4, C1, C6:C7)");
    model._set("B16", "=DCOUNTA(A1:C4, C1, C6:C7)");
    model._set("B17", "=DVAR(A1:C4, C1, C6:C7)");
    model._set("B18", "=DVARP(A1:C4, C1, C6:C7)");
    model._set("B19", "=DSTDEV(A1:C4, C1, C6:C7)");
    model._set("B20", "=DSTDEVP(A1:C4, C1, C6:C7)");

    model.evaluate();

    assert_eq!(model._get_text("A9"), *"900");
    assert_eq!(model._get_text("A10"), *"2100");
    assert_eq!(model._get_text("A11"), *"1400");
    assert_eq!(model._get_text("A12"), *"4200");
    assert_eq!(model._get_text("A13"), *"2268000000");
    assert_eq!(model._get_text("A14"), *"1200");
    assert_eq!(model._get_text("A15"), *"3");
    assert_eq!(model._get_text("A16"), *"3");
    assert_eq!(model._get_text("A17"), *"390000");
    assert_eq!(model._get_text("A18"), *"260000");
    assert_eq!(model._get_text("A19"), *"624.4997998");
    assert_eq!(model._get_text("A20"), *"509.9019514");

    assert_eq!(model._get_text("B9"), *"900");
    assert_eq!(model._get_text("B10"), *"2100");
    assert_eq!(model._get_text("B11"), *"1400");
    assert_eq!(model._get_text("B12"), *"4200");
    assert_eq!(model._get_text("B13"), *"2268000000");
    assert_eq!(model._get_text("B14"), *"1200");
    assert_eq!(model._get_text("B15"), *"3");
    assert_eq!(model._get_text("B16"), *"3");
    assert_eq!(model._get_text("B17"), *"390000");
    assert_eq!(model._get_text("B18"), *"260000");
    assert_eq!(model._get_text("B19"), *"624.4997998");
    assert_eq!(model._get_text("B20"), *"509.9019514");
}

// The database and criteria arguments go through `get_reference`, whose
// fallback arm evaluates non-literal references (OFFSET, INDIRECT) in
// reference context. The computed ranges must arrive whole.
#[test]
fn computed_database_and_criteria_references() {
    let mut model = new_empty_model();
    model._set("A1", "ID");
    model._set("B1", "Amount");
    model._set("A2", "1");
    model._set("B2", "1200");
    model._set("A3", "2");
    model._set("B3", "900");
    model._set("A4", "3");
    model._set("B4", "2100");

    model._set("A6", "Amount");
    model._set("A7", ">1000");

    // OFFSET(A1,0,0,4,2) -> A1:B4; INDIRECT("A6:A7") -> the criteria range.
    model._set(
        "D1",
        "=DSUM(OFFSET(A1, 0, 0, 4, 2), B1, INDIRECT(\"A6:A7\"))",
    );
    model._set("D2", "=DCOUNT(OFFSET(A1, 0, 0, 4, 2), B1, A6:A7)");
    model._set("D3", "=DMAX(OFFSET(A1, 0, 0, 4, 2), B1, A6:A7)");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"3300");
    assert_eq!(model._get_text("D2"), *"2");
    assert_eq!(model._get_text("D3"), *"2100");
}

// Criteria text is not trimmed ("  > 1 " matches nothing: smoke/d-function-criteria J2, excel-graph),
// `<>` is exact (non-text cells match), plain text matches by prefix.
#[test]
fn criteria_grammar_shared_with_countif() {
    let mut model = new_empty_model();
    model._set("A1", "Name");
    model._set("B1", "Qty");
    model._set("A2", "ab");
    model._set("B2", "1");
    model._set("A3", "abc");
    model._set("B3", "2");
    model._set("A4", "xab");
    model._set("B4", "4");
    // Criteria blocks (header + one criterion each)
    model._set("D1", "Name");
    model._set("D2", "ab");
    model._set("F1", "Qty");
    model._set("F2", "  > 1 ");
    model._set("H1", "Name");
    model._set("H2", "<>ab");

    model._set("J1", "=DSUM(A1:B4, \"Qty\", D1:D2)");
    model._set("J2", "=DSUM(A1:B4, \"Qty\", F1:F2)");
    model._set("J3", "=DSUM(A1:B4, \"Qty\", H1:H2)");
    model.evaluate();

    assert_eq!(model._get_text("J1"), *"3");
    assert_eq!(model._get_text("J2"), *"0");
    assert_eq!(model._get_text("J3"), *"6");
}

/// A huge or fractional `field` is out of range, never an overflow (T-1389 security review), and a
/// written empty `field` counts the matching records for DCOUNT and DCOUNTA only
/// (probes/db-argument-modes K12:K13).
#[test]
fn field_edge_cases() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "Name"),
        ("B1", "Qty"),
        ("A2", "a"),
        ("B2", "3"),
        ("A3", "b"),
        ("B3", "4"),
        ("D1", "Qty"),
        ("D2", ">2"),
    ] {
        model._set(cell, value);
    }
    model._set("F1", "=DSUM(A1:B3,1E10,D1:D2)");
    model._set("F2", "=DSUM(A1:B3,2147483647,D1:D2)");
    model._set("F3", "=DSUM(A1:B3,0/0,D1:D2)");
    model._set("F4", "=DCOUNT(A1:B3,,D1:D2)");
    model._set("F5", "=DCOUNTA(A1:B3,,D1:D2)");
    model._set("F6", "=DSUM(A1:B3,,D1:D2)");
    model._set("F7", "=DSUM({1,2;3,4},2,D1:D2)");
    model._set("F8", "=ROWS(DSUM(A1:B3,{1;2},D1:D2))");
    model.evaluate();
    assert_eq!(model._get_text("F1"), *"#VALUE!");
    assert_eq!(model._get_text("F2"), *"#VALUE!");
    assert_eq!(model._get_text("F3"), *"#DIV/0!");
    assert_eq!(model._get_text("F4"), *"2");
    assert_eq!(model._get_text("F5"), *"2");
    assert_eq!(model._get_text("F6"), *"#VALUE!");
    assert_eq!(model._get_text("F7"), *"#VALUE!");
    assert_eq!(model._get_text("F8"), *"2");
}

/// A lifted `field` is charged the criteria table per element, so a large one is refused before
/// it scans (security review of T-1389); the same call without a lift runs.
#[test]
fn lifted_field_is_charged_the_criteria_table() {
    let mut model = new_empty_model();
    model._set("A1", "Name");
    model._set("A2", "a");
    model._set("H1", "Name");
    model._set("H2", "a");
    model._set("F1", "=DCOUNTA(A1:C1000,1,H1:Z1000)");
    model._set("F2", "=ROWS(DCOUNTA(A1:C1000,SEQUENCE(10),H1:Z1000))");
    model.evaluate();
    assert_eq!(model._get_text("F1"), *"1");
    assert_eq!(model._get_text("F2"), *"#ERROR!");
}

/// A criteria reference of one row (headers only) is `#VALUE!`, as is a database of one row
/// (probes/db-header-criteria-shapes AB2-AB37, excel-graph).
#[test]
fn one_row_criteria_or_database_is_value_error() {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "Price"), ("A2", "2.5"), ("A3", "4"), ("E1", "Price")] {
        model._set(cell, value);
    }
    model._set("E2", ">3");
    model._set("F1", "=DSUM(A1:A3,\"Price\",E1)");
    model._set("F2", "=DCOUNTA(A1:A3,\"Price\",$E$1)");
    model._set("F3", "=DSUM(A1:A1,\"Price\",E1:E2)");
    model._set("F4", "=DSUM(A1:A3,\"Price\",E1:E2)");
    model.evaluate();
    assert_eq!(model._get_text("F1"), *"#VALUE!");
    assert_eq!(model._get_text("F2"), *"#VALUE!");
    assert_eq!(model._get_text("F3"), *"#VALUE!");
    assert_eq!(model._get_text("F4"), *"4");
}

/// Criteria headers name database columns by their text (a number by its coerced text), each
/// criteria column keeps its own header, and a header that names no column makes its conditions
/// computed criteria (probes/db-header-criteria-shapes, probes/db-computed-criteria-shapes).
#[test]
fn criteria_headers_and_computed_criteria() {
    let mut model = new_empty_model();
    model._set("A1", "Name");
    model._set("B1", "Qty");
    model._set("C1", "0.5");
    for row in 2..=6 {
        model._set(&format!("A{row}"), &format!("n{row}"));
        model._set(&format!("B{row}"), &(row - 1).to_string());
        model._set(&format!("C{row}"), &(10 * (row - 1)).to_string());
    }
    // An unknown header over a blank cell is ignored; the second column still tests Qty.
    model._set("H1", "Zzz");
    model._set("I1", "Qty");
    model._set("I2", ">3");
    // The text "0.5" names the number header 0.5.
    model._set("J1", "'0.5");
    model._set("J2", ">20");
    // Computed criteria under a blank or unknown header: TRUE, "TRUE" and 1 hold, ">2" does not.
    model._set("K2", "TRUE");
    model._set("L1", "Zzz");
    model._set("L2", ">2");
    model._set("M1", "Zzz");
    model._set("M2", "'TRUE");
    // A formula condition is evaluated again for each record.
    model._set("N2", "=B2>2");
    // A blank criteria row matches every record.
    model._set("O1", "Qty");
    model._set("O2", ">4");
    model._set("F1", "=DSUM(A1:C6,2,H1:I2)");
    model._set("F2", "=DSUM(A1:C6,2,J1:J2)");
    model._set("F3", "=DSUM(A1:C6,2,K1:K2)");
    model._set("F4", "=DSUM(A1:C6,2,L1:L2)");
    model._set("F5", "=DSUM(A1:C6,2,M1:M2)");
    model._set("F6", "=DSUM(A1:C6,2,N1:N2)");
    model._set("F7", "=DSUM(A1:C6,2,O1:O3)");
    model.evaluate();
    assert_eq!(model._get_text("F1"), *"9");
    assert_eq!(model._get_text("F2"), *"12");
    assert_eq!(model._get_text("F3"), *"15");
    assert_eq!(model._get_text("F4"), *"0");
    assert_eq!(model._get_text("F5"), *"15");
    assert_eq!(model._get_text("F6"), *"12");
    assert_eq!(model._get_text("F7"), *"15");
}

/// A formula condition that runs a D-function over its own criteria is bounded (directive 5).
#[test]
fn self_referencing_formula_condition_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "Qty");
    for row in 2..=6 {
        model._set(&format!("A{row}"), &row.to_string());
    }
    model._set("E2", "=DSUM($A$1:$A$6,1,$E$1:$E$2)>0");
    model._set("F1", "=DSUM(A1:A6,1,E1:E2)");
    model.evaluate();
    // The nested condition in E2 is not re-evaluated (one level only, a DEVIATION in
    // docs/excel-semantics/statistics.md), so it fails for every record and nothing is summed.
    assert_eq!(model._get_text("F1"), *"0");
}

/// A cell first computed while a formula condition is re-evaluated runs its own D-functions in
/// full: the one-level guard belongs to one formula run (T-1390 fidelity review).
#[test]
fn cells_reached_from_a_formula_condition_evaluate_in_full() {
    let mut model = new_empty_model();
    model._set("A1", "Qty");
    for row in 2..=6 {
        model._set(&format!("A{row}"), &(row - 1).to_string());
        model._set(&format!("K{row}"), "=DCOUNT($A$1:$A$6,1,$G$1:$G$2)");
    }
    model._set("G2", "=A2>2");
    model._set("H2", "=K2>0");
    // The database spans column K so that `=K2>0` reads its record area
    // (probes/db-computed-criteria-cross-sheet: a reference outside the columns matches nothing).
    model._set("M1", "=DCOUNT(A1:K6,1,H1:H2)");
    model._set("N1", "=DCOUNT(A1:A6,1,G1:G2)");
    model.evaluate();
    for row in 2..=6 {
        assert_eq!(model._get_text(&format!("K{row}")), *"3");
    }
    assert_eq!(model._get_text("M1"), *"5");
    assert_eq!(model._get_text("N1"), *"3");
}

/// A record whose moved formula reference lands on the database's header row does not match:
/// over A2:D9 (header row 2) `=B2>3` reads B2 for the first record, and that record fails
/// (gen-m9 4dc761698ca23149 A129, f066968376660329 A292, f61a73cd9af21e6d A289; T-2553).
#[test]
fn formula_condition_on_the_header_row_fails() {
    let mut model = new_empty_model();
    for (row, (qty, price)) in [
        ("Qty", "Price"),
        ("4", "2.5"),
        ("10", "x"),
        ("0", "4"),
        ("2", "TRUE"),
        ("1", "10"),
        ("-4", "0.5"),
        ("7", "100"),
        ("5", "-1"),
    ]
    .into_iter()
    .enumerate()
    {
        model._set(&format!("B{}", row + 1), qty);
        model._set(&format!("C{}", row + 1), price);
    }
    model._set("O1", "Calc");
    model._set("O2", "=B2>3");
    model._set("F1", "=DPRODUCT(B2:C9,2,O1:O2)");
    model._set("F2", "=DMAX(B2:C9,2,O1:O2)");
    model._set("F3", "=DCOUNTA(B2:C9,2,O1:O2)");
    // The same condition over B1:C9 starts at its first record: rows 2, 3, 8 and 9 match.
    model._set("F4", "=DCOUNTA(B1:C9,2,O1:O2)");
    // An absolute row stays put and is not checked: every record reads B2 = 4 and matches.
    model._set("P1", "Calc");
    model._set("P2", "=B$2>3");
    model._set("F5", "=DCOUNTA(B2:C9,2,P1:P2)");
    model.evaluate();
    // Rows 4 (B3 = 10) and 9 (B8 = 7) match; row 3 (B2 = 4, the header row) does not.
    assert_eq!(model._get_text("F1"), *"-4");
    assert_eq!(model._get_text("F2"), *"4");
    assert_eq!(model._get_text("F3"), *"2");
    assert_eq!(model._get_text("F4"), *"4");
    assert_eq!(model._get_text("F5"), *"7");
}
