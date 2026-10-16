#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
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

    let formula = "SUM(tblIncome[[#This Row],[Jan]:[Dec]])";
    let t = parser.parse(formula, &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "SUM($A$2:$E$2)"
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
        "SUBTOTAL(109,$A$2:$A$3)"
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
        "SUBTOTAL(109,'Sheet One'!$A$2:$A$3)"
    );
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
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "SUM(#REF!)"
    );
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
            assert_eq!(
                to_english_localized_string(&t, &cell_reference),
                "SUM(#REF!)",
                "{formula} with header {header}, totals {totals}"
            );
        }
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
    let expected = to_english_localized_string(
        &parser.parse("ROWS(tblIncome[])", &cell_reference),
        &cell_reference,
    );
    assert_eq!(
        to_english_localized_string(&bare, &cell_reference),
        expected
    );
    assert_eq!(expected, "ROWS($A$2:$C$3)");
    // Names match ignoring case.
    let t = parser.parse("ROWS(TBLINCOME)", &cell_reference);
    assert_eq!(to_english_localized_string(&t, &cell_reference), expected);
}
