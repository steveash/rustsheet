#![allow(clippy::unwrap_used)]

//! Conditional formats are evaluated on the first style read after a load, an `evaluate()` or
//! `set_theme`, not by those (SPEC-0021, T-0468).

use crate::cf_types::CfRuleInput;
use crate::test::util::new_empty_model;
use crate::types::Color;
use crate::{Model, UserModel};

/// `=$A1>5` over A1:A10, red.
fn rule() -> CfRuleInput {
    CfRuleInput::Formula {
        formula: "=$A1>5".to_string(),
        format: super::red_fill(),
        stop_if_true: false,
    }
}

fn red(model: &mut Model, row: i32) -> bool {
    model
        .get_extended_style_for_cell(0, row, 1)
        .unwrap()
        .style
        .fill
        .color
        == Color::Rgb("#FF0000".to_string())
}

fn model_with_rule() -> Model {
    let mut model = new_empty_model();
    for row in 1..=10 {
        model.set_user_input(0, row, 1, row.to_string()).unwrap();
    }
    model
        .add_conditional_formatting(0, "A1:A10", rule())
        .unwrap();
    model
}

#[test]
fn evaluating_without_a_style_read_evaluates_no_rule() {
    let mut model = model_with_rule();
    model.evaluate();
    model.evaluate();
    assert_eq!(model.cf_runs, 0);
}

#[test]
fn the_first_read_evaluates_once_and_later_reads_use_the_cache() {
    let mut model = model_with_rule();
    model.evaluate();
    assert!(red(&mut model, 6));
    assert!(!red(&mut model, 5));
    assert!(red(&mut model, 10));
    assert_eq!(model.cf_runs, 1);
    // A new evaluation: the next read sees its values.
    model.set_user_input(0, 5, 1, "50".to_string()).unwrap();
    model.evaluate();
    assert!(red(&mut model, 5));
    assert_eq!(model.cf_runs, 2);
}

#[test]
fn set_theme_makes_the_next_read_evaluate() {
    let mut model = model_with_rule();
    model.evaluate();
    assert!(red(&mut model, 6));
    let theme = model.get_theme();
    model.set_theme(theme);
    assert_eq!(model.cf_runs, 1);
    assert!(red(&mut model, 6));
    assert_eq!(model.cf_runs, 2);
}

fn user_red(model: &mut UserModel, row: i32) -> bool {
    model
        .get_extended_cell_style(0, row, 1)
        .unwrap()
        .style
        .fill
        .color
        == Color::Rgb("#FF0000".to_string())
}

fn user_model() -> UserModel {
    let mut model = UserModel::new_empty("cf", "en", "UTC", "en").unwrap();
    for row in 1..=10 {
        model.set_user_input(0, row, 1, &row.to_string()).unwrap();
    }
    model
}

#[test]
fn user_model_edits_undo_and_redo_are_seen_by_the_next_read() {
    let mut model = user_model();
    assert!(!user_red(&mut model, 6));
    model
        .add_conditional_formatting(0, "A1:A10", rule())
        .unwrap();
    assert!(user_red(&mut model, 6));
    model.undo().unwrap();
    assert!(!user_red(&mut model, 6));
    model.redo().unwrap();
    assert!(user_red(&mut model, 6));
}

/// Under paused evaluation the styles a read built stay until `evaluate()`, as the snapshot
/// taken at `evaluate()` did; a read that comes first after the edit sees it (SPEC-0021).
#[test]
fn paused_edits_after_a_read_wait_for_evaluate() {
    let mut model = user_model();
    assert!(!user_red(&mut model, 6));
    model.pause_evaluation();
    model
        .add_conditional_formatting(0, "A1:A10", rule())
        .unwrap();
    assert!(!user_red(&mut model, 6));
    model.resume_evaluation();
    model.evaluate();
    assert!(user_red(&mut model, 6));

    let mut fresh = user_model();
    fresh.pause_evaluation();
    fresh
        .add_conditional_formatting(0, "A1:A10", rule())
        .unwrap();
    assert!(user_red(&mut fresh, 6));
}

/// A formula written after `evaluate()` where that pass evaluated nothing is computed by the
/// style read whose rule reads it, and keeps that value (SPEC-0021).
#[test]
fn a_read_computes_a_formula_written_since_evaluate() {
    let mut model = model_with_rule();
    model.evaluate();
    model.set_user_input(0, 3, 1, "=7*1".to_string()).unwrap();
    assert!(red(&mut model, 3));
    assert_eq!(model._get_text("A3"), "7");
}
