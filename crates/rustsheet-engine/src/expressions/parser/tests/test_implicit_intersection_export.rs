use std::collections::HashMap;

use crate::expressions::{
    parser::{
        stringify::{to_excel_array_formula_string, to_excel_string},
        tests::utils::{new_parser, to_english_localized_string},
    },
    types::CellReferenceRC,
};

use crate::expressions::parser::static_analysis::add_implicit_intersection;

// Round-trips a formula the way an xlsx import/export does:
//   1. parse the Excel formula
//   2. add the automatic implicit intersection operators (the import step)
//   3. check the internal representation (where every II is shown as `@`)
//   4. stringify back to Excel and check we get `expected_excel`
//
// The interesting bit is step 4: an explicit `@` (`_xlfn.SINGLE`) sitting in a
// *vector* argument (GCD, LCM, SUM, ...) must be preserved, because on the next
// import nothing would re-insert it. An `@` in a *scalar* context (SIN, the
// formula root, ...) is dropped, because the import re-adds it automatically.
fn check(cases: &[(&str, &str, &str)]) {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Reference cell is Sheet1!A1
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };

    for (excel_input, internal, expected_excel) in cases {
        let mut t = parser.parse(excel_input, &cell_reference);
        add_implicit_intersection(&mut t, true);

        let r = to_english_localized_string(&t, &cell_reference);
        assert_eq!(&r, internal, "internal form of `{excel_input}`");

        let excel_formula = to_excel_string(&t, &cell_reference);
        assert_eq!(
            &excel_formula, expected_excel,
            "Excel export of `{excel_input}`"
        );
    }
}

// GCD/LCM accept ranges (vector arguments), so an explicit `@` on a range argument
// is meaningful and must survive the export. This is the regression behind the
// LCM_GCD_minimal calc test: dropping `_xlfn.SINGLE` turned `@J:J` into the whole
// column, making LCM collapse to 0 and GCD to 1.
#[test]
fn gcd_lcm_keep_explicit_intersection() {
    check(&[
        (
            "GCD(A1,B1,_xlfn.SINGLE(J1:J10))",
            "GCD(A1,B1,@J1:J10)",
            "GCD(A1,B1,_xlfn.SINGLE(J1:J10))",
        ),
        (
            "LCM(A1,B1,_xlfn.SINGLE(J1:J10))",
            "LCM(A1,B1,@J1:J10)",
            "LCM(A1,B1,_xlfn.SINGLE(J1:J10))",
        ),
        // Full-column form, exactly as in the failing workbook.
        (
            "LCM(A1,B1,_xlfn.SINGLE(J:J))",
            "LCM(A1,B1,@J:J)",
            "LCM(A1,B1,_xlfn.SINGLE(J:J))",
        ),
    ]);
}

// SUM is also a vector argument: the explicit operator must be preserved.
#[test]
fn sum_keeps_explicit_intersection() {
    check(&[
        (
            "SUM(_xlfn.SINGLE(A1:A10))",
            "SUM(@A1:A10)",
            "SUM(_xlfn.SINGLE(A1:A10))",
        ),
        (
            "SUM(A1:A10,_xlfn.SINGLE(B1:B10))",
            "SUM(A1:A10,@B1:B10)",
            "SUM(A1:A10,_xlfn.SINGLE(B1:B10))",
        ),
    ]);
}

// In a scalar context the operator is redundant: the import re-inserts it, so the
// export drops `_xlfn.SINGLE`. This is the behaviour we must NOT lose for vector
// arguments above.
#[test]
fn scalar_context_drops_redundant_intersection() {
    check(&[
        // SIN forces intersection on its argument anyway.
        ("SIN(_xlfn.SINGLE(A1:A10))", "SIN(@A1:A10)", "SIN(A1:A10)"),
        // The formula root is a scalar context.
        ("_xlfn.SINGLE(A1:A10)", "@A1:A10", "A1:A10"),
    ]);
}

// A `@` on something that is already a single cell is kept (Excel would not
// re-add it, and `add_implicit_intersection` does not either).
#[test]
fn explicit_intersection_on_single_cell_is_kept() {
    check(&[
        ("_xlfn.SINGLE(A1)", "@A1", "_xlfn.SINGLE(A1)"),
        ("SUM(_xlfn.SINGLE(A1))", "SUM(@A1)", "SUM(_xlfn.SINGLE(A1))"),
    ]);
}

// The same range `@J:J` is exported differently depending on the surrounding
// function: kept inside the vector argument of LCM, dropped inside the scalar
// argument of SIN.
#[test]
fn same_range_depends_on_context() {
    check(&[
        (
            "LCM(_xlfn.SINGLE(J:J))",
            "LCM(@J:J)",
            "LCM(_xlfn.SINGLE(J:J))",
        ),
        ("SIN(_xlfn.SINGLE(J:J))", "SIN(@J:J)", "SIN(J:J)"),
    ]);
}

// SUMIF's criterion is a scalar argument: a legacy formula intersects a multi-cell
// criterion (T-0278, fixtures/ifs-legacy-range-criteria), so the import adds the automatic
// `@`. It is redundant there, so the export drops it again.
#[test]
fn sumif_unique() {
    let test_str = r#"LARGE(SUMIF(E2:E7,@UNIQUE(FILTER(E2:E7,E2:E7<>""))),B27)"#;
    let test_excel = r#"LARGE(SUMIF(E2:E7,_xlfn.UNIQUE(_xlfn._xlws.FILTER(E2:E7,E2:E7<>""))),B27)"#;
    check(&[(test_excel, test_str, test_excel)]);
}

// Nested redundant operators must collapse completely. After dropping the outer
// `@` in a scalar context the child is still in that same scalar context, so the
// cleanup must continue there: `_xlfn.SINGLE(_xlfn.SINGLE(A1:A10))` has to export
// as `A1:A10`, not `_xlfn.SINGLE(A1:A10)`.
//
// Note: this parses the formula directly (without `add_implicit_intersection`,
// which collapses double-`@` on its own) to exercise `to_excel_string`'s own
// normalization on a nested tree.
#[test]
fn nested_redundant_intersections_are_fully_removed() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let t = parser.parse("_xlfn.SINGLE(_xlfn.SINGLE(A1:A10))", &cell_reference);
    let excel_formula = to_excel_string(&t, &cell_reference);
    assert_eq!(excel_formula, "A1:A10");
}

// The formula of an array, CSE or dynamic, is read as it is written: no
// implicit intersection is added to it. So none is redundant there, and they
// are all exported. As the formula of a normal cell the same `@` would be
// dropped, to be added again when read.
#[test]
fn array_formulas_keep_every_intersection() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    // (the formula in Excel, in a normal cell, in an array)
    let cases = [
        (
            "POWER(_xlfn.SINGLE(A1:A3),3)",
            "POWER(A1:A3,3)",
            "POWER(_xlfn.SINGLE(A1:A3),3)",
        ),
        (
            "SIN(_xlfn.SINGLE(A1:A3))+B1:B3",
            "SIN(A1:A3)+B1:B3",
            "SIN(_xlfn.SINGLE(A1:A3))+B1:B3",
        ),
        ("_xlfn.SINGLE(A1:A3)", "A1:A3", "_xlfn.SINGLE(A1:A3)"),
        (
            "SUM(_xlfn.SINGLE(A1:A3))",
            "SUM(_xlfn.SINGLE(A1:A3))",
            "SUM(_xlfn.SINGLE(A1:A3))",
        ),
        // Nothing is added to a formula that has none
        ("POWER(A1:A3,3)", "POWER(A1:A3,3)", "POWER(A1:A3,3)"),
        ("A1:A3*2", "A1:A3*2", "A1:A3*2"),
    ];
    for (excel_input, normal_cell, array) in cases {
        let t = parser.parse(excel_input, &cell_reference);
        assert_eq!(
            to_excel_string(&t, &cell_reference),
            normal_cell,
            "`{excel_input}` in a normal cell"
        );
        assert_eq!(
            to_excel_array_formula_string(&t, &cell_reference),
            array,
            "`{excel_input}` in an array"
        );
    }
}

// Fixed-length signatures (the D-functions, FREQUENCY, SMALL, ...) used to be indexed past
// their end when a formula passed too many arguments, panicking on import and export (found by
// fuzzing, T-0021). The extra arguments are simply not scalar positions.
#[test]
fn too_many_arguments_for_a_fixed_signature_do_not_panic() {
    for formula in [
        "DMIN(A1:C4,C1,,C1,A6:A7)",
        "DMIN(A1:C4,C1,,C1,A6:A6:A7)",
        "SMALL(A1:A3,1,B1:B2)",
    ] {
        check(&[(formula, formula, formula)]);
    }
}

// SUMPRODUCT's arguments are an array context in a legacy formula: the import adds no `@` inside
// (a range criterion lifts, probes/ifs-legacy-sumproduct, T-0278), and an explicit one there is
// never redundant, however deep.
#[test]
fn sumproduct_is_an_array_context() {
    check(&[
        (
            "SUMPRODUCT(COUNTIF(A1:A6,D1:D2))",
            "SUMPRODUCT(COUNTIF(A1:A6,D1:D2))",
            "SUMPRODUCT(COUNTIF(A1:A6,D1:D2))",
        ),
        (
            "SUMPRODUCT(ABS(_xlfn.SINGLE(A1:A2)))",
            "SUMPRODUCT(ABS(@A1:A2))",
            "SUMPRODUCT(ABS(_xlfn.SINGLE(A1:A2)))",
        ),
        (
            "SUM(COUNTIF(A1:A6,D1:D2))",
            "SUM(COUNTIF(A1:A6,@D1:D2))",
            "SUM(COUNTIF(A1:A6,D1:D2))",
        ),
    ]);
}

// Excel stores every written parenthesis in the `<f>` text, and an `@` with a written group as
// `_xlfn.SINGLE(` + the operand with its group + `)` (excel-graph, T-1244; SPEC-0007 "Excel
// behavior to match"; docs/excel-semantics/xlsx-storage.md). Export → import → print is the
// identity (T-1343).
#[test]
fn written_parentheses_are_exported_as_excel_stores_them() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let cases = [
        ("Z1+(Z2)", "Z1+(Z2)"),
        ("((Z1))", "((Z1))"),
        ("-(Z1)", "-(Z1)"),
        ("IF((Z1>0),(Z2),Z3)", "IF((Z1>0),(Z2),Z3)"),
        ("(Z1^2)^3", "(Z1^2)^3"),
        ("SUM((Z1:Z2))", "SUM((Z1:Z2))"),
        ("(1)+((2))", "(1)+((2))"),
        ("@(Z1+(Z2))", "_xlfn.SINGLE((Z1+(Z2)))"),
        ("@(Z1:Z3)", "_xlfn.SINGLE((Z1:Z3))"),
        ("@((Z1:Z3))", "_xlfn.SINGLE(((Z1:Z3)))"),
        ("SIN(@(Z1:Z3))", "SIN(_xlfn.SINGLE((Z1:Z3)))"),
        ("SIN(@((Z1:Z3)))", "SIN(_xlfn.SINGLE(((Z1:Z3))))"),
    ];
    for (typed, stored) in cases {
        let t = parser.parse(typed, &cell_reference);
        assert_eq!(
            to_excel_string(&t, &cell_reference),
            stored,
            "export of `{typed}`"
        );
        let mut imported = parser.parse(stored, &cell_reference);
        add_implicit_intersection(&mut imported, true);
        assert_eq!(
            to_english_localized_string(&imported, &cell_reference),
            typed,
            "import of `{stored}`"
        );
    }
}

// A legacy formula intersects the ranges in arithmetic under a scalar argument, in an
// aggregate's arithmetic argument, TYPE's argument, a range of a linked workbook, and IF's
// condition and IFERROR's value under SUMPRODUCT; those `@`s are automatic, so the export drops
// them (probes/legacy-value-argument-intersection, legacy-type-intersection,
// array-context-legacy-sumproduct; fixtures/external-link-cached-ranges; T-1526).
#[test]
fn legacy_value_arguments_intersect_and_export_unchanged() {
    check(&[
        (
            "SUM(IFERROR(B1:B6*1,100))",
            "SUM(IFERROR(@B1:B6*1,100))",
            "SUM(IFERROR(B1:B6*1,100))",
        ),
        ("SUM(B1:B6+0)", "SUM(@B1:B6+0)", "SUM(B1:B6+0)"),
        ("SUM(-B1:B6)", "SUM(-@B1:B6)", "SUM(-B1:B6)"),
        (
            "SUM(B1:B6,(B1:B6))",
            "SUM(B1:B6,(B1:B6))",
            "SUM(B1:B6,(B1:B6))",
        ),
        ("TYPE(B1:B6)", "TYPE(@B1:B6)", "TYPE(B1:B6)"),
        (
            "SUMPRODUCT(IF(B1:B6>1,B1:B6,0))",
            "SUMPRODUCT(IF(@B1:B6>1,B1:B6,0))",
            "SUMPRODUCT(IF(B1:B6>1,B1:B6,0))",
        ),
        (
            "SUMPRODUCT(ABS(IFERROR(B1:B6,100)))",
            "SUMPRODUCT(ABS(IFERROR(@B1:B6,100)))",
            "SUMPRODUCT(ABS(IFERROR(B1:B6,100)))",
        ),
        (
            "SUMPRODUCT(ABS(B1:B6-2))",
            "SUMPRODUCT(ABS(B1:B6-2))",
            "SUMPRODUCT(ABS(B1:B6-2))",
        ),
        (
            "SUM(MMULT(B1:B6*1,{1}))",
            "SUM(MMULT(B1:B6*1,{1}))",
            "SUM(MMULT(B1:B6*1,{1}))",
        ),
        (
            "SUM([1]Uses!$A$2:$A$3+0)",
            "SUM(@'[1]Uses'!$A$2:$A$3+0)",
            "SUM('[1]Uses'!$A$2:$A$3+0)",
        ),
    ]);
}

// A legacy CHOOSE with range arms is reference-shaped, so the import wraps the call in `@` (Excel
// upgrades `=CHOOSE(2,A1:A10,B1:B10)` to `=@CHOOSE(...)`), with Range extents from its arms' counts
// (T-1265).
#[test]
fn legacy_choose_of_ranges_is_intersected_and_exports_unchanged() {
    check(&[
        (
            "CHOOSE(2,A1:A10,B1:B10)",
            "@CHOOSE(2,A1:A10,B1:B10)",
            "CHOOSE(2,A1:A10,B1:B10)",
        ),
        ("CHOOSE(1,A1,B1)", "CHOOSE(1,A1,B1)", "CHOOSE(1,A1,B1)"),
    ]);
}

// A one-cell range and a 1x1 array are not scalars: Excel shows the legacy `=ABS(@E1:E1+1)`,
// `=ABS(@CHOOSE(1,{5}))`, `=AGGREGATE(@CHOOSE(1,{9}),0,E1:E6)`, `=CHOOSE(1,@E1:E1+1)` and
// `=ABS(@E1:E1)`, so these are dynamic formulas; CHOOSE of a single cell stays a scalar
// (probes/legacy-one-cell-shape-intersection, T-1265).
#[test]
fn one_cell_shapes_are_not_scalar() {
    use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    for (formula, scalar) in [
        ("ABS(E1:E1+1)", false),
        ("ABS(CHOOSE(1,{5}))", false),
        ("AGGREGATE(CHOOSE(1,{9}),0,E1:E6)", false),
        ("CHOOSE(1,E1:E1+1)", false),
        ("ABS(E1:E1)", false),
        ("ABS({5})", false),
        ("AGGREGATE(CHOOSE(1,G1),0,E1:E6)", true),
        ("ABS(E1+1)", true),
    ] {
        let node = parser.parse(formula, &cell_reference);
        let result = run_static_analysis_on_node(&node);
        assert_eq!(matches!(result, StaticResult::Scalar), scalar, "{formula}");
    }
}

// A legacy formula reads an array constant in INDEX's indices as its first element, under
// SUMPRODUCT and in arithmetic too, while a function call there keeps its own rules and INDEX's
// first argument keeps SUMPRODUCT's array context; the `@`s
// are automatic, so the export drops them (probes/legacy-index-array-index, T-0294).
#[test]
fn legacy_index_array_indices_intersect_and_export_unchanged() {
    check(&[
        (
            "SUMPRODUCT(INDEX(B1:B6,{1;2;3}))",
            "SUMPRODUCT(INDEX(B1:B6,@{1;2;3}))",
            "SUMPRODUCT(INDEX(B1:B6,{1;2;3}))",
        ),
        (
            "SUM(INDEX(B1:B6,{2;3}+0))",
            "SUM(INDEX(B1:B6,@{2;3}+0))",
            "SUM(INDEX(B1:B6,{2;3}+0))",
        ),
        (
            "SUMPRODUCT(INDEX(B1:B6,C1:C3))",
            "SUMPRODUCT(INDEX(B1:B6,@C1:C3))",
            "SUMPRODUCT(INDEX(B1:B6,C1:C3))",
        ),
        (
            "SUMPRODUCT(INDEX((B1:B6,B1:B6),1,1,{2;1}))",
            "SUMPRODUCT(INDEX((B1:B6,B1:B6),1,1,@{2;1}))",
            "SUMPRODUCT(INDEX((B1:B6,B1:B6),1,1,{2;1}))",
        ),
        (
            "SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))",
            "SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))",
            "SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))",
        ),
        // INDEX's first argument keeps SUMPRODUCT's array context
        (
            "SUMPRODUCT(INDEX(ABS(B1:B6),0))",
            "SUMPRODUCT(INDEX(ABS(B1:B6),0))",
            "SUMPRODUCT(INDEX(ABS(B1:B6),0))",
        ),
        (
            "SUMPRODUCT(INDEX(IF(B1:B6>1,B1:B6,0),{1;2}))",
            "SUMPRODUCT(INDEX(IF(@B1:B6>1,B1:B6,0),@{1;2}))",
            "SUMPRODUCT(INDEX(IF(B1:B6>1,B1:B6,0),{1;2}))",
        ),
    ]);
}

// SPEC-0011: the lifting table decides the legacy `@`. EDATE, EOMONTH, WEEKNUM and YEARFRAC are
// `LiftArrays`: a bare range keeps its text (Excel shows legacy `=EDATE(A1:A60,1)` and
// `=YEARFRAC(A1:A60,46000)` as typed), a value operation on the range gets the `@` of an
// aggregate's argument (`=EDATE(@A1:A60+0,1)`), and an explicit `@` is kept on export
// (probes/scalar-lift-shapes-c E-H, P-S; -shapes J2).
#[test]
fn lift_arrays_arguments_get_no_at_on_a_bare_range() {
    check(&[
        ("EDATE(A1:A60,1)", "EDATE(A1:A60,1)", "EDATE(A1:A60,1)"),
        (
            "YEARFRAC(A1:A60,46000)",
            "YEARFRAC(A1:A60,46000)",
            "YEARFRAC(A1:A60,46000)",
        ),
        (
            "EOMONTH(45000,B1:B60)",
            "EOMONTH(45000,B1:B60)",
            "EOMONTH(45000,B1:B60)",
        ),
        (
            "WEEKNUM(A1:A60,B1:B60)",
            "WEEKNUM(A1:A60,B1:B60)",
            "WEEKNUM(A1:A60,B1:B60)",
        ),
        (
            "EDATE(A1:A60+0,1)",
            "EDATE(@A1:A60+0,1)",
            "EDATE(A1:A60+0,1)",
        ),
        (
            "EDATE(45000,M1:M60*1)",
            "EDATE(45000,@M1:M60*1)",
            "EDATE(45000,M1:M60*1)",
        ),
        (
            "EDATE(_xlfn.SINGLE(A1:A60),1)",
            "EDATE(@A1:A60,1)",
            "EDATE(_xlfn.SINGLE(A1:A60),1)",
        ),
        // Under SUMPRODUCT nothing is intersected (-shapes J6, J7).
        (
            "SUMPRODUCT(EDATE(A1:A60,1))",
            "SUMPRODUCT(EDATE(A1:A60,1))",
            "SUMPRODUCT(EDATE(A1:A60,1))",
        ),
    ]);
}

// The static shape of a `LiftArrays` call: an array argument adds its shape, a bare range none
// (SPEC-0011 Test plan).
#[test]
fn lift_arrays_static_shapes_come_from_arrays_only() {
    use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let shape = |parser: &mut crate::expressions::parser::Parser, formula: &str| {
        run_static_analysis_on_node(&parser.parse(formula, &cell_reference))
    };
    assert!(matches!(
        shape(&mut parser, "EDATE({1;2},A1:A3)"),
        StaticResult::Array(2, 1)
    ));
    assert!(matches!(
        shape(&mut parser, "EDATE(A1:A3,1)"),
        StaticResult::Scalar
    ));
    assert!(matches!(
        shape(&mut parser, "YEARFRAC(1,{2,3,4},A1:A3)"),
        StaticResult::Array(1, 3)
    ));
    assert!(matches!(
        shape(&mut parser, "EDATE(A1:A3+0,1)"),
        StaticResult::Array(3, 1)
    ));
    assert!(matches!(
        shape(&mut parser, "EDATE(1)"),
        StaticResult::Scalar
    ));
}

// SPEC-0011 for the engineering functions (T-1383): the Analysis ToolPak ones are `LiftArrays`,
// so a legacy bare range keeps its text; BIT* are `Lift`, so a legacy range gets its `@`
// (docs/excel-semantics/engineering.md "Engineering function argument modes").
#[test]
fn engineering_legacy_ranges_follow_the_table() {
    check(&[
        (
            "BESSELJ(A1:A60,1)",
            "BESSELJ(A1:A60,1)",
            "BESSELJ(A1:A60,1)",
        ),
        (
            "DEC2BIN(9,B1:B60)",
            "DEC2BIN(9,B1:B60)",
            "DEC2BIN(9,B1:B60)",
        ),
        (
            "CONVERT(1,P1:P60,\"ft\")",
            "CONVERT(1,P1:P60,\"ft\")",
            "CONVERT(1,P1:P60,\"ft\")",
        ),
        ("IMABS(A1:A60+0)", "IMABS(@A1:A60+0)", "IMABS(A1:A60+0)"),
        // IMSUM and IMPRODUCT read their arguments whole (`WholeByCaller`, T-1384).
        (
            "IMSUM(A1:A60,B1:B60)",
            "IMSUM(A1:A60,B1:B60)",
            "IMSUM(A1:A60,B1:B60)",
        ),
        (
            "BITAND(A1:A60,1)",
            "BITAND(@A1:A60,1)",
            "_xlfn.BITAND(A1:A60,1)",
        ),
        (
            "BITLSHIFT(13,C1:C60)",
            "BITLSHIFT(13,@C1:C60)",
            "_xlfn.BITLSHIFT(13,C1:C60)",
        ),
    ]);
}

#[test]
fn engineering_static_shapes_follow_the_table() {
    use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let mut shape =
        |formula: &str| run_static_analysis_on_node(&parser.parse(formula, &cell_reference));
    assert!(matches!(
        shape("BITAND(A1:A3,1)"),
        StaticResult::Array(3, 1)
    ));
    assert!(matches!(
        shape("BITOR(13,{1,2})"),
        StaticResult::Array(1, 2)
    ));
    assert!(matches!(shape("BESSELJ(A1:A3,1)"), StaticResult::Scalar));
    assert!(matches!(
        shape("BESSELJ(2.5,{1;2})"),
        StaticResult::Array(2, 1)
    ));
    assert!(matches!(
        shape("CONVERT(1,\"m\",{\"ft\",\"in\"})"),
        StaticResult::Array(1, 2)
    ));
    assert!(matches!(shape("DELTA({1;2;3})"), StaticResult::Array(3, 1)));
    // IMSUM and IMPRODUCT fold their arguments into one value (T-1384).
    assert!(matches!(
        shape("IMSUM({\"1\";\"2\"},\"1-2i\")"),
        StaticResult::Scalar
    ));
    assert!(matches!(
        shape("IMPRODUCT(A1:A3,{1,2})"),
        StaticResult::Scalar
    ));
}

// SPEC-0011 for the financial functions (T-1372): a core worksheet function's argument is `Lift`,
// so a legacy range gets the `@` and exports without it (`=PMT(@C1:C60,10,100)`,
// probes/scalar-lift-shapes-c G, R; -shapes J1, J4); an Analysis ToolPak argument is
// `LiftArrays`, so a bare range keeps its text and an operation on it gets the `@`.
#[test]
fn financial_legacy_at_follows_the_lifting_table() {
    check(&[
        (
            "PMT(C1:C60,10,100)",
            "PMT(@C1:C60,10,100)",
            "PMT(C1:C60,10,100)",
        ),
        (
            "RATE(36,-300,8000,0,0,D1:D60)",
            "RATE(36,-300,8000,0,0,@D1:D60)",
            "RATE(36,-300,8000,0,0,D1:D60)",
        ),
        ("EFFECT(C1:C60,4)", "EFFECT(C1:C60,4)", "EFFECT(C1:C60,4)"),
        (
            "ACCRINTM(A1:A60,46000,0.1,1000,B1:B60)",
            "ACCRINTM(A1:A60,46000,0.1,1000,B1:B60)",
            "ACCRINTM(A1:A60,46000,0.1,1000,B1:B60)",
        ),
        (
            "TBILLPRICE(A1:A60+0,46000,0.05)",
            "TBILLPRICE(@A1:A60+0,46000,0.05)",
            "TBILLPRICE(A1:A60+0,46000,0.05)",
        ),
        (
            "SUMPRODUCT(PMT(C1:C60,10,100))",
            "SUMPRODUCT(PMT(C1:C60,10,100))",
            "SUMPRODUCT(PMT(C1:C60,10,100))",
        ),
    ]);
}

// The static shapes: a `Lift` range adds its shape, a `LiftArrays` range none.
#[test]
fn financial_static_shapes_come_from_the_lifting_table() {
    use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let mut shape =
        |formula: &str| run_static_analysis_on_node(&parser.parse(formula, &cell_reference));
    assert!(matches!(
        shape("PMT(C1:C3,10,100)"),
        StaticResult::Array(3, 1)
    ));
    assert!(matches!(
        shape("PMT({0.1,0.2},C1:C3,100)"),
        StaticResult::Array(3, 2)
    ));
    assert!(matches!(shape("PMT(0.1,10,100)"), StaticResult::Scalar));
    assert!(matches!(shape("EFFECT(C1:C3,4)"), StaticResult::Scalar));
    assert!(matches!(
        shape("PRICEDISC(39448,43831,{0.05;0.06},C1:C3)"),
        StaticResult::Array(2, 1)
    ));
}

// SPEC-0011 for the statistical functions whose every argument is scalar (T-1290): they are
// `Lift`, so a legacy range gets its `@` and the call's static shape is the lifted one
// (docs/excel-semantics/statistics.md "Statistical function argument modes").
#[test]
fn statistical_legacy_ranges_and_shapes_follow_the_table() {
    use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
    check(&[
        (
            "NORMDIST(A1:A60,0,1,TRUE)",
            "NORMDIST(@A1:A60,0,1,TRUE)",
            "NORMDIST(A1:A60,0,1,TRUE)",
        ),
        (
            "BETA.DIST(0.5,2,3,TRUE,0,B1:B60)",
            "BETA.DIST(0.5,2,3,TRUE,0,@B1:B60)",
            "_xlfn.BETA.DIST(0.5,2,3,TRUE,0,B1:B60)",
        ),
        ("PERMUT(5,C1:C60)", "PERMUT(5,@C1:C60)", "PERMUT(5,C1:C60)"),
    ]);
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let mut shape =
        |formula: &str| run_static_analysis_on_node(&parser.parse(formula, &cell_reference));
    assert!(matches!(
        shape("NORM.S.DIST(A1:A3,TRUE)"),
        StaticResult::Array(3, 1)
    ));
    assert!(matches!(
        shape("BINOM.INV(5,0.4,{0.4,0.9})"),
        StaticResult::Array(1, 2)
    ));
    assert!(matches!(shape("GAMMA(2.5)"), StaticResult::Scalar));
    // Not entered yet (T-1830): the old scalar shape.
    assert!(matches!(
        shape("CHISQ.DIST(A1:A3,2,TRUE)"),
        StaticResult::Scalar
    ));
}
