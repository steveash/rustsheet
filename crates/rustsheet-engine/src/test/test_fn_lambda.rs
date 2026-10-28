#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_evaluation() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(x, y, x + y)(1, 2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3");
}

#[test]
fn evaluation_in_let() {
    let mut model = new_empty_model();
    model._set("A1", "=LET(x, 1, y, LAMBDA(a, b, a + b), y(x, 22))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"23");
}

#[test]
fn evaluation_with_defined_name() {
    let mut model = new_empty_model();
    model
        .new_defined_name("MySum", None, "=LAMBDA(x, y, x + y)")
        .unwrap();
    model._set("A1", "=MySum(1, 2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3");
}

// Regression: a defined-name LAMBDA that returns an array must spill.
// Calling it (a `NamedFunctionKind`) was statically classified as a scalar,
// so the cell was never marked dynamic and the array result reached a
// scalar context instead of spilling.
#[test]
fn defined_name_lambda_spills_array() {
    let mut model = new_empty_model();
    model
        .new_defined_name("MySeq", None, "=LAMBDA(n, SEQUENCE(n))")
        .unwrap();
    model._set("A1", "=MySeq(3)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"2");
    assert_eq!(model._get_text("A3"), *"3");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(x, y, x + y)(1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
}

#[test]
fn returns_calculation_error() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(x, y, x + y)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#CALC!");
}

#[test]
fn sheet_local_name_takes_precedence_over_global() {
    // Global MyFn = LAMBDA(x, x * 2)
    // Sheet2-local MyFn = LAMBDA(x, x * 3)
    // Sheet1!A1 calls MyFn(5) => uses global => 10
    // Sheet2!A1 calls MyFn(5) => uses local  => 15
    let mut model = new_empty_model();
    model.new_sheet(); // adds Sheet2 at index 1

    model
        .new_defined_name("MyFn", None, "=LAMBDA(x, x * 2)")
        .unwrap();
    model
        .new_defined_name("MyFn", Some(1), "=LAMBDA(x, x * 3)")
        .unwrap();

    model._set("A1", "=MyFn(5)");
    model._set("Sheet2!A1", "=MyFn(5)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"10");
    assert_eq!(model._get_text("Sheet2!A1"), *"15");
}

#[test]
fn let_bound_variable_captured_in_lambda_body() {
    let mut model = new_empty_model();
    // `a` is bound by LET; the LAMBDA body references `a` as a closure variable.
    // f(2) should return 2 + 1 = 3.
    model._set("A1", "=LET(a, 1, f, LAMBDA(x, x + a), f(2))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"3");
}

#[test]
fn optional_parameter_omitted() {
    let mut model = new_empty_model();
    // b is optional; calling with one arg should use default (a/2 = 6).
    model._set("A1", "=LAMBDA(a, [b], IF(ISOMITTED(b), a/2, a*b))(12)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"6");
}

#[test]
fn optional_parameter_provided() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(a, [b], IF(ISOMITTED(b), a/2, a*b))(12, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"36");
}

#[test]
fn optional_parameter_too_many_args_is_error() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(a, [b], a+1)(1, 2, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
}

#[test]
fn required_arg_missing_is_error() {
    let mut model = new_empty_model();
    // `a` is required; calling with no args is an error.
    model._set("A1", "=LAMBDA(a, [b], a+1)()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
}

// A reference given to a LAMBDA, or bound by a LET, stays a reference, also
// when it is a single cell. Where a value is wanted it is the cell's value.
#[test]
fn a_single_cell_is_passed_as_a_reference() {
    let mut model = new_empty_model();
    model._set("A1", "text");
    model._set("A2", "7");
    model._set("B5", "3");
    model._set("C1", "=LAMBDA(x, MAX(x))(A1)");
    model._set("C2", "=LAMBDA(x, MAX(x))(A2)");
    model._set("C3", "=LAMBDA(x, ROW(x))(B5)");
    model._set("C4", "=LAMBDA(x, COLUMN(x))(B5)");
    model._set("C5", "=LAMBDA(x, OFFSET(x, 1, 0))(A1)");
    model._set("C6", "=LAMBDA(x, ROWS(x))(B5)");
    model._set("C9", "=LET(x, A1, MAX(x))");
    model._set("C10", "=LET(x, B5, ROW(x))");
    model._set("C11", "=LET(x, A2, y, B5, OFFSET(x, 0, 0)+y)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"0");
    assert_eq!(model._get_text("C2"), *"7");
    assert_eq!(model._get_text("C3"), *"5");
    assert_eq!(model._get_text("C4"), *"2");
    assert_eq!(model._get_text("C5"), *"7");
    assert_eq!(model._get_text("C6"), *"1");
    assert_eq!(model._get_text("C9"), *"0");
    assert_eq!(model._get_text("C10"), *"5");
    assert_eq!(model._get_text("C11"), *"10");
}

#[test]
fn a_single_cell_parameter_is_a_value_where_a_value_is_wanted() {
    let mut model = new_empty_model();
    model._set("A1", "text");
    model._set("A2", "7");
    model._set("C1", "=LAMBDA(x, x)(A1)");
    model._set("C2", "=LAMBDA(x, x)(A2)");
    model._set("C3", "=LAMBDA(x, x&\"!\")(A1)");
    model._set("C4", "=LAMBDA(x, x+1)(A2)");
    model._set("C5", "=LAMBDA(x, IF(x=\"text\", 1, 2))(A1)");
    model._set("C6", "=LAMBDA(x, ISBLANK(x))(A3)");
    model._set("C7", "=LAMBDA(x, ISTEXT(x))(A1)");
    model._set("C8", "=LAMBDA(x, LEN(x))(A1)");
    model._set("C9", "=LET(x, A2, x*2)");
    model._set("C10", "=LAMBDA(x, TYPE(x))(A2)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"text");
    assert_eq!(model._get_text("C2"), *"7");
    assert_eq!(model._get_text("C3"), *"text!");
    assert_eq!(model._get_text("C4"), *"8");
    assert_eq!(model._get_text("C5"), *"1");
    assert_eq!(model._get_text("C6"), *"TRUE");
    assert_eq!(model._get_text("C7"), *"TRUE");
    assert_eq!(model._get_text("C8"), *"4");
    assert_eq!(model._get_text("C9"), *"14");
    assert_eq!(model._get_text("C10"), *"1");
    // Nothing spills from a single cell
    assert_eq!(model._get_text("D1"), *"");
    assert_eq!(model._get_text("C11"), *"");
}

// A defined name that stands for a cell is a reference wherever one is
// wanted, and the cell's value wherever a value is wanted
#[test]
fn a_defined_name_for_a_single_cell_is_a_reference() {
    let mut model = new_empty_model();
    model._set("K1", "text");
    model._set("K2", "7");
    model
        .new_defined_name("TextCell", None, "Sheet1!$K$1")
        .unwrap();
    model
        .new_defined_name("Later", None, "=LAMBDA(x, x+1)")
        .unwrap();
    model._set("A1", "=ROW(TextCell)");
    model._set("A2", "=COLUMN(TextCell)");
    model._set("A3", "=OFFSET(TextCell, 1, 0)");
    model._set("A4", "=ROWS(TextCell)");
    model._set("A5", "=TextCell&\"!\"");
    model._set("A6", "=LEN(TextCell)");
    model._set("A7", "=IF(TextCell=\"text\", 1, 2)");
    model._set("A8", "=TextCell");
    model._set("A9", "=Later(1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"11");
    assert_eq!(model._get_text("A3"), *"7");
    assert_eq!(model._get_text("A4"), *"1");
    assert_eq!(model._get_text("A5"), *"text!");
    assert_eq!(model._get_text("A6"), *"4");
    assert_eq!(model._get_text("A7"), *"1");
    assert_eq!(model._get_text("A8"), *"text");
    assert_eq!(model._get_text("A9"), *"2");
}

// A called LET name or LAMBDA parameter that holds a value: a number or boolean is #NAME?, text
// is 0, an error is itself, an array is called per element and a reference (INDEX's too) is
// #REF! (probes/let-name-call*, lambda-param-call; T-1092).
#[test]
fn called_variable_holding_a_value() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("C1", "=LET(f, 1, f(2))");
    model._set("C2", "=LET(f, \"a\", f(2))");
    model._set("C3", "=LET(f, 1/0, f(2))");
    model._set("C4", "=LET(f, INDEX(A1:A2, 2), f(2))");
    model._set("C5", "=LAMBDA(f, f(2))(TRUE)");
    model._set("C6", "=LET(f, {1;\"a\"}, f(2))");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"#NAME?");
    assert_eq!(model._get_text("C2"), *"0");
    assert_eq!(model._get_text("C3"), *"#DIV/0!");
    assert_eq!(model._get_text("C4"), *"#REF!");
    assert_eq!(model._get_text("C5"), *"#NAME?");
    assert_eq!(model._get_text("C6"), *"#NAME?");
    // C7 is C6's spill.
    assert_eq!(model._get_text("C7"), *"0");
}

#[test]
fn duplicate_parameters_are_refused_in_any_case() {
    // Excel for the web refuses `LAMBDA(x,x,x)(1,2)`, `LAMBDA(x,X,x+X)(1,2)` and
    // `LAMBDA(ä,Ä,ä+Ä)(1,2)` (docs/excel-semantics/references.md, T-1387).
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(x,x,x)(1,2)");
    model._set("A2", "=LAMBDA(x,X,x+X)(1,2)");
    model._set("A3", "=LAMBDA(ä,Ä,ä+Ä)(1,2)");
    model._set("A4", "=LAMBDA(x,[X],x)(1)");
    model._set("A5", "=LAMBDA(ß,ß*2)(3)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), *"#ERROR!", "{cell}");
    }
    assert_eq!(model._get_text("A5"), *"6");
}

// Messages are not Excel-visible; the value stays `#NAME?`.
#[test]
fn calling_a_number_or_boolean_variable_is_name_with_a_message() {
    let mut model = new_empty_model();
    model._set("A1", "=LET(f, 1, f(2))");
    model._set("A2", "=LET(f, TRUE, f(2))");
    model.evaluate();

    for cell in ["A1", "A2"] {
        assert_eq!(model._get_text(cell), *"#NAME?");
        assert_eq!(model._get_error_message(cell), "A value cannot be called");
    }
}

#[test]
fn a_call_of_a_call_is_chained() {
    // probes/lambda-escaping-closures H8 and H10 (excel-graph): a LAMBDA returned by a call is
    // called by the next `(...)` (T-2038).
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(a,LAMBDA(b,a*b))(3)(4)");
    model._set("A2", "=LAMBDA(a,LAMBDA(b,LAMBDA(c,a*100+b*10+c)))(1)(2)(3)");
    model._set("A4", "=(LAMBDA(a,LAMBDA(b,a&b)))(\"x\")(\"y\")");
    model._set("A3", "=LET(f,LAMBDA(a,LAMBDA(b,a-b)),f(10)(4))");
    model._set("A5", "=LAMBDA(x,x)(1)(2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "12");
    assert_eq!(model._get_text("A2"), "123");
    assert_eq!(model._get_text("A4"), "xy");
    assert_eq!(model._get_text("A3"), "6");
    // A returned value is called like a LET-bound one (probes/chained-calls-lambda B4).
    assert_eq!(model._get_text("A5"), "#NAME?");
    model._set("A6", "=SUM(LAMBDA(x,x)(1)(2))");
    model._set("A7", "=LAMBDA(x,x)(\"a\")(2)");
    model.evaluate();
    assert_eq!(model._get_text("A6"), "#NAME?");
    assert_eq!(model._get_text("A7"), "0");
    // Excel for the web refuses a workbook holding a call followed by a space and `(4)`, or a
    // call of SUM's result (probes T-2038, docs/excel-semantics/references.md "Calling a call's
    // result"): the engine reads them as syntax errors.
    for (cell, formula) in [
        ("B1", "=LAMBDA(a,LAMBDA(b,a*b))(3) (4)"),
        ("B2", "=SUM(1)(2)"),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#ERROR!");
    assert_eq!(model._get_text("B2"), "#ERROR!");
    // The text round-trips.
    assert_eq!(model._get_formula("A1"), "=LAMBDA(a,LAMBDA(b,a*b))(3)(4)");
    assert_eq!(
        model._get_formula("A2"),
        "=LAMBDA(a,LAMBDA(b,LAMBDA(c,a*100+b*10+c)))(1)(2)(3)"
    );
    assert_eq!(
        model._get_formula("A3"),
        "=LET(f,LAMBDA(a,LAMBDA(b,a-b)),f(10)(4))"
    );
    assert_eq!(
        model._get_formula("A4"),
        "=(LAMBDA(a,LAMBDA(b,a&b)))(\"x\")(\"y\")"
    );
}

#[test]
fn a_call_of_a_reference_function_result_is_chained() {
    // probes/chained-calls-builtin-* and chained-calls-ref-4, -6 (excel-graph, T-2405, T-2499):
    // the result of a built-in that can return a reference (or a LAMBDA) may be called; Excel
    // refuses the others (`SUM(1)(2)`, `ABS(1)(2)`, `IFERROR`, `IFNA`), which stay syntax errors.
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3")] {
        model._set(cell, value);
    }
    let cases = [
        ("=IF(TRUE,LAMBDA(x,x*2))(3)", "6"),
        ("=CHOOSE(1,LAMBDA(x,x*2))(3)", "6"),
        ("=IFS(TRUE,LAMBDA(x,x*2))(3)", "6"),
        ("=SWITCH(1,1,LAMBDA(x,x*2))(3)", "6"),
        ("=LET(f,LAMBDA(x,x*2),f)(3)", "6"),
        ("=INDEX(A1:A3,1)(2)", "#REF!"),
        ("=OFFSET(A1,0,0)(2)", "#REF!"),
        ("=INDIRECT(\"A1\")(2)", "#REF!"),
        ("=CHOOSE(1,A1:A3)(2)", "#REF!"),
        // A called TAKE, DROP, TRIMRANGE or XLOOKUP over a range is the range it picks (T-2499).
        ("=TAKE(A1:A3,1)(2)", "#REF!"),
        ("=DROP(A1:A3,1)(2)", "#REF!"),
        ("=TRIMRANGE(A1:A3)(2)", "#REF!"),
        ("=XLOOKUP(1,A1:A3,A1:A3)(2)", "#REF!"),
        ("=INDEX({1,2},1)(2)", "#NAME?"),
        ("=SUM(1)(2)", "#ERROR!"),
        ("=ABS(1)(2)", "#ERROR!"),
        ("=IFERROR(LAMBDA(x,x*2),0)(3)", "#ERROR!"),
        ("=IFNA(LAMBDA(x,x*2),0)(3)", "#ERROR!"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("C{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, value)) in cases.iter().enumerate() {
        let cell = format!("C{}", row + 1);
        assert_eq!(model._get_text(&cell), *value, "{formula}");
        if *value != "#ERROR!" {
            // FORMULATEXT prints the callee bare (probes/chained-calls-builtin-if-text C1).
            assert_eq!(model._get_formula(&cell), *formula);
        }
    }
}

#[test]
fn a_long_call_chain_is_bounded_like_nesting() {
    // Each chained call is a level of the tree, so a chain past the depth limit is refused
    // rather than overflowing the stack (CLAUDE.md directive 5).
    let mut model = new_empty_model();
    let chain = "(1)".repeat(10_000);
    let refused = model.set_user_input(0, 1, 1, format!("=LAMBDA(x,x){chain}"));
    assert!(refused.unwrap_err().contains("nested too deeply"));
    let short = "(1)".repeat(20);
    model._set("A2", &format!("=LAMBDA(x,x){short}"));
    model.evaluate();
    assert_eq!(model._get_formula("A2"), format!("=LAMBDA(x,x){short}"));
}

// T-2925: no LAMBDA outlives the formula that made it, so a root cell's LAMBDAs are dropped once it
// is evaluated, and a later formula's calls do not slow down with what earlier ones made.
#[test]
fn a_root_cells_lambdas_are_dropped_once_it_is_evaluated() {
    let mut model = new_empty_model();
    model
        .new_defined_name("AddOne", None, "=LAMBDA(x, x + 1)")
        .unwrap();
    model._set(
        "A1",
        "=SUM(MAP(SEQUENCE(10),LAMBDA(z,MAP(z,LAMBDA(y,y+z)))))",
    );
    model._set(
        "A2",
        "=A1+SUM(MAP(SEQUENCE(5),LAMBDA(z,AddOne(z))))+REDUCE(0,{1,2},_xleta.SUM)",
    );
    model._set("A3", "=LET(f,LAMBDA(x,LAMBDA(y,x*y)),f(3)(4))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"110");
    assert_eq!(model._get_text("A2"), *"133");
    assert_eq!(model._get_text("A3"), *"12");
    assert!(model.lambdas.is_empty());
    assert!(model.eta_lambdas.is_empty());
    assert!(model.lambda_captures.is_empty());
    // Ids keep growing across root cells: a patched body's mark compares them.
    assert!(model.last_lambda_id > 0);

    // A second evaluation gives the same values from a model with no LAMBDA left over.
    model.evaluate();
    assert_eq!(model._get_text("A2"), *"133");
    assert!(model.lambdas.is_empty());
}
