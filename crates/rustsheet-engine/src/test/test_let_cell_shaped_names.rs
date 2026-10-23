#![allow(clippy::unwrap_used)]

//! LET names and LAMBDA parameters spelled like a cell (`f1`, `add5`): the variable within their
//! call, the cell outside it (docs/excel-semantics/references.md "LET and LAMBDA names spelled
//! like a cell"; probes/let-cell-shaped-name-stored, lambda-escaping-closures H4, H5, excel-graph,
//! T-2040).

use crate::test::util::new_empty_model;
use crate::Model;

#[test]
fn let_and_lambda_names_spelled_like_cells_are_variables_in_scope() {
    let mut model = new_empty_model();
    model._set("F1", "7");
    // The formulas are written as Excel prints them (FORMULATEXT, probes/let-cell-shaped-name-stored).
    let cases = [
        ("=LET(f1,1,f1+1)", "2"),
        ("=LAMBDA(f1,f1*2)(3)", "6"),
        ("=LET(c1,10,r1,2,c1*r1)", "20"),
        ("=LET(A1,5,A1*2)", "10"),
        ("=LET(x,LET(xy1,3,xy1+1),x)", "4"),
        // Outside the LET's call, `F1` is the cell again.
        ("=LET(f1,1,LAMBDA(f1,f1*10)(2))+F1", "27"),
        ("=LAMBDA(x,[f1],ISOMITTED(f1))(1)", "TRUE"),
        // Called (probes/lambda-escaping-closures H4, H5).
        ("=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),add5,mk(5),add5(1))", "6"),
        (
            "=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),f1,mk(1),f2,mk(100),f1(0)+f2(0)*1000)",
            "100001",
        ),
        // Unprobed as typed: a use before the binding is the cell (the probe file would prefix it).
        ("=LET(x,F1,F1,2,x+F1)", "9"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("H{}", row + 1), formula);
    }
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for (row, (formula, value)) in cases.iter().enumerate() {
        let cell = format!("H{}", row + 1);
        for m in [&model, &reloaded] {
            assert_eq!(m._get_text(&cell), *value, "{formula}");
            assert_eq!(m._get_formula(&cell), *formula);
        }
    }
}

// The use takes its declaration's spelling, as for any LET name (T-1720); a `$` or a sheet makes
// it a cell, which a LET cannot bind.
#[test]
fn cell_shaped_names_print_in_their_declaration_spelling() {
    let mut model = new_empty_model();
    model._set("A1", "=LET(f1,1,F1+1)");
    model._set("A2", "=LET($F$1,1,F1+1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model._get_formula("A1"), "=LET(f1,1,f1+1)");
    assert_eq!(model._get_text("A2"), "#VALUE!");
}
