//! Tables and structured references through the export (SPEC-0016 step 3, T-2196).

use std::fs;

use rustsheet_engine::Model;

use super::test_export::{new_empty_model, part_text};
use crate::{export::save_to_xlsx, import::load_from_xlsx};

/// Table `Sales` over A1:C4 of Sheet1, with a totals row (a label and a SUBTOTAL), a column whose
/// name needs an escape (`C#`) and formulas outside it with each kind of structured reference.
fn model_with_table() -> Model {
    let mut model = new_empty_model();
    let cells = [
        (1, 1, "Item"),
        (1, 2, "Qty"),
        (1, 3, "C#"),
        (2, 1, "a"),
        (2, 2, "2"),
        (2, 3, "10"),
        (3, 1, "b"),
        (3, 2, "3"),
        (3, 3, "20"),
        (4, 1, "Total"),
    ];
    for (row, column, input) in cells {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    model
        .add_table_with_totals(0, "Sales", (1, 1, 4, 3), false, true)
        .unwrap();
    let formulas = [
        (4, 2, "=SUBTOTAL(109,Sales[Qty])"),
        (2, 5, "=Sales[@Qty]*2"),
        (5, 5, "=ROWS(Sales)"),
        (6, 5, "=SUM(Sales[[#Totals],[Qty]])"),
        (7, 5, "=SUM(Sales[C'#])"),
        (8, 5, "=SUM(Sales[#All])"),
    ];
    for (row, column, input) in formulas {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    model.evaluate();
    model
}

#[test]
fn table_parts_and_structured_references_round_trip() {
    let model = model_with_table();
    let temp_file_name = "temp_file_test_table_parts.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let sheet_xml = part_text(temp_file_name, "xl/worksheets/sheet1.xml");
    let table_xml = part_text(temp_file_name, "xl/tables/table1.xml");
    let rels = part_text(temp_file_name, "xl/worksheets/_rels/sheet1.xml.rels");
    let content_types = part_text(temp_file_name, "[Content_Types].xml");
    let mut reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    // The file forms of xlsx-storage.md "Structured references in the file": a bare name as
    // `Sales[]`, `@` as `[#This Row]`, the escape before the special character.
    for formula in [
        "<f>Sales[[#This Row],[Qty]]*2</f>",
        "<f>ROWS(Sales[])</f>",
        "<f>SUM(Sales[[#Totals],[Qty]])</f>",
        "<f>SUM(Sales[C&apos;#])</f>",
        "<f>SUM(Sales[#All])</f>",
        "<f>SUBTOTAL(109,Sales[Qty])</f>",
    ] {
        assert!(sheet_xml.contains(formula), "{formula} in {sheet_xml}");
    }
    assert!(
        sheet_xml.contains(r#"<tableParts count="1"><tablePart r:id="rId1"/></tableParts>"#),
        "{sheet_xml}"
    );
    assert!(rels.contains(r#"Target="../tables/table1.xml""#), "{rels}");
    assert!(
        content_types.contains(r#"PartName="/xl/tables/table1.xml""#),
        "{content_types}"
    );
    // The totals row needs its metadata, or Excel refuses the file.
    assert!(
        table_xml.contains(r#"<tableColumn id="1" name="Item" totalsRowLabel="Total">"#),
        "{table_xml}"
    );
    assert!(
        table_xml.contains(
            r#"totalsRowFunction="custom"><totalsRowFormula>SUBTOTAL(109,Sales[Qty])</totalsRowFormula>"#
        ),
        "{table_xml}"
    );
    assert!(
        table_xml.contains(r#"ref="A1:C4" totalsRowCount="1""#),
        "{table_xml}"
    );
    assert!(
        table_xml.contains(r#"<autoFilter ref="A1:C3"></autoFilter>"#),
        "{table_xml}"
    );

    assert_eq!(reloaded.workbook.tables, model.workbook.tables);
    reloaded.evaluate();
    for (row, column) in [(4, 2), (2, 5), (5, 5), (6, 5), (7, 5), (8, 5)] {
        assert_eq!(
            reloaded.get_cell_formula(0, row, column).unwrap(),
            model.get_cell_formula(0, row, column).unwrap(),
            "formula of row {row} column {column}"
        );
        assert_eq!(
            reloaded.get_formatted_cell_value(0, row, column).unwrap(),
            model.get_formatted_cell_value(0, row, column).unwrap(),
            "value of row {row} column {column}"
        );
    }
    assert_eq!(reloaded.get_formatted_cell_value(0, 2, 5).unwrap(), "4");
    assert_eq!(reloaded.get_formatted_cell_value(0, 5, 5).unwrap(), "2");
    assert_eq!(reloaded.get_formatted_cell_value(0, 7, 5).unwrap(), "30");
}

#[test]
fn a_filtered_table_keeps_its_filter_flag() {
    let mut model = new_empty_model();
    for (row, input) in [(1, "Qty"), (2, "1"), (3, "2")] {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
    }
    model.add_table(0, "Sales", (1, 1, 3, 1), true).unwrap();
    let temp_file_name = "temp_file_test_filtered_table.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let table_xml = part_text(temp_file_name, "xl/tables/table1.xml");
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert!(
        table_xml.contains(r#"<autoFilter ref="A1:A3"><filterColumn colId="0"/></autoFilter>"#),
        "{table_xml}"
    );
    assert!(reloaded.workbook.tables["Sales"].has_filters);
}

#[test]
fn tables_on_two_sheets_get_their_own_parts() {
    let mut model = new_empty_model();
    model.add_sheet("Second").unwrap();
    for sheet in 0..2 {
        for (row, input) in [(1, "Qty"), (2, "1"), (3, "2")] {
            model
                .set_user_input(sheet, row, 1, input.to_string())
                .unwrap();
        }
    }
    model.add_table(0, "First", (1, 1, 3, 1), false).unwrap();
    model.add_table(1, "Other", (1, 1, 3, 1), false).unwrap();
    model
        .set_user_input(0, 5, 1, "=SUM(Other[Qty])+SUM(First[Qty])".to_string())
        .unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_tables_two_sheets.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let second = part_text(temp_file_name, "xl/tables/table2.xml");
    let second_rels = part_text(temp_file_name, "xl/worksheets/_rels/sheet2.xml.rels");
    let mut reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert!(second.contains(r#"id="2" name="Other""#), "{second}");
    assert!(
        second_rels.contains(r#"Target="../tables/table2.xml""#),
        "{second_rels}"
    );
    assert_eq!(reloaded.workbook.tables, model.workbook.tables);
    reloaded.evaluate();
    assert_eq!(reloaded.get_formatted_cell_value(0, 5, 1).unwrap(), "6");
}

/// The Excel-authored tables.xlsx keeps the text of every formula through an export, and writes
/// structured references as Excel's own file has them.
#[test]
fn excel_authored_tables_keep_their_formula_text() {
    let model = load_from_xlsx("tests/calc_test_tables/tables.xlsx", "en", "UTC", "en").unwrap();
    let temp_file_name = "temp_file_test_excel_tables.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    // Excel's forms (the original's `<f>`): `#Totals` of a table without a totals row (cached
    // `#REF!`), a column name that ends with a space, an `@` inside a column name.
    for (part, formula) in [
        ("xl/worksheets/sheet2.xml", "<f>SUM(Table2[#Totals])</f>"),
        ("xl/worksheets/sheet3.xml", "<f>SUM(Table3[[the two ]])</f>"),
        (
            "xl/worksheets/sheet8.xml",
            "<f>SUM(Table9[Totals &apos;[&apos;]^&amp;*@&apos;#$,,&lt;&gt;])</f>",
        ),
    ] {
        let xml = part_text(temp_file_name, part);
        assert!(xml.contains(formula), "{formula} in {part}");
    }
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert_eq!(reloaded.workbook.tables.len(), model.workbook.tables.len());
    let mut formulas = 0;
    for (sheet, worksheet) in model.workbook.worksheets.iter().enumerate() {
        let sheet = u32::try_from(sheet).unwrap();
        for (row, column, cell) in worksheet.sheet_data.cells() {
            if cell.get_formula().is_none() {
                continue;
            }
            formulas += 1;
            // The export drops the space after an argument comma (`SUBTOTAL(9, T[C])`), as it
            // did before table parts; only that spacing is set aside.
            let text = |m: &Model| {
                m.get_cell_formula(sheet, row, column)
                    .unwrap()
                    .map(|f| f.replace(", ", ","))
            };
            assert_eq!(
                text(&reloaded),
                text(&model),
                "{} row {row} column {column}",
                worksheet.name
            );
        }
    }
    assert!(formulas > 100, "{formulas}");
}
