#![allow(clippy::unwrap_used)]
//! `#This Row` resolved at the evaluating cell's row (SPEC-0016 step 2, T-1699).
//!
//! The parity tests pin today's values with a twin: each `#This Row` formula is paired with the
//! formula its parse-time lowering gave before step 2, the cell or range on the formula's row
//! (`$A$3`) on a data row and `#VALUE!` off them (gen-m8b this-row e1bf5aa0ffb4ed87).

use crate::expressions::parser::stringify::to_excel_string;
use crate::expressions::types::CellReferenceRC;
use crate::Model;

/// `T` with the table `Sales` over A1:B5 (data rows 2-5), whose `Qty` cells are TRUE, the text
/// "5", a blank and 3, and values under it on rows 6 and 7; and an empty sheet `C`.
fn model() -> Model {
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    model.rename_sheet("Sheet1", "T").unwrap();
    model.add_sheet("C").unwrap();
    for (row, qty, val) in [
        (1, "Qty", "Val"),
        (2, "TRUE", "10"),
        (3, "'5", "20"),
        (4, "", "30"),
        (5, "3", "40"),
        (6, "7", "60"),
        (7, "9", "70"),
    ] {
        if !qty.is_empty() {
            model.set_user_input(0, row, 1, qty.to_string()).unwrap();
        }
        model.set_user_input(0, row, 2, val.to_string()).unwrap();
    }
    model.add_table(0, "Sales", (1, 1, 5, 2), false).unwrap();
    model
}

/// The formula of the cell, empty when it has none.
fn formula(model: &Model, sheet: u32, row: i32, column: i32) -> String {
    model
        .get_cell_formula(sheet, row, column)
        .unwrap()
        .unwrap_or_default()
}

fn value(model: &Model, sheet: u32, row: i32, column: i32) -> String {
    model.get_formatted_cell_value(sheet, row, column).unwrap()
}

/// Formulas over `{x}` (`Sales[@Qty]`, a cell) and `{y}` (`Sales[@[Qty]:[Val]]`, a range): the
/// functions that read a cell reference apart from a value, and the reference operators.
const TEMPLATES: [&str; 30] = [
    "={x}",
    "=ISREF({x})",
    "=SUM({x})",
    "=COUNT({x})",
    "=COUNTA({x})",
    "=CONCAT({x})",
    "=QUARTILE({x},1)",
    "=MODE({x})",
    "=PERCENTILE({x},0.5)",
    "=ROW({x})",
    "=COLUMN({x})",
    "=N({x})",
    "=T({x})",
    "=ISBLANK({x})",
    "=AVERAGE({x})",
    "=MAX({x})",
    "=MATCH(5,{x},0)",
    "={x}+0",
    "=IMSUM({x})",
    "=NPV(0.1,{x})",
    "=SHEETS({x})",
    "=XLOOKUP(5,{x},{x})",
    "=SUM({x}:T!$B$5)",
    "=SUM({x} T!$A$1:$B$7)",
    "=SUM({y})",
    "=ISREF({y})",
    "=COLUMNS({y})",
    "=INDEX({y},1,2)",
    "=@{y}",
    "=COUNTA({y})",
];

/// Each template on `C` rows 1-8 (column `2 * i + 1`) next to its twin, and the same on `T`
/// (columns 5 and up), so the formula sits on the table's sheet and on another one.
fn parity_model() -> Model {
    let mut model = model();
    for (sheet, prefix, first_column) in [(1, "T!", 1), (0, "", 5)] {
        for (i, template) in TEMPLATES.iter().enumerate() {
            let column = first_column + 2 * i as i32;
            for row in 1..=8 {
                let on_data_row = (2..=5).contains(&row);
                let (x, y) = if on_data_row {
                    (
                        format!("{prefix}$A${row}"),
                        format!("{prefix}$A${row}:$B${row}"),
                    )
                } else {
                    ("#VALUE!".to_string(), "#VALUE!".to_string())
                };
                let structured = template
                    .replace("{x}", "Sales[@Qty]")
                    .replace("{y}", "Sales[@[Qty]:[Val]]");
                // `#VALUE!` does not parse as an operand of `:` or the space, so off the data rows
                // the twin of a reference operator over the lowered `#VALUE!` is its value.
                let operator = template.contains(":T!") || template.contains(" T!");
                let twin = if operator && !on_data_row {
                    "=#VALUE!".to_string()
                } else {
                    template.replace("{x}", &x).replace("{y}", &y)
                };
                model
                    .set_user_input(sheet, row, column, structured)
                    .unwrap();
                model.set_user_input(sheet, row, column + 1, twin).unwrap();
            }
        }
    }
    model.evaluate();
    model
}

/// The cells of `sheet` from `first_column` whose `#This Row` formula and twin differ. After an
/// edit the stand-in twin `=#VALUE!` of a reference operator is skipped: an edit that moves the
/// formula to another row prints it as its lowered text, `#VALUE!:T!$B$5`, which does not parse
/// back, as before SPEC-0016 step 2.
fn mismatches(model: &Model, sheet: u32, first_column: i32, edited: bool) -> Vec<String> {
    let mut found = Vec::new();
    for (i, template) in TEMPLATES.iter().enumerate() {
        let column = first_column + 2 * i as i32;
        for row in 1..=10 {
            let (structured, twin) = (
                value(model, sheet, row, column),
                value(model, sheet, row, column + 1),
            );
            let stand_in = formula(model, sheet, row, column + 1) == "=#VALUE!";
            if structured != twin && !(edited && stand_in) {
                found.push(format!(
                    "{template} on sheet {sheet} row {row}: {} is {structured}, {} is {twin}",
                    formula(model, sheet, row, column),
                    formula(model, sheet, row, column + 1),
                ));
            }
        }
    }
    found
}

#[test]
fn every_site_reads_this_row_as_its_lowering() {
    let model = parity_model();
    let mut found = mismatches(&model, 1, 1, false);
    found.extend(mismatches(&model, 0, 5, false));
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn the_stored_text_keeps_the_reference() {
    let model = parity_model();
    assert_eq!(formula(&model, 1, 3, 1), "=Sales[[#This Row],[Qty]]");
    assert_eq!(formula(&model, 1, 7, 1), "=Sales[[#This Row],[Qty]]");
    // One stored formula serves the whole column (SPEC-0016 "Shared formulas and storage").
    let stored = &model.workbook.worksheets[1].shared_formulas;
    let this_row: Vec<_> = stored
        .iter()
        .filter(|f| *f == "Sales[[#This Row],[Qty]]")
        .collect();
    assert_eq!(this_row.len(), 1);
}

/// gen-m8b in-table 19fdc25163755713 D11 (`SUM(Sales[@[2024]:[Val]])`), and ISREF: TRUE on a data
/// row and FALSE off them, from another sheet (references.md "Structured references: this row,
/// totals and ranges", probes/table-edits/table-this-row-off-data-isref, T-2197).
#[test]
fn isref_and_sum_of_this_row() {
    let mut model = model();
    model
        .set_user_input(1, 5, 1, "=SUM(Sales[@[Qty]:[Val]])".to_string())
        .unwrap();
    model
        .set_user_input(1, 5, 2, "=ISREF(Sales[@Qty])".to_string())
        .unwrap();
    model
        .set_user_input(1, 3, 1, "=SUM(Sales[@[Qty]:[Val]])".to_string())
        .unwrap();
    model
        .set_user_input(1, 1, 2, "=ISREF(Sales[@Qty])".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(value(&model, 1, 1, 2), "FALSE");
    assert_eq!(value(&model, 1, 5, 1), "43");
    assert_eq!(value(&model, 1, 5, 2), "TRUE");
    // The text "5" is not summed from a reference.
    assert_eq!(value(&model, 1, 3, 1), "20");
}

#[test]
fn a_grown_table_reads_its_new_data_row() {
    let mut model = model();
    model
        .set_user_input(1, 6, 1, "=Sales[@Qty]*2".to_string())
        .unwrap();
    model
        .set_user_input(1, 5, 1, "=Sales[@Qty]*2".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(value(&model, 1, 6, 1), "#VALUE!");
    assert_eq!(value(&model, 1, 5, 1), "6");
    // There is no resize API: grow the table by adding it again over A1:B6. That the formula then
    // reads its new data row is a hand expectation (SPEC-0016 goal 2): Excel's rule when a table
    // grows is probed for inserts only (references.md, T-2197).
    model.remove_table("Sales").unwrap();
    model.add_table(0, "Sales", (1, 1, 6, 2), false).unwrap();
    model.evaluate();
    assert_eq!(value(&model, 1, 6, 1), "14");
    assert_eq!(value(&model, 1, 5, 1), "6");
    assert_eq!(formula(&model, 1, 6, 1), "=Sales[[#This Row],[Qty]]*2");
    // And shrink it: row 5 is off the table now.
    model.remove_table("Sales").unwrap();
    model.add_table(0, "Sales", (1, 1, 4, 2), false).unwrap();
    model.evaluate();
    assert_eq!(value(&model, 1, 5, 1), "#VALUE!");
}

/// The export writes table parts, so it keeps `#This Row` in its file form on every row; Excel
/// evaluates it at the cell's row (T-2196).
#[test]
fn the_export_keeps_this_row_on_every_row() {
    let mut model = model();
    for row in [3, 6] {
        model
            .set_user_input(
                0,
                row,
                4,
                "=Sales[@Qty]+SUM(Sales[@[Qty]:[Val]])".to_string(),
            )
            .unwrap();
    }
    let cell = model.workbook.worksheets[0].cell(3, 4).unwrap();
    let index = cell.get_formula().unwrap();
    let node = &model.parsed_formulas[0][index as usize].evaluated;
    let at = |row| CellReferenceRC {
        sheet: "T".to_string(),
        row,
        column: 4,
    };
    for row in [3, 6, 1] {
        assert_eq!(
            to_excel_string(node, &at(row)),
            "Sales[[#This Row],[Qty]]+SUM(Sales[[#This Row],[Qty]:[Val]])"
        );
    }
}

/// Re-enters, on `sheet` from `first_column`, the text of every `#This Row` formula into its twin
/// cell, so each twin reads what a formula typed there after the edit reads.
fn retype_twins(model: &mut Model, sheet: u32, first_column: i32) {
    for i in 0..TEMPLATES.len() {
        let column = first_column + 2 * i as i32;
        for row in 1..=10 {
            let text = formula(model, sheet, row, column);
            if !text.is_empty() {
                model.set_user_input(sheet, row, column + 1, text).unwrap();
            }
        }
    }
    model.evaluate();
}

/// Structural edits other than a row insert the table follows keep today's values; one whose
/// displaced area differs, or that moves the formula's cell to another row, prints that area
/// (T-2297; Excel's rule for them is unprobed, T-3739). Through a row insert the table follows
/// (it moves or grows), every `#This Row` reference to it keeps its text and reads the cell's
/// row in the edited table: on the table's sheet the cells move with their rows and read the
/// same values, and on another sheet they stay and read the row they are on now, as Excel does
/// (T-2845 table-insert-row-this-row-above-header, -inside-data; T-3037).
#[test]
fn structural_edits_keep_values() {
    type Edit = fn(&mut Model);
    let edits: [(&str, Edit); 13] = [
        ("move a row on the formula sheet", |m| {
            m.move_rows_action(1, 3, 1, 4).unwrap();
        }),
        ("move a table row", |m| {
            m.move_rows_action(0, 3, 1, 1).unwrap();
        }),
        ("move a row out of the table", |m| {
            m.move_rows_action(0, 4, 1, 4).unwrap();
        }),
        ("insert above", |m| m.insert_rows(0, 1, 1).unwrap()),
        ("insert inside", |m| m.insert_rows(0, 3, 1).unwrap()),
        ("insert below", |m| m.insert_rows(0, 7, 2).unwrap()),
        ("delete inside", |m| m.delete_rows(0, 3, 1).unwrap()),
        ("delete below", |m| m.delete_rows(0, 6, 1).unwrap()),
        ("insert on the formula sheet", |m| {
            m.insert_rows(1, 1, 1).unwrap();
        }),
        ("insert inside on the formula sheet", |m| {
            m.insert_rows(1, 4, 2).unwrap();
        }),
        ("delete on the formula sheet", |m| {
            m.delete_rows(1, 2, 1).unwrap();
        }),
        ("insert on another sheet", |m| {
            m.add_sheet("Other").unwrap();
            m.insert_rows(2, 1, 3).unwrap();
        }),
        ("insert column left", |m| m.insert_columns(1, 1, 1).unwrap()),
    ];
    let followed = ["insert above", "insert inside"];
    let row_column = 2 * TEMPLATES.iter().position(|t| *t == "=ROW({x})").unwrap() as i32 + 1;
    let mut all = Vec::new();
    for (name, edit) in edits {
        let mut model = parity_model();
        edit(&mut model);
        model.evaluate();
        let first_column = if name == "insert column left" { 2 } else { 1 };
        if followed.contains(&name) {
            for row in 1..=8 {
                assert_eq!(
                    formula(&model, 1, row, row_column),
                    "=ROW(Sales[[#This Row],[Qty]])",
                    "{name} row {row}"
                );
            }
            retype_twins(&mut model, 1, first_column);
        }
        let mut found = mismatches(&model, 1, first_column, true);
        found.extend(mismatches(&model, 0, 5, true));
        all.extend(found.into_iter().map(|f| format!("{name}: {f}")));
    }
    assert!(all.is_empty(), "{}", all.join("\n"));
    // The insert above moves the table to A2:B6: row 2 is its header row now (`#VALUE!`) and row 6
    // its last data row, as Excel reads them (T-2845 table-insert-row-this-row-above-header).
    let mut model = parity_model();
    model.insert_rows(0, 1, 1).unwrap();
    model.evaluate();
    let rows: Vec<String> = (2..=6).map(|r| value(&model, 1, r, row_column)).collect();
    assert_eq!(rows, ["#VALUE!", "3", "4", "5", "6"]);
    // An edit of another sheet leaves the text of a formula on the table's sheet as it is.
    let mut model = parity_model();
    model.add_sheet("Other").unwrap();
    model.insert_rows(2, 1, 3).unwrap();
    assert_eq!(formula(&model, 0, 3, 5), "=Sales[[#This Row],[Qty]]");
    // So does an insert below the formula on its own sheet.
    model.insert_rows(1, 9, 1).unwrap();
    assert_eq!(formula(&model, 1, 3, 1), "=Sales[[#This Row],[Qty]]");
}

/// A cut and paste of the formula to another row reads the row it read; on the same row the
/// text is kept.
#[test]
fn cut_and_paste_of_the_formula() {
    for (target_row, target_column, keeps_text) in [(9, 3, false), (4, 3, false), (3, 6, true)] {
        let mut user = crate::UserModel::from_model(model());
        user.set_user_input(1, 3, 1, "=Sales[@Qty]&\"!\"").unwrap();
        user.set_user_input(1, 3, 2, "=T!$A$3&\"!\"").unwrap();
        user.set_selected_sheet(1).unwrap();
        user.set_selected_cell(3, 1).unwrap();
        user.set_selected_range(3, 1, 3, 2).unwrap();
        let clipboard = user.copy_to_clipboard().unwrap();
        user.set_selected_cell(target_row, target_column).unwrap();
        user.paste_from_clipboard(1, (3, 1, 3, 2), &clipboard.data, true)
            .unwrap();
        let model = user.get_model();
        let (row, column) = (target_row, target_column);
        assert_eq!(
            value(model, 1, row, column),
            value(model, 1, row, column + 1),
            "to {row},{column}"
        );
        assert_eq!(value(model, 1, row, column), "5!", "to {row},{column}");
        let text = formula(model, 1, row, column);
        assert_eq!(
            text == "=Sales[[#This Row],[Qty]]&\"!\"",
            keeps_text,
            "{text}"
        );
    }
}

/// A sheet rename keeps the text and the values; a delete of the table's sheet is `#REF!` on
/// every row, off the data rows too, as every structured reference to it is in Excel
/// (references.md "Delete the table's sheet", table-delete-sheet).
#[test]
fn sheet_rename_and_delete() {
    let mut model = model();
    model
        .set_user_input(1, 5, 1, "=Sales[@Qty]".to_string())
        .unwrap();
    model
        .set_user_input(1, 5, 2, "=T!$A$5".to_string())
        .unwrap();
    model
        .set_user_input(1, 8, 1, "=Sales[@Qty]".to_string())
        .unwrap();
    model.rename_sheet("T", "Facts").unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 1, 5, 1), "=Sales[[#This Row],[Qty]]");
    assert_eq!(value(&model, 1, 5, 1), "3");
    model.add_sheet("Spare").unwrap();
    model.take_sheet(0).unwrap();
    model.evaluate();
    assert_eq!(value(&model, 0, 5, 1), "#REF!");
    assert_eq!(value(&model, 0, 5, 1), value(&model, 0, 5, 2));
    assert_eq!(value(&model, 0, 8, 1), "#REF!");
}

/// A duplicated sheet gets no table (as before SPEC-0016): each `#This Row` formula on the copy
/// reads what it read on the source, the copy's cell on a data row and `#VALUE!` off them.
#[test]
fn a_duplicated_sheet_reads_its_own_row() {
    let mut model = parity_model();
    let (_, copy) = model.duplicate_sheet(0).unwrap();
    model.set_user_input(copy, 5, 1, "100".to_string()).unwrap();
    model.evaluate();
    let mut found = mismatches(&model, copy, 5, true);
    found.extend(mismatches(&model, 0, 5, false));
    assert!(found.is_empty(), "{}", found.join("\n"));
    assert_eq!(value(&model, copy, 5, 5), "100");
    assert_eq!(value(&model, copy, 6, 5), "#VALUE!");
    assert_eq!(value(&model, copy, 1, 5), "#VALUE!");
    assert_eq!(value(&model, 0, 5, 5), "3");
}

/// A copy and paste (not a cut) keeps the reference as written, which reads the row it is pasted
/// on, `#VALUE!` off the data rows. Hand expectation, unprobed: Excel copies `Sales[@Qty]`
/// unchanged, as filling a table column does (references.md, T-2297). Before SPEC-0016 step 2 the
/// copy held `T!$A$3` and read row 3 wherever it went.
#[test]
fn copy_and_paste_of_the_formula() {
    let mut user = crate::UserModel::from_model(model());
    user.set_user_input(1, 3, 1, "=Sales[@Qty]&\"!\"").unwrap();
    user.set_selected_sheet(1).unwrap();
    user.set_selected_cell(3, 1).unwrap();
    user.set_selected_range(3, 1, 3, 1).unwrap();
    let clipboard = user.copy_to_clipboard().unwrap();
    for (row, column, expected) in [(5, 1, "3!"), (9, 1, "#VALUE!"), (3, 4, "5!")] {
        user.set_selected_cell(row, column).unwrap();
        user.paste_from_clipboard(1, (3, 1, 3, 1), &clipboard.data, false)
            .unwrap();
        let model = user.get_model();
        assert_eq!(
            formula(model, 1, row, column),
            "=Sales[[#This Row],[Qty]]&\"!\""
        );
        assert_eq!(value(model, 1, row, column), expected, "to {row},{column}");
    }
}

/// A defined name has no cell: its `#This Row` area is resolved at the row its body is parsed at
/// (row 1, the header here), so it is `#VALUE!` wherever it is used, as before SPEC-0016 step 2.
/// Hand-flagged: Excel's rule is unprobed (T-2595).
#[test]
fn a_defined_name_resolves_at_row_one() {
    let mut model = model();
    model.define_name("ThisQty", None, "Sales[@Qty]").unwrap();
    model
        .define_name("ThisQtyTwice", None, "Sales[@Qty]*2")
        .unwrap();
    for row in [1, 3, 5, 8] {
        model
            .set_user_input(1, row, 1, "=ThisQty".to_string())
            .unwrap();
        model
            .set_user_input(1, row, 2, "=ThisQtyTwice".to_string())
            .unwrap();
    }
    model.evaluate();
    for row in [1, 3, 5, 8] {
        assert_eq!(value(&model, 1, row, 1), "#VALUE!", "row {row}");
        assert_eq!(value(&model, 1, row, 2), "#VALUE!", "row {row}");
    }
}

/// `Sales` on S!B3:D7 (Qty 1, 2, 4, 8; a calculated column D `=Sales[@Qty]*2`), `=Sales[@Qty]`
/// beside it on S!E4:E7 and on F!A2:A8: the layout of T-2845's table-insert-row-this-row-* and
/// -calc-column-* probes.
fn probe_model() -> Model {
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    model.rename_sheet("Sheet1", "S").unwrap();
    model.add_sheet("F").unwrap();
    for (row, qty) in [(3, "Qty"), (4, "1"), (5, "2"), (6, "4"), (7, "8")] {
        model.set_user_input(0, row, 2, "x".to_string()).unwrap();
        model.set_user_input(0, row, 3, qty.to_string()).unwrap();
    }
    model.set_user_input(0, 3, 4, "Dbl".to_string()).unwrap();
    model.add_table(0, "Sales", (3, 2, 7, 4), false).unwrap();
    for row in 4..=7 {
        let double = "=Sales[@Qty]*2".to_string();
        model.set_user_input(0, row, 4, double).unwrap();
        model
            .set_user_input(0, row, 5, "=Sales[@Qty]".to_string())
            .unwrap();
    }
    for row in 2..=8 {
        model
            .set_user_input(1, row, 1, "=Sales[@Qty]".to_string())
            .unwrap();
    }
    model.evaluate();
    model
}

/// The values of `sheet` column `column` on `rows`.
fn column_values(
    model: &Model,
    sheet: u32,
    column: i32,
    rows: std::ops::RangeInclusive<i32>,
) -> Vec<String> {
    rows.map(|row| value(model, sheet, row, column)).collect()
}

/// A row insert the table follows keeps every `#This Row` reference to it as written, and each
/// reads its cell's row in the moved or grown table (T-3037; excel-graph, T-2845).
#[test]
fn a_row_insert_the_table_follows_keeps_this_row() {
    // Above the header row: the table moves to B4:D8.
    let mut model = probe_model();
    model.insert_rows(0, 3, 1).unwrap();
    model.evaluate();
    // A fixed cell on another sheet: row 4 is the header row now, row 8 comes onto the data rows.
    assert_eq!(
        column_values(&model, 1, 1, 2..=8),
        ["#VALUE!", "#VALUE!", "#VALUE!", "1", "2", "4", "8"]
    );
    for row in 2..=8 {
        assert_eq!(formula(&model, 1, row, 1), "=Sales[[#This Row],[Qty]]");
    }
    // The cells beside the table move with it and read the same values.
    assert_eq!(column_values(&model, 0, 5, 5..=8), ["1", "2", "4", "8"]);
    assert_eq!(formula(&model, 0, 8, 5), "=Sales[[#This Row],[Qty]]");
    // The calculated column moves with the table, its last cell below the table's old last row.
    assert_eq!(column_values(&model, 0, 4, 5..=8), ["2", "4", "8", "16"]);
    assert_eq!(formula(&model, 0, 8, 4), "=[@Qty]*2");

    // Inside the data rows: the table grows to B3:D8 and row 6 is a new blank data row.
    let mut model = probe_model();
    model.insert_rows(0, 6, 1).unwrap();
    model.evaluate();
    assert_eq!(
        column_values(&model, 1, 1, 2..=8),
        ["#VALUE!", "#VALUE!", "1", "2", "0", "4", "8"]
    );
    assert_eq!(column_values(&model, 0, 5, 4..=8), ["1", "2", "", "4", "8"]);
    assert_eq!(formula(&model, 0, 8, 5), "=Sales[[#This Row],[Qty]]");
    assert_eq!(column_values(&model, 0, 3, 7..=8), ["4", "8"]);
}

/// An unqualified structured reference in a formula a row insert or delete moves keeps its text,
/// written without the table name, as an unmoved one beside it does (Excel never rewrites a
/// structured reference to a table that follows the edit); the insert above the table moves its
/// last row's formula below the table's old range, where it still names the table (T-3037).
#[test]
fn a_moved_unqualified_reference_keeps_its_text() {
    type Edit = fn(&mut Model);
    // Each edit with the first and last rows of the formulas after it.
    let edits: [(Edit, [i32; 2]); 3] = [
        (|m| m.insert_rows(0, 3, 1).unwrap(), [5, 8]),
        (|m| m.insert_rows(0, 6, 1).unwrap(), [4, 8]),
        (|m| m.delete_rows(0, 1, 1).unwrap(), [3, 6]),
    ];
    for (edit, rows) in edits {
        let mut model = probe_model();
        for row in 4..=7 {
            let sum = "=SUM([Qty])".to_string();
            model.set_user_input(0, row, 4, sum).unwrap();
        }
        edit(&mut model);
        model.evaluate();
        for row in rows {
            assert_eq!(formula(&model, 0, row, 4), "=SUM([Qty])", "row {row}");
            assert_eq!(value(&model, 0, row, 4), "15", "row {row}");
        }
    }
}
