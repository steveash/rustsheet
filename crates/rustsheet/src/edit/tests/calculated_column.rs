//! A formula typed into a table column fills it as a calculated column (T-2958;
//! docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table column").

use super::*;

/// `T` over A1:C4: `Col` 1, 2, 4, `Other` 10, 20, 40 and an empty `Calc` column, as in the
/// probes/t2957-* cases.
fn table_book() -> Workbook {
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
    let range = wb.range_ref("A1:C4").unwrap();
    wb.edit(|tx| tx.add_table(range, "T", false)).unwrap();
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
