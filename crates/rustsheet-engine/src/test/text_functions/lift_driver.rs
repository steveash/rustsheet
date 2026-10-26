//! The element-wise text functions on the SPEC-0011 driver (T-1775): what the driver decides for
//! them, beyond the per-function tests next door.

use crate::test::util::{check_cells, new_empty_model};

/// A 1x1 array lifts the call into a 1x1 array, as the other lifted families do; a one-cell
/// range is its cell's value (UNVERIFIED for the text functions, T-1963).
#[test]
fn a_one_by_one_array_lifts_and_a_one_cell_range_does_not() {
    check_cells(
        &[("A1", "abc")],
        &[
            ("B1", "=TYPE(LEFT({\"abc\"},2))", "64"),
            ("B2", "=TYPE(LEFT(A1:A1,2))", "2"),
            (
                "B3",
                "=ARRAYTOTEXT(MID({\"abc\"},\"x\",NA()),1)",
                "{#VALUE!}",
            ),
            ("B4", "=TYPE(CONCATENATE({\"a\"},\"b\"))", "64"),
        ],
    );
}

/// A LAMBDA argument makes the element `#VALUE!` whatever the other arguments hold
/// (probes/filterxml-array-forms A9 for a later error), and an array argument keeps its shape
/// (UNVERIFIED, T-1963).
#[test]
fn a_lambda_argument_is_value_in_every_element() {
    check_cells(
        &[],
        &[
            ("A1", "=LEFT(NA(),LAMBDA(x,x))", "#VALUE!"),
            ("A5", "=VALUETOTEXT(LAMBDA(x,x),NA())", "#VALUE!"),
            ("A6", "=FILTERXML(LAMBDA(x,x),NA())", "#VALUE!"),
            ("A2", "=LEFT(LAMBDA(x,x),1)", "#VALUE!"),
            ("A3", "=COLUMNS(LEFT({\"a\",\"b\"},LAMBDA(x,x)))", "2"),
            ("A4", "=CONCATENATE(LAMBDA(x,x))", "#VALUE!"),
        ],
    );
}

/// Every element-wise text function lifts its arguments over a range and an array, scalars
/// broadcast, and a position past a shorter array is `#N/A`.
#[test]
fn every_text_function_broadcasts_its_arguments() {
    check_cells(
        &[("A1", "abc"), ("A2", "2"), ("A3", "1/2/2020")],
        &[
            (
                "B1",
                "=ARRAYTOTEXT(LEFT({\"ab\";\"cd\";\"ef\"},{1;2}),1)",
                "{\"a\";\"cd\";#N/A}",
            ),
            (
                "B2",
                "=ARRAYTOTEXT(FIND({\"a\",\"b\",\"z\"},\"abc\"),1)",
                "{1,2,#VALUE!}",
            ),
            (
                "B3",
                "=ARRAYTOTEXT(SUBSTITUTE(\"aaa\",{\"a\",\"b\"},\"c\",{1;2}),1)",
                "{\"caa\",\"aaa\";\"aca\",\"aaa\"}",
            ),
            (
                "B4",
                "=ARRAYTOTEXT(REPLACE(\"abc\",{1,2},1,{\"x\";\"y\"}),1)",
                "{\"xbc\",\"axc\";\"ybc\",\"ayc\"}",
            ),
            (
                "B5",
                "=ARRAYTOTEXT(TEXT(A2:A3,{\"0\",\"0.0\"}),1)",
                "{\"2\",\"2.0\";\"43832\",\"43832.0\"}",
            ),
            (
                "B6",
                "=ARRAYTOTEXT(VALUETOTEXT({\"a\",1},{0;1}),1)",
                "{\"a\",\"1\";\"\"\"a\"\"\",\"1\"}",
            ),
            ("B7", "=ARRAYTOTEXT(CODE(A1:A2),1)", "{97;50}"),
            (
                "B8",
                "=ARRAYTOTEXT(DATEVALUE({\"1/2/2020\",\"x\",1}),1)",
                "{43832,#VALUE!,#VALUE!}",
            ),
            (
                "B9",
                "=ARRAYTOTEXT(CONCATENATE(A1:A2,{\"x\",\"y\"}),1)",
                "{\"abcx\",\"abcy\";\"2x\",\"2y\"}",
            ),
            (
                "B10",
                "=ARRAYTOTEXT(FILTERXML({\"<a/>\";\"<b/>\"},NA()),1)",
                "{#N/A;#N/A}",
            ),
        ],
    );
}

/// A lifted FIND or SEARCH is charged by its texts' lengths (T-1961): one over long texts is
/// refused with the visit cap's `#ERROR!` before its first element (DEVIATION, decision 0014:
/// Excel has no such limit), where each element would compare about 10^9 bytes; short texts and
/// one long text against a short one are far under it.
#[test]
fn a_lifted_search_over_long_texts_is_refused() {
    let mut model = new_empty_model();
    for (cell, function) in [("B1", "FIND"), ("B2", "SEARCH"), ("B3", "FINDB")] {
        model._set(
            cell,
            &format!("={function}(REPT(\"a\",32766)&\"b\",REPT(\"a\",32763)&SEQUENCE(1000))"),
        );
    }
    model._set(
        "C1",
        "=SUM(FIND(\"b\",REPT(\"a\",32766)&\"b\"&SEQUENCE(1000)))",
    );
    model._set(
        "C2",
        "=SUM(SEARCH(\"B\",REPT(\"a\",32766)&\"b\"&SEQUENCE(1000)))",
    );
    model._set("C3", "=FIND(REPT(\"a\",32766)&\"b\",REPT(\"a\",32767))");
    model.evaluate();
    for cell in ["B1", "B2", "B3"] {
        assert_eq!(model._get_text(cell), "#ERROR!", "{cell}");
        assert_eq!(model._get_error_message(cell), "Too many cells to visit");
    }
    assert_eq!(model._get_text("C1"), "32767000");
    assert_eq!(model._get_text("C2"), "32767000");
    // One call is not lifted, so not charged: its one scan runs.
    assert_eq!(model._get_text("C3"), "#VALUE!");
}

/// The charge leaves ordinary columns alone: a 100,000-row column of FINDs, one call each, and a
/// SUM over one FIND lifted across the 100,000 texts.
#[test]
fn a_long_column_of_searches_runs() {
    let mut model = new_empty_model();
    for row in 1..=100_000 {
        model._set(&format!("A{row}"), &format!("xab{row}"));
        model._set(&format!("B{row}"), &format!("=FIND(\"b\",A{row})"));
    }
    model._set("C1", "=SUM(FIND(\"a\",A1:A100000))");
    model._set("C2", "=SUM(B1:B100000)");
    model._set("C3", "=SUM(SEARCH(\"B\",A1:A100000))");
    model.evaluate();
    assert_eq!(model._get_text("B100000"), "3");
    assert_eq!(model._get_text("C1"), "200000");
    assert_eq!(model._get_text("C2"), "300000");
    assert_eq!(model._get_text("C3"), "300000");
}
