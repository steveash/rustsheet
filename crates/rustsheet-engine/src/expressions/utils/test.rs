use super::*;

#[test]
fn test_column_to_number() {
    assert_eq!(column_to_number("A"), Ok(1));
    assert_eq!(column_to_number("Z"), Ok(26));
    assert_eq!(column_to_number("AA"), Ok(27));
    assert_eq!(column_to_number("AB"), Ok(28));
    assert_eq!(column_to_number("XFD"), Ok(16_384));
    assert_eq!(column_to_number("XFD"), Ok(LAST_COLUMN));

    assert_eq!(
        column_to_number("XFE"),
        Err("Column is not valid.".to_string())
    );
    assert_eq!(
        column_to_number(""),
        Err("Column identifier cannot be empty.".to_string())
    );
    assert_eq!(
        column_to_number("💥"),
        Err("Column identifier must be ASCII.".to_string())
    );
    assert_eq!(
        column_to_number("A1"),
        Err("Column identifier can use only A-Z characters".to_string())
    );
    assert_eq!(
        column_to_number("ab"),
        Err("Column identifier can use only A-Z characters".to_string())
    );
    // Long letter runs overflow i32: they must be rejected, not wrap (T-0021). Wrapping,
    // `NKJDCLBT` would come out as column 1584 and `NKJDCLBT1` would lex as a reference.
    for column in ["ISERROR", "NKJDCLBT", &"Z".repeat(100)] {
        assert_eq!(
            column_to_number(column),
            Err("Column is not valid.".to_string())
        );
    }
}

#[test]
fn test_is_valid_column() {
    assert!(is_valid_column("A"));
    assert!(is_valid_column("AA"));
    assert!(is_valid_column("XFD"));

    assert!(!is_valid_column("a"));
    assert!(!is_valid_column("aa"));
    assert!(!is_valid_column("xfd"));

    assert!(!is_valid_column("1"));
    assert!(!is_valid_column("-1"));
    assert!(!is_valid_column("XFE"));
    assert!(!is_valid_column(""));
}

#[test]
fn test_number_to_column() {
    assert_eq!(number_to_column(1), Some("A".to_string()));
    assert_eq!(number_to_column(26), Some("Z".to_string()));
    assert_eq!(number_to_column(27), Some("AA".to_string()));
    assert_eq!(number_to_column(28), Some("AB".to_string()));
    assert_eq!(number_to_column(16_384), Some("XFD".to_string()));

    assert_eq!(number_to_column(0), None);
    assert_eq!(number_to_column(16_385), None);
}

#[test]
fn test_references() {
    assert_eq!(
        parse_reference_a1("A1"),
        Some(ParsedReference {
            row: 1,
            column: 1,
            absolute_column: false,
            absolute_row: false
        })
    );
}

#[test]
fn test_references_1() {
    assert_eq!(
        parse_reference_a1("AB$23"),
        Some(ParsedReference {
            row: 23,
            column: 28,
            absolute_column: false,
            absolute_row: true
        })
    );
}

#[test]
fn test_references_2() {
    assert_eq!(
        parse_reference_a1("$AB123"),
        Some(ParsedReference {
            row: 123,
            column: 28,
            absolute_column: true,
            absolute_row: false
        })
    );
}

#[test]
fn test_references_3() {
    assert_eq!(
        parse_reference_a1("$AB$123"),
        Some(ParsedReference {
            row: 123,
            column: 28,
            absolute_column: true,
            absolute_row: true
        })
    );
}

// probes/r1c1-shaped-sheet-names{,-2,-3,-4,-lower,-lower-2} (excel-graph, T-1652): the sheet
// names Excel quotes; A1-shaped ones (R1, C3, RC99999) are quoted by the A1 test instead.
#[test]
fn r1c1_shaped_names() {
    for name in [
        "RC",
        "R",
        "C",
        "rc",
        "r",
        "c",
        "Rc",
        "R1C",
        "R1C1",
        "r1c1",
        "R12C",
        "R1048576C1",
        "R1C16385",
        "R1C0D",
        "R1C99999999999",
        "R1CD",
        "R1C_",
        "R1C1.",
        "R1C.",
        "R2C3.4",
        "R1C1D",
        "R1048576D",
        "R0001D",
        "R01C01",
        "R1_",
        "R1R",
        "C1D",
        "c1d",
        "C1_",
        "C1R1",
        "c1r1",
        "C16384D",
        "RC1D",
        "RC1R",
        "RC16384D",
        "rC1",
        "R1.C1",
        "R1..C1",
        "R1.R1",
        "C1.R1",
        "RC1.C1",
        "R1.C1.5",
        "R1.C1.",
        "R1.C16385",
    ] {
        assert!(is_r1c1_shaped(name), "{name}");
    }
    for name in [
        "R0C1",
        "r0c1",
        "R0C0",
        "R1048577C1",
        "R0",
        "C0",
        "R1048577",
        "R2147483648",
        "C2147483648",
        "RC2147483648",
        "RCR",
        "rcr",
        "CR",
        "RCD",
        "RD",
        "CD",
        "RC_",
        "R_",
        "C_",
        "RCC",
        "RRC",
        "R1.",
        "R1.5",
        "R1.A",
        "C1.",
        "C1.5",
        "RC1.",
        "RC.",
        "RC.1",
        "R.1",
        "R2.3C",
        "R0D",
        "R0_",
        "R1048577D",
        "C0D",
        "R0C1D",
        "C16385D",
        "RC16385D",
        "RC0D",
        "R1.C1D",
        "R1.RC",
        "R1.R",
        "R1.D.C1",
        "R1.C0",
        "A.C1",
        "X.R1",
        "ABC.R1C1",
        "Q.R",
        "A1B",
        "Data",
    ] {
        assert!(!is_r1c1_shaped(name), "{name}");
    }
}

#[test]
fn test_names() {
    assert!(is_valid_identifier("hola1"));
    assert!(is_valid_identifier("hola_1"));
    assert!(is_valid_identifier("hola.1"));
    assert!(is_valid_identifier("sum_total_"));
    assert!(is_valid_identifier("sum.total"));
    assert!(is_valid_identifier("_hola"));
    assert!(is_valid_identifier("t"));
    assert!(is_valid_identifier("q"));
    assert!(is_valid_identifier("true_that"));
    assert!(is_valid_identifier("true1"));

    // weird names apparently  valid in Excel
    assert!(is_valid_identifier("_"));
    assert!(is_valid_identifier("\\hola1"));
    assert!(is_valid_identifier("__"));
    assert!(is_valid_identifier("_."));
    assert!(is_valid_identifier("_1"));
    assert!(is_valid_identifier("\\."));
    assert!(is_valid_identifier("truñe"));
    // A first letter of any script (T-1387).
    for name in ["é", "Ärger", "ß"] {
        assert!(is_valid_identifier(name), "{name}");
    }

    // invalid
    assert!(!is_valid_identifier("true"));
    assert!(!is_valid_identifier("false"));
    assert!(!is_valid_identifier("SUM THAT"));
    assert!(!is_valid_identifier("A23"));
    assert!(!is_valid_identifier("R1C1"));
    assert!(!is_valid_identifier("R23C"));
    assert!(!is_valid_identifier("R"));
    assert!(!is_valid_identifier("c"));
    // Graph rejects a workbook defining these (probes/r1c1-name-*, T-1652); RCR is fine.
    for name in ["RC", "rc", "R1C", "R12C", "C1D", "R1C1D", "R1C16385"] {
        assert!(!is_valid_identifier(name), "{name}");
    }
    assert!(is_valid_identifier("RCR"));
    assert!(is_valid_identifier("CR"));
    assert!(!is_valid_identifier("1true"));

    assert!(!is_valid_identifier("test€"));
    assert!(!is_valid_identifier("tr&ue"));

    assert!(!is_valid_identifier("LOG10"));
}

#[test]
fn test_a1_identifier_allows_r_and_c() {
    // is_valid_a1_identifier permits "R" and "C" (valid LAMBDA/LET variable names).
    assert!(is_valid_a1_identifier("A\\B"));
    assert!(is_valid_a1_identifier("\\name"));
    assert!(is_valid_a1_identifier("R"));
    assert!(is_valid_a1_identifier("r"));
    assert!(is_valid_a1_identifier("C"));
    assert!(is_valid_a1_identifier("c"));
    // And R1C1 references: `LET(R1C1,5,R1C1*2)` is 10 (probes/r1c1-let-name-*, T-1652).
    for name in ["R1C1", "R5C7", "R12C", "R0C1", "RC", "R1C1D"] {
        assert!(is_valid_a1_identifier(name), "{name}");
    }
    // All other is_valid_identifier rules still apply.
    assert!(!is_valid_a1_identifier("A23"));
    assert!(!is_valid_a1_identifier("true"));
}
