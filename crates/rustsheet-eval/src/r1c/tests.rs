use super::*;
use crate::accuracy::lists::R3Mark;
use crate::value::compare;

const CASE: &str = "probes/t.case.json";

/// A scored cell: Excel's `excel` against the engine's `engine`, with an optional math value.
fn cell(excel: Value, engine: Value, math: Option<f64>) -> CellResult {
    let mut c = CellResult::for_test("A1", Some("F(1)"), compare(&excel, &engine));
    c.expected = excel;
    c.actual = engine;
    c.math = math.map(MathValue::Number);
    c
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

fn marks(id: &str) -> R3Marks {
    R3Marks::from([(
        id.to_string(),
        R3Mark {
            line: 1,
            semantics_file: "docs/excel-semantics/statistics.md".into(),
            anchor: "x".into(),
        },
    )])
}

fn owned(case: &str, key: &str) -> Phase1Exceptions {
    let mut e = Phase1Exceptions::default();
    e.desktop_owned
        .insert((case.to_string(), key.to_string()), "T-2425".into());
    e
}

fn class(c: &CellResult, r3: &R3Marks, exceptions: &Phase1Exceptions) -> Option<R1c> {
    classify(CASE, c, r3, exceptions)
}

const MATH: f64 = 0.123_456_789_012_345_6;

#[test]
fn a_cell_that_is_not_visibly_off_has_no_class() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    assert_eq!(class(&cell(n(MATH), n(MATH), None), &none.0, &none.1), None);
    // 1e-15 relative: within 15 digits, so not visibly off.
    let close = cell(n(MATH), n(MATH * (1.0 + 1e-15)), None);
    assert_eq!(class(&close, &none.0, &none.1), None);
}

#[test]
fn a_type_or_error_cell_counts_even_inside_the_envelope() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    let error = cell(n(MATH), Value::Error("#NUM!".into()), Some(MATH));
    assert_eq!(class(&error, &none.0, &none.1), Some(R1c::Counts));
    let text = cell(Value::Text("a".into()), n(MATH), Some(MATH));
    assert_eq!(class(&text, &none.0, &none.1), Some(R1c::Counts));
}

#[test]
fn a_number_inside_the_envelope_is_phase_1b_evidence() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    // 1e-12 off Excel (visibly off) and 1e-12 off the math (within ε); Excel 2e-12 off the math.
    let c = cell(n(MATH * (1.0 + 1e-12)), n(MATH * (1.0 - 1e-12)), Some(MATH));
    assert_eq!(class(&c, &none.0, &none.1), Some(R1c::Phase1b));
}

#[test]
fn a_number_outside_the_envelope_or_without_math_counts() {
    let none = (R3Marks::new(), Phase1Exceptions::default());
    let far = cell(n(MATH), n(MATH * (1.0 + 1e-6)), Some(MATH));
    assert_eq!(class(&far, &none.0, &none.1), Some(R1c::Counts));
    let no_math = cell(n(MATH * (1.0 + 1e-12)), n(MATH), None);
    assert_eq!(class(&no_math, &none.0, &none.1), Some(R1c::Counts));
}

#[test]
fn excel_more_than_g_off_the_math_is_a_semantic_cell_that_counts_unless_r3_marks_it() {
    // The engine returns the math; Excel is 1e-3 off it (more than G = 1e-4).
    let c = cell(n(MATH * 1.001), n(MATH), Some(MATH));
    let none = Phase1Exceptions::default();
    assert_eq!(class(&c, &R3Marks::new(), &none), Some(R1c::Counts));
    let marked = marks(&cell_id(CASE, "S!A1"));
    assert_eq!(class(&c, &marked, &none), Some(R1c::R3Pass));
    // An R3 cell where the engine misses the math still counts.
    let miss = cell(n(MATH * 1.001), n(MATH * (1.0 + 1e-6)), Some(MATH));
    assert_eq!(class(&miss, &marked, &none), Some(R1c::Counts));
}

#[test]
fn a_cell_only_m14_tasks_own_leaves_the_row_whatever_its_values() {
    let exceptions = owned(CASE, "S!A1");
    let error = cell(n(MATH), Value::Error("#NUM!".into()), None);
    assert_eq!(
        class(&error, &R3Marks::new(), &exceptions),
        Some(R1c::M14Owned)
    );
    let other_case = owned("probes/other.case.json", "S!A1");
    assert_eq!(
        class(&error, &R3Marks::new(), &other_case),
        Some(R1c::Counts)
    );
}

#[test]
fn the_run_classes_and_the_phase_1b_counts_per_suite() {
    let mut probe = CaseResult::for_test(CASE, "probes");
    let inside = cell(n(MATH * (1.0 + 1e-12)), n(MATH * (1.0 - 1e-12)), Some(MATH));
    let mut far = cell(n(MATH), n(MATH * 1.5), Some(MATH));
    far.cell = "A2".into();
    probe.cells = vec![inside, far];
    let mut numeric = CaseResult::for_test("probes-numeric/n.case.json", "probes-numeric");
    numeric.cells = vec![cell(n(MATH), Value::Error("#NUM!".into()), None)];
    let mut generated = CaseResult::for_test("gen-m9/g.case.json", "gen-m9");
    generated.cells = vec![cell(n(MATH), n(MATH * 1.5), None)];
    let run = R1cCells::build(
        &[probe, numeric, generated],
        &R3Marks::new(),
        &Phase1Exceptions::default(),
    );
    assert_eq!(run.class("probes/t.case.json!S!A1"), R1c::Phase1b);
    assert!(run.counts("probes/t.case.json!S!A2"));
    assert!(run.counts("probes/never-classified.case.json!S!A1"));
    let evidence = run.phase1b();
    assert_eq!(
        evidence.suites.keys().collect::<Vec<_>>(),
        ["probes", "probes-numeric"]
    );
    assert_eq!(
        evidence.suites["probes"],
        R1cCounts {
            counts: 1,
            phase1b: 1,
            r3_pass: 0,
            m14_owned: 0
        }
    );
    assert_eq!(evidence.suites["probes-numeric"].counts, 1);
    let line = evidence.line().unwrap_or_default();
    assert!(line.contains("probes 1 counted, 1 Phase 1b evidence, 0 R3 passing, 0 M14-owned"));
    assert_eq!(Phase1bEvidence::default().line(), None);
}
