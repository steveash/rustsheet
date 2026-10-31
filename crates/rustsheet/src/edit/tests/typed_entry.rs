//! Typed formulas Excel refuses or reads differently from the same text stored (decision 0103).

use super::*;

/// Excel refuses these typed (T-2040, Graph PATCH 400) and binds them stored as `_xlpm.f1`
/// (probes/let-cell-shaped-name-stored): decision 0103.
#[test]
fn typed_cell_shaped_let_and_lambda_names_are_refused() {
    let mut wb = book();
    for (a1, text) in [
        ("A1", "=LET(f1,1,f1+1)"),
        ("A2", "=LAMBDA(f1,f1*2)(3)"),
        ("A3", "=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),add5,mk(5),add5(1))"),
    ] {
        let err = wb.set(a1, Input::Formula(text.into())).unwrap_err();
        assert!(
            matches!(err, EditError::InvalidInput { .. }),
            "{text}: {err:?}"
        );
        let err = wb.set(a1, Input::Parse(text.into())).unwrap_err();
        assert!(
            matches!(err, EditError::InvalidInput { .. }),
            "{text}: {err:?}"
        );
        assert_eq!(wb.value(a1).unwrap(), Value::Empty, "{text}");
    }
    // The control binds typed; the stored form binds the cell-shaped name.
    wb.set("B1", Input::Formula("=LET(x,1,x+1)".into()))
        .unwrap();
    wb.set("B2", Input::StoredFormula("=LET(f1,1,f1+1)".into()))
        .unwrap();
    assert_eq!(wb.value("B1").unwrap(), Value::Number(2.0));
    assert_eq!(wb.value("B2").unwrap(), Value::Number(2.0));
}

/// Excel refuses a typed call with an argument count the function does not take, also where the
/// callee is a LET name or LAMBDA parameter spelled like the function, since typed the call is
/// the function (probes/let-function-name-typed-*, T-2567 Graph PATCH 400). Stored, the name is
/// the variable (probes/let-name-function-spelled-call: 1, 3 and `#NAME?`). T-2568; every catalogued
/// function and whole SORTBY cycles, T-2632.
#[test]
fn typed_calls_with_a_wrong_argument_count_are_refused() {
    let mut wb = book();
    for (a1, text) in [
        ("A1", "=LET(Pv,LAMBDA(a,a),Pv(1))"),
        ("A2", "=LAMBDA(Pv,Pv(2))(LAMBDA(a,a+1))"),
        ("A3", "=LET(Pv,2,Pv(1))"),
        ("A4", "=PV(1)"),
        ("A5", "=ABS(1,2)"),
        ("A6", "=1+SUM(PV(0.1,2,3,4,0,1))"),
        // Functions outside the lifting table, from the catalog's counts (Graph PATCH 400, T-2632).
        ("A8", "=SUM()"),
        ("A9", "=IF(1)"),
        ("A10", "=XLOOKUP(1,{1},{2},0,0,1,1)"),
        ("A11", "=ACCRINTM(DATE(2020,1,1),DATE(2021,1,1),0.1)"),
        ("A12", "=SORTBY({1;2},{2;1},1,{1;1})"),
        ("A13", "=PI(1)"),
        // One past the maximum, and off-cycle IFS and LET (Graph PATCH 400, T-2682).
        ("A14", "=IF(TRUE,2,3,4)"),
        ("A15", "=INDEX((E1:F2,G1:H2),1,1,2,1)"),
        ("A16", "=IFS(TRUE,1,2)"),
        ("A17", "=LET(x,1,y,2)"),
        ("A18", "=FIND(\"a\",\"abc\",1,1)"),
        ("A19", "=LENB(\"abc\",1)"),
        ("A20", "=SWITCH(1,2)"),
    ] {
        for input in [Input::Formula(text.into()), Input::Parse(text.into())] {
            let err = wb.set(a1, input).unwrap_err();
            assert!(
                matches!(err, EditError::InvalidInput { .. }),
                "{text}: {err:?}"
            );
        }
        assert_eq!(wb.value(a1).unwrap(), Value::Empty, "{text}");
    }
    for (a1, text) in [
        ("B1", "=LET(Pv,LAMBDA(a,a),Pv(1))"),
        ("B2", "=LAMBDA(Pv,Pv(2))(LAMBDA(a,a+1))"),
        ("B3", "=LET(Pv,2,Pv(1))"),
    ] {
        wb.set(a1, Input::StoredFormula(text.into())).unwrap();
    }
    assert_eq!(wb.value("B1").unwrap(), Value::Number(1.0));
    assert_eq!(wb.value("B2").unwrap(), Value::Number(3.0));
    assert!(matches!(wb.value("B3").unwrap(), Value::Error(_)));
    // The first wrong-count call to end, left to right, is the one named.
    let err = wb
        .set("A7", Input::Formula("=ABS(1,2)+PV(1)".into()))
        .unwrap_err();
    assert!(format!("{err:?}").contains("ABS"), "{err:?}");
    // The counts the function takes stay accepted typed, its optional arguments included.
    for (a1, text) in [
        ("C3", "=ROWS(SORTBY({1;2},{2;1},1,{1;1},1))"),
        ("C4", "=SUM(1)"),
        ("C5", "=IF(1,2)"),
        ("C6", "=XLOOKUP(1,{1},{2},0,0,1)"),
        ("C7", "=PI()"),
        // INDEX's reference form takes `area_num` (probes/index-area-num-snap).
        ("C8", "=INDEX((E1:F2,G1:H2),1,1,2)"),
        // Whole IFS and LET pairs (probes/t2682-typed-max-arg-counts A4:A7).
        ("C9", "=IFS(FALSE,1,TRUE,2)"),
        ("C10", "=LET(x,1,y,2,x+y)"),
        ("C11", "=SWITCH(1,2,3,4)"),
    ] {
        wb.set(a1, Input::Formula(text.into())).unwrap();
    }
    assert_eq!(wb.value("C3").unwrap(), Value::Number(2.0));
    assert_eq!(wb.value("C6").unwrap(), Value::Number(2.0));
    // Stored, a wrong-count call is kept and evaluates to an error.
    wb.set("D1", Input::StoredFormula("=SUM()".into())).unwrap();
    assert!(matches!(wb.value("D1").unwrap(), Value::Error(_)));
    wb.set("C1", Input::Formula("=PV(0,2,-3)".into())).unwrap();
    wb.set("C2", Input::Formula("=PV(0,2,-3,0,1)".into()))
        .unwrap();
    assert_eq!(wb.value("C1").unwrap(), Value::Number(6.0));
    assert_eq!(wb.value("C2").unwrap(), Value::Number(6.0));
}

/// A LET name spelled like a function: typed, a call of it is the function (`=LET(Sum,..,Sum(3))`
/// is `SUM(3)`, probes/let-function-name-typed-12); stored as `_xlpm.Sum(3)`, the variable
/// (probes/let-name-function-spelled-call A1). Excel's own stored text declares `_xlpm.Sum`, and a
/// bare `SUM(` there is the function (typed-12, typed-05 as Excel saves them). T-2567.
#[test]
fn let_name_spelled_like_function_typed_and_stored() {
    let mut wb = book();
    let text = "=LET(Sum,LAMBDA(a,a*2),Sum(3))";
    wb.set("A1", Input::Formula(text.into())).unwrap();
    wb.set("A2", Input::StoredFormula(text.into())).unwrap();
    wb.set("A3", Input::Formula("=LET(sum,SUM(1,2),sum*2)".into()))
        .unwrap();
    assert_eq!(wb.value("A1").unwrap(), Value::Number(3.0));
    assert_eq!(wb.value("A2").unwrap(), Value::Number(6.0));
    assert_eq!(wb.value("A3").unwrap(), Value::Number(6.0));
    for (a1, text, expected) in [
        (
            "B1",
            "=_xlfn.LET(_xlpm.Sum,_xlfn.LAMBDA(_xlpm.a,_xlpm.a*2),SUM(3))",
            3.0,
        ),
        ("B2", "=_xlfn.LET(_xlpm.sum,SUM(1,2),_xlpm.sum*2)", 6.0),
        (
            "B3",
            "=_xlfn.LET(_xlpm.Sum,_xlfn.LAMBDA(_xlpm.a,_xlpm.a*2),_xlpm.Sum(3))",
            6.0,
        ),
    ] {
        wb.set(a1, Input::StoredFormula(text.into())).unwrap();
        assert_eq!(wb.value(a1).unwrap(), Value::Number(expected), "{text}");
    }
}

/// `calls` nested function calls in the shape of `form`, as typed: the innermost call is the LET or
/// the called LAMBDA for those forms (probes/t3097-nesting-*).
fn nested_calls(form: &str, calls: usize) -> String {
    let (open, inner, close) = match form {
        "abs" => ("ABS(", "1", ")"),
        "if" => ("IF(TRUE,", "1", ")"),
        "sum" => ("SUM(", "1", ")"),
        "op-abs" => ("ABS(1+", "1", ")"),
        "mixed" => ("ABS(SUM(INT(", "1", ")))"),
        "let" => ("ABS(", "LET(x,1,x)", ")"),
        _ => ("ABS(", "LAMBDA(x,x)(1)", ")"),
    };
    // A mixed level is three calls (`calls` is then a multiple of three); a LET or a called
    // LAMBDA innermost is one of the calls.
    let outer = match (form, inner) {
        ("mixed", _) => calls / 3,
        (_, "1") => calls,
        _ => calls - 1,
    };
    format!("={}{inner}{}", open.repeat(outer), close.repeat(outer))
}

/// Excel takes 65 nested function calls typed and refuses 66 (Graph PATCH 400; syntax.md "Formula
/// nesting depth cap", probes/t3097-nesting-*, T-3097), whatever the function, with an operator
/// between levels, and with a LET or a called LAMBDA innermost.
#[test]
fn typed_formulas_nesting_more_than_65_calls_are_refused() {
    let mut wb = book();
    for (row, form) in ["abs", "if", "sum", "op-abs", "let", "lambda", "mixed"]
        .into_iter()
        .enumerate()
    {
        let accepted = format!("A{}", row + 1);
        let refused = format!("B{}", row + 1);
        // 63 and 66 for the mixed form, whose levels come three at a time.
        let (fits, past) = if form == "mixed" { (63, 66) } else { (65, 66) };
        wb.set(accepted.as_str(), Input::Formula(nested_calls(form, fits)))
            .unwrap();
        assert!(
            matches!(wb.value(accepted.as_str()).unwrap(), Value::Number(_)),
            "{form}"
        );
        for input in [
            Input::Formula(nested_calls(form, past)),
            Input::Parse(nested_calls(form, past)),
        ] {
            let err = wb.set(refused.as_str(), input).unwrap_err();
            assert!(
                matches!(err, EditError::InvalidInput { .. }),
                "{form}: {err:?}"
            );
        }
        assert_eq!(wb.value(refused.as_str()).unwrap(), Value::Empty, "{form}");
    }
    // Parentheses add no level: 66 calls are refused, 65 calls inside 100 parentheses are not.
    let deep = format!("=(((((((((({}))))))))))", &nested_calls("abs", 65)[1..]);
    wb.set("C1", Input::Formula(deep)).unwrap();
    assert_eq!(wb.value("C1").unwrap(), Value::Number(1.0));
}

/// Typed, the `@` and `#` operators add no level (65 ABS calls around `@C1:C2`, or 64 around
/// `SUM(C1#)`, are accepted), but `SINGLE(` spelled as a call counts like any call: 64 ABS calls
/// around it are accepted and 65 refused (Graph PATCH 400; probes/t3143-single-*-typed,
/// t3143-anchor-inner-*-typed and t3143-single-spelled-*-typed, T-3143). A stored formula is not
/// checked: [`Input::StoredFormula`] keeps 66 calls, as it keeps `=SUM()`.
#[test]
fn typed_at_and_hash_operators_add_no_nesting_level() {
    let abs = |n: usize, inner: &str| format!("={}{inner}{}", "ABS(".repeat(n), ")".repeat(n));
    let mut wb = book();
    wb.set("C1", Input::Formula("=SEQUENCE(2)".into())).unwrap();
    // The `@` and SINGLE cells sit in rows 1-2, where `@C1:C2` meets the spill.
    for (a1, formula) in [
        ("A1", abs(65, "@C1:C2")),
        ("A2", abs(64, "SINGLE(C1:C2)")),
        ("A3", format!("=ABS(@{})", &abs(64, "1")[1..])),
        ("A4", abs(64, "SUM(C1#)")),
    ] {
        wb.set(a1, Input::Formula(formula)).unwrap();
        assert!(matches!(wb.value(a1).unwrap(), Value::Number(_)), "{a1}");
    }
    let err = wb
        .set("B1", Input::Formula(abs(65, "SINGLE(C1:C2)")))
        .unwrap_err();
    assert!(matches!(err, EditError::InvalidInput { .. }), "{err:?}");
    wb.set("B2", Input::StoredFormula(abs(66, "1"))).unwrap();
    assert_eq!(wb.value("B2").unwrap(), Value::Number(1.0));
}
