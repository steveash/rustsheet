#![allow(clippy::unwrap_used)]
//! Structured references kept in the parsed formula (SPEC-0016 step 1, T-2195).
//!
//! The edit tests pin today's values with a twin: each structured reference on `Calc` is paired
//! with the plain range it stood for before SPEC-0016. After every edit both have the same
//! value, and the structured formula keeps its text unless the edit rewrote its twin, in which
//! case it has the twin's text (the fallback of SPEC-0016 "Structural edits").

use crate::expressions::parser::stringify::to_excel_string;
use crate::expressions::types::CellReferenceRC;
use crate::Model;

/// Structured formulas and their twins, in `Calc` columns A and B.
const PAIRS: [(&str, &str); 6] = [
    ("=SUM(Sales[Qty])", "=SUM(Data!$A$2:$A$3)"),
    ("=SUM(Sales[#All])", "=SUM(Data!$A$1:$B$3)"),
    ("=ROWS(Sales)", "=ROWS(Data!$A$2:$B$3)"),
    ("=SUM(Sales[[Qty]:[Val]])", "=SUM(Data!$A$2:$B$3)"),
    ("=COUNTA(Sales[#Headers])", "=COUNTA(Data!$A$1:$B$1)"),
    ("=SUM(Sales[[#Data],[Val]])", "=SUM(Data!$B$2:$B$3)"),
];

/// `Data` with the table `Sales` over A1:B3 (data rows 2-3) and a value under it, and `Calc`
/// with the [`PAIRS`].
fn model() -> Model {
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model.add_sheet("Calc").unwrap();
    for (row, values) in [
        ["Qty", "Val"],
        ["1", "10"],
        ["2", "20"],
        ["4", "40"],
        ["8", "80"],
    ]
    .iter()
    .enumerate()
    {
        for (column, value) in values.iter().enumerate() {
            let (row, column) = (row as i32 + 1, column as i32 + 1);
            model
                .set_user_input(0, row, column, value.to_string())
                .unwrap();
        }
    }
    model.add_table(0, "Sales", (1, 1, 3, 2), false).unwrap();
    for (row, (formula, twin)) in PAIRS.iter().enumerate() {
        let row = row as i32 + 1;
        model
            .set_user_input(1, row, 1, formula.to_string())
            .unwrap();
        model.set_user_input(1, row, 2, twin.to_string()).unwrap();
    }
    model.evaluate();
    model
}

fn formula(model: &Model, sheet: u32, row: i32, column: i32) -> String {
    model.get_cell_formula(sheet, row, column).unwrap().unwrap()
}

fn value(model: &Model, sheet: u32, row: i32, column: i32) -> String {
    model.get_formatted_cell_value(sheet, row, column).unwrap()
}

/// Applies `edit` and checks every pair as the module doc says.
fn check_edit(name: &str, edit: impl FnOnce(&mut Model)) {
    let mut model = model();
    edit(&mut model);
    model.evaluate();
    for (row, (structured, twin)) in PAIRS.iter().enumerate() {
        let row = row as i32 + 1;
        // A twin rewritten to the same text (`$A$3:$A$2` read back as `$A$2:$A$3` after a row
        // move) leaves either text on the structured formula.
        let twin_now = formula(&model, 1, row, 2);
        let structured_now = formula(&model, 1, row, 1);
        if twin_now == *twin {
            assert!(
                structured_now == *structured || structured_now == twin_now,
                "{name}: {structured} is {structured_now}"
            );
        } else {
            assert_eq!(structured_now, twin_now, "{name}: {structured}");
        }
        assert_eq!(
            value(&model, 1, row, 1),
            value(&model, 1, row, 2),
            "{name}: {structured}"
        );
    }
    // A later reparse (any change to the table list) keeps them equal.
    model.add_table(1, "Other", (20, 1, 21, 1), false).ok();
    model.evaluate();
    for (row, (structured, _)) in PAIRS.iter().enumerate() {
        let row = row as i32 + 1;
        assert_eq!(
            value(&model, 1, row, 1),
            value(&model, 1, row, 2),
            "{name} after a reparse: {structured}"
        );
    }
}

#[test]
fn keeps_the_text_and_follows_the_table() {
    let mut model = model();
    assert_eq!(formula(&model, 1, 1, 1), "=SUM(Sales[Qty])");
    assert_eq!(value(&model, 1, 1, 1), "3");
    model
        .set_user_input(0, 10, 4, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 0, 10, 4), "=SUM(Sales[Qty])");

    // Without the table the text stays; with the table over A1:B4 it sums A2:A4.
    model.remove_table("Sales").unwrap();
    assert_eq!(formula(&model, 1, 1, 1), "=SUM(Sales[Qty])");
    assert_eq!(formula(&model, 0, 10, 4), "=SUM(Sales[Qty])");
    model.add_table(0, "Sales", (1, 1, 4, 2), false).unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 1, 1, 1), "=SUM(Sales[Qty])");
    assert_eq!(value(&model, 1, 1, 1), "7");
    assert_eq!(value(&model, 0, 10, 4), "7");
}

#[test]
fn every_written_form_is_kept() {
    let mut model = model();
    for (row, (written, stored)) in [
        ("=SUM(sales[qty])", "=SUM(sales[qty])"),
        ("=SUM(Sales[[#All],[Qty]])", "=SUM(Sales[[#All],[Qty]])"),
        (
            "=SUM(Sales[[#Headers],[#Data]])",
            "=SUM(Sales[[#Headers],[#Data]])",
        ),
        (
            "=SUM(Sales[[#Data],[#Totals]])",
            "=SUM(Sales[[#Data],[#Totals]])",
        ),
        (
            "=SUM(Sales[[#data],[Qty]:[Val]])",
            "=SUM(Sales[[#Data],[Qty]:[Val]])",
        ),
        ("=SUM(Sales[[Qty]])", "=SUM(Sales[Qty])"),
        ("=SUM(Sales[])", "=SUM(Sales)"),
        // `#Totals` without a totals row is #REF! and keeps its text too.
        ("=ISREF(Sales[#Totals])", "=ISREF(Sales[#Totals])"),
    ]
    .iter()
    .enumerate()
    {
        let row = row as i32 + 10;
        model
            .set_user_input(1, row, 4, written.to_string())
            .unwrap();
        assert_eq!(formula(&model, 1, row, 4), *stored, "{written}");
    }
}

#[test]
fn the_export_prints_the_area() {
    let model = model();
    let context = CellReferenceRC {
        sheet: "Calc".to_string(),
        row: 1,
        column: 1,
    };
    let node = &model.parsed_formulas[1][0].0;
    assert_eq!(to_excel_string(node, &context), "SUM(Data!$A$2:$A$3)");
    assert_eq!(
        crate::expressions::parser::stringify::to_excel_array_formula_string(node, &context),
        "SUM(Data!$A$2:$A$3)"
    );
}

#[test]
fn the_export_of_a_missing_table_is_a_ref_error() {
    let mut model = model();
    model.remove_table("Sales").unwrap();
    let context = CellReferenceRC {
        sheet: "Calc".to_string(),
        row: 1,
        column: 1,
    };
    let node = &model.parsed_formulas[1][0].0;
    assert_eq!(to_excel_string(node, &context), "#REF!");
}

#[test]
fn row_inserts_keep_values() {
    for row in [1, 2, 3, 4, 5, 10] {
        check_edit(&format!("insert row {row}"), |m| {
            m.insert_rows(0, row, 1).unwrap()
        });
    }
}

#[test]
fn row_deletes_keep_values() {
    for (row, count) in [(1, 1), (2, 1), (3, 1), (2, 2), (4, 1), (5, 1), (1, 5)] {
        check_edit(&format!("delete rows {row}+{count}"), |m| {
            m.delete_rows(0, row, count).unwrap();
        });
    }
}

#[test]
fn column_inserts_and_deletes_keep_values() {
    for column in [1, 2, 3, 5] {
        check_edit(&format!("insert column {column}"), |m| {
            m.insert_columns(0, column, 1).unwrap();
        });
        check_edit(&format!("delete column {column}"), |m| {
            m.delete_columns(0, column, 1).unwrap();
        });
    }
}

#[test]
fn row_and_column_moves_keep_values() {
    for (row, delta) in [(2, 1), (3, -1), (1, 3), (5, -4)] {
        check_edit(&format!("move row {row} by {delta}"), |m| {
            m.move_rows_action(0, row, 1, delta).unwrap();
        });
    }
    for (column, delta) in [(1, 1), (2, -1), (3, -2)] {
        check_edit(&format!("move column {column} by {delta}"), |m| {
            m.move_columns_action(0, column, 1, delta).unwrap();
        });
    }
}

#[test]
fn a_formula_on_the_table_sheet_keeps_its_value() {
    let mut model = model();
    model
        .set_user_input(0, 1, 6, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model.insert_rows(0, 1, 1).unwrap();
    model.evaluate();
    // Today's text and value: the area moved with the data, the table did not.
    assert_eq!(formula(&model, 0, 2, 6), "=SUM($A$3:$A$4)");
    assert_eq!(value(&model, 0, 2, 6), "3");
    model.insert_rows(0, 20, 1).unwrap();
    model
        .set_user_input(0, 10, 6, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model.insert_rows(0, 20, 1).unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 0, 10, 6), "=SUM(Sales[Qty])");
}

#[test]
fn cut_and_paste_keeps_values() {
    for (source, target) in [((2, 1), (10, 5)), ((1, 1), (10, 5)), ((5, 1), (10, 5))] {
        let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
        model.rename_sheet("Sheet1", "Data").unwrap();
        for (row, value) in [(1, "Qty"), (2, "1"), (3, "2"), (5, "4")] {
            model.set_user_input(0, row, 1, value.to_string()).unwrap();
        }
        model.add_table(0, "Sales", (1, 1, 3, 1), false).unwrap();
        let mut user = crate::UserModel::from_model(model);
        user.set_user_input(0, 1, 3, "=SUM(Sales[Qty])").unwrap();
        user.set_user_input(0, 1, 4, "=SUM($A$2:$A$3)").unwrap();
        user.set_selected_cell(source.0, source.1).unwrap();
        user.set_selected_range(source.0, source.1, source.0, source.1)
            .unwrap();
        let clipboard = user.copy_to_clipboard().unwrap();
        user.set_selected_cell(target.0, target.1).unwrap();
        user.paste_from_clipboard(
            0,
            (source.0, source.1, source.0, source.1),
            &clipboard.data,
            true,
        )
        .unwrap();
        let model = user.get_model();
        let twin = formula(model, 0, 1, 4);
        let expected = if twin == "=SUM($A$2:$A$3)" {
            "=SUM(Sales[Qty])".to_string()
        } else {
            twin
        };
        assert_eq!(formula(model, 0, 1, 3), expected, "cut {source:?}");
        assert_eq!(
            value(model, 0, 1, 3),
            value(model, 0, 1, 4),
            "cut {source:?}"
        );
    }
}

#[test]
fn a_sheet_rename_keeps_text_and_values() {
    let mut model = model();
    model
        .set_user_input(0, 10, 6, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model.rename_sheet("Data", "Facts").unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 1, 1, 1), "=SUM(Sales[Qty])");
    assert_eq!(formula(&model, 0, 10, 6), "=SUM(Sales[Qty])");
    assert_eq!(value(&model, 1, 1, 1), "3");
    assert_eq!(value(&model, 0, 10, 6), "3");
    let context = CellReferenceRC {
        sheet: "Calc".to_string(),
        row: 1,
        column: 1,
    };
    let node = &model.parsed_formulas[1][0].0;
    assert_eq!(to_excel_string(node, &context), "SUM(Facts!$A$2:$A$3)");
}

#[test]
fn a_sheet_delete_is_a_ref_error_and_restore_brings_the_tables_back() {
    let mut model = model();
    model.add_sheet("Spare").unwrap();
    model.define_name("Total", None, "SUM(Sales[Qty])").unwrap();
    let taken = model.take_sheet(0).unwrap();
    model.evaluate();
    // `Calc` is now sheet 0.
    assert_eq!(value(&model, 0, 1, 1), "#REF!");
    for (row, _) in PAIRS.iter().enumerate() {
        let row = row as i32 + 1;
        assert_eq!(formula(&model, 0, row, 1), formula(&model, 0, row, 2));
        assert_eq!(value(&model, 0, row, 1), value(&model, 0, row, 2));
    }
    let names = model.workbook.defined_names.clone();
    assert_eq!(names[0].formula, "SUM(Data!$A$2:$A$3)");
    assert!(model.workbook.tables.is_empty());
    // A new sheet with the old name does not bring the table back.
    model.add_sheet("Data").unwrap();
    model.evaluate();
    assert_eq!(value(&model, 0, 1, 1), value(&model, 0, 1, 2));
    model
        .delete_sheet(model.get_sheet_index_by_name("Data").unwrap())
        .unwrap();
    model.restore_sheet(0, taken).unwrap();
    model.evaluate();
    assert!(model.workbook.tables.contains_key("Sales"));
    model
        .set_user_input(1, 20, 1, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(value(&model, 1, 20, 1), "3");
    assert_eq!(value(&model, 1, 1, 1), value(&model, 1, 1, 2));
}

#[test]
fn many_structured_references_parse_and_print() {
    let mut model = model();
    let chain = vec!["Sales[Qty]"; 500].join("+");
    model
        .set_user_input(1, 30, 1, format!("=SUM({chain})"))
        .unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 1, 30, 1), format!("=SUM({chain})"));
    let nested = format!("{}Sales[Qty]{}", "SUM(".repeat(60), ")".repeat(60));
    model
        .set_user_input(1, 31, 1, format!("={nested}"))
        .unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 1, 31, 1), format!("={nested}"));
    assert_eq!(value(&model, 1, 31, 1), "3");
}
