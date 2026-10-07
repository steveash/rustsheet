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
