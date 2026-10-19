#![allow(clippy::unwrap_used, clippy::panic)]

//! The legacy N/T marker (SPEC-0012): the internal kinds `NLegacy` and `TLegacy`, their R1C1
//! storage spelling and their evaluation (T-1851, from stored text written by hand), and where
//! the legacy rewrite places them (T-1713).

use std::collections::HashMap;

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::new_parser_english as new_parser;
use crate::expressions::parser::static_analysis::{
    add_implicit_intersection, run_static_analysis_on_node, StaticResult,
};
use crate::expressions::parser::stringify::{
    to_english_string, to_excel_array_formula_string, to_excel_string, to_localized_string,
    to_rc_format,
};
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceRC;
use crate::expressions::utils::parse_reference_a1;
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

/// Whether stored (R1C1) text holds the marker.
fn is_marked(stored: &str) -> bool {
    stored.contains("NLG1(") || stored.contains("TLG1(")
}

// In A1 text the spelling is a cell, so a call of it is a call of that cell, `#REF!` as in Excel
// (probes/legacy-marker-spelling-name-nlg1 A5, T-1936), stored by its reference.
#[test]
fn the_spelling_is_the_marker_in_r1c1_and_a_cell_in_a1() {
    for (spelling, legacy) in [("NLG1", Function::NLegacy), ("TLG1", Function::TLegacy)] {
        let formula = format!("{spelling}(IF(1,{{2;3}}))");
        assert_eq!(top_kind(&parse(&formula, LexerMode::R1C1)), Some(legacy));
        let typed = parse(&formula, LexerMode::A1);
        assert!(matches!(typed, Node::LambdaCallKind { .. }), "{typed:?}");
        assert!(!is_marked(&to_rc_format(&typed)), "{formula}");
        let mut model = new_empty_model();
        model._set("A1", &format!("={formula}"));
        model.evaluate();
        assert_eq!(model._get_text("A1"), "#REF!", "{formula}");
    }
}

#[test]
fn only_the_storage_text_prints_the_spelling() {
    let (locale, language) = (get_default_locale(), get_default_language());
    for (stored, shown) in [
        ("NLG1(IF(1,{2;3}))", "N(IF(1,{2;3}))"),
        ("TLG1(IF(1,{\"a\";\"b\"}))", "T(IF(1,{\"a\";\"b\"}))"),
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
    let mut model = stored_in_a1("NLG1(IF(1,{2;3}))", &[]);
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
        ("NLG1(IF(1,{2;3}))", "#VALUE!"),
        ("NLG1(IF(1,{2,3}))", "#VALUE!"),
        ("NLG1(IF(1,{2}))", "#VALUE!"),
        ("NLG1(IF(1,{\"x\";3}))", "#VALUE!"),
        ("NLG1(IF(1,R1C2:R2C2))", "7"),
        ("NLG1(IF(1,2))", "2"),
        ("TLG1(IF(1,{\"a\";\"b\"}))", "#VALUE!"),
        ("TLG1(IF(1,{\"a\"}))", "#VALUE!"),
        ("TLG1(IF(1,R1C3:R2C3))", "x"),
        ("TLG1(\"a\")", "a"),
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
    let node = parse("NLG1(IF(1,{2;3}))", LexerMode::R1C1);
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
    assert_eq!(stored, &["NLG1(IF(1,{2;3}))".to_string()]);
}

// A LAMBDA name spelled like the marker of T-1851 is called by its name, as in Excel (17,
// probes/legacy-marker-spelling-name-plain A1:A2, T-1912), also after a reload and a sheet rename,
// which reparse the stored text (T-1936).
#[test]
fn a_typed_call_of_the_old_spelling_calls_the_name() {
    let mut model = new_empty_model();
    for name in ["_RS_LEGACY_N", "_RS_LEGACY_T"] {
        model.define_name(name, None, "=LAMBDA(x,x*10+7)").unwrap();
    }
    model._set("A1", "=_RS_LEGACY_N(1)");
    model._set("B1", "=_RS_LEGACY_T(1)");
    model._set("C1", "=N(1)");
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for model in [&mut model, &mut reloaded] {
        assert_eq!(model._get_formula("A1"), "=_RS_LEGACY_N(1)");
        assert_eq!(model._get_text("A1"), "17");
        assert_eq!(model._get_text("B1"), "17");
        assert_eq!(model._get_text("C1"), "1");
        model.rename_sheet("Sheet1", "Other").unwrap();
        model.evaluate();
        assert_eq!(model._get_text("A1"), "17");
        assert_eq!(model._get_text("B1"), "17");
    }
}

// A model file saved between T-1851 and T-1936 stores the marker as `_RS_LEGACY_N(` and
// `_RS_LEGACY_T(`; it still reloads as the marker while the workbook defines no name of that
// spelling (a file of that time stored no call of such a name: its A1 text read as plain N).
#[test]
fn the_old_stored_spelling_still_reloads_as_the_marker() {
    let cells = [("B1", "7"), ("B2", "8")];
    for (stored, value, shown) in [
        ("_RS_LEGACY_N(IF(1,{2;3}))", "#VALUE!", "=N(IF(1,{2;3}))"),
        ("_RS_LEGACY_N(IF(1,R1C2:R2C2))", "7", "=N(IF(1,$B$1:$B$2))"),
        (
            "_RS_LEGACY_T(IF(1,{\"a\";\"b\"}))",
            "#VALUE!",
            "=T(IF(1,{\"a\";\"b\"}))",
        ),
    ] {
        let model = stored_in_a1(stored, &cells);
        assert_eq!(model._get_text("A1"), value, "{stored}");
        assert_eq!(model._get_text("A2"), "", "{stored}");
        assert_eq!(model._get_formula("A1"), shown, "{stored}");
        // Stored again, it takes the new spelling.
        let restored = to_rc_format(&parse(stored, LexerMode::R1C1));
        assert!(is_marked(&restored), "{restored}");
    }
}

// With a name of the old spelling defined (any scope, any case), its stored text is the name call.
#[test]
fn the_old_stored_spelling_is_a_name_call_when_the_name_exists() {
    for (defined, scope) in [("_RS_LEGACY_N", None), ("_rs_legacy_n", Some(0))] {
        let names = vec![(defined.to_string(), scope, "LAMBDA(x,x)".to_string())];
        let mut parser = new_parser(vec!["Sheet1".to_string()], names, HashMap::new());
        parser.set_lexer_mode(LexerMode::R1C1);
        let node = parser.parse("_RS_LEGACY_N(1)", &context());
        assert!(matches!(node, Node::NamedFunctionKind { .. }), "{node:?}");
        // The new spelling stays the marker whatever names exist.
        let node = parser.parse("NLG1(1)", &context());
        assert_eq!(top_kind(&node), Some(Function::NLegacy));
    }
}

// Since T-1805 a formula that does not parse is stored marked as a parse error and never
// re-parsed, so the spelling in it does not come back as a marker after a reload.
#[test]
fn an_unparsed_formula_with_the_spelling_stays_a_parse_error() {
    let mut model = new_empty_model();
    model._set("A1", "=NLG1(IF(1,{2;3}))+R[1]C");
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    assert_eq!(reloaded._get_formula("A1"), "=NLG1(IF(1,{2;3}))+R[1]C");
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
        "_xlfn.NLG1",
        "NLG1_",
        "_NLG1",
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
    // A case variant of the new spelling is a name in R1C1 text (in A1 text it is the cell).
    for variant in ["nlg1", "Tlg1"] {
        let node = parse(&format!("{variant}(1)"), LexerMode::R1C1);
        assert!(matches!(node, Node::NamedFunctionKind { .. }), "{node:?}");
    }
}

/// `formula` (no `=`) read as a legacy formula in Sheet1!A1, as the xlsx import reads a plain
/// `<f>`: its stored (R1C1) text and its display text.
fn legacy_reading(formula: &str) -> (String, String) {
    let mut node = parse(formula, LexerMode::A1);
    add_implicit_intersection(&mut node, true);
    (to_rc_format(&node), to_english_string(&node, &context()))
}

// SPEC-0012 rule 2 and "Placement": one row per row of the spec's Problem table, the negative
// ones included. `marked` is whether the stored text holds the marker; the display text never
// does, and shows what Excel's FORMULATEXT shows (probes/legacy-n-t-reference-args row 20,
// legacy-array-result row 20, n-array-lift A26).
#[test]
fn the_legacy_rewrite_marks_n_and_t_over_an_array_valued_call() {
    for (formula, marked, shown) in [
        ("N(IF(1,{2;3}))", true, "N(IF(1,{2;3}))"),
        ("T(IF(1,{\"a\";\"b\"}))", true, "T(IF(1,{\"a\";\"b\"}))"),
        ("T(IF(1,{\"a\"}))", true, "T(IF(1,{\"a\"}))"),
        ("N(IF(1,{2,3}))", true, "N(IF(1,{2,3}))"),
        ("N(IF(1,{2,3;4,5}))", true, "N(IF(1,{2,3;4,5}))"),
        ("N(IF(1,{2;3},{4;5}))", true, "N(IF(1,{2;3},{4;5}))"),
        ("N(IF({1;0},2,3))", true, "N(IF({1;0},2,3))"),
        ("N(CHOOSE(1,{2;3}))", true, "N(CHOOSE(1,{2;3}))"),
        (
            "T(CHOOSE(1,{\"a\";\"b\"}))",
            true,
            "T(CHOOSE(1,{\"a\";\"b\"}))",
        ),
        ("N(INDEX({2;3},0,1))", true, "N(INDEX({2;3},0,1))"),
        ("N(IFS(1,{2;3}))", true, "N(IFS(1,{2;3}))"),
        ("ABS(N(IF(1,{2;3})))", true, "ABS(N(IF(1,{2;3})))"),
        ("N(IF(1,{2;3}))+0", true, "N(IF(1,{2;3}))+0"),
        ("N(IF(1,{2;3}))&\"\"", true, "N(IF(1,{2;3}))&\"\""),
        ("T(IF(1,{2;3}))", true, "T(IF(1,{2;3}))"),
        ("N(IF(1,{\"x\";3}))", true, "N(IF(1,{\"x\";3}))"),
        // N20: no one-element `@` under the marker.
        ("N(IF(1,{2}))", true, "N(IF(1,{2}))"),
        // probes/legacy-n-t-marker-shapes: one-element CHOOSE and IFS arms, INDEX of an array
        // constant, an operator or a reference in IF's arms, N under LEN, and CHOOSE's array
        // index, all `#VALUE!` with no `@` (A-F, H-L); `N(INDEX({2},1,1))` is 2 (G).
        ("N(CHOOSE(1,{2}))", true, "N(CHOOSE(1,{2}))"),
        ("T(CHOOSE(1,{\"a\"}))", true, "T(CHOOSE(1,{\"a\"}))"),
        ("N(IFS(1,{2}))", true, "N(IFS(1,{2}))"),
        ("N(INDEX({2,3;4,5},1))", true, "N(INDEX({2,3;4,5},1))"),
        ("N(INDEX({2},1,1))", true, "N(INDEX({2},1,1))"),
        ("N(IF(1,{2;3}+0))", true, "N(IF(1,{2;3}+0))"),
        ("LEN(N(IF(1,{2;3})))", true, "LEN(N(IF(1,{2;3})))"),
        ("N(IF(1,{2;3},Z1))", true, "N(IF(1,{2;3},Z1))"),
        (
            "N(CHOOSE({1;2},{2;3},{4;5}))",
            true,
            "N(CHOOSE({1;2},{2;3},{4;5}))",
        ),
        // A reference result reads the same marked or not (J10, 7).
        ("N(IF(1,Z1:Z2))", true, "N(IF(1,Z1:Z2))"),
        // Not a value position: a vector argument, IF's arms, SUMPRODUCT's array context.
        ("SUM(N(IF(1,{2;3})))", false, "SUM(N(IF(1,{2;3})))"),
        ("IF(1,N(IF(1,{2;3})))", false, "@IF(1,N(IF(1,{2;3})))"),
        (
            "SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))",
            false,
            "SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))",
        ),
        // Not an argument the marker covers: SWITCH spills, IFERROR and an array constant or
        // another array-valued call take the `@`, an explicit `@` or parentheses are kept.
        ("N(SWITCH(1,1,{2;3}))", false, "N(SWITCH(1,1,{2;3}))"),
        ("N(IFERROR({2;3},0))", false, "N(@IFERROR({2;3},0))"),
        ("N({2;3})", false, "N(@{2;3})"),
        ("N(TRANSPOSE({2,3}))", false, "N(@TRANSPOSE({2,3}))"),
        ("N(_xlfn.SINGLE(IF(1,{2;3})))", false, "N(@IF(1,{2;3}))"),
        // CHOOSE and INDEX over a reference are read as references, unmarked and with no `@`
        // (T-1908); a CHOOSE with a literal index mixing references and array constants is marked with no
        // `@` (T-1918), other mixed arms keep it.
        ("N(CHOOSE(1,Z1:Z2))", false, "N(CHOOSE(1,Z1:Z2))"),
        ("N(CHOOSE(1,{2;3},Z1))", true, "N(CHOOSE(1,{2;3},Z1))"),
        ("N(CHOOSE(2,Z1,{2;3}))", true, "N(CHOOSE(2,Z1,{2;3}))"),
        ("N(CHOOSE(1,Z1:Z2,{2;3}))", true, "N(CHOOSE(1,Z1:Z2,{2;3}))"),
        ("N(CHOOSE(TRUE,Z1,{2;3}))", true, "N(CHOOSE(TRUE,Z1,{2;3}))"),
        ("N(INDEX(Z1:Z2,1))", false, "N(INDEX(Z1:Z2,1))"),
        // A one-element arm next to the marker. `@`s survive where the call is not marked: an
        // IFS with a reference arm takes the `@` on the call and keeps the arm's, and so do an IF
        // under SUM or under an operator. Where the call is marked they are dropped, also with a
        // non-literal condition or a multi-cell arm beside them (extrapolated from N20 and
        // probes/legacy-n-t-marker-shapes A-D).
        ("N(IFS(A1,{2},1,Z1))", false, "N(@IFS(A1,@{2},1,Z1))"),
        ("SUM(N(IF(1,{2})))", false, "SUM(N(IF(1,@{2})))"),
        ("N(IF(1,{2})+0)", false, "N(IF(1,@{2})+0)"),
        ("N(CHOOSE(1,{2},Z1))", false, "N(@CHOOSE(1,{2},Z1))"),
        ("N(IF(A1,{2},3))", true, "N(IF(A1,{2},3))"),
        ("N(CHOOSE(1,{2},{3;4}))", true, "N(CHOOSE(1,{2},{3;4}))"),
        // A statically scalar argument reads the same either way.
        ("N(IF(A1,1,2))", false, "N(IF(A1,1,2))"),
        ("N(IF(1,2))", false, "N(IF(1,2))"),
    ] {
        let (stored, display) = legacy_reading(formula);
        assert_eq!(is_marked(&stored), marked, "{formula}: {stored}");
        assert_eq!(display, shown, "{formula}");
    }
    // A dynamic formula is never rewritten, so never marked.
    let mut model = new_empty_model();
    model._set("A1", "=N(IF(A2,1,2))");
    let stored = &model.workbook.worksheet(0).unwrap().shared_formulas;
    assert_eq!(stored, &["N(IF(R[1]C[0],1,2))".to_string()]);
}

// The legacy cells of the spec's Problem table evaluate as in Excel, while their dynamic twins
// keep lifting (probes/legacy-n-t-reference-args, legacy-array-result A, G, I, J).
#[test]
fn a_legacy_cell_reads_n_and_t_over_an_array_as_value_error() {
    for (formula, legacy, dynamic) in [
        ("=N(IF(1,{2;3}))", "#VALUE!", "2"),
        ("=T(IF(1,{\"a\";\"b\"}))", "#VALUE!", "a"),
        ("=N(IF(1,{2}))", "#VALUE!", "2"),
        ("=N(CHOOSE(1,{2;3}))", "#VALUE!", "2"),
        ("=ABS(N(IF(1,{2;3})))", "#VALUE!", "2"),
        ("=N(IF(1,{2;3}))&\"\"", "#VALUE!", "2"),
        ("=SUM(N(IF(1,{2;3})))", "5", "5"),
        ("=IF(1,N(IF(1,{2;3})))", "2", "2"),
        ("=N(IF(1,Z1:Z2))", "7", "7"),
        ("=N(IF(1,2))", "2", "2"),
        ("=N(CHOOSE(1,{2}))", "#VALUE!", "2"),
        ("=N(INDEX({2},1,1))", "2", "2"),
        ("=N(CHOOSE({1;2},{2;3},{4;5}))", "#VALUE!", "2"),
    ] {
        let mut model = new_empty_model();
        model._set("Z1", "7");
        model._set("Z2", "8");
        model
            .set_cell_with_legacy_formula(0, 1, 1, formula)
            .unwrap();
        model._set("B1", formula);
        model.evaluate();
        assert_eq!(model._get_text("A1"), legacy, "legacy {formula}");
        assert_eq!(model._get_text("A2"), "", "legacy {formula} spills");
        assert_eq!(model._get_text("B1"), dynamic, "dynamic {formula}");
    }
}

/// Every `legacy_formulas` cell of the probes suite: (sheet names of its case, its sheet, row,
/// column, formula without `=`).
fn probes_legacy_cells() -> Vec<(Vec<String>, String, i32, i32, String)> {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../eval/suites/probes/cases"
    );
    let mut cells = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.to_string_lossy().ends_with(".case.json") {
            continue;
        }
        let case: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let sheets = case["sheets"].as_array().cloned().unwrap_or_default();
        let names: Vec<String> = sheets
            .iter()
            .map(|sheet| sheet["name"].as_str().unwrap().to_string())
            .collect();
        for sheet in &sheets {
            let name = sheet["name"].as_str().unwrap();
            for cell in sheet["legacy_formulas"]
                .as_array()
                .cloned()
                .unwrap_or_default()
            {
                let a1 = cell.as_str().unwrap();
                let formula = sheet["cells"][a1].as_str().unwrap();
                let reference = parse_reference_a1(a1).unwrap();
                let body = formula.strip_prefix('=').unwrap().to_string();
                cells.push((
                    names.clone(),
                    name.to_string(),
                    reference.row,
                    reference.column,
                    body,
                ));
            }
        }
    }
    cells
}

// SPEC-0012 "Entry points": the rewrite is idempotent, on its own tree and over its own display
// text, as stored (R1C1) text, so the marker survives a re-entry of the text (T-1852) and the
// export's remove-then-add (`legacy_reading_intersects`).
#[test]
fn the_legacy_rewrite_is_idempotent_over_the_probes_legacy_cells() {
    let cells = probes_legacy_cells();
    assert!(cells.len() > 400, "{}", cells.len());
    let mut marked = 0;
    for (names, sheet, row, column, body) in cells {
        let context = CellReferenceRC { sheet, row, column };
        let mut parser = new_parser(names, vec![], HashMap::new());
        let mut once = parser.parse(&body, &context);
        add_implicit_intersection(&mut once, true);
        let stored = to_rc_format(&once);
        marked += usize::from(is_marked(&stored));
        let mut twice = once.clone();
        add_implicit_intersection(&mut twice, true);
        assert_eq!(to_rc_format(&twice), stored, "{body}");
        let display = to_english_string(&once, &context);
        let mut again = parser.parse(&display, &context);
        add_implicit_intersection(&mut again, true);
        assert_eq!(to_rc_format(&again), stored, "{body} shown as {display}");
    }
    assert!(marked >= 20, "{marked}");
}
