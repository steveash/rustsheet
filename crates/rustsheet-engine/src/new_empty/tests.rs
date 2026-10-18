#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

use crate::cf_types::{CfRuleInput, ValueOperator};
use crate::test::util::new_empty_model;
use crate::types::{Color, Dxf, Fill};

fn red_fill() -> Dxf {
    Dxf {
        font: None,
        fill: Some(Fill {
            color: Color::Rgb("#FF0000".to_string()),
        }),
        border: None,
        num_fmt: None,
        alignment: None,
    }
}

#[test]
fn test_duplicate_sheet_naming() {
    let mut model = new_empty_model();

    // Sheet1 -> Sheet1 (1)
    let (name1, index1) = model.duplicate_sheet(0).unwrap();
    assert_eq!(name1, "Sheet1 (1)");
    assert_eq!(index1, 1); // inserted right after the source

    // Duplicating Sheet1 again -> Sheet1 (2)
    let (name2, _) = model.duplicate_sheet(0).unwrap();
    assert_eq!(name2, "Sheet1 (2)");

    // Duplicating the copy -> Sheet1 (1) (1)
    let source = model.get_sheet_index_by_name("Sheet1 (1)").unwrap();
    let (name3, _) = model.duplicate_sheet(source).unwrap();
    assert_eq!(name3, "Sheet1 (1) (1)");

    // Out of range
    assert!(model.duplicate_sheet(100).is_err());
}

#[test]
fn test_duplicate_sheet_name_respects_length_limit() {
    let mut model = new_empty_model();
    // A 31-character name (the maximum allowed).
    let long_name = "AAAAAAAAAABBBBBBBBBBCCCCCCCCCCD";
    assert_eq!(long_name.chars().count(), 31);
    model.rename_sheet_by_index(0, long_name).unwrap();

    // Naively "{name} (1)" would be 35 chars and thus invalid. The base
    // name must be truncated so the result stays within the limit.
    let (new_name, new_index) = model.duplicate_sheet(0).unwrap();
    assert!(is_valid_sheet_name(&new_name));
    assert!(new_name.chars().count() <= 31);
    assert!(new_name.ends_with(" (1)"));
    assert_eq!(
        model.workbook.worksheet(new_index).unwrap().get_name(),
        new_name
    );

    // A second copy still produces a distinct, valid name.
    let (new_name2, _) = model.duplicate_sheet(0).unwrap();
    assert!(is_valid_sheet_name(&new_name2));
    assert!(new_name2.chars().count() <= 31);
    assert_ne!(new_name2, new_name);
}

#[test]
fn test_duplicate_sheet_formulas() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "10".to_string()).unwrap();
    model.set_user_input(0, 1, 2, "=A1*2".to_string()).unwrap();
    model.evaluate();

    let (_, new_index) = model.duplicate_sheet(0).unwrap();

    // The implicit self-reference is preserved and points to the copy.
    assert_eq!(
        model.get_cell_formula(new_index, 1, 2).unwrap(),
        Some("=A1*2".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(new_index, 1, 2).unwrap(),
        "20"
    );

    // The copy is independent: changing the copy doesn't touch the original.
    model
        .set_user_input(new_index, 1, 1, "100".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(
        model.get_formatted_cell_value(new_index, 1, 2).unwrap(),
        "200"
    );
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "20");
}

#[test]
fn test_duplicate_sheet_formulas_to_other_sheets() {
    let mut model = new_empty_model();
    model.add_sheet("Other").unwrap();
    let other = model.get_sheet_index_by_name("Other").unwrap();
    model.set_user_input(other, 1, 1, "7".to_string()).unwrap();

    // A reference to another sheet, and an explicit self-reference.
    model
        .set_user_input(0, 1, 1, "=Other!A1".to_string())
        .unwrap();
    model.set_user_input(0, 2, 1, "42".to_string()).unwrap();
    model
        .set_user_input(0, 1, 2, "=Sheet1!A2".to_string())
        .unwrap();
    model.evaluate();

    let (_, new_index) = model.duplicate_sheet(0).unwrap();

    // The cross-sheet reference is unchanged.
    assert_eq!(
        model.get_cell_formula(new_index, 1, 1).unwrap(),
        Some("=Other!A1".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(new_index, 1, 1).unwrap(),
        "7"
    );

    // The explicit self-reference is retargeted to the copy.
    assert_eq!(
        model.get_cell_formula(new_index, 1, 2).unwrap(),
        Some("='Sheet1 (1)'!A2".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(new_index, 1, 2).unwrap(),
        "42"
    );
}

#[test]
fn test_duplicate_sheet_local_defined_names() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "5".to_string()).unwrap();
    model
        .new_defined_name("local_name", Some(0), "Sheet1!$A$1")
        .unwrap();
    model.evaluate();

    let (_, new_index) = model.duplicate_sheet(0).unwrap();

    // The local name is duplicated, scoped to the copy, retargeted to it.
    let names = model.get_defined_name_list();
    let copy = names
        .iter()
        .find(|(name, scope, _)| name == "local_name" && *scope == Some(new_index))
        .expect("local name should be duplicated on the copy");
    assert_eq!(copy.2, "'Sheet1 (1)'!$A$1");

    // The original local name is left untouched.
    assert!(names
        .iter()
        .any(|(name, scope, formula)| name == "local_name"
            && *scope == Some(0)
            && formula == "Sheet1!$A$1"));
}

#[test]
fn test_duplicate_sheet_global_names_made_local() {
    let mut model = new_empty_model();
    model.add_sheet("Other").unwrap();
    model.set_user_input(0, 1, 1, "5".to_string()).unwrap();
    // A global name referencing the source sheet, and one that doesn't.
    model
        .new_defined_name("from_source", None, "Sheet1!$A$1")
        .unwrap();
    model
        .new_defined_name("from_other", None, "Other!$A$1")
        .unwrap();
    model.evaluate();

    let (_, new_index) = model.duplicate_sheet(0).unwrap();
    let names = model.get_defined_name_list();

    // The original global names are preserved.
    assert!(names
        .iter()
        .any(|(name, scope, _)| name == "from_source" && scope.is_none()));
    assert!(names
        .iter()
        .any(|(name, scope, _)| name == "from_other" && scope.is_none()));

    // The global name referencing the source is duplicated as a sheet-local
    // name on the copy, retargeted to it.
    let copy = names
        .iter()
        .find(|(name, scope, _)| name == "from_source" && *scope == Some(new_index))
        .expect("global name referencing source should become local on copy");
    assert_eq!(copy.2, "'Sheet1 (1)'!$A$1");

    // The global name that does not reference the source is NOT duplicated.
    assert!(!names
        .iter()
        .any(|(name, scope, _)| name == "from_other" && *scope == Some(new_index)));
}

#[test]
fn test_duplicate_sheet_prefers_local_over_global_name() {
    // When a name exists both globally and as sheet-local to the source,
    // resolution prefers the sheet-local one, so the copy must inherit the
    // sheet-local definition — regardless of which is stored first.
    let mut model = new_empty_model();
    // The global is created first (so it comes earlier in `defined_names`).
    model
        .new_defined_name("shared", None, "Sheet1!$A$1")
        .unwrap();
    model
        .new_defined_name("shared", Some(0), "Sheet1!$B$2")
        .unwrap();
    model.evaluate();

    let (_, new_index) = model.duplicate_sheet(0).unwrap();
    let names = model.get_defined_name_list();

    // Exactly one "shared" name is local to the copy and it comes from the
    // sheet-local definition ($B$2), not the global one ($A$1).
    let local_copies: Vec<_> = names
        .iter()
        .filter(|(name, scope, _)| name == "shared" && *scope == Some(new_index))
        .collect();
    assert_eq!(local_copies.len(), 1);
    assert_eq!(local_copies[0].2, "'Sheet1 (1)'!$B$2");
}

#[test]
fn test_duplicate_sheet_conditional_formatting() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "10".to_string()).unwrap();
    model
        .add_conditional_formatting(
            0,
            "A1:A10",
            CfRuleInput::CellIs {
                operator: ValueOperator::GreaterThan,
                formula: "5".to_string(),
                formula2: None,
                format: red_fill(),
                stop_if_true: false,
            },
        )
        .unwrap();
    model.evaluate();

    let (_, new_index) = model.duplicate_sheet(0).unwrap();

    let source_rules = model.get_conditional_formatting_list(0).unwrap();
    let copy_rules = model.get_conditional_formatting_list(new_index).unwrap();
    assert_eq!(copy_rules.len(), 1);
    assert_eq!(copy_rules[0].range, "A1:A10");
    assert_eq!(copy_rules[0].cf_rule, source_rules[0].cf_rule);
}

#[test]
fn test_duplicate_sheet_color() {
    let mut model = new_empty_model();
    let color = Color::Rgb("#123456".to_string());
    model.set_sheet_color(0, &color).unwrap();

    let (_, new_index) = model.duplicate_sheet(0).unwrap();
    assert_eq!(model.workbook.worksheet(new_index).unwrap().color, color);
}

#[test]
fn test_is_valid_sheet_name() {
    assert!(is_valid_sheet_name("Sheet1"));
    assert!(is_valid_sheet_name("Zażółć gęślą jaźń"));

    assert!(is_valid_sheet_name(" "));
    assert!(!is_valid_sheet_name(""));

    assert!(is_valid_sheet_name("🙈"));

    assert!(is_valid_sheet_name("AAAAAAAAAABBBBBBBBBBCCCCCCCCCCD")); // 31
    assert!(!is_valid_sheet_name("AAAAAAAAAABBBBBBBBBBCCCCCCCCCCDE")); // 32
}
