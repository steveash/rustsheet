use std::collections::HashMap;

use crate::expressions::{
    parser::{
        stringify::to_excel_string,
        tests::utils::{new_parser, to_english_localized_string},
    },
    types::CellReferenceRC,
};

use crate::expressions::parser::static_analysis::add_implicit_intersection;

#[test]
fn simple_test() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Reference cell is Sheet1!A1
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let cases = vec![
        ("A1:A10*SUM(A1:A10)", "@A1:A10*SUM(A1:A10)"),
        ("A1:A10", "@A1:A10"),
        // The anchor of a spill operator is not intersected (T-1109).
        ("_xlfn.ANCHORARRAY(INDEX(A1:A10,1))", "INDEX(A1:A10,1)#"),
        // Math and trigonometry functions
        ("SUM(A1:A10)", "SUM(A1:A10)"),
        ("SIN(A1:A10)", "SIN(@A1:A10)"),
        ("COS(A1:A10)", "COS(@A1:A10)"),
        ("TAN(A1:A10)", "TAN(@A1:A10)"),
        ("ASIN(A1:A10)", "ASIN(@A1:A10)"),
        ("ACOS(A1:A10)", "ACOS(@A1:A10)"),
        ("ATAN(A1:A10)", "ATAN(@A1:A10)"),
        ("SINH(A1:A10)", "SINH(@A1:A10)"),
        ("COSH(A1:A10)", "COSH(@A1:A10)"),
        ("TANH(A1:A10)", "TANH(@A1:A10)"),
        ("ASINH(A1:A10)", "ASINH(@A1:A10)"),
        ("ACOSH(A1:A10)", "ACOSH(@A1:A10)"),
        ("ATANH(A1:A10)", "ATANH(@A1:A10)"),
        ("ATAN2(A1:A10,B1:B10)", "ATAN2(@A1:A10,@B1:B10)"),
        ("ATAN2(A1:A10,A1)", "ATAN2(@A1:A10,A1)"),
        ("SQRT(A1:A10)", "SQRT(@A1:A10)"),
        // A one-sheet span is that sheet's reference: its range gets the `@` (T-2351).
        ("SIN(Sheet1:Sheet1!A1:A10)", "SIN(@Sheet1!A1:A10)"),
        ("SIN(Sheet1:Sheet1!A1)", "SIN(Sheet1!A1)"),
        // An Analysis ToolPak argument's bare range takes no `@`: legacy `SQRTPI(E1:E3)` and
        // `MROUND(E1:E3,2)` are #VALUE! on the range's rows (probes/date-math-lift-driver-edges K2-K3).
        ("SQRTPI(A1:A10)", "SQRTPI(A1:A10)"),
        ("MROUND(A1:A10,2)", "MROUND(A1:A10,2)"),
        // Legacy DATEDIF intersects its unit (K1) and DAY its date (K5).
        ("DATEDIF(A1,B1,C1:C10)", "DATEDIF(A1,B1,@C1:C10)"),
        ("DAY(A1:A10)", "DAY(@A1:A10)"),
        ("POWER(A1:A10,A1)", "POWER(@A1:A10,A1)"),
        ("POWER(A1:A10,B1:B10)", "POWER(@A1:A10,@B1:B10)"),
        ("MAX(A1:A10)", "MAX(A1:A10)"),
        ("MIN(A1:A10)", "MIN(A1:A10)"),
        ("ABS(A1:A10)", "ABS(@A1:A10)"),
        ("FALSE()", "FALSE()"),
        ("TRUE()", "TRUE()"),
        // Defined names
        ("BADNMAE", "@BADNMAE"),
        // Logical
        ("AND(A1:A10)", "AND(A1:A10)"),
        ("OR(A1:A10)", "OR(A1:A10)"),
        ("NOT(A1:A10)", "NOT(@A1:A10)"),
        // IF passes its range arms whole and takes the `@` itself (`=@IF(TRUE,B1:B6)`,
        // probes/legacy-if-arm-range-argument Q12, T-1935).
        ("IF(A1:A10,B1:B10,C1:C10)", "@IF(@A1:A10,B1:B10,C1:C10)"),
        // A single-cell INDEX arm shows no `@` at all; a whole-column INDEX arm still does
        // (probes/legacy-if-arm-other-references M12, T-2326).
        ("IF(TRUE,INDEX(B1:B6,3))", "IF(TRUE,INDEX(B1:B6,3))"),
        ("IF(TRUE,INDEX(B1:C6,0,1))", "@IF(TRUE,INDEX(B1:C6,0,1))"),
        // Any range shape with literal whole-number indexes, a literal arm and CHOOSE lose it; a
        // computed or non-integer index and IFS keep it (probes/t3134-legacy-if-index-arm-shapes).
        ("IF(TRUE,INDEX(B1:C6,3))", "IF(TRUE,INDEX(B1:C6,3))"),
        (
            "ABS(IF(TRUE,INDEX(B1:B6,3),5))",
            "ABS(IF(TRUE,INDEX(B1:B6,3),5))",
        ),
        ("CHOOSE(1,INDEX(B1:B6,3))", "CHOOSE(1,INDEX(B1:B6,3))"),
        ("IF(TRUE,INDEX(B1:B6,3.5))", "@IF(TRUE,INDEX(B1:B6,3.5))"),
        (
            "ABS(IF(TRUE,INDEX(B1:B6,1+2)))",
            "ABS(@IF(TRUE,INDEX(B1:B6,1+2)))",
        ),
        (
            "IF(TRUE,INDEX(B1:B6,3),B1:B6)",
            "@IF(TRUE,INDEX(B1:B6,3),B1:B6)",
        ),
        // A bare range condition (unprobed) is intersected first, so the call is still one cell; an
        // array one is not a scalar, so the call keeps its `@` (probes/t3134-legacy-if-index-arm-shapes I50).
        ("IF(B1:B6,INDEX(B1:B6,3))", "IF(@B1:B6,INDEX(B1:B6,3))"),
        (
            "IF({TRUE,FALSE},INDEX(B1:B6,3))",
            "@IF({TRUE,FALSE},INDEX(B1:B6,3))",
        ),
        // An array-constant CHOOSE index over one-cell arms stays whole and the `@` goes on the
        // call; range arms keep both (probes/t3134-legacy-if-index-arm-shapes I54, K54,
        // legacy-choose-array-index; T-3168).
        (
            "CHOOSE({1,2},INDEX(B1:B6,3),5)",
            "@CHOOSE({1,2},INDEX(B1:B6,3),5)",
        ),
        (
            "ABS(CHOOSE({1,2},INDEX(B1:B6,3),5))",
            "ABS(@CHOOSE({1,2},INDEX(B1:B6,3),5))",
        ),
        (
            "SUM(CHOOSE({1,2},INDEX(B1:B6,3),5))",
            "SUM(CHOOSE({1,2},INDEX(B1:B6,3),5))",
        ),
        ("CHOOSE({1,2},H1:H2,I1:I2)", "@CHOOSE(@{1,2},H1:H2,I1:I2)"),
        (
            "_xlfn.IFS(TRUE,INDEX(B1:B6,3))",
            "@IFS(TRUE,INDEX(B1:B6,3))",
        ),
        // CHOOSE value args pass references through: no `@` on the arms. In a
        // scalar position the whole call is wrapped instead; in a range
        // position (e.g. inside SUM) nothing is wrapped.
        ("CHOOSE(2,A1:A10,B1:B10)", "@CHOOSE(2,A1:A10,B1:B10)"),
        (
            "SUM(CHOOSE(2,A1:A10,B1:B10))",
            "SUM(CHOOSE(2,A1:A10,B1:B10))",
        ),
        ("CHOOSE(2,A1,B1)", "CHOOSE(2,A1,B1)"),
        // An operator arm gets the `@` on its range operand, not on the call (T-1633).
        ("CHOOSE(1,A1:A2+1)", "CHOOSE(1,@A1:A2+1)"),
        ("CHOOSE(1,A1:A1+1)", "CHOOSE(1,@A1:A1+1)"),
        // A one-element array constant in a scalar argument (T-1633).
        ("ABS({5})", "ABS(@{5})"),
        // TYPE reads an array whole (T-1638).
        ("TYPE({5})", "TYPE({5})"),
        // INDIRECT takes one text and gives a reference
        ("INDIRECT(A1:A10)", "@INDIRECT(@A1:A10)"),
        ("INDIRECT(A1:A10,FALSE)", "@INDIRECT(@A1:A10,FALSE)"),
        ("SUM(INDIRECT(A1,FALSE))", "SUM(INDIRECT(A1,FALSE))"),
        // ROW and COLUMN of a range are vectors: the whole call is wrapped in
        // a scalar position, and nothing is wrapped in a range position.
        ("ROW(A1:A10)", "@ROW(A1:A10)"),
        ("COLUMN(A1:J1)", "@COLUMN(A1:J1)"),
        ("ROW(A1:J1)", "ROW(A1:J1)"),
        ("COLUMN(A1:A10)", "COLUMN(A1:A10)"),
        ("ROW(A1)", "ROW(A1)"),
        ("COLUMN()", "COLUMN()"),
        ("SUM(ROW(A1:A10))", "SUM(ROW(A1:A10))"),
        ("SIN(ROW(A1:A10))", "SIN(@ROW(A1:A10))"),
        // `holidays` takes the whole range (poi/59106.xlsx Test!B2, docs/excel-semantics/dates.md).
        // The date and day-count arguments refuse a range with no intersection, in a legacy
        // formula too, so they get no `@`; the weekend lifts over a range and is a scalar
        // argument (arrays.md "Scalar arguments lift", SPEC-0011; T-1236).
        ("WORKDAY(A1:A10,6,B1:B4)", "WORKDAY(A1:A10,6,B1:B4)"),
        ("WORKDAY(A1,B1:B10)", "WORKDAY(A1,B1:B10)"),
        ("WORKDAY.INTL(A1,6,1,B1:B4)", "WORKDAY.INTL(A1,6,1,B1:B4)"),
        ("WORKDAY.INTL(A1,6,B1:B4)", "WORKDAY.INTL(A1,6,@B1:B4)"),
        ("NETWORKDAYS(A1,B1,C1:C4)", "NETWORKDAYS(A1,B1,C1:C4)"),
        (
            "NETWORKDAYS.INTL(A1,B1,C1:C4)",
            "NETWORKDAYS.INTL(A1,B1,@C1:C4)",
        ),
        // Information
        // ("ISBLANK(A1:A10)", "ISBLANK(A1:A10)"),
        // ("ISERR(A1:A10)", "ISERR(A1:A10)"),
        // ("ISERROR(A1:A10)", "ISERROR(A1:A10)"),
        ("ISEVEN(A1:A10)", "ISEVEN(A1:A10)"),
        // ("ISLOGICAL(A1:A10)", "ISLOGICAL(A1:A10)"),
        // ("ISNA(A1:A10)", "ISNA(A1:A10)"),
        // ("ISNONTEXT(A1:A10)", "ISNONTEXT(A1:A10)"),
        // ("ISNUMBER(A1:A10)", "ISNUMBER(A1:A10)"),
        ("ISODD(A1:A10)", "ISODD(A1:A10)"),
        // ("ISREF(A1:A10)", "ISREF(A1:A10)"),
        // ("ISTEXT(A1:A10)", "ISTEXT(A1:A10)"),
    ];
    for (formula, expected) in cases {
        let mut t = parser.parse(formula, &cell_reference);
        add_implicit_intersection(&mut t, true);
        let r = to_english_localized_string(&t, &cell_reference);
        assert_eq!(r, expected);
        let excel_formula = to_excel_string(&t, &cell_reference);
        // A one-sheet span is written as the plain reference (T-2357).
        assert_eq!(excel_formula, formula.replace("Sheet1:Sheet1!", "Sheet1!"));
    }
}

// T-2440: a defined name and a defined-name lambda's call take the `@`, an IF arm of a name
// stays whole, the dynamic-array-era functions and a LET-bound lambda's call remove every `@`,
// and TEXTBEFORE over scalars takes none (probes/legacy-at-defined-name-dynamic-funcs,
// legacy-at-dynamic-funcs-2).
#[test]
fn defined_names_and_dynamic_functions() {
    let worksheets = vec!["Sheet1".to_string()];
    let names = [
        ("swOne", "SWITCH(1,1,2)"),
        ("rngZ", "Sheet1!$Z$30:$Z$32"),
        ("myId", "LAMBDA(x,x)"),
    ]
    .map(|(name, body)| (name.to_string(), None, body.to_string()))
    .to_vec();
    let mut parser = new_parser(worksheets, names, HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let cases = [
        ("ABS(swOne)", "ABS(@swOne)"),
        ("ABS(rngZ)", "ABS(@rngZ)"),
        ("ABS(A1:A10)+swOne", "ABS(@A1:A10)+@swOne"),
        ("IF(TRUE,swOne)", "@IF(TRUE,swOne)"),
        ("SUM(IF(TRUE,rngZ))", "SUM(IF(TRUE,rngZ))"),
        ("ABS(A1:A10)+myId(1)", "ABS(@A1:A10)+@myId(1)"),
        ("ABS(A1:A10)+VSTACK(1)", "ABS(A1:A10)+VSTACK(1)"),
        (
            "ABS(A1:A10)+MAP({1},LAMBDA(x,x))",
            "ABS(A1:A10)+MAP({1},LAMBDA(x,x))",
        ),
        (
            "ABS(A1:A10)+LET(f,LAMBDA(x,x),f(1))",
            "ABS(A1:A10)+LET(f,LAMBDA(x,x),f(1))",
        ),
        (
            "ABS(A1:A10)+XMATCH(2,A1:A10)",
            "ABS(@A1:A10)+XMATCH(2,A1:A10)",
        ),
        (
            "LEN(TEXTBEFORE(\"ab\",\"b\"))",
            "LEN(TEXTBEFORE(\"ab\",\"b\"))",
        ),
        (
            "LEN(REGEXEXTRACT(\"a1\",\"[0-9]\"))",
            "LEN(@REGEXEXTRACT(\"a1\",\"[0-9]\"))",
        ),
    ];
    for (formula, expected) in cases {
        let mut t = parser.parse(formula, &cell_reference);
        add_implicit_intersection(&mut t, true);
        assert_eq!(
            to_english_localized_string(&t, &cell_reference),
            expected,
            "{formula}"
        );
    }
}

// T-2723: a unary plus is the identity, so a legacy formula gains the `@` it gains without it,
// a literal under a plus included (`IF(+TRUE,…)`, `CHOOSE(+1,…)`).
#[test]
fn a_unary_plus_changes_no_implicit_intersection() {
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let mut legacy = |formula: &str| {
        let mut t = parser.parse(formula, &cell_reference);
        add_implicit_intersection(&mut t, true);
        to_english_localized_string(&t, &cell_reference)
    };
    for (formula, plus) in [
        ("N(IF(TRUE,A1:A3))", "N(IF(+TRUE,A1:A3))"),
        ("N(CHOOSE(1,A1:A3,{2;3}))", "N(CHOOSE(+1,A1:A3,{2;3}))"),
        ("SIN(A1:A10)", "SIN(+A1:A10)"),
        ("SUM((A1:A10*1))", "SUM(+(A1:A10*1))"),
    ] {
        let expected = legacy(formula);
        let with_plus = legacy(plus);
        assert_eq!(
            with_plus.replace('+', ""),
            expected.replace('+', ""),
            "{plus}"
        );
    }
}

// T-3134 review: the single-cell IF test decides on the shape the analysis already has, so IFs
// nested in each other's condition stay linear (an extra pass over each condition doubled the work
// per level: depth 22 took 2 s).
#[test]
fn nested_if_conditions_stay_linear() {
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let depth = 60;
    let formula = format!(
        "{}A1:A2,1,1){}",
        "IF(".repeat(depth),
        ",1,1)".repeat(depth - 1)
    );
    let start = std::time::Instant::now();
    let mut t = parser.parse(&formula, &cell_reference);
    add_implicit_intersection(&mut t, true);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "{:?}",
        start.elapsed()
    );
}
