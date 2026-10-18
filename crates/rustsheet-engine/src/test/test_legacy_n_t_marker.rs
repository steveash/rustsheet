#![allow(clippy::unwrap_used, clippy::panic)]

//! The legacy N/T marker (SPEC-0012, T-1851): the internal kinds `NLegacy` and `TLegacy`, their
//! R1C1 storage spelling and their evaluation. Nothing places a marker yet (T-1713), so the
//! marked formulas here are written by hand as stored (R1C1) text.

use std::collections::HashMap;

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::new_parser_english as new_parser;
use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
use crate::expressions::parser::stringify::{
    to_english_string, to_excel_array_formula_string, to_excel_string, to_localized_string,
    to_rc_format,
};
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceRC;
use crate::functions::Function;
use crate::language::get_default_language;
use crate::locale::get_default_locale;
use crate::test::util::new_empty_model;
use crate::types::Cell;
use crate::Model;

fn context() -> CellReferenceRC {
    CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    }
}

/// `formula` (no `=`) parsed in `mode`.
fn parse(formula: &str, mode: LexerMode) -> Node {
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    parser.set_lexer_mode(mode);
    parser.parse(formula, &context())
}

/// The kind of the call at the top of `node`.
fn top_kind(node: &Node) -> Option<Function> {
    match node {
        Node::FunctionKind { kind, .. } => Some(kind.clone()),
        _ => None,
    }
}

/// A model whose A1 holds the stored (R1C1) formula `stored`, as a reload reads it, with `cells`
/// set first.
fn stored_in_a1(stored: &str, cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    for (cell, value) in cells {
        model._set(cell, value);
    }
    model._set("A1", "=N(0)");
    let worksheet = model.workbook.worksheet_mut(0).unwrap();
    worksheet.shared_formulas = vec![stored.to_string()];
    model.reset_parsed_structures();
    model.evaluate();
    model
}

#[test]
fn the_spelling_is_the_marker_in_r1c1_and_plain_n_or_t_in_a1() {
    for (spelling, legacy, plain) in [
        ("_RS_LEGACY_N", Function::NLegacy, Function::N),
        ("_RS_LEGACY_T", Function::TLegacy, Function::T),
    ] {
        let formula = format!("{spelling}(IF(1,{{2;3}}))");
        assert_eq!(top_kind(&parse(&formula, LexerMode::R1C1)), Some(legacy));
        assert_eq!(top_kind(&parse(&formula, LexerMode::A1)), Some(plain));
    }
}

#[test]
fn only_the_storage_text_prints_the_spelling() {
    let (locale, language) = (get_default_locale(), get_default_language());
    for (stored, shown) in [
        ("_RS_LEGACY_N(IF(1,{2;3}))", "N(IF(1,{2;3}))"),
        (
            "_RS_LEGACY_T(IF(1,{\"a\";\"b\"}))",
            "T(IF(1,{\"a\";\"b\"}))",
        ),
    ] {
        let node = parse(stored, LexerMode::R1C1);
        assert_eq!(to_rc_format(&node), stored);
        assert_eq!(
            to_localized_string(&node, &context(), locale, language),
            shown
        );
        assert_eq!(to_english_string(&node, &context()), shown);
        assert_eq!(to_excel_string(&node, &context()), shown);
        assert_eq!(to_excel_array_formula_string(&node, &context()), shown);
    }
    let mut model = stored_in_a1("_RS_LEGACY_N(IF(1,{2;3}))", &[]);
    model._set("B1", "=FORMULATEXT(A1)");
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=N(IF(1,{2;3}))");
    assert_eq!(model._get_text("B1"), "=N(IF(1,{2;3}))");
}

// SPEC-0012 rule 3: an array argument is `#VALUE!` whatever its shape, a range is its top-left
// cell and a scalar is as it was (probes/legacy-n-t-reference-args, legacy-array-result).
#[test]
fn a_marked_n_or_t_reads_an_array_as_value_error() {
    let cells = [("B1", "7"), ("B2", "8"), ("C1", "x"), ("C2", "y")];
    for (stored, value) in [
        ("_RS_LEGACY_N(IF(1,{2;3}))", "#VALUE!"),
        ("_RS_LEGACY_N(IF(1,{2,3}))", "#VALUE!"),
        ("_RS_LEGACY_N(IF(1,{2}))", "#VALUE!"),
        ("_RS_LEGACY_N(IF(1,{\"x\";3}))", "#VALUE!"),
        ("_RS_LEGACY_N(IF(1,R1C2:R2C2))", "7"),
        ("_RS_LEGACY_N(IF(1,2))", "2"),
        ("_RS_LEGACY_T(IF(1,{\"a\";\"b\"}))", "#VALUE!"),
        ("_RS_LEGACY_T(IF(1,{\"a\"}))", "#VALUE!"),
        ("_RS_LEGACY_T(IF(1,R1C3:R2C3))", "x"),
        ("_RS_LEGACY_T(\"a\")", "a"),
    ] {
        let model = stored_in_a1(stored, &cells);
        assert_eq!(model._get_text("A1"), value, "{stored}");
        // A scalar cell: no spill below it.
        assert_eq!(model._get_text("A2"), "", "{stored}");
    }
    // Unmarked, the formula keeps today's reading: N lifts.
    let mut model = new_empty_model();
    model._set("A1", "=N(IF(1,{2;3}))");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "3");
}

// A marked formula's static result is a scalar, so its cell is a plain formula, never a dynamic
// anchor (SPEC-0012 "The marker").
#[test]
fn a_marked_formula_is_a_scalar_cell() {
    let node = parse("_RS_LEGACY_N(IF(1,{2;3}))", LexerMode::R1C1);
    assert!(matches!(
        run_static_analysis_on_node(&node),
        StaticResult::Scalar
    ));
    let mut model = new_empty_model();
    model.set_cell_with_formula_node(0, 1, 1, node, 0).unwrap();
    model.evaluate();
    let cell = model.workbook.worksheet(0).unwrap().cell(1, 1);
    assert!(matches!(cell, Some(Cell::CellFormula { .. })), "{cell:?}");
    assert_eq!(model._get_text("A1"), "#VALUE!");
    let stored = &model.workbook.worksheet(0).unwrap().shared_formulas;
    assert_eq!(stored, &["_RS_LEGACY_N(IF(1,{2;3}))".to_string()]);
}

// A typed call of the spelling is plain N, also over a LAMBDA name of that spelling, which it
// shadows (a DEVIATION, references.md "Legacy N and T marker spelling").
#[test]
fn a_typed_call_of_the_spelling_is_plain_n() {
    let mut model = new_empty_model();
    model
        .define_name("_RS_LEGACY_N", None, "=LAMBDA(x,x*10)")
        .unwrap();
    model._set("A1", "=_RS_LEGACY_N(IF(1,{2;3}))");
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=N(IF(1,{2;3}))");
    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model._get_text("A2"), "3");
    model.rename_sheet("Sheet1", "Other").unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A2"), "3");
}

// Since T-1805 a formula that does not parse is stored marked as a parse error and never
// re-parsed, so the spelling in it does not come back as a marker after a reload.
#[test]
fn an_unparsed_formula_with_the_spelling_stays_a_parse_error() {
    let mut model = new_empty_model();
    model._set("A1", "=_RS_LEGACY_N(IF(1,{2;3}))+R[1]C");
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    assert_eq!(
        reloaded._get_formula("A1"),
        "=_RS_LEGACY_N(IF(1,{2;3}))+R[1]C"
    );
    assert_eq!(reloaded._get_text("A1"), "#ERROR!");
}

// The matcher is exact: a case or `_xlfn.` variant is an unknown name in both modes, and never
// reparses to the internal kind.
#[test]
fn spelling_variants_are_no_marker() {
    for variant in [
        "_rs_legacy_n",
        "_Rs_Legacy_N",
        "_xlfn._RS_LEGACY_N",
        "_RS_LEGACY_N_",
    ] {
        let formula = format!("{variant}(IF(1,{{2;3}}))");
        for mode in [LexerMode::A1, LexerMode::R1C1] {
            let node = parse(&formula, mode.clone());
            assert!(
                matches!(node, Node::NamedFunctionKind { .. }),
                "{formula}: {node:?}"
            );
            let stored = to_rc_format(&node);
            assert!(
                top_kind(&parse(&stored, LexerMode::R1C1)).is_none(),
                "{stored}"
            );
        }
        let mut model = new_empty_model();
        model._set("A1", &format!("={formula}"));
        model.rename_sheet("Sheet1", "Other").unwrap();
        model.evaluate();
        assert_eq!(model._get_text("A1"), "#NAME?", "{formula}");
    }
}
