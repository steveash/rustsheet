//! The element-wise text functions on the SPEC-0011 driver (T-1775): what the driver decides for
//! them, beyond the per-function tests next door.

use crate::test::util::check_cells;

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

/// A LAMBDA argument is `#VALUE!` in its own position, so an earlier argument's error comes first
/// and an array argument keeps its shape (UNVERIFIED, T-1963).
#[test]
fn a_lambda_argument_is_value_per_element() {
    check_cells(
        &[],
        &[
            ("A1", "=LEFT(NA(),LAMBDA(x,x))", "#N/A"),
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
