#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::Model;

// Unparsable formulas, with spaces and with R1C1-shaped references (an error in A1, a reference in
// R1C1, where formulas are stored).
const UNPARSED: [&str; 6] = [
    "=Ab1 (1)",
    "=SUM(1 2)",
    "=R[1]C[1]+1",
    "=R[1]C[1](1)",
    "=SUM(R[1]C[1])",
    "=(R[1]C[1])(1)",
];

fn assert_unparsed(model: &Model, cell: &str, formula: &str) {
    assert_eq!(model._get_formula(cell), formula, "{formula} in {cell}");
    assert_eq!(model._get_text(cell), "#ERROR!", "{formula} in {cell}");
}

// A stored parse error reads back exactly after every structural edit (T-1805).
#[test]
fn unparsed_formula_keeps_its_text_through_structural_edits() {
    for formula in UNPARSED {
        for (edit, cell) in [("rename", "A8"), ("rows", "A9"), ("columns", "B8")] {
            let mut model = new_empty_model();
            model._set("A8", formula);
            model.evaluate();
            assert_unparsed(&model, "A8", formula);
            match edit {
                "rename" => model.rename_sheet("Sheet1", "Other").unwrap(),
                "rows" => model.insert_rows(0, 1, 1).unwrap(),
                _ => model.insert_columns(0, 1, 1).unwrap(),
            }
            model.evaluate();
            assert_unparsed(&model, cell, formula);
        }
    }
}

// The bytes keep the stored form, so a reload does not re-read the A1 text in R1C1.
#[test]
fn unparsed_formula_keeps_its_text_through_bytes() {
    let mut model = new_empty_model();
    for (row, formula) in UNPARSED.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for (row, formula) in UNPARSED.iter().enumerate() {
        assert_unparsed(&reloaded, &format!("A{}", row + 1), formula);
    }
}

// The same R1C1 text typed where it parses (`=B10+1` in A9 is `R[1]C[1]+1`) is a different
// stored formula from the parse error, so neither cell takes the other's formula.
#[test]
fn unparsed_formula_is_not_shared_with_its_r1c1_twin() {
    let mut model = new_empty_model();
    model._set("A8", "=R[1]C[1]+1");
    model._set("A9", "=B10+1");
    model.evaluate();
    assert_unparsed(&model, "A8", "=R[1]C[1]+1");
    assert_eq!(model._get_formula("A9"), "=B10+1");
    assert_eq!(model._get_text("A9"), "1");
}

// A typed formula that starts with the stored form's mark is itself a parse error, and keeps
// its text: the mark is only read back in R1C1.
#[test]
fn typed_mark_is_a_parse_error() {
    let mut model = new_empty_model();
    model._set("A1", "=\u{1}0:0:B1");
    model.evaluate();
    assert_unparsed(&model, "A1", "=\u{1}0:0:B1");
    model.rename_sheet("Sheet1", "Other").unwrap();
    model.evaluate();
    assert_unparsed(&model, "A1", "=\u{1}0:0:B1");
}
