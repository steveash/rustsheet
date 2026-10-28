#![allow(clippy::unwrap_used)]

//! Whole-column aggregate arguments read as a used head and one repeated tail row (SPEC-0020,
//! T-0469) give what the 1,048,576-row arrays give.

use crate::tailed_array::{TAILED_OFF, TAILED_WALKS};
use crate::test::util::new_empty_model;
use crate::Model;

/// A1:C40 of Sheet1: numbers, text, booleans, blanks, empty text and errors; H1:H3 the scalars
/// the formulas read. Sheet2!A1:A60 holds numbers, so its used area is longer than Sheet1's.
fn fill(model: &mut Model) {
    const A: &[&str] = &["1", "2", "", "x", "TRUE", "=1/0", "3", "=\"\"", "1", "0"];
    const B: &[&str] = &["5", "", "7", "y", "FALSE", "2.5", "=NA()", "-1", "0.1", "9"];
    const C: &[&str] = &["0.2", "3", "", "z", "1", "4", "TRUE", "", "=\"\"", "8"];
    for row in 1..=40 {
        let i = (row - 1) % 10;
        let j = (row * 3) % 10;
        model._set(&format!("A{row}"), A[i]);
        model._set(&format!("B{row}"), B[j]);
        model._set(&format!("C{row}"), C[(i + j) % 10]);
    }
    model._set("H1", "1");
    model._set("H2", "=1/0");
    model._set("H3", "TRUE");
    model.new_sheet();
    for row in 1..=60 {
        model._set(&format!("Sheet2!A{row}"), &(row % 7).to_string());
    }
}

/// Each formula's text with the tailed reading on and with it off; how many arguments the
/// first evaluation walked as a head and a tail.
fn both_ways(formulas: &[String]) -> (Vec<(String, String, String)>, usize) {
    let mut values = Vec::new();
    let mut walks = 0;
    for off in [false, true] {
        TAILED_OFF.with(|flag| flag.set(off));
        TAILED_WALKS.with(|count| count.set(0));
        let mut model = new_empty_model();
        fill(&mut model);
        for (index, formula) in formulas.iter().enumerate() {
            model._set(&format!("E{}", index + 1), formula);
        }
        model.evaluate();
        if !off {
            walks = TAILED_WALKS.with(std::cell::Cell::get);
        }
        values.push(
            (0..formulas.len())
                .map(|index| model._get_text(&format!("E{}", index + 1)))
                .collect::<Vec<_>>(),
        );
    }
    TAILED_OFF.with(|flag| flag.set(false));
    let results = formulas
        .iter()
        .enumerate()
        .map(|(index, formula)| {
            (
                formula.clone(),
                values[0][index].clone(),
                values[1][index].clone(),
            )
        })
        .collect();
    (results, walks)
}

/// Every aggregate of `aggregates` over every expression of `expressions` gives the same text
/// both ways, and all of them were tailed.
fn check(aggregates: &[&str], expressions: &[&str]) {
    let formulas: Vec<String> = aggregates
        .iter()
        .flat_map(|f| expressions.iter().map(move |e| format!("={f}({e})")))
        .collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, formulas.len(), "every argument is tailed");
}

/// One aggregate per kind of walk: sums, extremes, counts of numbers and of values, logicals,
/// the A variants' coercions and the variance family's two passes.
const AGGREGATES: &[&str] = &["SUM", "MIN", "COUNT", "COUNTA", "AND", "MAXA", "VAR"];

#[test]
fn if_over_whole_column_conditions() {
    check(
        AGGREGATES,
        &[
            "IF(A:A=$H$1,B:B,\"\")",
            "IF(A:A>1,B:B*2,C:C)",
            // An empty tail through a branch; COUNTA counts it in every row.
            "IF(A:A=\"\",C:C)",
        ],
    );
}

#[test]
fn operators_over_whole_columns() {
    check(
        AGGREGATES,
        &[
            "(A:A=1)*B:B",
            "-A:A",
            "A:A%",
            "B:B/A:A",
            "A:A&\"x\"",
            "A:A^2",
            "B:B<>A:A",
            "A:A<=1",
        ],
    );
}

#[test]
fn numeric_and_error_tails() {
    check(
        AGGREGATES,
        &[
            // A numeric tail, summed once per row.
            "IF(A:A=\"\",0.1,0)",
            // Scalar error operands: the tail keeps the left error (`array_of_errors`).
            "((A:A+$H$2)=$H$2)",
            "$H$2+A:A",
            // A bare range as the condition.
            "IF(C:C,B:B,A:A)",
        ],
    );
}

/// The number-folding aggregates skip a repeated tail only when their rules ignore it: a
/// numeric tail counts in every row, and so does a text tail for VARA (text is 0). Sheet2's
/// head has no blanks, so only the tail rows give these values.
#[test]
fn number_folding_aggregates_skip_only_ignored_tails() {
    check(
        &["AVERAGE", "MAX", "MIN", "PRODUCT", "VARA", "STDEV"],
        &[
            "IF(Sheet2!A:A=\"\",0.5,Sheet2!A:A)",
            "IF(Sheet2!A:A=\"\",\"t\",Sheet2!A:A)",
            "IF(Sheet2!A:A=\"\",TRUE,Sheet2!A:A)",
        ],
    );
}

#[test]
fn operands_on_another_sheet() {
    // Sheet2's used area is longer than Sheet1's: Sheet1's columns are padded with blanks.
    check(
        &["SUM", "MAX", "COUNTA"],
        &["Sheet2!A:A*(A:A=1)", "IF(Sheet2!A:A>3,A:A,B:B)"],
    );
}

/// A chain of 4,000 operators is walked without recursion, and is tailed.
#[test]
fn a_long_operator_chain_is_tailed() {
    let mut model = new_empty_model();
    for row in 1..=10 {
        model._set(&format!("A{row}"), &row.to_string());
    }
    let chain = vec!["A:A"; 4000].join("+");
    model._set("E1", &format!("=SUM({chain})"));
    model._set("E2", &format!("=COUNTA({chain})"));
    TAILED_WALKS.with(|count| count.set(0));
    model.evaluate();
    assert_eq!(model._get_text("E1"), "220000");
    assert_eq!(model._get_text("E2"), "1048576");
    assert_eq!(TAILED_WALKS.with(std::cell::Cell::get), 2);
}

/// What a tailed reading must leave to the ordinary one: references walked as references, an IF
/// over a scalar condition, functions and bounded ranges.
#[test]
fn excluded_forms_are_not_tailed() {
    let formulas: Vec<String> = [
        "=SUM(A:A)",
        "=SUM((A:A))",
        "=COUNTA(+A:A)",
        "=MIN(IF($H$3,A:A,B:B))",
        "=SUM(IF(A1>0,A:A,B:B)*1)",
        "=SUM(ISNUMBER(A:A)*1)",
        "=SUM(A1:A100*1)",
        "=SUM(A:B*1)",
        "=SUM(ROW(A:A)*0)",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, 0);
}

/// A dynamic array spills into the tail rows of a column a tailed argument read: the pass
/// restarts, and the aggregate counts the spilled values.
#[test]
fn a_spill_into_the_tail_restarts_the_pass() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("E1", "=SUM(IF(A:A>0,A:A))");
    model._set("F1", "=SEQUENCE(5,1,10)");
    model._set("A100", "=F1:F5*1");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "61");
}

/// A head of one row (only A1 is used): a 1x1 array where the ordinary reading has the whole
/// column, which IF and the operators treat as an array all the same.
#[test]
fn a_head_of_one_row() {
    let formulas: Vec<String> = [
        "=SUM(IF(A:A=1,A:A*2))",
        "=COUNTA(A:A&\"\")",
        "=MAX((A:A>0)*A:A)",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let mut values = Vec::new();
    for off in [false, true] {
        TAILED_OFF.with(|flag| flag.set(off));
        TAILED_WALKS.with(|count| count.set(0));
        let mut model = new_empty_model();
        model._set("A1", "3");
        for (index, formula) in formulas.iter().enumerate() {
            model._set(&format!("E{}", index + 1), formula);
        }
        model.evaluate();
        if !off {
            assert_eq!(TAILED_WALKS.with(std::cell::Cell::get), formulas.len());
        }
        values.push(
            (1..=formulas.len())
                .map(|r| model._get_text(&format!("E{r}")))
                .collect::<Vec<_>>(),
        );
    }
    TAILED_OFF.with(|flag| flag.set(false));
    assert_eq!(values[0], values[1]);
}

/// A one-cell reference that closes a cycle is read again by the tail: the cycle is reported as
/// the ordinary reading reports it.
#[test]
fn a_cycle_through_a_scalar_operand() {
    let mut texts = Vec::new();
    for off in [false, true] {
        TAILED_OFF.with(|flag| flag.set(off));
        let mut model = new_empty_model();
        for row in 1..=5 {
            model._set(&format!("A{row}"), &row.to_string());
        }
        model._set("E1", "=SUM(IF(A:A>E2,A:A))");
        model._set("E2", "=E1/100");
        model._set("E3", "=SUM(E1:E2)");
        model.evaluate();
        texts.push(["E1", "E2", "E3"].map(|c| model._get_text(c)));
    }
    TAILED_OFF.with(|flag| flag.set(false));
    assert_eq!(texts[0], texts[1]);
}

/// SUMPRODUCT reads element-wise whole-column arguments as a head and a tail too (T-0471): one
/// argument over every expression of the aggregate tests.
#[test]
fn sumproduct_of_one_tailed_argument() {
    check(
        &["SUMPRODUCT"],
        &[
            "(A:A=1)*B:B",
            "IF(A:A=$H$1,B:B,\"\")",
            "A:A=$H$1",
            "B:B/A:A",
            "IF(A:A=\"\",0.1,0)",
            "$H$2+A:A",
            "-A:A",
            // Columns without errors, so the products are summed: C holds text, blanks and
            // empty text, Sheet2!A numbers only. A tail of 0.1 in every row, added row by row.
            "IF(C:C=\"\",0.1,C:C)",
            "(C:C=\"\")*1",
            "C:C*2",
            "Sheet2!A:A/3",
            "IF(Sheet2!A:A=\"\",1/3,Sheet2!A:A)",
        ],
    );
}

/// SUMPRODUCT over several arguments, tailed or not: shapes compared at their logical size, the
/// first element error in argument order, a whole-argument error, tails that differ by sheet and
/// a nonzero product past every head.
#[test]
fn sumproduct_of_several_arguments() {
    let cases = [
        ("=SUMPRODUCT((A:A=1)*1,B:B*1)", 2),
        ("=SUMPRODUCT((A:A=1)*1,B:B)", 1),
        ("=SUMPRODUCT(A:A*1,A1:A10)", 1),
        ("=SUMPRODUCT(A1:A10,A:A*1)", 1),
        ("=SUMPRODUCT(B:B*1,(A:A=\"\")*1)", 2),
        ("=SUMPRODUCT((C:C=\"\")*1,NA())", 1),
        ("=SUMPRODUCT(Sheet2!A:A*1,(A:A=\"\")*1)", 2),
        ("=SUMPRODUCT((Sheet2!A:A=\"\")*0.5,(A:A=\"\")*3)", 2),
        ("=SUMPRODUCT((A:A>0)*1,(A:A>0)*1,(B:B<>0)*1)", 3),
        ("=SUMPRODUCT((C:C=\"\")*0.1,(Sheet2!A:A=\"\")*3)", 2),
        ("=SUMPRODUCT(C:C*1,Sheet2!A:A+1)", 2),
        ("=SUMPRODUCT((C:C=\"\")*1,Sheet2!A:A)", 1),
        // A tail product of -0: skipped, as adding it leaves the total as it is.
        ("=SUMPRODUCT(-(C:C=\"x\")*1,Sheet2!A:A+1)", 2),
    ];
    let formulas: Vec<String> = cases.iter().map(|(f, _)| (*f).to_string()).collect();
    let expected_walks: usize = cases.iter().map(|(_, walks)| walks).sum();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, expected_walks);
}

/// Bounded single-column ranges past the used area are tailed like whole columns (SPEC-0020
/// Amendment 1, T-0491): spans that start inside, at the end of and below the used area, on
/// another sheet, and `ROW()` of the span where the tail row does not reach it.
#[test]
fn bounded_spans_and_row_positions_are_tailed() {
    let formulas: Vec<String> = [
        "=SUM(A2:A70000*1)",
        "=COUNTA(IF(B2:B70000=5,A2:A70000))",
        "=MIN(IF(C1:C9000<>\"\",C1:C9000))",
        "=SUM(A40:A70000+1)",
        "=COUNTA(A50:A70000&\"\")",
        "=SUM(Sheet2!A3:A80000*2)",
        "=SUM(IF(B2:B70000=5,ROW(B2:B70000)))",
        "=COUNT(IF(A2:A70000>0,ROW(A2:A70000)))",
        "=MIN(IF(C1:C9000=3,ROW(C1:C9000)*10))",
        "=SUMPRODUCT(IF(A2:A70000=1,ROW(A2:A70000)))",
        "=SUM(IF(A2:A70000=1,ROW(A2:A70000),A2:A70000))",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, formulas.len(), "every argument is tailed");
}

/// What bounded spans and `ROW()` leave to the ordinary reading: ranges of two spans, a tail row
/// that reaches `ROW()` (its value differs by row: the sentinel), `ROW()` in an IF's condition, `ROW()` with no range operand
/// beside it, and a span whose rows past the used area are too few.
#[test]
fn ragged_spans_and_row_dependent_tails_are_not_tailed() {
    let formulas: Vec<String> = [
        "=SUM(A1:A70000*B2:B70000)",
        "=SUM(A:A*A1:A70000)",
        "=SUM(IF(B2:B70000=\"\",ROW(B2:B70000)))",
        "=SUM(A2:A70000+ROW(A2:A70000))",
        "=SUMPRODUCT((A2:A70000=1)*ROW(A2:A70000))",
        "=SUM(IF(ROW(A2:A70000)>2,1))",
        "=MAX(IF(ROW(A2:A70000)>30,A2:A70000))",
        "=SUM(IF(A2:A70000=1,ROW(A1:A70000)))",
        "=SUM(A1:A4000*1)",
        "=SUM(IF(A:A=1,COLUMN(A:A)))",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, 0);
}

/// A dynamic array spills into the tail rows of a bounded range a tailed argument read: its
/// rows past the used area are on record as read empty, so the pass restarts.
#[test]
fn a_spill_into_a_bounded_tail_restarts_the_pass() {
    let mut model = new_empty_model();
    model._set("A2", "1");
    model._set("E1", "=SUM(IF(A2:A70000>0,A2:A70000))");
    model._set("F1", "=SEQUENCE(5,1,10)");
    model._set("A100", "=F1:F5*1");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "61");
}
