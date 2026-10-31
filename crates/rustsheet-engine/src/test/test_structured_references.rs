#![allow(clippy::unwrap_used)]
//! Structured references kept in the parsed formula (SPEC-0016 step 1, T-2195).
//!
//! The edit tests pin today's values with a twin: each structured reference on `Calc` is paired
//! with the plain range it stood for before SPEC-0016. After every edit both have the same
//! value. An edit the table follows (it moves, grows or shrinks, T-2297) keeps the structured
//! text; under any other, the structured formula keeps its text unless the edit rewrote its twin,
//! in which case it has the twin's text (the fallback of SPEC-0016 "Structural edits").

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

/// Applies `edit`, which the table follows when `follows` is set, and checks every pair as the
/// module doc says.
fn check_edit(name: &str, follows: bool, edit: impl FnOnce(&mut Model)) {
    let mut model = model();
    edit(&mut model);
    model.evaluate();
    for (row, (structured, twin)) in PAIRS.iter().enumerate() {
        let row = row as i32 + 1;
        if follows {
            assert_eq!(formula(&model, 1, row, 1), *structured, "{name}");
        }
        // A twin rewritten to the same text (`$A$3:$A$2` read back as `$A$2:$A$3` after a row
        // move) leaves either text on the structured formula.
        let twin_now = formula(&model, 1, row, 2);
        let structured_now = formula(&model, 1, row, 1);
        if follows {
            // The twin moved with the data; the table moved with it.
        } else if twin_now == *twin {
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
fn the_export_keeps_the_reference() {
    let model = model();
    let context = CellReferenceRC {
        sheet: "Calc".to_string(),
        row: 1,
        column: 1,
    };
    let node = &model.parsed_formulas[1][0].evaluated;
    // The export writes the table parts, so the file keeps the reference (T-2196).
    assert_eq!(to_excel_string(node, &context), "SUM(Sales[Qty])");
    assert_eq!(
        crate::expressions::parser::stringify::to_excel_array_formula_string(node, &context),
        "SUM(Sales[Qty])"
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
    let node = &model.parsed_formulas[1][0].evaluated;
    assert_eq!(to_excel_string(node, &context), "#REF!");
}

/// The table (header row 1, data rows 2-3) moves with an insert at its header row, grows with
/// one at its first data row or inside its data (T-2321) and is left alone by one below it.
#[test]
fn row_inserts_keep_values() {
    for (row, follows) in [
        (1, true),
        (2, true),
        (3, true),
        (4, true),
        (5, true),
        (10, true),
    ] {
        check_edit(&format!("insert row {row}"), follows, |m| {
            m.insert_rows(0, row, 1).unwrap();
        });
    }
}

/// The table (header row 1, data rows 2-3) shrinks with a delete of either data row (T-2338) and
/// is left alone by one below it; a delete touching its header row, or of both data rows at once,
/// is a fallback.
#[test]
fn row_deletes_keep_values() {
    for (row, count, follows) in [
        (1, 1, false),
        (2, 1, true),
        (3, 1, true),
        (2, 2, false),
        (4, 1, true),
        (5, 1, true),
        (1, 5, false),
    ] {
        check_edit(&format!("delete rows {row}+{count}"), follows, |m| {
            m.delete_rows(0, row, count).unwrap();
        });
    }
}

/// The table over columns A:B moves with an insert at A, grows with one at B and is left alone
/// right of B (T-2339); column deletes inside it are `column_deletes_inside_drop_the_column`.
#[test]
fn column_inserts_and_deletes_keep_values() {
    // An insert at the second column grows the table (T-2339).
    for column in [1, 2, 3, 5] {
        check_edit(&format!("insert column {column}"), true, |m| {
            m.insert_columns(0, column, 1).unwrap();
        });
    }
    for column in [3, 5] {
        check_edit(&format!("delete column {column}"), true, |m| {
            m.delete_columns(0, column, 1).unwrap();
        });
    }
}

/// A column delete inside the table shrinks it and drops the column (T-2339): a reference to the
/// deleted column prints `#REF!`, as Excel does (probes/table-edits/table-delete-column-qty), and
/// the others keep their text and read the shrunk table. A column range that loses one end prints
/// the area it is displaced to, as under a fallback; its value is the plain range's, whose own
/// column-delete gaps are T-2851's (Excel is unprobed for both).
#[test]
fn column_deletes_inside_drop_the_column() {
    let qty = [
        ("=SUM(#REF!)", "#REF!"),
        ("=SUM(Sales[#All])", "30"),
        ("=ROWS(Sales)", "2"),
        // The plain range's column-delete corner gap, as above (T-2851).
        ("=SUM(#REF!:$A$3)", "#REF!"),
        ("=COUNTA(Sales[#Headers])", "1"),
        ("=SUM(Sales[[#Data],[Val]])", "30"),
    ];
    let val = [
        ("=SUM(Sales[Qty])", "3"),
        ("=SUM(Sales[#All])", "3"),
        ("=ROWS(Sales)", "2"),
        // Not Excel's value: the plain range's column-delete corner gap (T-2851).
        ("=SUM(Data!$A$2:#REF!)", "#ERROR!"),
        ("=COUNTA(Sales[#Headers])", "1"),
        ("=SUM(#REF!)", "#REF!"),
    ];
    for (column, expected) in [(1, qty), (2, val)] {
        let mut model = model();
        model.delete_columns(0, column, 1).unwrap();
        model.evaluate();
        for (row, (text, result)) in (1..).zip(expected) {
            assert_eq!(formula(&model, 1, row, 1), text, "delete column {column}");
            assert_eq!(
                value(&model, 1, row, 1),
                result,
                "delete column {column}: {text}"
            );
        }
    }
}

/// A column inserted inside the table is named `Column1`, in its header cell too (T-2339,
/// probes/table-edits/table-insert-column-inside-header).
#[test]
fn a_column_inserted_inside_is_named_in_its_header() {
    let mut model = model();
    model.insert_columns(0, 2, 1).unwrap();
    model
        .set_user_input(1, 10, 1, "=INDEX(Sales[#Headers],1,2)".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(value(&model, 0, 1, 2), "Column1");
    assert_eq!(value(&model, 1, 10, 1), "Column1");
    assert_eq!(formula(&model, 1, 1, 1), "=SUM(Sales[Qty])");
}

#[test]
fn row_and_column_moves_keep_values() {
    for (row, delta) in [(2, 1), (3, -1), (1, 3), (5, -4)] {
        check_edit(&format!("move row {row} by {delta}"), false, |m| {
            m.move_rows_action(0, row, 1, delta).unwrap();
        });
    }
    for (column, delta) in [(1, 1), (2, -1), (3, -2)] {
        check_edit(&format!("move column {column} by {delta}"), false, |m| {
            m.move_columns_action(0, column, 1, delta).unwrap();
        });
    }
}

/// `Sales` over `Data!B2:C7` (header row 2, Qty 1, 2, 4, 8, 16 in B3:B7), referred to from
/// `Calc`, from its own sheet, through a defined name and from a conditional format.
fn wide_model() -> Model {
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model.add_sheet("Calc").unwrap();
    model.set_user_input(0, 2, 2, "Qty".to_string()).unwrap();
    model.set_user_input(0, 2, 3, "Val".to_string()).unwrap();
    for (row, qty) in [(3, "1"), (4, "2"), (5, "4"), (6, "8"), (7, "16")] {
        model.set_user_input(0, row, 2, qty.to_string()).unwrap();
    }
    model.add_table(0, "Sales", (2, 2, 7, 3), false).unwrap();
    model.define_name("QtyName", None, "Sales[Qty]").unwrap();
    for (row, text) in [
        (1, "=SUM(Sales[Qty])"),
        (2, "=SUM(QtyName)"),
        (3, "=ROWS(Sales[#All])"),
        (4, "=COLUMN(Sales[Qty])"),
    ] {
        model.set_user_input(1, row, 1, text.to_string()).unwrap();
    }
    model
        .set_user_input(0, 1, 6, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model.workbook.worksheets[0].conditional_formatting.push(
        crate::cf_types::ConditionalFormatting {
            range: "E1:E2".to_string(),
            cf_rule: crate::cf_types::CfRule::Formula {
                formula: "SUM(Sales[Qty])>30".to_string(),
                dxf_id: 0,
                stop_if_true: false,
            },
            priority: 1,
        },
    );
    model.evaluate();
    model
}

/// The `Calc` values, the formula on the table's sheet (wherever the edit moved it), its value,
/// and the conditional format's formula.
fn wide_state(model: &Model) -> (Vec<String>, String, String, String) {
    let calc = (1..=4).map(|row| value(model, 1, row, 1)).collect();
    let (row, column) = model.workbook.worksheets[0]
        .sheet_data
        .cells()
        .find(|(_, _, cell)| cell.get_formula().is_some())
        .map(|(row, column, _)| (row, column))
        .unwrap();
    let cf = match &model.workbook.worksheets[0].conditional_formatting[0].cf_rule {
        crate::cf_types::CfRule::Formula { formula, .. } => formula.clone(),
        other => panic!("{other:?}"),
    };
    (
        calc,
        formula(model, 0, row, column),
        value(model, 0, row, column),
        cf,
    )
}

/// A delete inside the data rows shrinks the table, a column delete left of it moves it
/// left, and a row insert inside its data grows it: every reference keeps its text, from another
/// sheet, from the table's sheet, through a defined name and from a conditional format, and reads
/// the table as it now is (references.md "Structured references under row deletes, column edits,
/// convert, table and sheet delete", T-2324), a delete at an edge of the data too (T-2338).
#[test]
fn a_table_shrinks_and_moves_left() {
    let kept = |calc: [&str; 4]| {
        (
            calc.map(str::to_string).to_vec(),
            "=SUM(Sales[Qty])".to_string(),
            calc[0].to_string(),
            "SUM(Sales[Qty])>30".to_string(),
        )
    };
    let mut model = wide_model();
    assert_eq!(wide_state(&model), kept(["31", "31", "6", "2"]));
    model.delete_rows(0, 4, 2).unwrap();
    model.evaluate();
    assert_eq!(wide_state(&model), kept(["25", "25", "4", "2"]));
    model.delete_columns(0, 1, 1).unwrap();
    model.evaluate();
    assert_eq!(wide_state(&model), kept(["25", "25", "4", "1"]));
    model.insert_rows(0, 4, 3).unwrap();
    model.evaluate();
    assert_eq!(wide_state(&model), kept(["25", "25", "7", "1"]));
    assert_eq!(model.workbook.tables["Sales"].reference, "A2:B8");
    // A formula typed after the edits reads the table where it is now.
    model
        .set_user_input(1, 9, 1, "=ROWS(Sales)".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(value(&model, 1, 9, 1), "6");

    // A delete of the last data row (8) shrinks the table too (T-2338).
    model.delete_rows(0, 8, 1).unwrap();
    model.evaluate();
    let (_, text, _, cf) = wide_state(&model);
    assert_eq!(formula(&model, 1, 1, 1), "=SUM(Sales[Qty])");
    assert_eq!(text, "=SUM(Sales[Qty])");
    assert_eq!(cf, "SUM(Sales[Qty])>30");
    assert_eq!(model.workbook.tables["Sales"].reference, "A2:B7");
    assert_eq!(value(&model, 1, 9, 1), "5");
}

/// An edit on another sheet leaves the table, and so its references, alone.
#[test]
fn an_edit_of_another_sheet_keeps_the_table() {
    let mut model = wide_model();
    model.insert_rows(1, 1, 2).unwrap();
    model.delete_columns(1, 2, 1).unwrap();
    model.evaluate();
    assert_eq!(model.workbook.tables["Sales"].reference, "B2:C7");
    assert_eq!(formula(&model, 1, 3, 1), "=SUM(Sales[Qty])");
    assert_eq!(value(&model, 1, 3, 1), "31");
}

#[test]
fn a_formula_on_the_table_sheet_keeps_its_value() {
    let mut model = model();
    model
        .set_user_input(0, 1, 6, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model.insert_rows(0, 1, 1).unwrap();
    model.evaluate();
    // The table moved with the data (T-2297).
    assert_eq!(formula(&model, 0, 2, 6), "=SUM(Sales[Qty])");
    assert_eq!(value(&model, 0, 2, 6), "3");
    // An insert at its first data row grows it (T-2321).
    model.insert_rows(0, 3, 1).unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 0, 2, 6), "=SUM(Sales[Qty])");
    assert_eq!(value(&model, 0, 2, 6), "3");
    assert_eq!(model.workbook.tables["Sales"].reference, "A2:B5");
    model.insert_rows(0, 20, 1).unwrap();
    model
        .set_user_input(0, 10, 6, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model.insert_rows(0, 20, 1).unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 0, 10, 6), "=SUM(Sales[Qty])");
}

/// T-2197's `Sales` on S!B3:C7 (header row 3, data rows 4-7, Qty 1, 2, 4, 8), with a totals row
/// 8 when `totals` is set, and on `F` the formulas `=SUM(Sales[Qty])`, `=ROWS(Sales)` and the
/// plain ranges `=SUM(S!C4:C7)` (the data column), `=SUM(S!C5:C7)` (inside it, off its edges)
/// and `=SUM(S!B4:D7)` (wider than the table).
fn edge_model(totals: bool) -> Model {
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    model.rename_sheet("Sheet1", "S").unwrap();
    model.add_sheet("F").unwrap();
    let mut cells = vec![(3, 2, "Region"), (3, 3, "Qty")];
    for (row, (region, qty)) in [("E", "1"), ("W", "2"), ("N", "4"), ("S", "8")]
        .into_iter()
        .enumerate()
    {
        let row = row as i32 + 4;
        cells.extend([(row, 2, region), (row, 3, qty)]);
    }
    for (row, column, input) in cells {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    let bottom = if totals { 8 } else { 7 };
    model
        .add_table_with_totals(0, "Sales", (3, 2, bottom, 3), false, totals)
        .unwrap();
    if totals {
        let total = "=SUBTOTAL(109,Sales[Qty])".to_string();
        model.set_user_input(0, 8, 3, total).unwrap();
    }
    for (row, input) in [
        "=SUM(Sales[Qty])",
        "=ROWS(Sales)",
        "=SUM(S!C4:C7)",
        "=SUM(S!C5:C7)",
        "=SUM(S!B4:D7)",
    ]
    .into_iter()
    .enumerate()
    {
        model
            .set_user_input(1, row as i32 + 1, 1, input.to_string())
            .unwrap();
    }
    model.evaluate();
    model
}

/// The formulas of `F` column A after inserting one row above `row`, and the table's reference.
fn edge_insert(totals: bool, row: i32) -> (Vec<String>, String) {
    let mut model = edge_model(totals);
    model.insert_rows(0, row, 1).unwrap();
    model.evaluate();
    let formulas = (1..=4).map(|r| formula(&model, 1, r, 1)).collect();
    (formulas, model.workbook.tables["Sales"].reference.clone())
}

/// Excel grows the table on an insert at its first data row and at its totals row, and a plain
/// range onto its data column grows with it (T-2197 table-insert-row-above-first-data and
/// -above-totals: `=SUM(S!C4:C7)` becomes `=SUM(S!C4:C8)`); a range off that edge, or wider than
/// the table, displaces as usual.
#[test]
fn a_table_and_ranges_onto_its_data_grow_at_its_edges() {
    let formulas = |plain: [&str; 3]| {
        let mut all = vec!["=SUM(Sales[Qty])".to_string(), "=ROWS(Sales)".to_string()];
        all.extend(plain.map(str::to_string));
        all
    };
    // Only `=SUM(S!C4:C8)` is oracle-backed; the off-edge and wider ranges are hand expectations
    // (an ordinary displacement) until T-2846 probes them.
    // Above the first data row.
    let grown = formulas(["=SUM(S!C4:C8)", "=SUM(S!C6:C8)", "=SUM(S!B5:D8)"]);
    for totals in [false, true] {
        let bottom = if totals { 9 } else { 8 };
        assert_eq!(
            edge_insert(totals, 4),
            (grown.clone(), format!("B3:C{bottom}"))
        );
    }
    // Above the totals row.
    let above_totals = formulas(["=SUM(S!C4:C8)", "=SUM(S!C5:C8)", "=SUM(S!B4:D7)"]);
    assert_eq!(edge_insert(true, 8), (above_totals, "B3:C9".to_string()));
    // Just below the table: nothing changes.
    let unchanged = formulas(["=SUM(S!C4:C7)", "=SUM(S!C5:C7)", "=SUM(S!B4:D7)"]);
    assert_eq!(
        edge_insert(false, 8),
        (unchanged.clone(), "B3:C7".to_string())
    );
    assert_eq!(edge_insert(true, 9), (unchanged, "B3:C8".to_string()));

    // The values read the grown table: one blank data row, the totals row still sums the data.
    let mut model = edge_model(true);
    model.insert_rows(0, 8, 1).unwrap();
    model.evaluate();
    let values: Vec<String> = (1..=3).map(|r| value(&model, 1, r, 1)).collect();
    assert_eq!(values, ["15", "5", "15"]);
    assert_eq!(value(&model, 0, 9, 3), "15");
}

/// A row delete that takes a plain range's first or last rows shrinks it to the rows left
/// (plain-range-delete-row-first and -last, T-2338); one that takes all of it is `#REF!`.
#[test]
fn a_row_delete_at_a_range_edge_shrinks_it() {
    let text = |delete: (i32, i32), input: &str| {
        let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
        model.rename_sheet("Sheet1", "S").unwrap();
        model.add_sheet("F").unwrap();
        model.set_user_input(1, 1, 1, input.to_string()).unwrap();
        model.delete_rows(0, delete.0, delete.1).unwrap();
        formula(&model, 1, 1, 1)
    };
    assert_eq!(text((4, 1), "=SUM(S!C4:C7)"), "=SUM(S!C4:C6)");
    assert_eq!(text((7, 1), "=SUM(S!C4:C7)"), "=SUM(S!C4:C6)");
    assert_eq!(text((4, 1), "=SUM(S!$C$4:$C$7)"), "=SUM(S!$C$4:$C$6)");
    assert_eq!(text((7, 1), "=SUM(S!C7:C9)"), "=SUM(S!C7:C8)");
    // Across an edge with more than one row (hand: only one-row deletes are probed).
    assert_eq!(text((3, 3), "=SUM(S!C4:C7)"), "=SUM(S!C3:C4)");
    assert_eq!(text((6, 3), "=SUM(S!C4:C7)"), "=SUM(S!C4:C5)");
    // Inside or around the range: as before.
    assert_eq!(text((5, 1), "=SUM(S!C4:C7)"), "=SUM(S!C4:C6)");
    assert_eq!(text((1, 1), "=SUM(S!C4:C7)"), "=SUM(S!C3:C6)");
    assert!(text((4, 1), "=SUM(S!C4:C4)").contains("#REF!"));
}

/// Deleting a table's only data row keeps the row, cleared: nothing below moves
/// (table-delete-row-all-data-below, T-2338), and undo restores the row without moving anything.
#[test]
fn deleting_the_last_data_row_clears_it_and_undo_restores_it() {
    let mut model = edge_model(false);
    model.set_user_input(0, 10, 3, "99".to_string()).unwrap();
    model.set_user_input(1, 6, 1, "=S!C10".to_string()).unwrap();
    for _ in 0..3 {
        model.delete_rows(0, 4, 1).unwrap();
    }
    model.evaluate();
    assert_eq!(model.workbook.tables["Sales"].reference, "B3:C4");
    assert_eq!(formula(&model, 1, 6, 1), "=S!C7");
    let mut user = crate::UserModel::from_model(model);
    user.delete_rows(0, 4, 1).unwrap();
    let model = user.get_model();
    assert_eq!(model.workbook.tables["Sales"].reference, "B3:C4");
    assert_eq!(formula(model, 1, 6, 1), "=S!C7");
    assert_eq!(value(model, 1, 2, 1), "1");
    assert_eq!(value(model, 1, 1, 1), "0");
    assert_eq!(value(model, 0, 4, 3), "");
    user.undo().unwrap();
    let model = user.get_model();
    assert_eq!(model.workbook.tables["Sales"].reference, "B3:C4");
    assert_eq!(value(model, 0, 4, 3), "8");
    assert_eq!(value(model, 0, 7, 3), "99");
    user.redo().unwrap();
    assert_eq!(value(user.get_model(), 0, 4, 3), "");
    assert_eq!(value(user.get_model(), 0, 7, 3), "99");
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
    let node = &model.parsed_formulas[1][0].evaluated;
    // The table part is written on the renamed sheet, so the file keeps the reference (T-2196).
    assert_eq!(to_excel_string(node, &context), "SUM(Sales[Qty])");
    assert_eq!(model.workbook.tables["Sales"].sheet_name, "Facts");
}

#[test]
fn a_sheet_delete_is_a_ref_error_and_restore_brings_the_tables_back() {
    let mut model = model();
    model.add_sheet("Spare").unwrap();
    model.define_name("Total", None, "SUM(Sales[Qty])").unwrap();
    let taken = model.take_sheet(0).unwrap();
    model.evaluate();
    // `Calc` is now sheet 0. As in Excel, a structured reference to the deleted sheet's table
    // becomes `#REF!` and a plain one is displayed with `#REF!` for its sheet (T-2340).
    assert_eq!(value(&model, 0, 1, 1), "#REF!");
    assert_eq!(formula(&model, 0, 1, 1), "=SUM(#REF!)");
    assert_eq!(formula(&model, 0, 1, 2), "=SUM(#REF!$A$2:$A$3)");
    for (row, _) in PAIRS.iter().enumerate() {
        let row = row as i32 + 1;
        assert!(!formula(&model, 0, row, 1).contains("Sales"));
        assert_eq!(value(&model, 0, row, 1), value(&model, 0, row, 2));
    }
    let names = model.workbook.defined_names.clone();
    assert_eq!(names[0].formula, "SUM(#REF!)");
    assert!(model.workbook.tables.is_empty());
    // A new sheet with the old name does not bring the table back. The plain twin, whose stored
    // text keeps the sheet name, reads the new sheet: a known gap from Excel (T-2612).
    model.add_sheet("Data").unwrap();
    model.evaluate();
    assert_eq!(value(&model, 0, 1, 1), "#REF!");
    assert_eq!(value(&model, 0, 1, 2), "0");
    model
        .delete_sheet(model.get_sheet_index_by_name("Data").unwrap())
        .unwrap();
    model.restore_sheet(0, taken).unwrap();
    model.evaluate();
    assert!(model.workbook.tables.contains_key("Sales"));
    // The restore puts the structured references back.
    assert_eq!(formula(&model, 1, 1, 1), "=SUM(Sales[Qty])");
    assert_eq!(model.workbook.defined_names[0].formula, "SUM(Sales[Qty])");
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

#[test]
fn a_duplicated_sheet_reads_its_own_cells() {
    let mut model = model();
    model
        .set_user_input(0, 10, 6, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model
        .set_user_input(0, 11, 6, "=SUM($A$2:$A$3)".to_string())
        .unwrap();
    let (_, copy) = model.duplicate_sheet(0).unwrap();
    model.set_user_input(copy, 2, 1, "100".to_string()).unwrap();
    model.evaluate();
    // As before SPEC-0016: the copy has no table, and its formula reads the copy's cells.
    assert_eq!(value(&model, copy, 10, 6), value(&model, copy, 11, 6));
    assert_eq!(value(&model, copy, 10, 6), "102");
    assert_eq!(formula(&model, 0, 10, 6), "=SUM(Sales[Qty])");
    assert_eq!(value(&model, 0, 10, 6), "3");
}

#[test]
fn undoing_a_sheet_delete_brings_the_tables_back() {
    let mut user = crate::UserModel::from_model(model());
    user.delete_sheet(0).unwrap();
    assert_eq!(user.get_formatted_cell_value(0, 1, 1).unwrap(), "#REF!");
    user.undo().unwrap();
    assert_eq!(user.get_formatted_cell_value(1, 1, 1).unwrap(), "3");
    // The undo puts back the structured references the delete made `#REF!` (T-2340).
    assert_eq!(user.get_cell_content(1, 1, 1).unwrap(), "=SUM(Sales[Qty])");
    // The table is back: a new structured reference resolves.
    user.set_user_input(1, 20, 1, "=SUM(Sales[Qty])").unwrap();
    assert_eq!(user.get_formatted_cell_value(1, 20, 1).unwrap(), "3");
}

/// Table names made of `R`, `C` and digits that no relative R1C1 reference starts with still read
/// back from storage (R1C1) after a reparse.
#[test]
fn r_and_c_table_names_survive_a_reparse() {
    for name in ["CR", "RR", "CC", "RCR", "RCC"] {
        let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
        for (row, value) in [(1, "Qty"), (2, "1"), (3, "2")] {
            model.set_user_input(0, row, 1, value.to_string()).unwrap();
        }
        model.add_table(0, name, (1, 1, 3, 1), false).unwrap();
        model
            .set_user_input(0, 1, 5, format!("=SUM({name}[Qty])"))
            .unwrap();
        model.rename_sheet("Sheet1", "Moved").unwrap();
        model.evaluate();
        assert_eq!(value(&model, 0, 1, 5), "3", "{name}");
        assert_eq!(formula(&model, 0, 1, 5), format!("=SUM({name}[Qty])"));
    }
}

#[test]
fn delete_table_clears_the_cells_and_a_huge_range_is_clamped() {
    // T-2341: Graph's DELETE tables clears the range (probes/table-edits/table-delete-table).
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "Qty".to_string()).unwrap();
    model.set_user_input(0, 2, 1, "5".to_string()).unwrap();
    model.set_user_input(0, 4, 1, "7".to_string()).unwrap();
    model.add_table(0, "Sales", (1, 1, 2, 1), false).unwrap();
    model.delete_table("Sales").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "7");
    assert!(model.delete_table("Sales").is_err());
    // A table over every row of the sheet must not walk every cell.
    model.set_user_input(0, 1, 3, "Big".to_string()).unwrap();
    model.set_user_input(0, 2, 3, "1".to_string()).unwrap();
    model
        .add_table(0, "Whole", (1, 3, 1_048_576, 3), false)
        .unwrap();
    model.delete_table("Whole").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "");
    // Nor may clearing the whole sheet walk every position: cells in two far corners only.
    model
        .set_user_input(0, 1_048_576, 1, "x".to_string())
        .unwrap();
    model.set_user_input(0, 1, 16_384, "y".to_string()).unwrap();
    model
        .range_clear_all(&crate::expressions::types::Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 16_384,
            height: 1_048_576,
        })
        .unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1_048_576, 1).unwrap(), "");
}

#[test]
fn a_table_delete_makes_its_references_ref_errors_for_good() {
    // probes/table-edits/table-delete-table (T-2340): the text becomes `#REF!`, so a later
    // reparse (here from adding a sheet) keeps it.
    let mut model = model();
    model
        .set_user_input(1, 20, 1, "=ROWS(Sales)".to_string())
        .unwrap();
    model.delete_table("Sales").unwrap();
    model.add_sheet("Later").unwrap();
    model.evaluate();
    assert_eq!(formula(&model, 1, 1, 1), "=SUM(#REF!)");
    assert_eq!(value(&model, 1, 1, 1), "#REF!");
    assert_eq!(formula(&model, 1, 20, 1), "=ROWS(#REF!)");
    assert_eq!(value(&model, 1, 20, 1), "#REF!");
}

#[test]
fn a_displayed_deleted_sheet_reference_reads_back_as_ref() {
    // Excel displays `S!C4:C7` as `#REF!C4:C7` once S is deleted (T-2340); typed back, that
    // text is the #REF! error, not a parse error.
    let mut model = model();
    for (row, input) in ["=SUM(#REF!A1:A3)", "=#REF!$B$2", "=COUNTA(#REF!A1)"]
        .iter()
        .enumerate()
    {
        model
            .set_user_input(1, row as i32 + 30, 1, (*input).to_string())
            .unwrap();
    }
    model.evaluate();
    assert_eq!(value(&model, 1, 30, 1), "#REF!");
    assert_eq!(value(&model, 1, 31, 1), "#REF!");
    assert_eq!(value(&model, 1, 32, 1), "1");
}

#[test]
fn convert_to_range_keeps_cells_and_makes_references_absolute_ranges() {
    // T-2342: probes/table-edits/table-convert-to-range.
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "Qty".to_string()).unwrap();
    model.set_user_input(0, 2, 1, "5".to_string()).unwrap();
    model.set_user_input(0, 3, 1, "7".to_string()).unwrap();
    model.add_table(0, "Sales", (1, 1, 3, 1), false).unwrap();
    model
        .set_user_input(0, 1, 3, "=SUM(Sales[Qty])".to_string())
        .unwrap();
    model
        .set_user_input(0, 2, 3, "=ROWS(Sales[#All])".to_string())
        .unwrap();
    model.convert_table_to_range("Sales").unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "12");
    assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "3");
    assert_eq!(
        model.get_localized_cell_content(0, 1, 3).unwrap(),
        "=SUM($A$2:$A$3)"
    );
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "7");
    assert!(model.convert_table_to_range("Sales").is_err());
}

/// Convert to range: a `#This Row` reference off the data rows stays `#VALUE!` (below the
/// table and on another sheet), while one on a data row reads its own cell with the column
/// absolute and the row relative (probes/table-edits/table-convert-names-this-row and
/// table-convert-same-sheet, T-2643).
#[test]
fn convert_to_range_lowers_this_row_at_each_row() {
    let mut model = model();
    for row in [2, 3, 5] {
        model
            .set_user_input(0, row, 4, "=Sales[[#This Row],[Qty]]*2".to_string())
            .unwrap();
    }
    model
        .set_user_input(1, 5, 4, "=Sales[[#This Row],[Qty]]".to_string())
        .unwrap();
    model.convert_table_to_range("Sales").unwrap();
    model.evaluate();
    assert_eq!(value(&model, 0, 5, 4), "#VALUE!");
    assert_eq!(value(&model, 0, 2, 4), "2");
    assert_eq!(value(&model, 0, 3, 4), "4");
    assert_eq!(model.get_localized_cell_content(0, 3, 4).unwrap(), "=$A3*2");
    assert_eq!(value(&model, 1, 5, 4), "#VALUE!");
}

/// INDIRECT of structured-reference text is the table area; a column the table lacks, a table
/// the workbook lacks are `#REF!` (docs/excel-semantics/lookup.md, T-2911).
#[test]
fn indirect_of_structured_reference_text() {
    let mut model = model();
    for (row, formula) in [
        "=SUM(INDIRECT(\"Sales[Val]\"))",
        "=ROWS(INDIRECT(\"Sales[Val]\"))",
        "=INDIRECT(\"Sales[Nope]\")",
        "=INDIRECT(\"Nope[Val]\")",
    ]
    .iter()
    .enumerate()
    {
        model
            .set_user_input(1, row as i32 + 1, 5, formula.to_string())
            .unwrap();
    }
    model.evaluate();
    let got: Vec<String> = (1..=4).map(|row| value(&model, 1, row, 5)).collect();
    assert_eq!(got, ["30", "2", "#REF!", "#REF!"]);
}
