#![allow(clippy::unwrap_used)]

// Functions that walk every cell of a range read only the used area of a
// whole-row or whole-column argument (`clip_to_used_area`); the cells past it
// are empty, so the value is the one the used-area range gives (T-0048).

use std::time::{Duration, Instant};

use crate::test::util::new_empty_model;

//         ║   A   |   B   |   C   |
// ════════╬═══════╪═══════╪═══════╪
//    1    ║   2   |   1   |       |
//    2    ║   3   |       |   4   |
//    3    ║       |   5   | "x"   |
fn sample() -> crate::Model {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("B1", "1");
    model._set("A2", "3");
    model._set("C2", "4");
    model._set("B3", "5");
    model._set("C3", "x");
    // The formulas sit on the second sheet, outside the whole-sheet ranges.
    model.new_sheet();
    model
}

fn same_as_used_area(function: &str, whole: &str, used: &str) {
    let mut model = sample();
    model._set("Sheet2!E1", &format!("={function}(Sheet1!{whole})"));
    model._set("Sheet2!E2", &format!("={function}(Sheet1!{used})"));
    let start = Instant::now();
    model.evaluate();
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "{function}({whole}) was not clipped"
    );
    assert_eq!(
        model._get_text("Sheet2!E1"),
        model._get_text("Sheet2!E2"),
        "{function}({whole})"
    );
}

#[test]
fn multinomial_over_whole_rows() {
    same_as_used_area("MULTINOMIAL", "1:1", "A1:C1");
}

#[test]
fn networkdays_holidays_over_whole_column() {
    let mut model = new_empty_model();
    model._set("A1", "45300");
    model._set("A2", "45301");
    model.new_sheet();
    model._set("Sheet2!C1", "=NETWORKDAYS(45292,45330,Sheet1!A:A)");
    model._set("Sheet2!C2", "=NETWORKDAYS(45292,45330,Sheet1!A1:A2)");
    let start = Instant::now();
    model.evaluate();
    assert!(start.elapsed() < Duration::from_secs(10));
    assert_eq!(model._get_text("Sheet2!C1"), "27");
    assert_eq!(model._get_text("Sheet2!C2"), "27");
}

#[test]
fn count_family_over_whole_sheet_ranges() {
    same_as_used_area("COUNT", "A:XFD", "A1:C3");
    same_as_used_area("COUNTA", "A:XFD", "A1:C3");
    same_as_used_area("AVERAGEA", "A:XFD", "A1:C3");
    same_as_used_area("MAXA", "1:1048576", "A1:C3");
}

#[test]
fn countblank_counts_the_clipped_cells() {
    let mut model = sample();
    model._set("Sheet2!E1", "=COUNTBLANK(Sheet1!A:A)");
    model._set("Sheet2!E2", "=COUNTBLANK(Sheet1!1:1)");
    model._set("Sheet2!E3", "=COUNTBLANK(Sheet1!A1:C3)");
    model.evaluate();
    // Column A holds two values; row 1 holds two values; A1:C3 holds six.
    assert_eq!(model._get_text("Sheet2!E1"), "1048574");
    assert_eq!(model._get_text("Sheet2!E2"), "16382");
    assert_eq!(model._get_text("Sheet2!E3"), "3");
}

// Clipping costs O(1) per walk, not a visit of every cell of the sheet
// (T-0049): 10,000 walks of a whole column over a wide sheet of 160,000
// cells would visit 1.6 billion cells if each one scanned the sheet (about
// 30 s unoptimised; a fraction of a second without the scan).
#[test]
fn whole_column_walks_do_not_scan_the_sheet() {
    let mut model = new_empty_model();
    for row in 1..=10 {
        for column in 1..=16_000 {
            model
                .set_user_input(0, row, column, "1".to_string())
                .unwrap();
        }
    }
    model.new_sheet();
    for row in 1..=10_000 {
        model
            .set_user_input(1, row, 1, "=COUNT(Sheet1!A:A)".to_string())
            .unwrap();
    }
    let start = Instant::now();
    model.evaluate();
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "a whole-column walk scanned the sheet"
    );
    assert_eq!(model._get_text("Sheet2!A10000"), "10");
}

#[test]
fn used_area_end_over_approximates_and_follows_writes() {
    let mut model = new_empty_model();
    let end = |model: &crate::Model| model.workbook.worksheet(0).unwrap().used_area_end();
    // An empty sheet ends at A1, as `dimension` says.
    assert_eq!(end(&model), (1, 1));
    model._set("C7", "1");
    model._set("E2", "1");
    assert_eq!(end(&model), (7, 5));
    let dimension = model.workbook.worksheet(0).unwrap().dimension();
    assert_eq!(end(&model), (dimension.max_row, dimension.max_column));
    model
        .workbook
        .worksheet_mut(0)
        .unwrap()
        .remove_cell(7, 3)
        .unwrap();
    assert_eq!(end(&model), (2, 5));
}

// CONCAT walks a range through the shared clipped walk, so a whole column or the whole sheet
// costs the used cells and joins the same text as the used-area range (T-0261).
#[test]
fn concat_over_whole_sheet_ranges() {
    same_as_used_area("CONCAT", "A:A", "A1:A3");
    same_as_used_area("CONCAT", "1:1", "A1:C1");
    same_as_used_area("CONCAT", "A:XFD", "A1:C3");
    let mut model = sample();
    model._set("Sheet2!E1", "=CONCAT(Sheet1!A:XFD)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), "21345x");
}

// AND, OR and XOR ignore empty cells (T-0230). Sheet1 holds no logical values but
// numbers: 2, 1, 3, 4, 5 are all TRUE.
#[test]
fn and_or_xor_over_whole_sheet_and_columns() {
    for function in ["AND", "OR", "XOR"] {
        same_as_used_area(function, "A:XFD", "A1:C3");
        same_as_used_area(function, "1:1048576", "A1:C3");
        same_as_used_area(function, "B:B", "B1:B3");
        same_as_used_area(function, "2:3", "A2:C3");
    }
}

// A value set later beyond the used area is seen by the clipped walk: the
// skipped cells were recorded as read empty.
#[test]
fn and_over_whole_column_sees_later_cell() {
    let mut model = sample();
    model._set("Sheet2!E1", "=AND(Sheet1!B:B)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), *"TRUE");
    model._set("Sheet1!B1000", "FALSE");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), *"FALSE");
}

// The order statistics, MODE, FREQUENCY and the one-list tests keep only the numbers of a range,
// so they walk its used area like SUM. Before T-1681 they built one entry per cell, and
// `PERCENTRANK.INC(SE:RCH,2)` (7,000+ whole columns) aborted on a 3 GB allocation
// (nightly fuzz, formula_eval).
#[test]
fn order_statistics_over_whole_columns() {
    for function in [
        "PERCENTILE.INC({},0.5)",
        "PERCENTRANK.INC({},2)",
        "QUARTILE({},1)",
        "MODE.SNGL({})",
        "Z.TEST({},2)",
        "F.TEST({},{})",
        "PROB({},{},1)",
        "RANK.EQ(2,{})",
    ] {
        same_formula_as_used_area(function, "A:XFD", "A1:C3");
        same_formula_as_used_area(function, "A:C", "A1:C3");
    }
    same_formula_as_used_area("SUM(FREQUENCY({},2))", "A:XFD", "A1:C3");
}

// The paired functions match two lists by position; past the used area every pair is blank, so
// the lists stop there and the result is the full lists' (T-1681).
#[test]
fn paired_functions_over_whole_columns() {
    for function in [
        "SUMXMY2({0},{1})",
        "SUMX2PY2({0},{1})",
        "COVARIANCE.P({0},{1})",
        "COVARIANCE.S({0},{1})",
        "PEARSON({0},{1})",
        "SLOPE({0},{1})",
        "T.TEST({0},{1},2,1)",
        "T.TEST({0},{1},2,3)",
    ] {
        let paired = |a: &str, b: &str| function.replace("{0}", a).replace("{1}", b);
        same_as(
            &paired("Sheet1!A:A", "Sheet1!B:B"),
            &paired("Sheet1!A1:A3", "Sheet1!B1:B3"),
        );
        same_as(
            &paired("Sheet1!A:B", "Sheet1!B:C"),
            &paired("Sheet1!A1:B3", "Sheet1!B1:C3"),
        );
        // A whole column against the same column written out: same size, one clipped.
        same_as(
            &paired("Sheet1!A:A", "Sheet1!B1:B1048576"),
            &paired("Sheet1!A1:A3", "Sheet1!B1:B3"),
        );
        // A whole column against a column of another sheet, whose used area differs.
        same_as(
            &paired("Sheet1!A:A", "Sheet2!Z:Z"),
            &paired("Sheet1!A1:A3", "Sheet2!Z1:Z3"),
        );
    }
}

// CHISQ.TEST's degrees of freedom count every cell of the ranges, blank or not, so a whole column
// is not its used area; the blank pairs past it add nothing to the statistic. The expectation is
// the engine's behavior before T-1681, derived by hand, not oracle evidence (probe: T-1689).
#[test]
fn chisq_test_over_whole_columns_counts_every_cell() {
    // One numeric pair, (2, 1): the statistic is 1.
    same_as(
        "CHISQ.TEST(Sheet1!A:A,Sheet1!B:B)",
        "CHISQ.DIST.RT(1,1048575)",
    );
    same_as(
        "CHISQ.TEST(Sheet1!A:B,Sheet1!B:C)",
        "CHISQ.DIST.RT(1,1048575)",
    );
}

#[test]
fn paired_functions_keep_size_mismatch_and_late_errors() {
    // Different sizes are #N/A however far the data reaches.
    same_as("SUMXMY2(Sheet1!A:A,Sheet1!B1:B3)", "NA()");
    same_as("COVARIANCE.P(Sheet1!A:A,Sheet1!B:C)", "NA()");
    // An error past the paired positions of the longer argument is still found first by T.TEST.
    // SUMXMY2 checks the counts before any error inside a range (coercion.md "SUMX2MY2 / SUMX2PY2
    // / SUMXMY2 pairing", probes/sumx-blank-text-error-order G7, G23).
    let mut model = sample();
    model._set("A9", "=1/0");
    model._set("Sheet2!E1", "=SUMXMY2(Sheet1!A:A,Sheet1!B1:B3)");
    model._set("Sheet2!E2", "=T.TEST(Sheet1!A:A,Sheet1!B1:B3,2,2)");
    model._set("Sheet2!E3", "=T.TEST(Sheet1!B1:B3,Sheet1!A:A,2,2)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), "#N/A");
    for cell in ["Sheet2!E2", "Sheet2!E3"] {
        assert_eq!(model._get_text(cell), "#DIV/0!", "{cell}");
    }
}

// The fuzz input itself, on the fuzz target's almost empty sheet: whole columns SE to RCH.
#[test]
fn percentrank_over_thousands_of_whole_columns_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    for formula in [
        "=PERCENTRANK.INC(SE:RCH,\"\u{130}\u{130}x\",2)",
        "=PERCENTRANK.INC(SE:RCH,2)",
        "=PERCENTILE.INC(SE:RCH,0.5)",
        "=SUMXMY2(SE:RCH,SE:RCH)",
        "=COVARIANCE.P(SE:RCH,SF:RCI)",
        "=CHISQ.TEST(SE:RCH,SE:RCH)",
    ] {
        model._set("C3", formula);
        let start = Instant::now();
        model.evaluate();
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "{formula} was not bounded"
        );
    }
}

/// `formula` with `{}` standing for the whole range, against the same with the used range.
fn same_formula_as_used_area(formula: &str, whole: &str, used: &str) {
    same_as(
        &formula.replace("{}", &format!("Sheet1!{whole}")),
        &formula.replace("{}", &format!("Sheet1!{used}")),
    );
}

/// Evaluates two formulas over the sample sheet, quickly, and checks they agree.
fn same_as(formula: &str, expected: &str) {
    let mut model = sample();
    model._set("Sheet2!Z1", "7");
    model._set("Sheet2!Z2", "8");
    model._set("Sheet2!Z3", "6");
    model._set("Sheet2!Z9", "x");
    model._set("Sheet2!E1", &format!("={formula}"));
    model._set("Sheet2!E2", &format!("={expected}"));
    let start = Instant::now();
    model.evaluate();
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "{formula} was not clipped"
    );
    assert_eq!(
        model._get_text("Sheet2!E1"),
        model._get_text("Sheet2!E2"),
        "{formula}"
    );
}

// One far cell stretches the used area to the whole width of the ranges and 5,000 rows;
// the lists hold the sheet's few non-blank cells, not a slot per cell of that area (T-1681
// review: a crafted workbook must not turn a used-area walk into an allocation per cell).
#[test]
fn a_far_cell_does_not_stretch_the_lists() {
    let mut model = sample();
    model._set("CV5000", "x");
    for (formula, expected) in [
        ("SUMXMY2(A:CV,B:CW)", "SUMXMY2(A1:C3,B1:D3)"),
        ("SUMXMY2(1:5000,1:5000)", "SUMXMY2(A1:C3,A1:C3)"),
        ("COVARIANCE.P(A:CV,A:CV)", "COVARIANCE.P(A1:C3,A1:C3)"),
        ("PEARSON(A:CV,B:CW)", "PEARSON(A1:C3,B1:D3)"),
        ("T.TEST(A:CV,B:CW,2,1)", "T.TEST(A1:C3,B1:D3,2,1)"),
        ("PERCENTILE.INC(A:CV,0.5)", "PERCENTILE.INC(A1:C3,0.5)"),
    ] {
        model._set("Sheet2!E1", &format!("={}", qualify(formula)));
        model._set("Sheet2!E2", &format!("={}", qualify(expected)));
        let start = Instant::now();
        model.evaluate();
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "{formula} was not bounded"
        );
        assert_eq!(
            model._get_text("Sheet2!E1"),
            model._get_text("Sheet2!E2"),
            "{formula}"
        );
    }
}

/// `formula` with every range argument read from Sheet1.
fn qualify(formula: &str) -> String {
    let (name, args) = formula.split_once('(').unwrap();
    let args: Vec<String> = args
        .trim_end_matches(')')
        .split(',')
        .map(|arg| {
            if arg.contains(':') {
                format!("Sheet1!{arg}")
            } else {
                arg.to_string()
            }
        })
        .collect();
    format!("{name}({})", args.join(","))
}

/// Sheets Sheet1..Sheet50, each holding a number in A2 and Z3; the formulas go on a sheet after
/// them.
fn fifty_sheets() -> crate::Model {
    let mut model = new_empty_model();
    for _ in 2..=50 {
        model.new_sheet();
    }
    for index in 1..=50 {
        model._set(&format!("Sheet{index}!A2"), &index.to_string());
        model._set(&format!("Sheet{index}!Z3"), "1");
    }
    model.add_sheet("Out").unwrap();
    model
}

// A 3-D span walks each sheet's used area only, for a tall explicit block as for a whole column
// (T-0405): unclipped, the block below is 27 million cells on each of 50 sheets.
#[test]
fn three_d_walks_cost_the_data_of_each_sheet() {
    let mut model = fifty_sheets();
    model._set("Out!A1", "=SUM(Sheet1:Sheet50!A:A)");
    model._set("Out!A2", "=SUM(Sheet1:Sheet50!A2:Z1048575)");
    model._set("Out!A3", "=COUNTA(Sheet1:Sheet50!B1:Z1048576)");
    model._set("Out!A4", "=COUNTBLANK(Sheet1!A1:Z1048576)");
    let start = Instant::now();
    model.evaluate();
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "a 3-D block walk was not clipped"
    );
    assert_eq!(model._get_text("Out!A1"), "1275");
    assert_eq!(model._get_text("Out!A2"), "1325");
    assert_eq!(model._get_text("Out!A3"), "50");
    assert_eq!(model._get_text("Out!A4"), "27262974");
}

// A block that lies wholly past the used area walks nothing, and its cells are on record as
// read empty: a value set there later is seen. The blocks are tall enough to be cut
// (`MIN_CLIPPED_CELLS`); a short one is walked cell by cell.
#[test]
fn block_past_the_used_area_sees_later_cells() {
    let mut model = sample();
    model._set("Sheet2!E1", "=SUM(Sheet1!D5:F2000)");
    model._set("Sheet2!E2", "=SUM(Sheet1!B2:D2000)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), "0");
    assert_eq!(model._get_text("Sheet2!E2"), "9");
    model._set("Sheet1!F2000", "7");
    model._set("Sheet1!D1999", "1");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), "8");
    assert_eq!(model._get_text("Sheet2!E2"), "10");
}

// A spill into the clipped part of a block restarts the walk's formula: D1's spill is written
// after A1 read D1:D2000, whose used area was row 1 then.
#[test]
fn spill_into_a_clipped_block_is_seen() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(D1:D2000)");
    model._set("D1", "=SEQUENCE(10)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "55");
}

// The walkers that read a blank cell still see the blanks past the used area of an explicit
// block: GCD and LCM read one as 0 (coercion.md), TEXTJOIN with ignore_empty FALSE adds a
// delimiter for it, here on each sheet of a 3-D span.
#[test]
fn walkers_reading_blanks_see_the_whole_block() {
    let mut model = sample();
    model._set("Sheet2!E1", "=LCM(Sheet1!A1:A5)");
    model._set("Sheet2!E2", "=GCD(Sheet1!A1:A5)");
    model._set("Sheet2!E3", "=TEXTJOIN(\",\",FALSE,Sheet1:Sheet1!A1:A5)");
    model._set("Sheet2!E4", "=TEXTJOIN(\",\",FALSE,Sheet1!A1:A5)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), "0");
    assert_eq!(model._get_text("Sheet2!E2"), "1");
    assert_eq!(model._get_text("Sheet2!E3"), "2,3,,,");
    assert_eq!(model._get_text("Sheet2!E4"), "2,3,,,");
}

// Short blocks past the used area are walked cell by cell, not recorded as rectangles: a LAMBDA
// reading more of them than the seen-range cap allows rectangles is not `#VALUE!` (T-0405).
#[test]
fn many_short_blocks_do_not_reach_the_seen_range_cap() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set(
        "C1",
        "=SUM(MAKEARRAY(70000,1,LAMBDA(r,c,SUM(OFFSET($A$1,r,0,2,2)))))",
    );
    model.evaluate();
    assert_eq!(model._get_text("C1"), "0");
}

// One cell at XFD1048576 stretches the used area to the whole sheet, 1.7e10 positions. A single
// range that is mostly blank is walked over its existing cells, as a union's areas are, so these
// cost the sheet's few cells, not the area (T-1690; before it, each walked every position).
#[test]
fn a_far_cell_does_not_stretch_single_range_walks() {
    let mut model = sample();
    model._set("XFD1048576", "10");
    for (formula, expected) in [
        ("SUM(Sheet1!A:XFD)", "SUM(Sheet1!A1:C3)+10"),
        ("COUNT(Sheet1!A:XFD)", "COUNT(Sheet1!A1:C3)+1"),
        ("COUNTA(Sheet1!1:1048576)", "COUNTA(Sheet1!A1:C3)+1"),
        (
            "COUNTBLANK(Sheet1!A:XFD)",
            "17179869184-COUNTA(Sheet1!A1:C3)-1",
        ),
        ("SUMXMY2(Sheet1!A:XFD,Sheet1!A:XFD)", "0"),
        ("MAX(Sheet1!A:XFD)", "10"),
        ("SUBTOTAL(9,Sheet1!A:XFD)", "SUM(Sheet1!A1:C3)+10"),
        ("AGGREGATE(4,6,Sheet1!A:XFD)", "10"),
        ("LCM(Sheet1!A:XFD)", "LCM(Sheet1!A1:C3,0)"),
        ("CONCAT(Sheet1!A:XFD)", "CONCAT(Sheet1!A1:C3)&10"),
    ] {
        model._set("Sheet2!E1", &format!("={formula}"));
        model._set("Sheet2!E2", &format!("={expected}"));
        let start = Instant::now();
        model.evaluate();
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "{formula} was not bounded"
        );
        assert_eq!(
            model._get_text("Sheet2!E1"),
            model._get_text("Sheet2!E2"),
            "{formula}"
        );
    }
}

// A sparse single range finds the same first error in row-major order as the walk position by
// position.
#[test]
fn a_sparse_single_range_finds_the_first_error_in_row_major_order() {
    let mut model = sample();
    model._set("XFD1048576", "10");
    // Column-major order would meet B5 first.
    model._set("B5", "=1/0");
    model._set("Z4", "=NA()");
    model._set("Sheet2!E1", "=SUM(Sheet1!A:XFD)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), "#N/A");
}

// A single range walked over its existing cells does not draw on the formula's union walk budget
// (2^24 steps): 200 walks of 100,001 cells would spend it, but the dense walk they replace never
// did, so the value stays (T-1690 review).
#[test]
fn a_repeated_sparse_single_range_does_not_spend_the_union_budget() {
    let mut model = new_empty_model();
    for row in 1..=100_000 {
        model._set(&format!("A{row}"), "1");
    }
    model._set("Z1", "1");
    model.new_sheet();
    model._set(
        "Sheet2!A1",
        "=SUM(BYROW(SEQUENCE(200),LAMBDA(r,SUM(Sheet1!A:Z))))",
    );
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!A1"), "20000200");
}

// Either side of the size at which a mostly blank single range is walked over its existing cells
// (65,536 positions), the values are the same.
#[test]
fn single_ranges_either_side_of_the_sparse_walk_size_agree() {
    let mut model = sample();
    // 256 rows by 256 columns is 65,536 positions; one row fewer is 65,280.
    model._set("IV256", "10");
    model._set("Sheet2!E1", "=SUM(Sheet1!A1:IV256)");
    model._set("Sheet2!E2", "=SUM(Sheet1!A1:IV255)");
    model._set("Sheet2!E3", "=COUNTA(Sheet1!A1:IV256)");
    model._set("Sheet2!E4", "=COUNTA(Sheet1!A1:IV255)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!E1"), "25");
    assert_eq!(model._get_text("Sheet2!E2"), "15");
    assert_eq!(model._get_text("Sheet2!E3"), "7");
    assert_eq!(model._get_text("Sheet2!E4"), "6");
}
