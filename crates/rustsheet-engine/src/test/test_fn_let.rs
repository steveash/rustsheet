#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_let_wrong_arg_count() {
    let mut model = new_empty_model();
    // Too few args
    model._set("A1", "=LET(x,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");

    // Even number of args
    model._set("A2", "=LET(x,1,y,2)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

#[test]
fn test_let_two_variables() {
    let mut model = new_empty_model();
    // LET(x, 2, y, 4, x+y) → 6
    model._set("A1", "=LET(x,2,y,4,x+y)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "6");
}

#[test]
fn test_let_with_cell_reference() {
    let mut model = new_empty_model();
    model._set("A1", "5");
    // LET(x, A1, SUM(x*3, x)) → SUM(15, 5) = 20
    model._set("B1", "=LET(x,A1,SUM(x*3,x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "20");
}

#[test]
fn test_let_nested() {
    let mut model = new_empty_model();
    // LET(x, 3, LET(x, 4, x*5)*x) → inner x=4: 4*5=20, outer x=3: 20*3=60
    model._set("A1", "=LET(x,3,LET(x,4,x*5)*x)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "60");
}

#[test]
fn test_let_inner_binding_references_outer_value() {
    let mut model = new_empty_model();
    // LET(x,1,LET(x,x+1,x)) → inner x is bound from outer x+1 = 2, result = 2
    model._set("A1", "=LET(x,1,LET(x,x+1,x))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}
#[test]
fn test_let_rebind_same_name_within_single_let() {
    let mut model = new_empty_model();
    model._set("A1", "=LET(x,1,x,2,x)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn test_let_duplicate_names_differing_in_case_are_refused() {
    // Excel for the web refuses `LET(x,1,X,2,x)` and `LET(é,1,É,2,é)` as it refuses
    // `LET(x,1,x,2,x)` (docs/excel-semantics/references.md, T-1387).
    let mut model = new_empty_model();
    model._set("A1", "=LET(x,1,X,2,x)");
    model._set("A2", "=LET(é,1,É,2,é)");
    model._set("A3", "=LET(é,1,É+1)");
    model._set("A4", "=LET(Ä,2,ä*10)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "20");
}

#[test]
fn test_let_r_and_c_variables() {
    let mut model = new_empty_model();
    // r and c must work as LET variable names
    model._set("A1", "=LET(r,3,c,4,r+c)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
}

#[test]
fn test_lambda_r_and_c_parameters() {
    let mut model = new_empty_model();
    // r and c must work as LAMBDA parameter names
    model._set("A1", "=LAMBDA(r,c,r+c)(3,4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
}

#[test]
fn test_let_names_are_case_insensitive() {
    let mut model = new_empty_model();
    model._set("A1", "=LET(x,2,X+1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_subtotal_of_take_over_a_selector_holding_a_computed_array_spills() {
    let mut model = new_empty_model();
    model._set("A1", "=SUBTOTAL(9,TAKE(CHOOSE(1,SEQUENCE(4)),2))");
    model._set("C1", "=SUBTOTAL(9,TAKE(SWITCH(9,1,D1:D4,SEQUENCE(4)),2))");
    model._set("E1", "=SUBTOTAL(9,TAKE(IFS(TRUE,SEQUENCE(4)),2))");
    model.evaluate();
    // probes/subtotal-reference-operands-f C46, C111 and C66: two #VALUE! each.
    for cell in ["A1", "A2", "C1", "C2", "E1", "E2"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
}

#[test]
fn test_lambda_parameter_names_are_case_insensitive() {
    let mut model = new_empty_model();
    // probes/let-lambda-name-case A1, A2, A6, A10, A11 and A15, in that order.
    model._set("A1", "=LET(x,1,LAMBDA(X,x*10)(2))");
    model._set("A2", "=LAMBDA(x,X+1)(2)");
    model._set("A3", "=LET(X,1,LAMBDA(x,X*10)(2))");
    model._set("A4", "=LAMBDA(Ab,aB*2)(4)");
    model._set("A5", "=LET(f,LAMBDA(x,x+1),F(2))");
    model._set("A6", "=LET(x,1,LAMBDA(X,x)(2))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "20");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "20");
    assert_eq!(model._get_text("A4"), "8");
    assert_eq!(model._get_text("A5"), "3");
    assert_eq!(model._get_text("A6"), "2");
}

#[test]
fn test_let_binds_index_reference_and_at_index_reference() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("B2", "6");
    // An INDEX binding stays a reference (probes/let-bind-reference-forms).
    model._set("A1", "=LET(x,INDEX(B1:B2,2),ROW(x))");
    // `@INDEX(..)` stays a reference too (probes/let-bind-implicit-index, T-1503).
    model._set("A2", "=LET(x,@INDEX(B1:B2,2),ISREF(x))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model._get_text("A2"), "TRUE");
}

// probes/r1c1-let-name-* (excel-graph, T-1652): LET takes names Excel reads as R1C1 references,
// and they survive a reload (formulas are kept in R1C1 form, where an uppercase `R1C1` is
// stored as `_xlpm.R1C1`, T-1658).
#[test]
fn let_names_may_be_r1c1_shaped() {
    let mut model = new_empty_model();
    let names = [
        "R12C",
        "RC",
        "R1C",
        "R1C1D",
        "C1D",
        "R1.C1",
        "r1c1",
        "R1C1",
        "R5C7",
        "R1048576C16384",
        "R1c1",
        "r1C1",
        "R0",
        "R0C1",
        "R1048577C1",
    ];
    for (row, name) in names.iter().enumerate() {
        model._set(
            &format!("A{}", row + 1),
            &format!("=LET({name},5,{name}*2)"),
        );
    }
    model.evaluate();
    let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for (row, name) in names.iter().enumerate() {
        let cell = format!("A{}", row + 1);
        assert_eq!(model._get_text(&cell), "10", "{name}");
        assert_eq!(reloaded._get_text(&cell), "10", "{name}");
        assert_eq!(
            reloaded._get_formula(&cell),
            format!("=LET({name},5,{name}*2)")
        );
    }
}

// A LAMBDA parameter spelled like an R1C1 cell is a name too, through a reload (T-1658), and the
// stored form is R1C1 with the escaped name.
#[test]
fn lambda_parameters_spelled_like_r1c1_cells_survive_a_reload() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(R5C7,[R1C1],R5C7*2+ISOMITTED(R1C1))(5)");
    model._set("A2", "=LET(R1C1,LAMBDA(R2C2,R2C2+1),R1C1(9))");
    model.evaluate();
    let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for m in [&model, &reloaded] {
        assert_eq!(m._get_text("A1"), "11");
        assert_eq!(m._get_text("A2"), "10");
    }
    assert_eq!(
        reloaded._get_formula("A1"),
        "=LAMBDA(R5C7,[R1C1],R5C7*2+ISOMITTED(R1C1))(5)"
    );
}

// probes/let-name-shadows-defined-name: a LET name or LAMBDA parameter shadows a defined name
// of the same name in any case; a closure keeps the defined name it was written against.
#[test]
fn let_names_and_lambda_parameters_shadow_defined_names() {
    let mut model = new_empty_model();
    model.define_name("Mk", None, "=LAMBDA(a,a+100)").unwrap();
    model.define_name("Pv", None, "=7").unwrap();
    let cases = [
        ("=LET(mk,LAMBDA(a,a+1),mk(1))", "2"),
        ("=LET(Mk,LAMBDA(a,a+1),Mk(1))", "2"),
        ("=mk(1)", "101"),
        ("=LET(pv,2,pv+1)", "3"),
        ("=LET(PV,2,pv+1)", "3"),
        ("=Pv+1", "8"),
        ("=LAMBDA(pv,pv+1)(2)", "3"),
        ("=LAMBDA(PV,Pv*10)(2)", "20"),
        ("=LET(x,Pv,LET(pv,1,x+pv))", "8"),
        ("=LET(f,LAMBDA(a,a+Pv),LET(pv,50,f(1)))", "8"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    // The stored form (R1C1 grammar) prints the names without `_xlpm.`: a reload keeps the shadowing.
    let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for m in [&model, &reloaded] {
        for (row, (formula, expected)) in cases.iter().enumerate() {
            assert_eq!(
                m._get_text(&format!("A{}", row + 1)),
                *expected,
                "{formula}"
            );
        }
    }
}

// probes/let-lambda-body-own-name (excel-graph, T-2558): a LAMBDA body naming its own LET name
// is #NAME?, with or without a defined name of that name. (A LET name spelled like a function,
// `Pv`, called by name is a separate gap: T-2567.)
#[test]
fn lambda_body_naming_its_own_let_name_is_name_error() {
    let mut model = new_empty_model();
    model.define_name("Qv", None, "=7").unwrap();
    let cases = [
        ("=LET(y,LAMBDA(a,y),y(1))", "#NAME?"),
        ("=LET(Y,LAMBDA(a,y),y(1))", "#NAME?"),
        ("=LET(y,LAMBDA(a,a+1),y(1))", "2"),
        ("=LET(y,y+1,y)", "#NAME?"),
        ("=LET(f,LAMBDA(a,g(a)),g,LAMBDA(a,a+1),f(1))", "#NAME?"),
        ("=LET(Qv,LAMBDA(a,Qv),Qv(1))", "#NAME?"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn test_let_name_spelled_like_function_called() {
    // A bound name spelled like a function is the variable when called, in any case
    // (probes/let-name-function-spelled-call, let-lambda-body-own-name, T-2567). `_set` reads
    // the formula as stored A1 text; typed, the function wins (crates/rustsheet/src/edit/tests.rs).
    let mut model = new_empty_model();
    let cases = [
        ("=LET(Pv,LAMBDA(a,a),Pv(1))", "1"),
        ("=LET(Pv,LAMBDA(a,Pv),Pv(1))", "#NAME?"),
        ("=LET(Sum,LAMBDA(a,a*2),Sum(3))", "6"),
        ("=LET(abs,LAMBDA(a,a*2),ABS(-3))", "-6"),
        ("=LAMBDA(Pv,Pv(2))(LAMBDA(a,a+1))", "3"),
        ("=LET(Pv,2,Pv(1))", "#NAME?"),
        ("=LET(Pv,LAMBDA(a,a),PV(1,2,3))", "#VALUE!"),
        ("=LET(Pv,LAMBDA(a,a),LET(x,1,Pv(5)))", "5"),
        ("=LET(If,LAMBDA(a,a+1),If(1))", "2"),
        ("=LET(Sum,LAMBDA(a,a*2),SUM(1,2))", "#VALUE!"),
        ("=LET(Today,LAMBDA(a,a),Today(4))", "4"),
        ("=LET(Pv,LAMBDA(a,a),Pv())", "#VALUE!"),
        // Out of the LET's scope the function is back.
        ("=LET(x,LET(Sum,LAMBDA(a,a*2),Sum(1)),SUM(x,1))", "3"),
    ];
    for (row, (formula, _)) in (1..).zip(cases) {
        model._set(&format!("A{row}"), formula);
    }
    // All cells are set before one evaluation, then checked.
    model.evaluate();
    for (row, (formula, expected)) in (1..).zip(cases) {
        assert_eq!(model._get_text(&format!("A{row}")), expected, "{formula}");
    }
}

#[test]
fn test_lambda_parameter_under_a_unary_plus_is_refused() {
    // Excel for the web refuses `=LAMBDA(+x,x)(1)` typed (Graph PATCH 400,
    // docs/research/t2772-let-lambda-plus-refusal-2026-10-27.md), and so is `=LET(+x,1,x)`;
    // a plus inside the value or the body is valid (`LET(x,1,+x)`).
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(+x,x)(1)");
    model._set("A2", "=LET(+x,1,x)");
    model._set("A3", "=LET(x,1,+x)");
    model._set("A4", "=LET(x,+1,y,+x,x+y)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
    assert_eq!(model._get_text("A3"), "1");
    assert_eq!(model._get_text("A4"), "2");
}
