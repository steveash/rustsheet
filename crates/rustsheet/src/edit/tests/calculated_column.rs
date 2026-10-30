//! A formula typed into a table column fills it as a calculated column (T-2958;
//! docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table column").

use super::*;

/// `T` over A1:C4: `Col` 1, 2, 4, `Other` 10, 20, 40 and an empty `Calc` column, as in the
/// probes/t2957-* cases.
fn table_book() -> Workbook {
    table_book_with_totals(false)
}

/// [`table_book`], with a totals row 5 (`Total` in A5) when `totals` is set, as in
/// probes/t3003-totals-*.
fn table_book_with_totals(totals: bool) -> Workbook {
    let mut wb = book();
    for (a1, input) in [
        ("A1", "Col"),
        ("B1", "Other"),
        ("C1", "Calc"),
        ("A2", "1"),
        ("B2", "10"),
        ("A3", "2"),
        ("B3", "20"),
        ("A4", "4"),
        ("B4", "40"),
    ] {
        wb.set(a1, input).unwrap();
    }
    if totals {
        wb.set("A5", "Total").unwrap();
    }
    let range = wb
        .range_ref(if totals { "A1:C5" } else { "A1:C4" })
        .unwrap();
    wb.edit(|tx| tx.add_table_with_totals(range, "T", false, totals))
        .unwrap();
    wb
}

fn formula(wb: &Workbook, a1: &str) -> Option<String> {
    wb.cell(a1).unwrap().formula
}

#[test]
fn a_typed_formula_fills_an_empty_column_and_reports_every_row() {
    let mut wb = table_book();
    let changes = wb.set("C3", "=T[@Col]*2").unwrap();
    let typed_text = formula(&wb, "C3");
    assert!(typed_text.is_some());
    for (a1, n) in [("C2", 2.0), ("C3", 4.0), ("C4", 8.0)] {
        assert_eq!(wb.value(a1).unwrap(), Value::Number(n), "{a1}");
        assert_eq!(formula(&wb, a1), typed_text, "{a1}");
    }
    assert_eq!(changes.inputs.len(), 3);
}

#[test]
fn a_formula_input_fills_too_and_a_stored_one_does_not() {
    let mut wb = table_book();
    wb.set("C2", Input::StoredFormula("=T[@Col]*2".into()))
        .unwrap();
    assert_eq!(formula(&wb, "C3"), None);
    let mut wb = table_book();
    wb.set("C2", Input::Formula("=T[@Col]*2".into())).unwrap();
    assert_eq!(wb.value("C4").unwrap(), Value::Number(8.0));
}

/// probes/t2957-refill-second, -constant-exception, -constant-then-refill, -existing-constant.
#[test]
fn a_filled_column_refills_unless_it_holds_an_exception() {
    let mut wb = table_book();
    wb.set("C2", "=T[[#This Row],[Col]]*2").unwrap();
    wb.set("C3", "=T[@Col]*3").unwrap();
    assert_eq!(wb.value("C2").unwrap(), Value::Number(3.0));
    assert_eq!(formula(&wb, "C4"), formula(&wb, "C3"));
    // A constant is an exception, and a column with one is not re-filled.
    wb.set("C3", "5").unwrap();
    assert_eq!(wb.value("C4").unwrap(), Value::Number(12.0));
    wb.set("C4", "=T[@Col]*4").unwrap();
    assert_eq!(wb.value("C2").unwrap(), Value::Number(3.0));
    assert_eq!(wb.value("C3").unwrap(), Value::Number(5.0));
    assert_eq!(wb.value("C4").unwrap(), Value::Number(16.0));
}

/// probes/t2957-a1-reference: an A1 formula fills shifted.
#[test]
fn an_a1_formula_fills_shifted() {
    let mut wb = table_book();
    wb.set("C2", "=A2*2").unwrap();
    assert_eq!(formula(&wb, "C3").as_deref(), Some("=A3*2"));
    assert_eq!(wb.value("C4").unwrap(), Value::Number(8.0));
}

#[test]
fn rollback_restores_the_filled_column() {
    let mut wb = table_book();
    wb.set("C2", "=T[@Col]*2").unwrap();
    let before = dump(&wb);
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("C3", "=T[@Col]*3")?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);
}

/// A rolled-back fill leaves the column no calculated column (T-3024): identical stored formulas
/// written afterwards are not re-filled by a typed one, as probes/t2983-identical-stored-refill.
#[test]
fn rollback_restores_the_column_formula() {
    let mut wb = table_book();
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("C2", "=A2*2")?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(formula(&wb, "C3"), None);
    for row in [2, 4] {
        let stored = Input::StoredFormula(format!("=A{row}*2"));
        wb.set(format!("C{row}").as_str(), stored).unwrap();
    }
    wb.set("C3", "=A3*3").unwrap();
    assert_eq!(wb.value("C2").unwrap(), Value::Number(2.0));
    assert_eq!(wb.value("C3").unwrap(), Value::Number(6.0));
}

/// probes/t2957-expand-right-data: a formula typed just right of the table grows it by a column
/// named `Column1` and fills it, all in one edit (T-2968).
#[test]
fn a_formula_typed_beside_the_table_grows_it_in_one_edit() {
    let mut wb = table_book();
    wb.set("H1", "=COLUMNS(T[#All])").unwrap();
    let changes = wb.set("D2", "=A2*2").unwrap();
    assert_eq!(wb.value("H1").unwrap(), Value::Number(4.0));
    assert_eq!(wb.value("D1").unwrap(), Value::Text("Column1".into()));
    assert_eq!(formula(&wb, "D4").as_deref(), Some("=A4*2"));
    assert_eq!(wb.value("D4").unwrap(), Value::Number(8.0));
    // D1:D4 are the edit's inputs.
    assert_eq!(changes.inputs.len(), 4);
}

/// Rolling the growing edit back puts the table, its header cell and the filled rows back.
#[test]
fn rollback_restores_the_table_a_typed_formula_grew() {
    let mut wb = table_book();
    wb.set("H1", "=COLUMNS(T[#All])").unwrap();
    let before = dump(&wb);
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("D1", "=1+1")?;
        tx.set("D2", "=A2*2")?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);
    assert_eq!(wb.value("H1").unwrap(), Value::Number(3.0));
    // The table is its old size again: typing beside it grows it once more.
    wb.set("D2", "=A2*2").unwrap();
    assert_eq!(wb.value("H1").unwrap(), Value::Number(4.0));
}

/// probes/t3003-const-*: a constant typed just right of the table grows it too, but fills nothing;
/// typed into the header cell it becomes the header text, `5` as the text `5` (rule 7a, T-3218).
#[test]
fn a_constant_typed_beside_the_table_grows_it_and_fills_nothing() {
    let mut wb = table_book();
    wb.set("H1", "=COLUMNS(T[#All])").unwrap();
    let changes = wb.set("D2", "5").unwrap();
    assert_eq!(wb.value("H1").unwrap(), Value::Number(4.0));
    assert_eq!(wb.value("D1").unwrap(), Value::Text("Column1".into()));
    assert_eq!(wb.value("D2").unwrap(), Value::Number(5.0));
    assert_eq!(wb.value("D3").unwrap(), Value::Empty);
    // D2 and the header D1 are the edit's inputs.
    assert_eq!(changes.inputs.len(), 2);

    let mut wb = table_book();
    wb.set("H1", "=COLUMNS(T[#All])").unwrap();
    wb.set("D1", "5").unwrap();
    assert_eq!(wb.value("H1").unwrap(), Value::Number(4.0));
    assert_eq!(wb.value("D1").unwrap(), Value::Text("5".into()));
    assert_eq!(wb.value("D2").unwrap(), Value::Empty);
}

/// Rolling back a typed constant that grew the table puts the table and its header cell back.
#[test]
fn rollback_restores_the_table_a_typed_constant_grew() {
    let mut wb = table_book();
    wb.set("H1", "=COLUMNS(T[#All])").unwrap();
    let before = dump(&wb);
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("D2", "hello")?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);
    assert_eq!(wb.value("H1").unwrap(), Value::Number(3.0));
    assert_eq!(wb.value("D1").unwrap(), Value::Empty);
}

/// probes/t3003-totals-*: a totals row does not stop the growth. A formula typed beside a data row
/// fills the other data rows but not the totals cell; typed beside the totals row it fills
/// nothing (rule 7c, T-3218).
#[test]
fn a_table_with_a_totals_row_grows_too() {
    let totals_book = || {
        let mut wb = table_book_with_totals(true);
        wb.set("H1", "=COLUMNS(T[#All])").unwrap();
        wb
    };
    let mut wb = totals_book();
    let changes = wb.set("D2", "=A2*2").unwrap();
    assert_eq!(wb.value("H1").unwrap(), Value::Number(4.0));
    assert_eq!(wb.value("D1").unwrap(), Value::Text("Column1".into()));
    assert_eq!(wb.value("D4").unwrap(), Value::Number(8.0));
    assert_eq!(wb.value("D5").unwrap(), Value::Empty);
    // D1:D4 are the edit's inputs; the totals cell D5 is not written.
    assert_eq!(changes.inputs.len(), 4);

    let mut wb = totals_book();
    wb.set("D5", "=A2*2").unwrap();
    assert_eq!(wb.value("H1").unwrap(), Value::Number(4.0));
    assert_eq!(wb.value("D5").unwrap(), Value::Number(2.0));
    assert_eq!(wb.value("D2").unwrap(), Value::Empty);
    assert_eq!(wb.value("D4").unwrap(), Value::Empty);
}

/// probes/t3003-below-last-row: a formula typed just below the table grows it by a row in one
/// edit, and rolling that edit back puts the table back to four rows (rule 7e, T-3273).
#[test]
fn typing_below_the_table_grows_it_by_a_row_and_rollback_restores_it() {
    let mut wb = table_book();
    wb.set("H2", "=ROWS(T[#All])").unwrap();
    let before = dump(&wb);
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("A5", "=B4*2")?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);
    assert_eq!(wb.value("H2").unwrap(), Value::Number(4.0));

    wb.set("A5", "=B4*2").unwrap();
    assert_eq!(wb.value("H2").unwrap(), Value::Number(5.0));
    assert_eq!(formula(&wb, "A5").as_deref(), Some("=B4*2"));
    assert_eq!(wb.value("A5").unwrap(), Value::Number(80.0));
}

/// A formula typed just below an empty column grows the table but fills nothing, so rolling the
/// edit back leaves every cell as it was (unprobed, pinned, T-3273).
#[test]
fn a_formula_typed_below_an_empty_column_fills_nothing_and_rolls_back() {
    let mut wb = table_book();
    wb.set("H2", "=ROWS(T[#All])").unwrap();
    let before = dump(&wb);
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("C5", "=1")?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);

    wb.set("C5", "=1").unwrap();
    assert_eq!(wb.value("H2").unwrap(), Value::Number(5.0));
    assert_eq!(wb.value("C2").unwrap(), Value::Empty);
    assert_eq!(formula(&wb, "C4"), None);
}
