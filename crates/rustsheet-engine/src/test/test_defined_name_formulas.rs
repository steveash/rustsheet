#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A defined name whose body is any formula is evaluated where it is used (T-0158;
// probes/defined-name-formula-bodies, excel-graph).
#[test]
fn formula_bodies_evaluate_where_used() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model
        .define_name("total", None, "=SUM(Sheet1!$A$1:$A$2)")
        .unwrap();
    model.define_name("chain", None, "=total*2").unwrap();
    model._set("B1", "=chain");
    model._set("B2", "=ISREF(total)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"6");
    assert_eq!(model._get_text("B2"), *"FALSE");
}

// A name that reaches itself is #NAME? (probes/defined-name-formula-bodies H11, H13), and the
// evaluation stays bounded however it loops (directive 5).
#[test]
fn self_referencing_names_are_name_errors() {
    let mut model = new_empty_model();
    model.define_name("loopa", None, "=loopb").unwrap();
    model.define_name("loopb", None, "=loopa").unwrap();
    model.define_name("selfref", None, "=selfref+1").unwrap();
    model._set("A1", "=loopa");
    model._set("A2", "=selfref");
    model._set("A3", "=ISREF(loopa)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NAME?");
    assert_eq!(model._get_text("A2"), *"#NAME?");
    assert_eq!(model._get_text("A3"), *"FALSE");
}

// A chain of distinct names past the depth limit is #NAME?, not a stack overflow (DEVIATION:
// references.md "A defined name may be a formula").
#[test]
fn long_name_chains_are_bounded() {
    let mut model = new_empty_model();
    for i in 0..200 {
        model
            .define_name(&format!("n{i}x"), None, &format!("=n{}x+1", i + 1))
            .unwrap();
    }
    model.define_name("n200x", None, "=1").unwrap();
    model._set("A1", "=n190x");
    model._set("A2", "=n0x");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"11");
    assert_eq!(model._get_text("A2"), *"#NAME?");
}

// Bodies that use a name twice would double the work at every level; the evaluation budget
// keeps them fast (DEVIATION: references.md "A defined name may be a formula").
#[test]
fn branching_names_are_bounded_in_time() {
    let mut model = new_empty_model();
    model.define_name("twice", None, "=twice+twice").unwrap();
    for i in 0..60 {
        model
            .define_name(
                &format!("d{i}x"),
                None,
                &format!("=d{}x+d{}x", i + 1, i + 1),
            )
            .unwrap();
    }
    model.define_name("d60x", None, "=1").unwrap();
    model._set("A1", "=twice");
    model._set("A2", "=d0x");
    model._set("A3", "=d55x");
    model._set("A4", "=ISREF(twice)");
    // One budget per cell, however many times its formula uses the name.
    model._set("A5", &format!("=SUM({})", vec!["twice"; 200].join(",")));
    let start = std::time::Instant::now();
    model.evaluate();
    assert!(start.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(model._get_text("A1"), *"#NAME?");
    assert_eq!(model._get_text("A2"), *"#NAME?");
    // 2^5 = 32 within the budget.
    assert_eq!(model._get_text("A3"), *"32");
    assert_eq!(model._get_text("A4"), *"FALSE");
    assert_eq!(model._get_text("A5"), *"#NAME?");
}

// Every name in a chain nests its body: the depth and stack budgets stop the chain before the
// native stack runs out (directive 5). The nesting stays shallow so that the chain, not one
// cell's nesting, is what runs into the budgets (a cell alone: test_nested_formula_stack.rs).
#[test]
fn deeply_nested_name_chains_stay_on_the_stack() {
    let mut model = new_empty_model();
    let depth = 20;
    const CHAIN: usize = 64;
    for i in 0..CHAIN {
        let body = format!("={}n{}x{}", "ABS(".repeat(depth), i + 1, ")".repeat(depth));
        model.define_name(&format!("n{i}x"), None, &body).unwrap();
    }
    model
        .define_name(&format!("n{CHAIN}x"), None, "=1")
        .unwrap();
    model._set("A1", "=n0x");
    model._set("A2", &format!("=n{}x", CHAIN - 2));
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NAME?");
    // Two short links fit a release build's stack budget, not always a debug build's, whose
    // frames are many times larger: either way the cell gets a value, never a crash.
    let short_chain = model._get_text("A2");
    assert!(
        short_chain == "1" || short_chain == "#NAME?",
        "{short_chain}"
    );
}

// A self-referencing name with a nested body stops on the stack budget, not on a stack overflow
// (reproduced as an overflow on a 2 MB thread before the budget was shared with the cells).
#[test]
fn nested_self_reference_stays_on_the_stack() {
    let mut model = new_empty_model();
    let body = format!("={}selfref{}", "ABS(".repeat(20), ")".repeat(20));
    model.define_name("selfref", None, &body).unwrap();
    model._set("A1", "=selfref");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NAME?");
}

// Outside a cell every use gets its own budget: 12,000 uses after `evaluate()` all succeed.
#[test]
fn uses_outside_a_cell_do_not_share_a_budget() {
    let mut model = new_empty_model();
    model.define_name("thr", None, "=0.5").unwrap();
    model.evaluate();
    for _ in 0..12_000 {
        assert_eq!(
            model.evaluate_formula("=thr", 0),
            Some(0.5),
            "a use outside a cell ran out of budget"
        );
    }
}

// A range whose corners are on two sheets is `#VALUE!`, not a reference (T-1166;
// probes/sheet-qualified-corner-order Calc!A7, excel-graph).
#[test]
fn isref_false_for_cross_sheet_corner_range() {
    let mut model = new_empty_model();
    model.add_sheet("Other").unwrap();
    model._set("A1", "=ISREF(Sheet1!A1:Other!B2)");
    model._set("A2", "=ISREF(Sheet1!A1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
    assert_eq!(model._get_text("A2"), *"TRUE");
}

// `Sheet2!loc`, where Sheet2 has no local `loc`, is the workbook-scoped `loc`; with neither it is
// #NAME? (gen-m8a names-scope c238d93c128fd5ee, names-errors 6322c0f2638aa11b).
#[test]
fn sheet_qualified_name_falls_back_to_workbook_name() {
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model.define_name("loc", None, "10").unwrap();
    model.define_name("loc", Some(0), "30").unwrap();
    model.define_name("only1", Some(0), "5").unwrap();
    model._set("A1", "=Sheet2!loc");
    model._set("A2", "=ISERROR(Sheet2!loc)");
    model._set("A3", "=Sheet1!loc");
    model._set("A4", "=IFERROR(Sheet2!only1,\"x\")");
    model._set("A5", "=Sheet2!LOC");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"10");
    assert_eq!(model._get_text("A2"), *"FALSE");
    assert_eq!(model._get_text("A3"), *"30");
    assert_eq!(model._get_text("A4"), *"x");
    assert_eq!(model._get_text("A5"), *"10");
}

// A name spelled like an out-of-range R1C1 reference is defined but never resolved, also after a
// reload from the R1C1 form formulas are stored in (probes/r1c1-name-*, r1c1-name-use-*,
// excel-graph, T-1657).
#[test]
fn out_of_range_r1c1_names_are_never_resolved() {
    let names = ["R0", "C0", "R0C1", "r0c1", "R1048577", "R1048577C1"];
    for name in names {
        let mut model = new_empty_model();
        model._set("B1", "7");
        model.define_name(name, None, "=Sheet1!$B$1").unwrap();
        model._set("A1", &format!("={name}*1"));
        model._set("A2", &format!("=ISREF(INDIRECT(\"{name}\"))"));
        model._set("A3", &format!("=SUM({name})"));
        model.evaluate();
        let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
        reloaded.evaluate();
        for m in [&model, &reloaded] {
            assert_eq!(m._get_text("A1"), "#NAME?", "{name}");
            assert_eq!(m._get_text("A2"), "FALSE", "{name}");
            assert_eq!(m._get_text("A3"), "#NAME?", "{name}");
            assert_eq!(m._get_formula("A1"), format!("={name}*1"), "{name}");
        }
    }
}
