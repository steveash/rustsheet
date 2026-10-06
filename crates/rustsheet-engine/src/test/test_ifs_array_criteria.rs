#![allow(clippy::unwrap_used)]

//! Array and range criteria lift every *IF(S) function to an array of results
//! (probes/ifs-array-criteria, excel-graph; docs/excel-semantics/criteria.md).

use crate::test::util::new_empty_model;

// A1:A6 = a,b,a,c,b,a   B1:B6 = 1,2,3,1,2,3   E1 = #N/A, E3 = "q", F1:F3 = 1,2,3
fn model() -> crate::model::Model {
    let mut model = new_empty_model();
    for (row, (a, b)) in ["a", "b", "a", "c", "b", "a"]
        .iter()
        .zip([1, 2, 3, 1, 2, 3])
        .enumerate()
    {
        model._set(&format!("A{}", row + 1), a);
        model._set(&format!("B{}", row + 1), &b.to_string());
    }
    model._set("D1", "a");
    model._set("D2", "c");
    model._set("E1", "=NA()");
    model._set("E3", "q");
    for row in 1..=3 {
        model._set(&format!("F{row}"), &row.to_string());
    }
    model
}

fn texts(model: &crate::model::Model, cells: &[&str]) -> Vec<String> {
    cells.iter().map(|c| model._get_text(c)).collect()
}

#[test]
fn countif_array_and_range_criteria_keep_their_shape() {
    let mut model = model();
    model._set("H1", r#"=COUNTIF(A1:A6,{"a","b"})"#);
    model._set("H3", "=COUNTIF(A1:A6,D1:D2)");
    model._set("H6", r#"=COUNTIF(A1:A6,{"a","b";"c","z"})"#);
    model._set("H9", r#"=COUNTIFS(A1:A6,{"a"})"#);
    model.evaluate();
    assert_eq!(texts(&model, &["H1", "I1", "J1", "H2"]), ["3", "2", "", ""]);
    assert_eq!(texts(&model, &["H3", "H4", "I3"]), ["3", "1", ""]);
    assert_eq!(
        texts(&model, &["H6", "I6", "H7", "I7"]),
        ["3", "2", "1", "0"]
    );
    assert_eq!(model._get_text("H9"), "3");
}

#[test]
fn countifs_criteria_broadcast_like_operators() {
    let mut model = model();
    // A row and a column give a grid: 3 rows x 2 columns.
    model._set("H1", r#"=COUNTIFS(A1:A6,{"a","b"},B1:B6,{1;2;3})"#);
    // Two rows of different length: the shorter one is #N/A past its end, which counts 0.
    model._set("H5", r#"=COUNTIFS(A1:A6,{"a","b"},B1:B6,{1,2,3})"#);
    // ... and an #N/A criterion matches #N/A cells (E1, with F1 = 1).
    model._set("H7", r#"=COUNTIFS(E1:E3,{"q","q"},F1:F3,{1,1,1})"#);
    model._set("H8", r#"=COUNTIFS(E1:E3,{"q","q"},F1:F3,{2,2,2})"#);
    model.evaluate();
    assert_eq!(
        texts(&model, &["H1", "I1", "H2", "I2", "H3", "I3", "H4"]),
        ["1", "0", "0", "2", "2", "0", ""]
    );
    assert_eq!(texts(&model, &["H5", "I5", "J5"]), ["1", "2", "0"]);
    assert_eq!(texts(&model, &["J7", "J8"]), ["1", "0"]);
}

#[test]
fn sumifs_family_lifts_array_criteria() {
    let mut model = model();
    model._set("H1", r#"=SUMIFS(B1:B6,A1:A6,{"a","b"})"#);
    model._set("H2", r#"=SUMIFS(B1:B6,A1:A6,{"a";"b"},B1:B6,{">1",">2"})"#);
    model._set("H4", r#"=AVERAGEIFS(B1:B6,A1:A6,{"a","c"})"#);
    model._set("H5", r#"=MAXIFS(B1:B6,A1:A6,{"a","b"})"#);
    model._set("H6", r#"=MINIFS(B1:B6,A1:A6,{"a","b"})"#);
    model._set("H7", r#"=AVERAGEIF(A1:A6,{"a","z"},B1:B6)"#);
    model._set("H8", r#"=SUMIF(A1:A6,{"a","b"},B1:B6)"#);
    model.evaluate();
    assert_eq!(texts(&model, &["H1", "I1"]), ["7", "4"]);
    assert_eq!(
        texts(&model, &["H2", "I2", "H3", "I3"]),
        ["6", "6", "4", "0"]
    );
    assert_eq!(texts(&model, &["H4", "I4"]), ["2.333333333", "1"]);
    assert_eq!(texts(&model, &["H5", "I5"]), ["3", "2"]);
    assert_eq!(texts(&model, &["H6", "I6"]), ["1", "2"]);
    assert_eq!(texts(&model, &["H7", "I7"]), ["2.333333333", "#DIV/0!"]);
    assert_eq!(texts(&model, &["H8", "I8"]), ["7", "4"]);
}

#[test]
fn scalar_criteria_still_give_a_scalar() {
    let mut model = model();
    model._set("H1", r#"=COUNTIFS(A1:A6,"a",B1:B6,">1")"#);
    model._set("H2", r#"=SUMIFS(B1:B6,A1:A6,"b")"#);
    model.evaluate();
    assert_eq!(texts(&model, &["H1", "I1", "H2"]), ["2", "", "4"]);
}
