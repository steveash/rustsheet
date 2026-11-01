#![allow(clippy::unwrap_used)]

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::lexer::LexerError;
use crate::expressions::{
    lexer::{Lexer, LexerMode},
    token::TokenType::*,
    types::ParsedReference,
};
use crate::language::get_language;
use crate::locale::get_locale;

fn new_lexer(formula: &str) -> Lexer {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    Lexer::new(formula, LexerMode::A1, locale, language)
}

#[test]
fn test_range() {
    let mut lx = new_lexer("C4:D4");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 3,
                row: 4,
                absolute_column: false,
                absolute_row: false,
            },
            right: ParsedReference {
                column: 4,
                row: 4,
                absolute_column: false,
                absolute_row: false,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_absolute_column() {
    let mut lx = new_lexer("$A1:B$4");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 1,
                row: 1,
                absolute_column: true,
                absolute_row: false,
            },
            right: ParsedReference {
                column: 2,
                row: 4,
                absolute_column: false,
                absolute_row: true,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_with_sheet() {
    let mut lx = new_lexer("Sheet1!A1:B4");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: Some("Sheet1".to_string()),
            left: ParsedReference {
                column: 1,
                row: 1,
                absolute_column: false,
                absolute_row: false,
            },
            right: ParsedReference {
                column: 2,
                row: 4,
                absolute_column: false,
                absolute_row: false,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_with_sheet_with_space() {
    let mut lx = new_lexer("'New sheet'!$A$1:B44");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: Some("New sheet".to_string()),
            left: ParsedReference {
                column: 1,
                row: 1,
                absolute_column: true,
                absolute_row: true,
            },
            right: ParsedReference {
                column: 2,
                row: 44,
                absolute_column: false,
                absolute_row: false,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

/// Excel drops whitespace after a sheet prefix's `!` (probes/t2776-ws-after-bang).
#[test]
fn test_range_with_sheet_whitespace_after_bang() {
    for (formula, sheet) in [("Sheet1! A1", "Sheet1"), ("'New sheet'!  A1", "New sheet")] {
        let mut lx = new_lexer(formula);
        assert_eq!(
            lx.next_token(),
            Reference {
                sheet: Some(sheet.to_string()),
                row: 1,
                column: 1,
                absolute_column: false,
                absolute_row: false,
            }
        );
        assert_eq!(lx.next_token(), EOF);
    }
}

#[test]
fn test_range_error() {
    let mut lx = new_lexer("'Sheet 1'!3.4:5");
    assert_eq!(
        lx.next_token(),
        Illegal(LexerError {
            position: 10,
            message: "Expecting reference in range".to_string(),
        })
    );
    assert_eq!(lx.next_token(), EOF);

    let mut lx = new_lexer("'Sheet 1'!3:A2");
    assert_eq!(
        lx.next_token(),
        Illegal(LexerError {
            position: 14,
            message: "Error parsing Range".to_string()
        })
    );
    assert_eq!(lx.next_token(), EOF);

    let mut lx = new_lexer("'Sheet 1'!3:");
    assert_eq!(
        lx.next_token(),
        Illegal(LexerError {
            position: 12,
            message: "Error parsing Range".to_string()
        })
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_column() {
    let mut lx = new_lexer("C:D");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 3,
                row: 1,
                absolute_column: false,
                absolute_row: true,
            },
            right: ParsedReference {
                column: 4,
                row: LAST_ROW,
                absolute_column: false,
                absolute_row: true,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_column_out_of_range() {
    let mut lx = new_lexer("C:XFE");
    assert_eq!(
        lx.next_token(),
        Illegal(LexerError {
            position: 5,
            message: "Column is not valid.".to_string(),
        })
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_column_absolute1() {
    let mut lx = new_lexer("$C:D");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 3,
                row: 1,
                absolute_column: true,
                absolute_row: true,
            },
            right: ParsedReference {
                column: 4,
                row: LAST_ROW,
                absolute_column: false,
                absolute_row: true,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_column_absolute2() {
    let mut lx = new_lexer("$C:$AA");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 3,
                row: 1,
                absolute_column: true,
                absolute_row: true,
            },
            right: ParsedReference {
                column: 27,
                row: LAST_ROW,
                absolute_column: true,
                absolute_row: true,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_rows() {
    let mut lx = new_lexer("3:5");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 1,
                row: 3,
                absolute_column: true,
                absolute_row: false,
            },
            right: ParsedReference {
                column: LAST_COLUMN,
                row: 5,
                absolute_column: true,
                absolute_row: false,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_rows_absolute1() {
    let mut lx = new_lexer("$3:5");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 1,
                row: 3,
                absolute_column: true,
                absolute_row: true,
            },
            right: ParsedReference {
                column: LAST_COLUMN,
                row: 5,
                absolute_column: true,
                absolute_row: false,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_rows_absolute2() {
    let mut lx = new_lexer("$3:$55");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 1,
                row: 3,
                absolute_column: true,
                absolute_row: true,
            },
            right: ParsedReference {
                column: LAST_COLUMN,
                row: 55,
                absolute_column: true,
                absolute_row: true,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_column_sheet() {
    let mut lx = new_lexer("Sheet1!C:D");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: Some("Sheet1".to_string()),
            left: ParsedReference {
                column: 3,
                row: 1,
                absolute_column: false,
                absolute_row: true,
            },
            right: ParsedReference {
                column: 4,
                row: LAST_ROW,
                absolute_column: false,
                absolute_row: true,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_column_sheet_absolute() {
    let mut lx = new_lexer("Sheet1!$C:$D");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: Some("Sheet1".to_string()),
            left: ParsedReference {
                column: 3,
                row: 1,
                absolute_column: true,
                absolute_row: true,
            },
            right: ParsedReference {
                column: 4,
                row: LAST_ROW,
                absolute_column: true,
                absolute_row: true,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);

    let mut lx = new_lexer("'Woops ans'!$C:$D");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: Some("Woops ans".to_string()),
            left: ParsedReference {
                column: 3,
                row: 1,
                absolute_column: true,
                absolute_row: true,
            },
            right: ParsedReference {
                column: 4,
                row: LAST_ROW,
                absolute_column: true,
                absolute_row: true,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_rows_sheet() {
    let mut lx = new_lexer("'A new sheet'!3:5");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: Some("A new sheet".to_string()),
            left: ParsedReference {
                column: 1,
                row: 3,
                absolute_column: true,
                absolute_row: false,
            },
            right: ParsedReference {
                column: LAST_COLUMN,
                row: 5,
                absolute_column: true,
                absolute_row: false,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
    let mut lx = new_lexer("Sheet12!3:5");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: Some("Sheet12".to_string()),
            left: ParsedReference {
                column: 1,
                row: 3,
                absolute_column: true,
                absolute_row: false,
            },
            right: ParsedReference {
                column: LAST_COLUMN,
                row: 5,
                absolute_column: true,
                absolute_row: false,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}
// Non ranges

#[test]
fn test_non_range_variable_name() {
    let mut lx = new_lexer("AB");
    assert_eq!(lx.next_token(), Ident("AB".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_non_range_invalid_variable_name() {
    let mut lx = new_lexer("$AB");
    assert!(matches!(lx.next_token(), Illegal(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_non_range_invalid_variable_name_a03() {
    let mut lx = new_lexer("A03");
    assert_eq!(
        lx.next_token(),
        Reference {
            sheet: None,
            row: 3,
            column: 1,
            absolute_column: false,
            absolute_row: false
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_non_range_invalid_variable_name_sheet1_a03() {
    let mut lx = new_lexer("Sheet1!A03");
    assert_eq!(
        lx.next_token(),
        Reference {
            sheet: Some("Sheet1".to_string()),
            row: 3,
            column: 1,
            absolute_column: false,
            absolute_row: false
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_rows_with_0() {
    let mut lx = new_lexer("03:05");
    assert_eq!(
        lx.next_token(),
        Range {
            sheet: None,
            left: ParsedReference {
                column: 1,
                row: 3,
                absolute_column: true,
                absolute_row: false,
            },
            right: ParsedReference {
                column: LAST_COLUMN,
                row: 5,
                absolute_column: true,
                absolute_row: false,
            }
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_incomplete_row() {
    let mut lx = new_lexer("R[");
    lx.set_lexer_mode(LexerMode::R1C1);
    assert!(matches!(lx.next_token(), Illegal(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_range_incomplete_column() {
    let mut lx = new_lexer("R[3][");
    lx.set_lexer_mode(LexerMode::R1C1);
    assert!(matches!(lx.next_token(), Illegal(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn range_operator() {
    let mut lx = new_lexer("A1:OFFSET(B1,1,2)");
    lx.set_lexer_mode(LexerMode::A1);
    assert!(matches!(lx.next_token(), Reference { .. }));
    assert!(matches!(lx.next_token(), Colon));
    assert!(matches!(lx.next_token(), Ident(_)));
    assert!(matches!(lx.next_token(), LeftParenthesis));
    assert!(matches!(lx.next_token(), Reference { .. }));
    assert_eq!(lx.next_token(), Comma);
    assert!(matches!(lx.next_token(), Number(_)));
    assert_eq!(lx.next_token(), Comma);
    assert!(matches!(lx.next_token(), Number(_)));
    assert!(matches!(lx.next_token(), RightParenthesis));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_sheet_qualified_malformed_corner_pairs_are_names() {
    // Excel reads these as a sheet-scoped name, `#NAME?` (references.md, T-0145).
    for formula in [
        "Data!A:Data!B",
        "Data!A1:Data!B",
        "Data!A:Data!B1",
        "Data!A:Data!A",
        "Data!A1:B",
        "'My Data'!A:'My Data'!B",
    ] {
        let mut lx = new_lexer(formula);
        // The sheet prefix stays in the identifier so the formula text round-trips.
        assert_eq!(lx.next_token(), Ident(formula.to_string()), "{formula}");
        assert_eq!(lx.next_token(), EOF, "{formula}");
    }
}

#[test]
fn test_sheet_qualified_off_grid_cells_are_names() {
    // A row of 0 or past the last row is not a cell: Excel reads a name, `#NAME?`
    // (probes/aggregate-range-function-num-grid A22, A24, A25, T-0296).
    for formula in ["Data!H0", "Data!H00", "Data!A1048577", "'My Data'!H0"] {
        let mut lx = new_lexer(formula);
        assert_eq!(lx.next_token(), Ident(formula.to_string()), "{formula}");
        assert_eq!(lx.next_token(), EOF, "{formula}");
    }
    // `Data!H0:I1` is that name, then the range operator (A16, A29).
    let mut lx = new_lexer("Data!H0:I1");
    assert_eq!(lx.next_token(), Ident("Data!H0".to_string()));
    assert_eq!(lx.next_token(), Colon);
    assert!(matches!(lx.next_token(), Reference { row: 1, .. }));
    assert_eq!(lx.next_token(), EOF);
    // An off-grid second corner ends the token at the first cell; the corner is then a name
    // (probes/off-grid-corners A1:A6).
    for formula in ["Data!A1:B0", "A1:B0", "Data!A1:B1048577"] {
        let mut lx = new_lexer(formula);
        assert!(
            matches!(
                lx.next_token(),
                Reference {
                    row: 1,
                    column: 1,
                    ..
                }
            ),
            "{formula}"
        );
        assert_eq!(lx.next_token(), Colon, "{formula}");
        assert!(matches!(lx.next_token(), Ident(_)), "{formula}");
        assert_eq!(lx.next_token(), EOF, "{formula}");
    }
    // A leading zero is still a cell: `Data!H01` is H1 (A27).
    let mut lx = new_lexer("Data!H01");
    assert!(matches!(
        lx.next_token(),
        Reference {
            row: 1,
            column: 8,
            ..
        }
    ));
}

#[test]
fn test_sheet_qualified_valid_corner_pairs_stay_ranges() {
    // Unprobed shapes keep their original parse error.
    for formula in [
        "Data!A1:INDEX(Data!B:B,2)",
        "Data!A1:Data!$B",
        "Data!A1:Data!XFE1",
    ] {
        let mut lx = new_lexer(formula);
        assert!(!matches!(lx.next_token(), Ident(_)), "{formula}");
    }
    for formula in ["Data!A:B", "Data!A1:B2", "Data!1:2", "Data!A1:Data!B2"] {
        let mut lx = new_lexer(formula);
        assert!(
            !matches!(lx.next_token(), Ident(_) | Illegal(_)),
            "{formula}"
        );
    }
}

/// Reference lexing reads the column and row without building their text (T-3384). These tokens,
/// error positions included, were recorded from the lexer before that change: valid and
/// off-grid columns and rows, overflowing letter and digit runs, lower case, `$`, sheet prefixes,
/// ranges and their corners.
#[test]
fn reference_tokens_are_unchanged() {
    let expected = [
        ("A1", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 EOF@2"),
        ("a1", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 EOF@2"),
        ("$A$1", "Reference { sheet: None, row: 1, column: 1, absolute_column: true, absolute_row: true }@4 EOF@4"),
        ("XFD1048576", "Reference { sheet: None, row: 1048576, column: 16384, absolute_column: false, absolute_row: false }@10 EOF@10"),
        ("XFE1", "Ident(\"XFE1\")@4 EOF@4"),
        ("A1048577", "Ident(\"A1048577\")@8 EOF@8"),
        ("A0", "Ident(\"A0\")@2 EOF@2"),
        ("A00", "Ident(\"A00\")@3 EOF@3"),
        ("H01", "Reference { sheet: None, row: 1, column: 8, absolute_column: false, absolute_row: false }@3 EOF@3"),
        ("A2147483647", "Ident(\"A2147483647\")@11 EOF@11"),
        ("A2147483648", "Ident(\"A2147483648\")@11 EOF@11"),
        ("A99999999999999", "Ident(\"A99999999999999\")@15 EOF@15"),
        ("AAAAAAAAAAAA1", "Ident(\"AAAAAAAAAAAA1\")@13 EOF@13"),
        ("'S'!A1", "Reference { sheet: Some(\"S\"), row: 1, column: 1, absolute_column: false, absolute_row: false }@6 EOF@6"),
        ("'S'!XFE1", "Ident(\"S!XFE1\")@8 EOF@8"),
        ("'S'!A0", "Ident(\"S!A0\")@6 EOF@6"),
        ("'S'!AAAAAAAAAAAAAAA1", "Ident(\"S!AAAAAAAAAAAAAAA1\")@20 EOF@20"),
        ("'S'!A99999999999", "Ident(\"S!A99999999999\")@16 EOF@16"),
        ("'S'!A1:B2", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 1, row: 1, absolute_column: false, absolute_row: false }, right: ParsedReference { column: 2, row: 2, absolute_column: false, absolute_row: false } }@9 EOF@9"),
        ("'S'!A1:XFE2", "Illegal(LexerError { position: 7, message: \"Expecting reference in range\" })@11 EOF@11"),
        ("'S'!B9:A1", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 2, row: 9, absolute_column: false, absolute_row: false }, right: ParsedReference { column: 1, row: 1, absolute_column: false, absolute_row: false } }@9 EOF@9"),
        ("'S'!$A$1:$b2", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: true }, right: ParsedReference { column: 2, row: 2, absolute_column: true, absolute_row: false } }@12 EOF@12"),
        ("Sheet1!A1:B", "Ident(\"Sheet1!A1:B\")@11 EOF@11"),
        ("'S'!A", "Ident(\"S!A\")@5 EOF@5"),
        ("'S'!$A$", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@7 EOF@7"),
        ("'S'!$", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@5 EOF@5"),
        ("'S'!1", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@5 EOF@5"),
        ("'S'!1:2", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: false }, right: ParsedReference { column: 16384, row: 2, absolute_column: true, absolute_row: false } }@7 EOF@7"),
        ("'S'!A1048576:A1048577", "Reference { sheet: Some(\"S\"), row: 1048576, column: 1, absolute_column: false, absolute_row: false }@12 Colon@13 Ident(\"A1048577\")@21 EOF@21"),
        ("Data!ISERRORS1", "Ident(\"Data!ISERRORS1\")@14 EOF@14"),
        ("Data!ISERROR(", "Illegal(LexerError { position: 5, message: \"Expecting reference in range\" })@13 EOF@13"),
        ("$A$1:$B$2", "Range { sheet: None, left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: true }, right: ParsedReference { column: 2, row: 2, absolute_column: true, absolute_row: true } }@9 EOF@9"),
        ("A1:B2 C3", "Range { sheet: None, left: ParsedReference { column: 1, row: 1, absolute_column: false, absolute_row: false }, right: ParsedReference { column: 2, row: 2, absolute_column: false, absolute_row: false } }@5 Reference { sheet: None, row: 3, column: 3, absolute_column: false, absolute_row: false }@8 EOF@8"),
        ("'S'!a1:b2", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 1, row: 1, absolute_column: false, absolute_row: false }, right: ParsedReference { column: 2, row: 2, absolute_column: false, absolute_row: false } }@9 EOF@9"),
        ("'S'!ZZZZZZ1", "Ident(\"S!ZZZZZZ1\")@11 EOF@11"),
        ("'S'!A2147483647", "Ident(\"S!A2147483647\")@15 EOF@15"),
        // Absolute corners and the error paths of a reference read in a range.
        ("$A", "Illegal(LexerError { position: 0, message: \"Expecting reference in range\" })@2 EOF@2"),
        ("$A$", "Illegal(LexerError { position: 0, message: \"Expecting reference in range\" })@3 EOF@3"),
        ("A1:$B", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 Colon@3 Illegal(LexerError { position: 3, message: \"Expecting reference in range\" })@5 EOF@5"),
        ("$A$1:$B", "Illegal(LexerError { position: 5, message: \"Expecting reference in range\" })@7 EOF@7"),
        ("'S'!$A1:$B", "Illegal(LexerError { position: 8, message: \"Expecting reference in range\" })@10 EOF@10"),
        ("A1:B", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 Colon@3 Ident(\"B\")@4 EOF@4"),
        ("$A1:B$", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@6 EOF@6"),
        ("A1:$B$", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 Colon@3 Illegal(LexerError { position: 3, message: \"Expecting reference in range\" })@6 EOF@6"),
        ("$A1:$B$0", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@8 EOF@8"),
        ("A1:$B$2147483648", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 Colon@3 Illegal(LexerError { position: 3, message: \"Expecting reference in range\" })@16 EOF@16"),
        ("A1:$XFE$1", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 Colon@3 Illegal(LexerError { position: 3, message: \"Expecting reference in range\" })@9 EOF@9"),
        ("A1:$AAAAAAAAAAAAAA$1", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 Colon@3 Illegal(LexerError { position: 3, message: \"Expecting reference in range\" })@20 EOF@20"),
        ("$XFE$1", "Illegal(LexerError { position: 0, message: \"Expecting reference in range\" })@6 EOF@6"),
        ("$AAAAAAAAAAAAAAAA$1", "Illegal(LexerError { position: 0, message: \"Expecting reference in range\" })@19 EOF@19"),
        ("$A$0", "Illegal(LexerError { position: 0, message: \"Expecting reference in range\" })@4 EOF@4"),
        ("$A$2147483648", "Illegal(LexerError { position: 0, message: \"Expecting reference in range\" })@13 EOF@13"),
        ("$A$1048577", "Illegal(LexerError { position: 0, message: \"Expecting reference in range\" })@10 EOF@10"),
        ("'S'!$A$", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@7 EOF@7"),
        ("'S'!$A$0", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@8 EOF@8"),
        ("'S'!$XFE$1", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@10 EOF@10"),
        ("'S'!$A$2147483648", "Illegal(LexerError { position: 4, message: \"Expecting reference in range\" })@17 EOF@17"),
        ("A1:$A$", "Reference { sheet: None, row: 1, column: 1, absolute_column: false, absolute_row: false }@2 Colon@3 Illegal(LexerError { position: 3, message: \"Expecting reference in range\" })@6 EOF@6"),
        ("'S'!A1:$B$", "Illegal(LexerError { position: 7, message: \"Expecting reference in range\" })@10 EOF@10"),
        ("'S'!A1:$XFE$2", "Illegal(LexerError { position: 7, message: \"Expecting reference in range\" })@13 EOF@13"),
        ("'S'!A1:$AAAAAAAAAAAAAAAAAAA$2", "Illegal(LexerError { position: 7, message: \"Expecting reference in range\" })@29 EOF@29"),
        ("Sheet1!$A$2147483648", "Illegal(LexerError { position: 7, message: \"Expecting reference in range\" })@20 EOF@20"),
        ("$A1", "Reference { sheet: None, row: 1, column: 1, absolute_column: true, absolute_row: false }@3 EOF@3"),
        ("$1", "Illegal(LexerError { position: 0, message: \"Expecting reference in range\" })@2 EOF@2"),
        // Row and column ranges, and their error messages.
        ("1:2", "Range { sheet: None, left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: false }, right: ParsedReference { column: 16384, row: 2, absolute_column: true, absolute_row: false } }@3 EOF@3"),
        ("$1:$2", "Range { sheet: None, left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: true }, right: ParsedReference { column: 16384, row: 2, absolute_column: true, absolute_row: true } }@5 EOF@5"),
        ("0:5", "Range { sheet: None, left: ParsedReference { column: 1, row: 0, absolute_column: true, absolute_row: false }, right: ParsedReference { column: 16384, row: 5, absolute_column: true, absolute_row: false } }@3 EOF@3"),
        ("1048576:1048576", "Range { sheet: None, left: ParsedReference { column: 1, row: 1048576, absolute_column: true, absolute_row: false }, right: ParsedReference { column: 16384, row: 1048576, absolute_column: true, absolute_row: false } }@15 EOF@15"),
        ("1048577:2", "Illegal(LexerError { position: 9, message: \"Row too large in reference\" })@9 EOF@9"),
        ("2:1048577", "Illegal(LexerError { position: 9, message: \"Row too large in reference\" })@9 EOF@9"),
        ("99999999999:2", "Illegal(LexerError { position: 13, message: \"Failed parsing row 99999999999\" })@13 EOF@13"),
        ("2:99999999999", "Illegal(LexerError { position: 13, message: \"Failed parsing row 99999999999\" })@13 EOF@13"),
        ("A:B", "Range { sheet: None, left: ParsedReference { column: 1, row: 1, absolute_column: false, absolute_row: true }, right: ParsedReference { column: 2, row: 1048576, absolute_column: false, absolute_row: true } }@3 EOF@3"),
        ("$A:$B", "Range { sheet: None, left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: true }, right: ParsedReference { column: 2, row: 1048576, absolute_column: true, absolute_row: true } }@5 EOF@5"),
        ("a:b", "Range { sheet: None, left: ParsedReference { column: 1, row: 1, absolute_column: false, absolute_row: true }, right: ParsedReference { column: 2, row: 1048576, absolute_column: false, absolute_row: true } }@3 EOF@3"),
        ("A:XFD", "Range { sheet: None, left: ParsedReference { column: 1, row: 1, absolute_column: false, absolute_row: true }, right: ParsedReference { column: 16384, row: 1048576, absolute_column: false, absolute_row: true } }@5 EOF@5"),
        ("A:XFE", "Illegal(LexerError { position: 5, message: \"Column is not valid.\" })@5 EOF@5"),
        ("XFE:A", "Ident(\"XFE\")@3 Colon@4 Ident(\"A\")@5 EOF@5"),
        ("AAAAAAAAAAAAAAAA:B", "Ident(\"AAAAAAAAAAAAAAAA\")@16 Colon@17 Ident(\"B\")@18 EOF@18"),
        ("A:AAAAAAAAAAAAAAAA", "Illegal(LexerError { position: 18, message: \"Column is not valid.\" })@18 EOF@18"),
        ("'S'!$A:$A", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: true }, right: ParsedReference { column: 1, row: 1048576, absolute_column: true, absolute_row: true } }@9 EOF@9"),
        ("'S'!1:3", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: false }, right: ParsedReference { column: 16384, row: 3, absolute_column: true, absolute_row: false } }@7 EOF@7"),
        ("'S'!99999999999:3", "Illegal(LexerError { position: 17, message: \"Failed parsing row 99999999999\" })@17 EOF@17"),
        ("'S'!A:XFE", "Ident(\"S!A\")@5 Colon@6 Ident(\"XFE\")@9 EOF@9"),
        ("'S'!A1B:C", "Ident(\"S!A1B\")@7 Colon@8 Ident(\"C\")@9 EOF@9"),
        ("'S'!1A:2", "Illegal(LexerError { position: 8, message: \"Error parsing Range\" })@8 EOF@8"),
        ("'S'!A:2", "Ident(\"S!A\")@5 Colon@6 Number(2.0)@7 EOF@7"),
        ("'S'!1:B", "Illegal(LexerError { position: 7, message: \"Error parsing Range\" })@7 EOF@7"),
        ("'S'!:B", "Illegal(LexerError { position: 6, message: \"Column identifier cannot be empty.\" })@6 EOF@6"),
        ("'S'!A:", "Ident(\"S!A\")@5 Colon@6 EOF@6"),
        ("'S'!1:", "Illegal(LexerError { position: 6, message: \"Error parsing Range\" })@6 EOF@6"),
        ("Sheet1!$A:$C", "Range { sheet: Some(\"Sheet1\"), left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: true }, right: ParsedReference { column: 3, row: 1048576, absolute_column: true, absolute_row: true } }@12 EOF@12"),
        ("Sheet1!3:$5", "Range { sheet: Some(\"Sheet1\"), left: ParsedReference { column: 1, row: 3, absolute_column: true, absolute_row: false }, right: ParsedReference { column: 16384, row: 5, absolute_column: true, absolute_row: true } }@11 EOF@11"),
        ("'S'!$:$A", "Illegal(LexerError { position: 8, message: \"Column identifier cannot be empty.\" })@8 EOF@8"),
        ("'S'!0001:2", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 1, row: 1, absolute_column: true, absolute_row: false }, right: ParsedReference { column: 16384, row: 2, absolute_column: true, absolute_row: false } }@10 EOF@10"),
        ("'S'!A:B C:D", "Range { sheet: Some(\"S\"), left: ParsedReference { column: 1, row: 1, absolute_column: false, absolute_row: true }, right: ParsedReference { column: 2, row: 1048576, absolute_column: false, absolute_row: true } }@7 Range { sheet: None, left: ParsedReference { column: 3, row: 1, absolute_column: false, absolute_row: true }, right: ParsedReference { column: 4, row: 1048576, absolute_column: false, absolute_row: true } }@11 EOF@11"),
    ];
    for (formula, tokens) in expected {
        let mut lexer = new_lexer(formula);
        let mut read = Vec::new();
        loop {
            let token = lexer.next_token();
            let end = token == EOF;
            read.push(format!("{token:?}@{}", lexer.get_position()));
            if end || read.len() > 20 {
                break;
            }
        }
        assert_eq!(read.join(" "), tokens, "{formula}");
    }
}
