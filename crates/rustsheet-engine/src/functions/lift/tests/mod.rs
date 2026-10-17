#![allow(clippy::unwrap_used, clippy::panic)]

//! Unit tests of the lifting driver (SPEC-0011 Test plan): the read phase and `lift_call` with
//! counting kernels, `lift_values`' bounds, and the first consumers (EDATE, EOMONTH, WEEKNUM,
//! YEARFRAC) through formulas. Excel values: probes/scalar-lift-shapes, -b and -c,
//! probes/date-argument-modes (docs/excel-semantics/arrays.md "Scalar arguments lift").

use super::driver::{LiftCost, LiftOptions, LiftedArg};
use super::kinds::{ArgKind, ArgKinds, KernelWork};
use crate::calc_result::CalcResult;
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
use crate::functions::Function;
use crate::limits::MAX_LIFTED_CELL_VISITS;
use crate::model::Model;
use crate::test::util::new_empty_model;

/// Where the calls under test run: Sheet1!Z1, outside every range they read.
const AT: CellReferenceIndex = CellReferenceIndex {
    sheet: 0,
    row: 1,
    column: 26,
};

/// A model with `cells` set and evaluated.
fn model_with(cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    for (cell, value) in cells {
        model._set(cell, value);
    }
    model.evaluate();
    model
}

/// The arguments of the call `formula` (no `=`), parsed at [`AT`].
fn args_of(model: &mut Model, formula: &str) -> Vec<Node> {
    let context = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: AT.row,
        column: AT.column,
    };
    match model.parser.parse(formula, &context) {
        Node::FunctionKind { args, .. } => args,
        other => panic!("{formula} is not a call: {other:?}"),
    }
}

/// `lift_call_with` over `formula`'s arguments with a kernel that counts its calls and returns
/// `kernel(values)`: the result and the number of calls.
fn run_counting(
    model: &mut Model,
    kinds: &ArgKinds,
    formula: &str,
    cost: LiftCost<'_>,
    kernel: impl Fn(&[CalcResult]) -> CalcResult,
) -> (CalcResult, usize) {
    let args = args_of(model, formula);
    let mut calls = 0;
    let mut counting = |_: &mut Model, values: &[CalcResult]| {
        calls += 1;
        kernel(values)
    };
    let result = model.lift_call_with(kinds, &args, cost, AT, &mut counting);
    (result, calls)
}

fn is_error(result: &CalcResult, expected: &Error) -> bool {
    matches!(result, CalcResult::Error { error, .. } if error == expected)
}

/// The rows of an array result as numbers, an error element as `NaN`.
fn numbers(result: &CalcResult) -> Vec<Vec<f64>> {
    let CalcResult::Array(rows) = result else {
        panic!("not an array");
    };
    rows.iter()
        .map(|row| {
            row.iter()
                .map(|node| match node {
                    ArrayNode::Number(n) => *n,
                    _ => f64::NAN,
                })
                .collect()
        })
        .collect()
}

fn first(values: &[CalcResult]) -> CalcResult {
    values.first().cloned().unwrap_or(CalcResult::EmptyArg)
}

/// Two `LiftArrays` arguments, as EDATE's entry.
fn edate_kinds() -> &'static ArgKinds {
    Function::Edate.arg_kinds().unwrap()
}

/// Any number of `Lift` arguments.
const LIFT_ALL: ArgKinds = ArgKinds {
    required: 1,
    params: &[],
    repeat: Some(ArgKind::Lift),
    max: 255,
    work: KernelWork::Constant,
};

/// A whole argument, then a lifted one.
const WHOLE_THEN_LIFT: ArgKinds = ArgKinds::fixed(
    2,
    &[ArgKind::Whole, ArgKind::LiftArrays],
    KernelWork::Constant,
);

// ---------------------------------------------------------------- the table

#[test]
fn the_table_reads_its_kinds_and_counts() {
    let weeknum = Function::Weeknum.arg_kinds().unwrap();
    assert!(!weeknum.accepts(0) && weeknum.accepts(1) && weeknum.accepts(2));
    assert!(!weeknum.accepts(3));
    assert_eq!(weeknum.kind(1), Some(ArgKind::LiftArrays));
    assert_eq!(weeknum.kind(2), None);
    assert_eq!(LIFT_ALL.kind(200), Some(ArgKind::Lift));
    for function in [Function::Edate, Function::Eomonth, Function::Yearfrac] {
        let kinds = function.arg_kinds().unwrap();
        assert_eq!(kinds.work, KernelWork::Constant, "{function:?}");
    }
    assert!(Function::Pmt.arg_kinds().is_none());
}

// ---------------------------------------------------------------- the unlifted path

#[test]
fn unlifted_calls_run_the_kernel_once_on_inline_values() {
    let mut model = model_with(&[("A1", "45000")]);
    let args = args_of(&mut model, "EDATE(A1,1)");
    let read = model
        .read_lift_args(edate_kinds(), &args, MAX_LIFTED_CELL_VISITS, AT)
        .ok()
        .unwrap();
    // No grid and no heap: the kernel gets the inline buffer as it is.
    assert!(read.grids.is_empty());
    assert!(read.values.is_inline());
    assert!(matches!(
        read.values.as_slice(),
        [CalcResult::Number(_), CalcResult::Number(_)]
    ));
    let (result, calls) = run_counting(
        &mut model,
        edate_kinds(),
        "EDATE(A1,1)",
        LiftCost::Unit,
        first,
    );
    assert_eq!(calls, 1);
    assert!(matches!(result, CalcResult::Number(n) if n == 45000.0));
}

#[test]
fn a_one_cell_range_is_its_value_and_does_not_lift() {
    let mut model = model_with(&[("A1", "45000"), ("B1", "=EDATE(A1:A1,1)")]);
    let args = args_of(&mut model, "EDATE(A1:A1,INDEX(A1:A3,1))");
    let read = model
        .read_lift_args(edate_kinds(), &args, MAX_LIFTED_CELL_VISITS, AT)
        .ok()
        .unwrap();
    assert!(read.grids.is_empty());
    assert!(
        matches!(read.values.as_slice(), [CalcResult::Number(a), CalcResult::Number(b)] if *a == 45000.0 && *b == 45000.0)
    );
    // probes/scalar-lift-shapes H4.
    assert_eq!(model._get_text("B1"), "45031");
}

#[test]
fn a_wrong_count_is_the_count_error_before_anything_is_read() {
    let mut model = model_with(&[]);
    let (result, calls) =
        run_counting(&mut model, edate_kinds(), "EDATE(1)", LiftCost::Unit, first);
    assert!(is_error(&result, &Error::ERROR));
    assert_eq!(calls, 0);
    let model = model_with(&[("A1", "=EDATE(1)"), ("A2", "=WEEKNUM(1,2,3)")]);
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

// ---------------------------------------------------------------- shapes

#[test]
fn a_row_and_a_column_give_a_grid() {
    let mut model = model_with(&[]);
    let (result, calls) = run_counting(
        &mut model,
        edate_kinds(),
        "EDATE({1;2},{10,20,30})",
        LiftCost::Unit,
        |values| match values {
            [CalcResult::Number(a), CalcResult::Number(b)] => CalcResult::Number(a + b),
            _ => CalcResult::EmptyArg,
        },
    );
    assert_eq!(calls, 6);
    assert_eq!(
        numbers(&result),
        vec![vec![11.0, 21.0, 31.0], vec![12.0, 22.0, 32.0]]
    );
}

#[test]
fn positions_past_a_shorter_array_are_na_in_1d_and_2d() {
    let model = model_with(&[
        // arrays.md: EDATE({45000;45031;45061},{1;2}) element 3 is #N/A.
        ("A1", "=INDEX(EDATE({45000;45031;45061},{1;2}),3)"),
        ("A2", "=INDEX(EDATE({45000;45031;45061},{1;2}),2)"),
        ("A3", "=INDEX(EDATE({45000,45031;45061,45092},{0,0,0}),2,3)"),
        ("A4", "=INDEX(EDATE({45000,45031;45061,45092},{0,0,0}),2,2)"),
        ("A5", "=ROWS(EDATE({45000;45031;45061},{1;2}))"),
    ]);
    assert_eq!(model._get_text("A1"), "#N/A");
    assert_eq!(model._get_text("A2"), "45092");
    assert_eq!(model._get_text("A3"), "#N/A");
    assert_eq!(model._get_text("A4"), "45092");
    assert_eq!(model._get_text("A5"), "3");
}

#[test]
fn a_1x1_array_lifts_to_a_1x1_result() {
    let mut model = model_with(&[]);
    let (result, calls) = run_counting(
        &mut model,
        edate_kinds(),
        "EDATE({45000},0)",
        LiftCost::Unit,
        first,
    );
    assert_eq!(calls, 1);
    assert_eq!(numbers(&result), vec![vec![45000.0]]);
}

#[test]
fn a_single_grid_is_rewritten_in_place_with_the_scalars_beside_it() {
    let mut model = model_with(&[]);
    let (result, calls) = run_counting(
        &mut model,
        &LIFT_ALL,
        "SUM(100,{1,2;3,4},1000)",
        LiftCost::Unit,
        |values| match values {
            [CalcResult::Number(a), CalcResult::Number(b), CalcResult::Number(c)] => {
                CalcResult::Number(a + b + c)
            }
            _ => CalcResult::EmptyArg,
        },
    );
    assert_eq!(calls, 4);
    assert_eq!(
        numbers(&result),
        vec![vec![1101.0, 1102.0], vec![1103.0, 1104.0]]
    );
}

// ---------------------------------------------------------------- errors per element

#[test]
fn an_element_error_stays_in_its_position() {
    let model = model_with(&[
        ("A1", "=INDEX(EDATE({45000;-1},1),1)"),
        ("A2", "=INDEX(EDATE({45000;-1},1),2)"),
        // A typed boolean date is #VALUE! (scalar-lift-shapes H36, H37).
        ("A3", "=INDEX(EDATE({TRUE;45000},1),1)"),
        ("A4", "=INDEX(EDATE({TRUE;45000},1),2)"),
    ]);
    assert_eq!(model._get_text("A1"), "45031");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "45031");
}

#[test]
fn a_scalar_error_and_scalar_text_broadcast() {
    let model = model_with(&[
        // scalar-lift-shapes H30, H45: two rows (the element values are the design's).
        ("A1", "=ROWS(EDATE(NA(),{1;2}))"),
        ("A2", "=INDEX(EDATE(NA(),{1;2}),2)"),
        ("A3", "=ROWS(EDATE(\"x\",{1;2}))"),
        ("A4", "=INDEX(EDATE(\"x\",{1;2}),2)"),
    ]);
    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model._get_text("A2"), "#N/A");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "#VALUE!");
}

#[test]
fn the_kernel_check_order_decides_the_error_per_element() {
    let model = model_with(&[
        ("A1", "=INDEX(YEARFRAC({1;#N/A},{#DIV/0!;#NUM!}),1)"),
        ("A2", "=INDEX(YEARFRAC({1;#N/A},{#DIV/0!;#NUM!}),2)"),
    ]);
    assert_eq!(model._get_text("A1"), "#DIV/0!");
    assert_eq!(model._get_text("A2"), "#N/A");
}

#[test]
fn kernel_element_errors_carry_no_message() {
    let model = model_with(&[
        ("A1", "=EDATE(-1,1)"),
        ("A2", "=EOMONTH(-1,1)"),
        ("A3", "=WEEKNUM(1,99)"),
        ("A4", "=YEARFRAC(1,2,9)"),
    ]);
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), "#NUM!", "{cell}");
        assert_eq!(model._get_error_message(cell), "", "{cell}");
    }
}

// ---------------------------------------------------------------- LiftArrays

#[test]
fn a_multi_cell_range_alone_is_one_value_error_and_one_kernel_call() {
    let mut model = model_with(&[("A1", "45000"), ("A2", "45001"), ("A3", "45002")]);
    let (result, calls) = run_counting(
        &mut model,
        edate_kinds(),
        "EDATE(A1:A3,1)",
        LiftCost::Unit,
        first,
    );
    assert_eq!(calls, 1);
    assert!(is_error(&result, &Error::VALUE));
}

#[test]
fn a_range_next_to_an_array_makes_every_element_value_in_the_arrays_shape() {
    let mut model = model_with(&[("M1", "1"), ("M2", "2"), ("M3", "3")]);
    let (result, calls) = run_counting(
        &mut model,
        edate_kinds(),
        "EDATE({45000;45031},M1:M3)",
        LiftCost::Unit,
        |values| match values.get(1) {
            Some(error @ CalcResult::Error { .. }) => error.clone(),
            _ => CalcResult::Number(0.0),
        },
    );
    assert_eq!(calls, 2);
    let CalcResult::Array(rows) = result else {
        panic!("not an array");
    };
    assert_eq!(rows.len(), 2);
    assert!(rows
        .iter()
        .all(|row| matches!(row.as_slice(), [ArrayNode::Error(Error::VALUE)])));
}

#[test]
fn references_returned_by_functions_are_refused_and_one_cell_ones_are_values() {
    let model = model_with(&[
        ("A1", "45000"),
        ("A2", "45001"),
        ("A3", "45002"),
        // scalar-lift-shapes H50, -b K3, gen-m9/b58b6d7cd39d195f A9.
        ("B1", "=EDATE(IF(TRUE,A1:A3),1)"),
        ("B2", "=EDATE(INDEX(A1:A3,0,1),1)"),
        ("B3", "=ROWS(EOMONTH(45000,TAKE(A1:A3,2)))"),
        ("B4", "=EDATE(INDEX(A1:A3,2),1)"),
        // An operation on the range is an array, and that lifts (H5).
        ("B5", "=ROWS(EDATE(A1:A3+0,1))"),
        ("B6", "=ROWS(EDATE(TAKE(A1:A3,2)+0,1))"),
    ]);
    assert_eq!(model._get_text("B1"), "#VALUE!");
    assert_eq!(model._get_text("B2"), "#VALUE!");
    assert_eq!(model._get_text("B3"), "#VALUE!");
    assert_eq!(model._get_text("B4"), "45032");
    assert_eq!(model._get_text("B5"), "3");
    assert_eq!(model._get_text("B6"), "2");
}

#[test]
fn the_four_functions_lift_over_arrays_and_refuse_ranges() {
    let model = model_with(&[
        ("A1", "45000"),
        ("A2", "45031"),
        ("C1", "=INDEX(EOMONTH({45000;45031},0),2)"),
        ("C2", "=INDEX(WEEKNUM({45000;45031},{1;2}),2)"),
        ("C3", "=INDEX(YEARFRAC(45000,{45366;45731},{1;0}),2)"),
        ("C4", "=EOMONTH(A1:A2,0)"),
        ("C5", "=WEEKNUM(A1:A2)"),
        ("C6", "=YEARFRAC(45000,46000,A1:A2)"),
        ("C7", "=ROWS(EDATE(45000,A1:A2))"),
    ]);
    assert_eq!(model._get_text("C1"), "45046");
    assert_eq!(model._get_text("C2"), "16");
    assert_eq!(model._get_text("C3"), "2");
    for cell in ["C4", "C5", "C6", "C7"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
}

// ---------------------------------------------------------------- Whole

#[test]
fn a_whole_argument_is_never_lifted_and_read_once() {
    let mut model = model_with(&[("A1", "1"), ("A2", "2"), ("A3", "3")]);
    // RAND() read once is the same value in every element; a range is handed on whole.
    let mut seen: Vec<CalcResult> = Vec::new();
    let args = args_of(&mut model, "SUM(RAND(),{1;2;3})");
    let mut kernel = |_: &mut Model, values: &[CalcResult]| {
        seen.push(first(values));
        CalcResult::Number(0.0)
    };
    let result = model.lift_call_with(&WHOLE_THEN_LIFT, &args, LiftCost::Unit, AT, &mut kernel);
    assert_eq!(numbers(&result).len(), 3);
    assert_eq!(seen.len(), 3);
    assert!(matches!(seen[0], CalcResult::Number(_)));
    assert!(seen.iter().all(|value| *value == seen[0]));
    let (result, calls) = run_counting(
        &mut model,
        &WHOLE_THEN_LIFT,
        "SUM(A1:A3,{1;2})",
        LiftCost::Unit,
        |values| match first(values) {
            CalcResult::Range { left, right } if left.row == 1 && right.row == 3 => {
                CalcResult::Number(1.0)
            }
            _ => CalcResult::Number(0.0),
        },
    );
    assert_eq!(calls, 2);
    assert_eq!(numbers(&result), vec![vec![1.0], vec![1.0]]);
}

#[test]
fn a_whole_by_caller_argument_is_not_evaluated() {
    const BY_CALLER: ArgKinds = ArgKinds::fixed(
        2,
        &[ArgKind::WholeByCaller, ArgKind::LiftArrays],
        KernelWork::Constant,
    );
    let mut model = model_with(&[]);
    let args = args_of(&mut model, "SUM(1/0,{1;2})");
    let read = model
        .read_lift_args(&BY_CALLER, &args, MAX_LIFTED_CELL_VISITS, AT)
        .ok()
        .unwrap();
    assert!(matches!(
        read.values.as_slice().first(),
        Some(CalcResult::EmptyArg)
    ));
    assert_eq!(read.grids.len(), 1);
}

// ---------------------------------------------------------------- bounds (directive 5)

/// Column A filled to the last row and A1:B1 filled, so no bound passes on blank cells.
fn filled_column_model() -> Model {
    let mut model = new_empty_model();
    let sheet = model.workbook.worksheet_mut(0).unwrap();
    for row in 1..=1_048_576 {
        sheet.set_cell_with_number(row, 1, 45000.0, 0).unwrap();
    }
    sheet.set_cell_with_number(1, 2, 1.0, 0).unwrap();
    model.evaluate();
    model
}

#[test]
fn an_output_above_the_array_bound_is_refused_before_any_element() {
    let mut model = filled_column_model();
    let (result, calls) = run_counting(
        &mut model,
        edate_kinds(),
        "EDATE(A1:A1048576+0,A1:B1+0)",
        LiftCost::Unit,
        first,
    );
    assert_eq!(calls, 0);
    assert!(is_error(&result, &Error::ERROR));
    model._set("D1", "=ROWS(EDATE(A1:A1048576+0,A1:B1+0))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#ERROR!");
}

#[test]
fn summed_input_cells_are_refused_before_the_next_grid_is_read() {
    let mut model = model_with(&[("A1", "1"), ("A60", "1"), ("B1", "1"), ("B60", "1")]);
    let args = args_of(&mut model, "SUM(A1:A60,B1:B60)");
    // 60 + 60 cells: within a cap of 120, refused under a cap of 100 at the second range.
    let read = model
        .read_lift_args(&LIFT_ALL, &args, 120, AT)
        .ok()
        .unwrap();
    assert_eq!(read.grids.len(), 2);
    let refused = model.read_lift_args(&LIFT_ALL, &args, 100, AT);
    assert!(matches!(refused, Err(ref error) if is_error(error, &Error::ERROR)));
    // An array counts toward the total too.
    let args = args_of(&mut model, "SUM(A1:A60,SEQUENCE(41))");
    assert!(model.read_lift_args(&LIFT_ALL, &args, 100, AT).is_err());
    // A range is bounded by its written shape before it is read (2 whole columns).
    let args = args_of(&mut model, "SUM(A:B)");
    let refused = model.read_lift_args(&LIFT_ALL, &args, MAX_LIFTED_CELL_VISITS, AT);
    assert!(matches!(refused, Err(ref error) if is_error(error, &Error::ERROR)));
}

#[test]
fn per_element_cells_trip_the_work_bound_and_prepared_data_runs() {
    let mut model = filled_column_model();
    // A whole argument of 2^20 cells against 2^8 elements: 2^28 visits per element walk, but
    // 2^20 + 2^8 when the kernel prepares the data once.
    let formula = "SUM(A1:A1048576,SEQUENCE(256))";
    let (result, calls) = run_counting(
        &mut model,
        &WHOLE_THEN_LIFT,
        formula,
        LiftCost::Cells(1 << 20),
        |_| CalcResult::Number(1.0),
    );
    assert_eq!(calls, 0);
    assert!(is_error(&result, &Error::ERROR));
    let (result, calls) = run_counting(
        &mut model,
        &WHOLE_THEN_LIFT,
        formula,
        LiftCost::Prepared(1 << 20),
        |_| CalcResult::Number(1.0),
    );
    assert_eq!(calls, 256);
    assert_eq!(numbers(&result).len(), 256);
}

#[test]
fn a_weighted_kernel_is_charged_its_weight() {
    let mut model = model_with(&[]);
    let grid = || vec![CalcResult::Array(vec![vec![ArrayNode::Number(1.0); 1024]])];
    // 2^10 elements × 2^17 cells is the cap exactly; a weight of 2 is over it.
    for (weight, runs) in [(1, true), (2, false)] {
        let options = LiftOptions {
            reuse_equal_runs: false,
            work: KernelWork::Weighted(weight),
        };
        let mut calls = 0;
        let mut kernel = |_: &mut Model, _: &[CalcResult]| {
            calls += 1;
            CalcResult::Number(1.0)
        };
        let result = model.lift_values(grid(), LiftCost::Cells(1 << 17), &options, AT, &mut kernel);
        assert_eq!(calls, if runs { 1024 } else { 0 }, "weight {weight}");
        assert_eq!(is_error(&result, &Error::ERROR), !runs, "weight {weight}");
    }
}

#[test]
fn element_texts_count_against_the_text_budget() {
    let mut model = model_with(&[]);
    let long = CalcResult::String("x".repeat(32767).into());
    let elements = |n: usize| vec![CalcResult::Array(vec![vec![ArrayNode::Number(1.0); n]])];
    let options = LiftOptions {
        reuse_equal_runs: false,
        work: KernelWork::Constant,
    };
    let mut kernel = |_: &mut Model, _: &[CalcResult]| long.clone();
    // 2048 texts of 32,767 bytes fit 64 MiB; 2049 are #VALUE!.
    let result = model.lift_values(elements(2048), LiftCost::Unit, &options, AT, &mut kernel);
    assert!(matches!(result, CalcResult::Array(_)));
    let result = model.lift_values(elements(2049), LiftCost::Unit, &options, AT, &mut kernel);
    assert!(is_error(&result, &Error::VALUE));
}

#[test]
fn lifted_args_are_grids_for_arrays_and_ranges() {
    assert!(LiftedArg::is_grid(&CalcResult::Array(vec![])));
}

#[test]
fn a_3d_range_is_value_in_lift_arrays_and_ref_in_lift() {
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model._set("A1", "45000");
    model._set("B1", "=EDATE(Sheet1:Sheet2!A1,1)");
    model._set("B2", "=EDATE(Sheet1:Sheet2!A1:A2,1)");
    model.evaluate();
    // probes/scalar-lift-open-b W2-W11, -d W17-W29 (T-1777).
    assert_eq!(model._get_text("B1"), "#VALUE!");
    assert_eq!(model._get_text("B2"), "#VALUE!");
    let (result, calls) = run_counting(
        &mut model,
        &LIFT_ALL,
        "SUM(Sheet1:Sheet2!A1:A2,{1;2})",
        LiftCost::Unit,
        first,
    );
    assert_eq!(calls, 2);
    let CalcResult::Array(rows) = result else {
        panic!("not an array");
    };
    assert!(rows
        .iter()
        .all(|row| matches!(row.as_slice(), [ArrayNode::Error(Error::REF)])));
}
