#![allow(clippy::unwrap_used)]

//! Whole-column aggregate arguments read as a used head and one repeated tail row (SPEC-0020,
//! T-0469) give what the 1,048,576-row arrays give.

use crate::functions::lift::LIFT_TALLY;
use crate::tailed_array::{TAILED_OFF, TAILED_WALKS};
use crate::test::util::new_empty_model;
use crate::Model;

/// A1:C40 of Sheet1: numbers, text, booleans, blanks, empty text and errors; H1:H3 the scalars
/// the formulas read; the lookup tables of probes/t3096-lookup-value-blank-elements in J1:K6
/// (0, empty text, FALSE and a blank among the keys; also the name `Tbl`), L1:L5 (descending)
/// and N1:S2 (by row). Sheet2!A1:A60 holds numbers, so its used area is longer than Sheet1's.
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
    // #N/IMPL: the engine does not implement FORECAST.ETS.
    model._set("H4", "=FORECAST.ETS(5,{1,2,3},{1,2,3})");
    let keys = ["0", "=\"\"", "FALSE", "", "1", "x"];
    for (index, key) in keys.iter().enumerate() {
        let row = index + 1;
        model._set(&format!("J{row}"), key);
        model._set(&format!("K{row}"), &format!("{}", row * 10));
        let column = ["N", "O", "P", "Q", "R", "S"][index];
        model._set(&format!("{column}1"), key);
        model._set(&format!("{column}2"), &format!("{}", row * 100));
    }
    for (index, value) in ["3", "2", "1", "0", "-1"].iter().enumerate() {
        model._set(&format!("L{}", index + 1), value);
    }
    model
        .new_defined_name("Tbl", None, "Sheet1!$J$1:$K$6")
        .unwrap();
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
        // A real #N/IMPL tail with no ROW() is an ordinary tail error.
        "=COUNTA(IF(A2:A70000=1,A2:A70000,$H$4))",
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
/// that reaches `ROW()` (its value differs by row: the sentinel), `ROW()` in an IF's condition, a
/// real `#N/IMPL` tail beside `ROW()` (taken for the sentinel), `ROW()` with no range operand
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
        "=SUM(IF(A2:A70000=1,ROW(A2:A70000),$H$4))",
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

/// SUMPRODUCT of two bounded arguments of one length but different first rows: each is tailed
/// with its own head, and the rows still line up.
#[test]
fn sumproduct_of_bounded_arguments_with_different_tops() {
    let formulas: Vec<String> = [
        "=SUMPRODUCT(A2:A70000*1,B3:B70001*1)",
        "=SUMPRODUCT((A1:A9000=1)*1,(C30:C9029<>\"\")*1)",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, 2 * formulas.len(), "every argument is tailed");
}

/// UNIQUE reads an element-wise whole-column argument as its head and two copies of its tail row
/// (SPEC-0020 Amendment 1, T-0493): the whole result, as ARRAYTOTEXT's strict text, with
/// `exactly_once` TRUE and FALSE, literal `by_col` FALSE forms, tails equal to a head value (an
/// empty element, 0) and not, and an all-tail `#CALC!`.
#[test]
fn unique_of_a_tailed_argument() {
    let expressions = [
        "IF(A:A=$H$1,B:B,\"\")",
        // A tail of `Empty`, which the head holds too (C's blanks).
        "IF(A:A>1,B:B*2,C:C)",
        // A tail of 0, which the head holds too.
        "(A:A=1)*B:B",
        // A tail of 0.1 that no head row holds.
        "IF(C:C=\"\",0.1,C:C)",
        "Sheet2!A:A&\"\"",
        // Every row repeats: no row occurs once, `#CALC!` with exactly_once.
        "Sheet2!A:A*0",
    ];
    // Each expression with exactly_once FALSE and TRUE; the literal flag forms on one of them
    // (debug builds evaluate the ordinary 1,048,576-row reading slowly).
    let mut formulas: Vec<String> = expressions
        .iter()
        .flat_map(|e| ["", ",,TRUE"].map(|f| format!("=ARRAYTOTEXT(UNIQUE({e}{f}),1)")))
        .collect();
    formulas.extend(
        [",FALSE", ",0", ",,FALSE", ",FALSE,TRUE", ",0,1"]
            .map(|f| format!("=ARRAYTOTEXT(UNIQUE(IF(A:A>1,B:B*2,C:C){f}),1)")),
    );
    // A bounded span past the used area, and `ROW()` of it where the tail row does not pick it
    // (T-0491).
    formulas.extend(
        [
            "IF(A2:A70000=$H$1,B2:B70000,\"\")",
            "IF(A2:A70000=1,ROW(A2:A70000),0)",
        ]
        .map(|e| format!("=ARRAYTOTEXT(UNIQUE({e},,TRUE),1)")),
    );
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, formulas.len(), "every argument is tailed");
    let (results, _) = both_ways(&["=UNIQUE(Sheet2!A:A*0,,TRUE)".to_string()]);
    assert_eq!(results[0].1, "#CALC!");
}

/// UNIQUE keeps the ordinary reading when a flag is not a literal FALSE-or-absent `by_col` and
/// literal `exactly_once`: by columns (a whole column is one key), a flag from a reference or an
/// array (the call lifts), arguments that are not element-wise expressions, and a tail row that
/// picks `ROW()`.
#[test]
fn unique_forms_that_are_not_tailed() {
    let formulas: Vec<String> = [
        "=ARRAYTOTEXT(UNIQUE((A:A=1)*B:B,TRUE),1)",
        "=ARRAYTOTEXT(UNIQUE((A:A=1)*B:B,1,TRUE),1)",
        "=ARRAYTOTEXT(UNIQUE((A:A=1)*B:B,$H$3),1)",
        "=ARRAYTOTEXT(UNIQUE((A:A=1)*B:B,,$H$3),1)",
        "=ARRAYTOTEXT(UNIQUE((A:A=1)*B:B,{FALSE;TRUE}),1)",
        "=ARRAYTOTEXT(UNIQUE((A:A=1)*B:B,\"FALSE\"),1)",
        "=ARRAYTOTEXT(UNIQUE(A:A),1)",
        "=ARRAYTOTEXT(UNIQUE(A1:A100*1),1)",
        // The tail row picks `ROW()`: one value per row, not one repeated row (the sentinel).
        "=ROWS(UNIQUE(IF(A2:A70000=\"\",ROW(A2:A70000),0)))",
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

/// SMALL and LARGE read an element-wise argument as its head and its tail row (SPEC-0020
/// Amendment 1 section 3, T-0492): the head's first error, else the tail's error, else the head's
/// numbers and a numeric tail once per tail row. Errors in the head (A6, B's `#N/A`) and in the
/// tail (a scalar error operand), numeric, FALSE and empty tails, whole columns and bounded spans
/// with `ROW()` (corpus-bench 59969's shape), and k past the head's numbers.
#[test]
fn small_and_large_of_a_tailed_argument() {
    let expressions = [
        // An error in the head: A6 is #DIV/0!, B holds #N/A.
        "A:A*1",
        "IF(B:B=5,A:A)",
        // An error tail (and head): the left error first.
        "$H$2+C:C",
        // A tail error with no error in the head.
        "IF(Sheet2!A:A>3,Sheet2!A:A,$H$2)",
        // A numeric tail, counted once per tail row.
        "IF(C:C=\"\",0.1,0)",
        "Sheet2!A:A*2",
        // A FALSE tail: IF with no FALSE branch.
        "IF(Sheet2!A:A>3,Sheet2!A:A)",
        // An empty tail through a branch, and a text tail.
        "IF(Sheet2!A:A>3,Sheet2!A:A,C:C)",
        "Sheet2!A:A&\"\"",
        // Bounded spans and ROW(): the tail row picks FALSE.
        "IF((Sheet2!A2:A70000=3)*(Sheet2!A2:A70000<>\"\"),ROW(Sheet2!A2:A70000))",
        "IF(Sheet2!A2:A70000>4,ROW(Sheet2!A2:A70000)*10)",
    ];
    let formulas: Vec<String> = expressions
        .iter()
        .flat_map(|e| {
            ["SMALL", "LARGE"]
                .into_iter()
                .flat_map(move |f| [1, 3, 40, 5000].map(|k| format!("={f}({e},{k})")))
        })
        .collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, formulas.len(), "every argument is tailed");
}

/// The RANK family keeps the ordinary reading (its arrays are `#VALUE!`), and SMALL whose tail
/// row picks `ROW()` falls back (the sentinel).
#[test]
fn rank_and_row_dependent_small_are_not_tailed() {
    let formulas: Vec<String> = [
        "=RANK(1,Sheet2!A:A*1)",
        "=RANK.EQ(1,IF(Sheet2!A:A>3,Sheet2!A:A))",
        "=PERCENTRANK(Sheet2!A:A*1,2)",
        "=SMALL(IF(Sheet2!A2:A70000=\"\",ROW(Sheet2!A2:A70000)),2)",
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

/// LOOKUP's vector form searches an element-wise `lookup_vector` as its head and a tail it skips
/// (SPEC-0020 Amendment 1 section 3, T-0494): error tails (`1/FALSE`, corpus-bench 33935's
/// last-non-empty idiom) and empty tails, targets of each kind, including ones whose kind differs
/// from the head's last element (the search's midpoints skip forward), a lifted `lookup_value`, and
/// a bounded span.
#[test]
fn lookup_vector_form_of_a_tailed_argument() {
    let formulas: Vec<String> = [
        "=LOOKUP(2,1/(A:A<>\"\"),A:A)",
        "=LOOKUP(2,1/(B:B<>\"\"),C:C)",
        "=LOOKUP(2,1/(Sheet2!A:A<>\"\"),Sheet2!A:A)",
        // An empty tail (C's blanks through IF), searched for a number, text and a boolean.
        "=LOOKUP(3,IF(A:A=\"\",C:C,B:B),A:A)",
        "=LOOKUP(0.5,IF(A:A=\"\",C:C,B:B),A:A)",
        "=LOOKUP(\"z\",IF(A:A=\"\",C:C,A:A),B:B)",
        "=LOOKUP(TRUE,IF(A:A=\"\",C:C,A:A),B:B)",
        "=LOOKUP(\"a\",1/(A:A<>\"\"),B:B)",
        // A value smaller than every key: #N/A.
        "=LOOKUP(-5,IF(A:A=\"\",C:C,B:B),A:A)",
        // A lifted lookup_value: one search per element.
        "=ARRAYTOTEXT(LOOKUP({1;3;\"y\";TRUE},IF(A:A=\"\",C:C,A:A),B:B),1)",
        // A bounded span past the used area.
        "=LOOKUP(2,1/(A2:A70000<>\"\"),B2:B70000)",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(walks, formulas.len(), "every lookup_vector is tailed");
}

/// LOOKUP keeps the ordinary search for a tail of a searched kind (a number, FALSE, text: a run
/// of equal keys at the end), and for the array form, which picks from the table itself.
#[test]
fn lookup_forms_that_are_not_tailed() {
    let formulas: Vec<String> = [
        "=LOOKUP(2,A:A*0+1,B:B)",
        "=LOOKUP(2,IF(A:A<>\"\",1),B:B)",
        "=LOOKUP(\"b\",A:A&\"\",B:B)",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let (results, _) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    let (results, walks) = both_ways(&["=LOOKUP(2,1/(A:A<>\"\"))".to_string()]);
    assert_eq!(results[0].1, results[0].2);
    assert_eq!(walks, 0);
}

/// VLOOKUP, HLOOKUP and MATCH over a tailed key are interior nodes (SPEC-0020 Amendment 2,
/// T-3221): exact, approximate and descending searches of the t3096 tables, whose blank tail key
/// they pin, read by every kind of consumer.
#[test]
fn keyed_lookups_over_a_whole_column_key() {
    check(
        AGGREGATES,
        &[
            "--(VLOOKUP(A:A,$J$1:$K$6,2,FALSE)=10)",
            "VLOOKUP(A:A,$J$1:$K$6,2,FALSE)",
            "VLOOKUP(A:A,$J$1:$K$6,2)",
            "VLOOKUP(B:B,$J$1:$K$6,$H$1,FALSE)*0+1",
            "HLOOKUP(A:A,$N$1:$S$2,2,FALSE)",
            "MATCH(A:A,$J$1:$J$6,0)",
            "MATCH(C:C,$J$1:$J$6)",
            "MATCH(A:A,$L$1:$L$5,-1)",
            "MATCH(A:A,$L$1:$L$5,$H$1)",
            "MATCH(A:A,{0,1,\"x\"},0)",
            "MATCH(A:A,B:B,0)",
            "MATCH(A:A,Sheet2!$A$1:$A$9,1)",
        ],
    );
}

/// A bounded span, a key expression, a lookup under an operator and under IF (as a branch and
/// as the condition), nested lookups, and the failed-argument lift (`argument_error_each`).
#[test]
fn keyed_lookups_in_expressions() {
    check(
        &["SUM", "COUNT", "COUNTA", "MAXA"],
        &[
            "MATCH(A2:A70000,$J$1:$J$6,0)",
            "MATCH(A:A&\"\",$J$1:$J$6,0)",
            "MATCH(A:A,$J$1:$J$6,0)*2",
            "IF(A:A=1,MATCH(B:B,$J$1:$J$6,0),0)",
            "IF(MATCH(A:A,$J$1:$J$6,0)>1,1,0)",
            "MATCH(VLOOKUP(A:A,$J$1:$K$6,2,FALSE),$K$1:$K$6,0)",
            "VLOOKUP(A:A,$J$1:$K$6,\"x\")",
            "VLOOKUP(A:A,$H$2,2)",
            "VLOOKUP(A:A,$J$1:$K$6,$H$2)",
            "VLOOKUP(A:A,$J$1:$K$6,3)",
            "MATCH(A:A,$J$1:$J$6,\"x\")",
            "VLOOKUP(Sheet2!A:A,$J$1:$K$6,2)",
        ],
    );
    let consumers = [
        "=SUMPRODUCT(--(VLOOKUP(A:A,$J$1:$K$6,2)=10))",
        "=SMALL(MATCH(A:A,$J$1:$J$6,0),2)",
        "=ROWS(UNIQUE(MATCH(A:A&\"\",$J$1:$J$6,0)))",
        "=LOOKUP(2,1/(MATCH(A:A,$J$1:$J$6,0)>1),B:B)",
    ];
    let formulas: Vec<String> = consumers.iter().map(ToString::to_string).collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    assert_eq!(
        walks,
        formulas.len(),
        "every consumer reads its lookup tailed"
    );
}

/// A `ROW()` leaf in a key: the sentinel tail picked by the result leaves the argument to the
/// ordinary reading; an IF that does not pick it in the tail keeps it tailed.
#[test]
fn row_leaves_in_a_key() {
    let picked = ["=COUNT(MATCH(ROW(A:A),$L$1:$L$5,-1))".to_string()];
    let (results, walks) = both_ways(&picked);
    assert_eq!(results[0].1, results[0].2);
    assert_eq!(walks, 0);
    let not_picked = ["=SUM(IF(A:A=\"\",0,MATCH(ROW(A:A),$L$1:$L$5,-1)))".to_string()];
    let (results, walks) = both_ways(&not_picked);
    assert_eq!(results[0].1, results[0].2);
    assert_eq!(walks, 1);
}

/// Shapes a keyed lookup refuses (SPEC-0020 Amendment 2, section 1): an array `match_type` or
/// `range_lookup`, a function-call or named table, an operator in a fixed argument, a key that
/// reaches no leaf, and a multi-column key.
#[test]
fn keyed_lookup_shapes_that_are_not_tailed() {
    let formulas: Vec<String> = [
        "=COUNT(MATCH(A:A,$J$1:$J$6,{0;1}))",
        "=COUNTA(VLOOKUP(A:A,$J$1:$K$6,2,{TRUE;FALSE}))",
        "=COUNTA(VLOOKUP(A:A,$J$1:$K$6,{1,2}))",
        "=COUNT(MATCH(A:A,INDIRECT(\"J1:J6\"),0))",
        "=COUNTA(VLOOKUP(A:A,Tbl,2,FALSE))",
        "=COUNT(MATCH(A:A,$J$1:$J$6,0+0))",
        "=SUM(--(VLOOKUP(1,$J$1:$K$6,2,FALSE)=A:A))",
        "=COUNT(MATCH(A:B,$J$1:$J$6,0))",
        "=COUNT(XMATCH(A:A,$J$1:$J$6))",
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

/// `formula` in E1 of a model `setup` fills, evaluated with the tailed reading on and off: its
/// text, the arguments walked tailed, and the lift tally (kernel runs, visits charged).
fn tallied(setup: &dyn Fn(&mut Model), formula: &str) -> [(String, usize, (u64, u64)); 2] {
    [false, true].map(|off| {
        TAILED_OFF.with(|flag| flag.set(off));
        let mut model = new_empty_model();
        setup(&mut model);
        model._set("E1", formula);
        TAILED_WALKS.with(|count| count.set(0));
        LIFT_TALLY.with(|tally| tally.set((0, 0)));
        model.evaluate();
        TAILED_OFF.with(|flag| flag.set(false));
        (
            model._get_text("E1"),
            TAILED_WALKS.with(std::cell::Cell::get),
            LIFT_TALLY.with(std::cell::Cell::get),
        )
    })
}

/// Keys 1, 2, 2, 3, 3 in A1:A5; `last` in A6, the head's last row (Z6 keeps it used); the table
/// in J1:K4.
fn small_keys(model: &mut Model, last: &str) {
    for (row, key) in ["1", "2", "2", "3", "3", last].iter().enumerate() {
        model._set(&format!("A{}", row + 1), key);
    }
    model._set("Z6", "1");
    for row in 1..=4 {
        model._set(&format!("J{row}"), &row.to_string());
        model._set(&format!("K{row}"), &(row * 10).to_string());
    }
}

/// The continued lift (SPEC-0020 Amendment 2, section 2): a tail key equal to the head's last
/// key (a blank: the merged run) runs and charges nothing more, one that differs one walk; both
/// equal the one lift's kernel runs and visits, for each kernel (the search and the
/// failed-argument lift).
#[test]
fn the_continued_lift_charges_the_one_lift() {
    let formulas = [
        "=COUNTA(VLOOKUP(A:A,$J$1:$K$4,2,FALSE))",
        "=SUM(--(VLOOKUP(A:A,$J$1:$K$4,2)=10))",
        "=COUNT(MATCH(A:A,$J$1:$J$4,0))",
        "=COUNTA(HLOOKUP(A:A,$J$1:$K$4,2,FALSE))",
        "=COUNTA(VLOOKUP(A:A,$J$1:$K$4,\"x\"))",
    ];
    for (last, kind) in [("", "merged"), ("9", "unmerged")] {
        for formula in formulas {
            let [tailed, full] = tallied(&|model| small_keys(model, last), formula);
            assert_eq!(tailed.0, full.0, "{kind} {formula}");
            assert_eq!(tailed.1, 1, "{kind} {formula} is tailed");
            assert_eq!(tailed.2, full.2, "{kind} {formula}: runs and visits");
        }
    }
    // The merged run computes one element fewer than the unmerged: the tail reused the head's.
    let merged = tallied(&|model| small_keys(model, ""), formulas[0])[0].2;
    let unmerged = tallied(&|model| small_keys(model, "9"), formulas[0])[0].2;
    assert_eq!(merged.0, 4);
    assert_eq!(unmerged.0, 5);
}

/// Sorted keys 1..=32768 in Sheet2!A (a scan of 2^15 cells per walk, so 4,096 walks reach the
/// per-call cap of 2^27 visits) and on Sheet1 `distinct` distinct keys from A1, then a repeat of
/// the last one when `repeat` or else a blank, which Z keeps in the head.
fn keys_at_the_cap(model: &mut Model, distinct: i32, repeat: bool) {
    model.new_sheet();
    for row in 1..=32_768 {
        model._set(&format!("Sheet2!A{row}"), &row.to_string());
    }
    for row in 1..=distinct {
        model._set(&format!("A{row}"), &row.to_string());
    }
    let last = distinct + 1;
    if repeat {
        model._set(&format!("A{last}"), &distinct.to_string());
    }
    model._set(&format!("Z{last}"), "1");
}

/// The per-call cap at its boundary (SPEC-0020 Amendment 2, section 2): the head's walks and the
/// tail's one together, merged and not, fail exactly where the one lift fails.
#[test]
fn the_per_call_cap_at_its_boundary() {
    let formula = "=SUM(MATCH(A:A,Sheet2!$A$1:$A$32768,1)*0)";
    // (distinct keys, repeat, the one lift's walks, within the cap)
    let cases = [
        // 4,095 walks in the head, a tail key that differs: 4,096 walks.
        (4095, true, true),
        // 4,096 walks in the head, a tail key that differs: 4,097.
        (4096, true, false),
        // 4,096 walks in the head ending in a blank, the tail's merged: 4,096.
        (4095, false, true),
        // 4,097 walks in the head: the head fails, the ordinary reading too.
        (4096, false, false),
    ];
    for (distinct, repeat, within) in cases {
        let [tailed, full] = tallied(&|model| keys_at_the_cap(model, distinct, repeat), formula);
        let case = format!("{distinct} {repeat}");
        assert_eq!(tailed.0, full.0, "{case}");
        assert_eq!(full.0 != "#ERROR!", within, "{case}: {}", full.0);
        if within {
            assert_eq!(tailed.1, 1, "{case} is tailed");
            assert_eq!(tailed.2, full.2, "{case}: runs and visits");
        }
    }
}

/// A fixed one-cell reference that closes a cycle, or that is in a spill evaluated during the
/// head, reads as the ordinary reading reads it.
#[test]
fn fixed_references_in_a_cycle_or_a_spill() {
    for formula in [
        "=COUNT(MATCH(A:A,$J$1:$J$6,F1))",
        "=COUNTA(VLOOKUP(A:A,$J$1:$K$6,$G$2,FALSE))",
        "=COUNTA(VLOOKUP(A:A,$G$2,1,FALSE))",
    ] {
        let setup = |model: &mut Model| {
            fill(model);
            model._set("F1", "=E1");
            model._set("G1", "=SEQUENCE(3)");
        };
        let [tailed, full] = tallied(&setup, formula);
        assert_eq!(tailed.0, full.0, "{formula}");
    }
}

/// INDEX with scalar indices picks from a tailed argument (SPEC-0020 Amendment 2, section 3;
/// T-3222): rows in the head, in the tail and at the logical last row, in the three-argument
/// form (a value) and the two-argument form (a 1x1 array), over each kind of tailed argument.
#[test]
fn index_picks_from_a_tailed_argument() {
    let picks = ["3", "40", "41", "5000", "1048576"];
    let arguments = [
        "VLOOKUP(A:A,$J$1:$K$6,2,FALSE)",
        "MATCH(A:A,$J$1:$J$6,1)",
        "HLOOKUP(B:B,$N$1:$S$2,2)",
        "IF(A:A=\"\",\"blank\",A:A&\"!\")",
        "MATCH(A2:A70000,$L$1:$L$5,-1)",
        "ROW(A:A)*0+A:A",
    ];
    let mut formulas = Vec::new();
    for argument in arguments {
        for row in picks {
            formulas.push(format!("=INDEX({argument},{row})"));
            formulas.push(format!("=INDEX({argument},{row},1)"));
            formulas.push(format!("=ROWS(INDEX({argument},{row}))"));
            formulas.push(format!("=TYPE(INDEX({argument},{row},1,1))"));
        }
    }
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    // The `ROW()` argument's sentinel tail sends all its picks to the ordinary reading. The picks
    // past `MATCH(A2:A70000,..)`'s 69,999 rows read it tailed, then fall back too.
    let fallbacks = picks.len() * 4;
    assert_eq!(walks, formulas.len() - fallbacks);
}

/// INDEX keeps the ordinary reading for every other index (SPEC-0020 Amendment 2, section 3):
/// past the logical rows (`#REF!`), row 0, column 0 and 2, `area_num` 2, lifted indices, and an
/// error index before an error array; an error key element in the picked row is that error.
#[test]
fn index_forms_that_fall_back() {
    let formulas: Vec<String> = [
        "=INDEX(MATCH(A:A,$J$1:$J$6,0),1048577)",
        "=INDEX(MATCH(A:A,$J$1:$J$6,0),0)",
        "=ROWS(INDEX(MATCH(A:A,$J$1:$J$6,0),0,1))",
        "=INDEX(MATCH(A:A,$J$1:$J$6,0),5,0)",
        "=INDEX(MATCH(A:A,$J$1:$J$6,0),5,2)",
        "=INDEX(MATCH(A:A,$J$1:$J$6,0),5,1,2)",
        "=SUM(INDEX(MATCH(A:A,$J$1:$J$6,0),{1;2;5000}))",
        "=INDEX(MATCH(A:A,{\"\"},0)/0,#REF!)",
        "=INDEX(VLOOKUP(A:A,$H$2,2),5)",
        "=INDEX(IF(TRUE,MATCH(A:A,$J$1:$J$6,0)),5)",
        "=INDEX(XMATCH(A:A,$J$1:$J$6),5)",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let (results, walks) = both_ways(&formulas);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
    }
    // Only row 1048577 reads the tailed argument before falling back (past its rows), and the
    // table error lifted over the key (`$H$2`) is picked tailed; the other indices never read
    // it tailed, and the IF, lifted and XMATCH forms are not tailed.
    assert_eq!(walks, 2);
    let errors: Vec<String> = [
        // A6 is `=1/0`: the key element of the picked row is its own error.
        "=INDEX(VLOOKUP(A:A,$J$1:$K$6,2,FALSE),6)",
        "=INDEX(MATCH(A:A,$J$1:$J$6,0),6,1)",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let (results, walks) = both_ways(&errors);
    for (formula, tailed, full) in results {
        assert_eq!(tailed, full, "{formula}");
        assert_eq!(tailed, "#DIV/0!", "{formula}");
    }
    assert_eq!(walks, errors.len());
}
