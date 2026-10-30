#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::{to_excel_string, to_rc_format};
use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::parser::{Node, Parser};
use crate::expressions::types::CellReferenceRC;
use crate::expressions::utils::{number_to_column, parse_reference_a1};
use crate::types::{Table, TableColumn, TableStyleInfo};

fn create_test_table(
    table_name: &str,
    column_names: &[&str],
    cell_ref: &str,
    row_count: i32,
) -> HashMap<String, Table> {
    let mut table = HashMap::new();
    let mut columns = Vec::new();
    for (id, name) in column_names.iter().enumerate() {
        columns.push(TableColumn {
            id: id as u32,
            name: name.to_string(),
            ..Default::default()
        });
    }
    let init_cell = parse_reference_a1(cell_ref).unwrap();
    let start_row = init_cell.row;
    let start_column = number_to_column(init_cell.column).unwrap();
    let end_column = number_to_column(init_cell.column + column_names.len() as i32).unwrap();
    let end_row = start_row + row_count - 1;

    let area_ref = format!("{start_column}{start_row}:{end_column}{end_row}");

    table.insert(
        table_name.to_string(),
        Table {
            name: table_name.to_string(),
            display_name: table_name.to_string(),
            sheet_name: "Sheet One".to_string(),
            reference: area_ref,
            totals_row_count: 0,
            header_row_count: 1,
            header_row_dxf_id: None,
            data_dxf_id: None,
            columns,
            style_info: TableStyleInfo {
                ..Default::default()
            },
            totals_row_dxf_id: None,
            has_filters: false,
            filter_columns: Vec::new(),
        },
    );
    table
}

#[test]
fn simple_table() {
    let worksheets = vec!["Sheet One".to_string(), "Second Sheet".to_string()];

    // This is a table A1:F3, the column F has a formula
    let column_names = ["Jan", "Feb", "Mar", "Apr", "Dec", "Year"];
    let row_count = 3;
    let tables = create_test_table("tblIncome", &column_names, "A1", row_count);

    let mut parser = new_parser(worksheets, vec![], tables);
    // Reference cell is 'Sheet One'!F2
    let cell_reference = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 2,
        column: 6,
    };

    // In a cell of its own table `#This Row` prints in Excel's `@` form (T-3069); the export
    // keeps the long text, which writes table parts (T-2196).
    let formula = "SUM(tblIncome[[#This Row],[Jan]:[Dec]])";
    let t = parser.parse(formula, &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "SUM([@[Jan]:[Dec]])"
    );
    assert_eq!(
        to_excel_string(&t, &cell_reference),
        "SUM(tblIncome[[#This Row],[Jan]:[Dec]])"
    );

    // Cell A3
    let cell_reference = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 4,
        column: 1,
    };
    let formula = "SUBTOTAL(109, tblIncome[Jan])";
    let t = parser.parse(formula, &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "SUBTOTAL(109, tblIncome[Jan])"
    );
    assert_eq!(
        to_excel_string(&t, &cell_reference),
        "SUBTOTAL(109, tblIncome[Jan])"
    );

    // Cell A3 in 'Second Sheet'
    let cell_reference = CellReferenceRC {
        sheet: "Second Sheet".to_string(),
        row: 4,
        column: 1,
    };
    let formula = "SUBTOTAL(109, tblIncome[Jan])";
    let t = parser.parse(formula, &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "SUBTOTAL(109, tblIncome[Jan])"
    );
    assert_eq!(
        to_excel_string(&t, &cell_reference),
        "SUBTOTAL(109, tblIncome[Jan])"
    );
}

/// Whitespace inside a structured reference prints as Excel normalizes it: the runs inside the
/// outer brackets as typed, one space after a comma where any was typed, none before it, and a
/// single padded column without its own brackets (probes/t2786-whitespace-structured-refs-runs
/// F4:F15, T-2801). The printed text parses back to the same text.
#[test]
fn whitespace_inside_a_structured_reference() {
    let tables = create_test_table("T", &["Col", "Other"], "A1", 4);
    let mut parser = new_parser(vec!["Sheet One".to_string()], vec![], tables);
    let cell_reference = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 6,
        column: 5,
    };
    // `T[ [Col] ]` prints `T[ Col ]`, which Excel refuses typed and loads as `#REF!` (T-3069):
    // see `a_padded_bare_column_is_ours_only`.
    let t = parser.parse("SUM(T[ [Col] ])", &cell_reference);
    assert_eq!(to_excel_string(&t, &cell_reference), "SUM(T[ [Col] ])");
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "SUM(T[ Col ])"
    );
    for (typed, printed) in [
        ("SUM(T[ [Col]:[Other] ])", "SUM(T[ [Col]:[Other] ])"),
        ("SUM(T[ [#Data] , [Col] ])", "SUM(T[ [#Data], [Col] ])"),
        ("SUM(T[[#Data], [Col]])", "SUM(T[[#Data], [Col]])"),
        ("SUM(T[[#Data]  ,  [Col]])", "SUM(T[[#Data], [Col]])"),
        (
            "SUM(T[[#Data],[Col]:[Other]])",
            "SUM(T[[#Data],[Col]:[Other]])",
        ),
        ("SUM(T[[Col]:[Other]])", "SUM(T[[Col]:[Other]])"),
        ("SUM( T[Col] )", "SUM( T[Col] )"),
        ("T[#Headers]", "T[#Headers]"),
    ] {
        let t = parser.parse(typed, &cell_reference);
        assert_eq!(to_excel_string(&t, &cell_reference), printed, "{typed}");
        let again = parser.parse(printed, &cell_reference);
        assert_eq!(
            to_excel_string(&again, &cell_reference),
            printed,
            "{printed}"
        );
    }
}

/// An unterminated structured reference is a parse error, never a panic (directive 5): the
/// lexer used to step past the missing `]` and leave its position beyond the formula.
#[test]
fn unterminated_structured_reference_is_a_parse_error() {
    let tables = create_test_table("T", &["Col", "Other"], "A1", 4);
    let mut parser = new_parser(vec!["Sheet One".to_string()], vec![], tables);
    let cell_reference = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 6,
        column: 5,
    };
    for formula in [
        "SUM(T[[Col)",
        "T[[Col",
        "T[[A]:[B",
        "T[[#Data],[Col",
        "T[ [Col",
        "T[ Col ",
        "T[@[Col",
        "T[@Col",
    ] {
        let t = parser.parse(formula, &cell_reference);
        assert!(matches!(t, Node::ParseErrorKind { .. }), "{formula}: {t:?}");
    }
}

#[test]
fn malformed_table_reference_is_ref_error_not_a_panic() {
    // The table metadata comes from the workbook, which is untrusted (directive 5).
    let worksheets = vec!["Sheet One".to_string()];
    let mut tables = create_test_table("tblIncome", &["Jan", "Feb"], "A1", 3);
    if let Some(table) = tables.get_mut("tblIncome") {
        table.reference = "not a range".to_string();
    }
    let mut parser = new_parser(worksheets, vec![], tables);
    let cell_reference = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 2,
        column: 6,
    };
    let t = parser.parse("SUM(tblIncome[Jan])", &cell_reference);
    assert_eq!(to_excel_string(&t, &cell_reference), "SUM(#REF!)");
    assert_eq!(to_rc_format(&t), "SUM(tblIncome[Jan])");
}

#[test]
fn huge_header_or_totals_row_counts_are_ref_error_not_an_overflow() {
    // Row counts come from the workbook, which is untrusted (directive 5, T-1234).
    let cell_reference = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 2,
        column: 6,
    };
    for (header, totals) in [
        (u32::MAX, 0),
        (0, u32::MAX),
        (i32::MAX as u32, 0),
        (0, 1 << 31),
    ] {
        let mut tables = create_test_table("tblIncome", &["Jan", "Feb"], "A1", 3);
        if let Some(table) = tables.get_mut("tblIncome") {
            table.header_row_count = header;
            table.totals_row_count = totals;
        }
        let mut parser = new_parser(vec!["Sheet One".to_string()], vec![], tables);
        for formula in ["SUM(tblIncome[Jan])", "SUM(tblIncome[[#Totals],[Jan]])"] {
            let t = parser.parse(formula, &cell_reference);
            assert!(
                matches!(t.ungrouped(), Node::FunctionKind { args, .. }
                    if matches!(args[0].ungrouped(), Node::ErrorKind(_))),
                "{formula} with header {header}, totals {totals}"
            );
        }
        // The export writes such a reference as its error; `#Totals` keeps its text, as Excel
        // keeps it for a table without a totals row (T-2196).
        let t = parser.parse("SUM(tblIncome[Jan])", &cell_reference);
        assert_eq!(to_excel_string(&t, &cell_reference), "SUM(#REF!)");
    }
}

#[test]
fn bare_table_name_and_empty_brackets_are_the_data_area() {
    let worksheets = vec!["Sheet One".to_string()];
    let tables = create_test_table("tblIncome", &["Jan", "Feb"], "A1", 3);
    let mut parser = new_parser(worksheets, vec![], tables);
    let cell_reference = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 9,
        column: 9,
    };
    // Header row excluded (gen-m8b specials e804cf2f8af00680 K29, K116).
    let bare = parser.parse("ROWS(tblIncome)", &cell_reference);
    let expected = to_excel_string(
        &parser.parse("ROWS(tblIncome[])", &cell_reference),
        &cell_reference,
    );
    assert_eq!(to_excel_string(&bare, &cell_reference), expected);
    // The file stores a bare name as `T[]` (xlsx-storage.md "Structured references in the file").
    assert_eq!(expected, "ROWS(tblIncome[])");
    assert!(matches!(bare.ungrouped(), Node::FunctionKind { .. }));
    // A bare name keeps its text; `tblIncome[]` is read as the bare name (SPEC-0016).
    assert_eq!(to_rc_format(&bare), "ROWS(tblIncome)");
    // Names match ignoring case, and keep the case they were typed in.
    let t = parser.parse("ROWS(TBLINCOME)", &cell_reference);
    assert_eq!(to_excel_string(&t, &cell_reference), "ROWS(TBLINCOME[])");
    assert_eq!(to_rc_format(&t), "ROWS(TBLINCOME)");
}

#[test]
fn this_row_outside_the_data_rows_is_value_error() {
    let worksheets = vec!["Sheet One".to_string()];
    let tables = create_test_table("tblIncome", &["Jan", "Feb"], "A1", 3);
    let mut parser = new_parser(worksheets, vec![], tables);
    // Data rows are 2 and 3; the header row, and rows above and below, are #VALUE! (gen-m8b
    // this-row e1bf5aa0ffb4ed87).
    for row in [1, 4, 9, 3] {
        let cell_reference = CellReferenceRC {
            sheet: "Sheet One".to_string(),
            row,
            column: 9,
        };
        let t = parser.parse("tblIncome[[#This Row],[Jan]]", &cell_reference);
        // Excel stores the reference on any row, and evaluates it (T-2196).
        assert_eq!(
            to_excel_string(&t, &cell_reference),
            "tblIncome[[#This Row],[Jan]]"
        );
        assert_eq!(
            to_english_localized_string(&t, &cell_reference),
            "tblIncome[[#This Row],[Jan]]"
        );
        // The parsed area does not depend on the row it was parsed at (SPEC-0016 step 2).
        assert!(matches!(
            t.ungrouped(),
            Node::ThisRowKind {
                data_start: 2,
                data_end: 3,
                cell: true,
                ..
            }
        ));
    }
}

#[test]
fn reversed_column_range_is_the_same_area() {
    let worksheets = vec!["Sheet One".to_string()];
    let tables = create_test_table("tblIncome", &["Jan", "Feb", "Mar"], "A1", 3);
    let mut parser = new_parser(worksheets, vec![], tables);
    let cell_reference = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 9,
        column: 9,
    };
    // column-range 44cecbb1ecd5b2da K143: COLUMNS(Sales[[2024]:[a.b]]) is 2 either way round.
    let forward = parser.parse("tblIncome[[Jan]:[Mar]]", &cell_reference);
    let reversed = parser.parse("tblIncome[[Mar]:[Jan]]", &cell_reference);
    assert_eq!(forward.ungrouped(), reversed.ungrouped());
    assert!(matches!(forward.ungrouped(), Node::RangeKind { .. }));
    // The file keeps it as written: Excel computed gen-m8b's reversed form from the file.
    assert_eq!(
        to_excel_string(&reversed, &cell_reference),
        "tblIncome[[Mar]:[Jan]]"
    );
    assert_eq!(to_rc_format(&reversed), "tblIncome[[Mar]:[Jan]]");
}

/// Printing a structured reference and parsing the text gives the same reference, for every
/// specifier and column form over adversarial column names (the names come from untrusted
/// files). Printing never panics.
#[test]
fn every_form_round_trips_through_the_printer() {
    use crate::expressions::parser::{Node, StructuredReference};
    use crate::expressions::token::{TableReference, TableSpacing, TableSpecifier};

    let names = [
        "Qty",
        "a[b",
        "a]b",
        "#x",
        "x#",
        "it's",
        "@at",
        "a@",
        " lead",
        "trail ",
        "two  spaces",
        "''",
        "'[",
        "[']",
        "#All",
        "ñandú",
        "数量",
        "a'#b'@c",
        "]",
        "[",
        "'",
    ];
    let mut tables = create_test_table("T", &names, "A1", 4);
    if let Some(table) = tables.get_mut("T") {
        table.totals_row_count = 1;
    }
    let mut parser = new_parser(vec!["Sheet One".to_string()], vec![], tables);
    let context = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 30,
        column: 30,
    };
    let specifiers = [
        None,
        Some(TableSpecifier::All),
        Some(TableSpecifier::Data),
        Some(TableSpecifier::Headers),
        Some(TableSpecifier::Totals),
        Some(TableSpecifier::HeadersData),
        Some(TableSpecifier::DataTotals),
        Some(TableSpecifier::ThisRow),
    ];
    let mut references = vec![StructuredReference {
        table_name: "T".to_string(),
        specifier: None,
        table_reference: None,
        spacing: TableSpacing::default(),
        bare: true,
        unqualified: false,
    }];
    for specifier in &specifiers {
        if specifier.is_some() {
            references.push(StructuredReference {
                table_name: "T".to_string(),
                specifier: specifier.clone(),
                table_reference: None,
                spacing: TableSpacing::default(),
                bare: false,
                unqualified: false,
            });
        }
        for (i, left) in names.iter().enumerate() {
            let right = names[(i + 1) % names.len()];
            for table_reference in [
                TableReference::ColumnReference(left.to_string()),
                TableReference::RangeReference((left.to_string(), right.to_string())),
            ] {
                references.push(StructuredReference {
                    table_name: "T".to_string(),
                    specifier: specifier.clone(),
                    table_reference: Some(table_reference),
                    spacing: TableSpacing::default(),
                    bare: false,
                    unqualified: false,
                });
            }
        }
    }
    for reference in references {
        let text = reference.text();
        let node = parser.parse(&text, &context);
        let Node::StructuredReferenceKind {
            reference: parsed, ..
        } = &node
        else {
            panic!("{text} parsed as {node:?}");
        };
        assert_eq!(**parsed, reference, "{text}");
        assert_eq!(to_rc_format(&node), text);
    }
}

/// A structured reference typed without its table name in a cell of the table names that table
/// (T-2979): it prints as typed, in its `@` form, is stored with a `[T]` prefix that a re-parse
/// reads back, and is exported qualified; outside every table it is refused.
#[test]
fn unqualified_reference_inside_its_table() {
    let worksheets = vec!["Sheet One".to_string()];
    let tables = create_test_table("tblIncome", &["Jan", "Feb"], "A1", 3);
    let mut parser = new_parser(worksheets, vec![], tables);
    let inside = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 2,
        column: 3,
    };
    for (typed, shown, stored, exported) in [
        (
            "[@Jan]*2",
            "[@Jan]*2",
            "[tblIncome][[#This Row],[Jan]]*2",
            "tblIncome[[#This Row],[Jan]]*2",
        ),
        (
            "[[#This Row],[Feb]]",
            "[@Feb]",
            "[tblIncome][[#This Row],[Feb]]",
            "tblIncome[[#This Row],[Feb]]",
        ),
        (
            "SUM([Jan])",
            "SUM([Jan])",
            "SUM([tblIncome][Jan])",
            "SUM(tblIncome[Jan])",
        ),
    ] {
        let t = parser.parse(typed, &inside);
        assert_eq!(to_english_localized_string(&t, &inside), shown);
        assert_eq!(to_rc_format(&t), stored);
        assert_eq!(to_excel_string(&t, &inside), exported);
        parser.set_lexer_mode(crate::expressions::lexer::LexerMode::R1C1);
        let reparsed = parser.parse(stored, &inside);
        parser.set_lexer_mode(crate::expressions::lexer::LexerMode::A1);
        assert_eq!(to_english_localized_string(&reparsed, &inside), shown);
    }
    let outside = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 9,
        column: 9,
    };
    let t = parser.parse("[@Jan]*2", &outside);
    assert!(matches!(t, Node::ParseErrorKind { .. }), "{t:?}");
}

/// `node` read back from its stored (R1C1) text, as the engine re-parses a formula.
fn reparse_stored(parser: &mut Parser, node: &Node, cell: &CellReferenceRC) -> Node {
    parser.set_lexer_mode(LexerMode::R1C1);
    let again = parser.parse(&to_rc_format(node), cell);
    parser.set_lexer_mode(LexerMode::A1);
    again
}

/// A `#This Row` reference in a cell of its own table prints in Excel's `@` form without the
/// table name, keeping the padding inside the outer brackets, and a run after `@` is padding, not
/// part of the column (syntax.md, probes/t2800-this-row-whitespace, T-3069).
#[test]
fn this_row_prints_in_its_at_form_in_its_own_table() {
    let tables = create_test_table("T", &["Col", "Other"], "A1", 4);
    let mut parser = new_parser(vec!["Sheet One".to_string()], vec![], tables);
    let inside = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 2,
        column: 2,
    };
    for (typed, printed) in [
        ("T[[#This Row], [Col]]", "[@Col]"),
        ("T[[#This Row] , [Col]]", "[@Col]"),
        ("T[ [#This Row] , [Col] ]", "[ @Col ]"),
        ("T[ [#This Row],[Col] ]", "[ @Col ]"),
        ("[@ Col]", "[ @Col ]"),
        ("[@Col]", "[@Col]"),
    ] {
        let t = parser.parse(typed, &inside);
        assert_eq!(to_english_localized_string(&t, &inside), printed, "{typed}");
        // The stored text reads back to the same reference.
        let again = reparse_stored(&mut parser, &t, &inside);
        assert_eq!(
            to_english_localized_string(&again, &inside),
            printed,
            "{typed}"
        );
    }
    // Outside the table the name stays.
    let outside = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 8,
        column: 8,
    };
    let t = parser.parse("T[[#This Row], [Col]]", &outside);
    assert_eq!(
        to_english_localized_string(&t, &outside),
        "T[[#This Row], [Col]]"
    );
}

/// `T[ Col ]` is only the engine's own printed form of `T[ [Col] ]`: a padded bare column is a
/// parse error typed and `#REF!` loaded, as Excel loads `<f>SUM(T[ Col ])</f>` (syntax.md,
/// probes/t2800-stored-ws-0..2, T-3069).
#[test]
fn a_padded_bare_column_is_ours_only() {
    let tables = create_test_table("T", &["Col", "Other"], "A1", 4);
    let mut parser = new_parser(vec!["Sheet One".to_string()], vec![], tables);
    let cell = CellReferenceRC {
        sheet: "Sheet One".to_string(),
        row: 6,
        column: 5,
    };
    let t = parser.parse("SUM(T[ [Col] ])", &cell);
    let again = reparse_stored(&mut parser, &t, &cell);
    assert_eq!(to_english_localized_string(&again, &cell), "SUM(T[ Col ])");
    // The engine's own printed text reads back: edits re-parse it.
    let printed = parser.parse("SUM(T[ Col ])", &cell);
    assert_eq!(to_excel_string(&printed, &cell), "SUM(T[ [Col] ])");
    for formula in ["SUM(T[ Col ])", "SUM(T[ Col])", "SUM(T[Col ])"] {
        parser.set_typed(true);
        let typed = parser.parse(formula, &cell);
        parser.set_typed(false);
        assert!(matches!(typed, Node::ParseErrorKind { .. }), "{formula}");
        parser.set_loaded(true);
        let loaded = parser.parse(formula, &cell);
        parser.set_loaded(false);
        assert_eq!(to_excel_string(&loaded, &cell), "SUM(#REF!)", "{formula}");
    }
}
