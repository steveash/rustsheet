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
        // `holidays` takes the whole range (poi/59106.xlsx Test!B2, docs/excel-semantics/dates.md);
        // the date, day-count and weekend arguments stay scalar.
        ("WORKDAY(A1:A10,6,B1:B4)", "WORKDAY(@A1:A10,6,B1:B4)"),
        ("WORKDAY(A1,B1:B10)", "WORKDAY(A1,@B1:B10)"),
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
        assert_eq!(excel_formula, formula);
    }
}
