// A single-range aggregate argument walks the cells stored in its range, not every position of
// the used area, when a far cell leaves that area mostly blank (T-1690, T-2710). The work is
// counted in cell reads (`Evaluation::cell_reads`), not wall-clock time (T-2282).

use crate::calc_result::{CalcResult, Range};
use crate::constants::LAST_ROW;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::RangeItem;
use crate::test::util::new_empty_model;

/// Evaluates `formula` in `at` and returns its text and the cells the evaluation read.
fn evaluate_counting_reads(model: &mut crate::Model, at: &str, formula: &str) -> (String, u64) {
    model._set(at, formula);
    let before = model.evaluation.cell_reads;
    model.evaluate();
    (model._get_text(at), model.evaluation.cell_reads - before)
}

#[test]
fn aggregates_beside_one_cell_in_the_last_row_read_a_few_cells() {
    let cases = [
        ("Z", "=SUM(A:Z)", "1"),
        ("CV", "=COUNTA(A:CV)", "1"),
        ("J", "=SUM(A:J)", "1"),
        ("J", "=PRODUCT(A:J)", "1"),
        ("J", "=AVERAGE(A:J)", "1"),
        ("J", "=AVERAGEA(A:J)", "1"),
        ("J", "=LARGE(A:J,1)", "1"),
        ("J", "=GCD(A:J)", "1"),
        ("J", &format!("=MIN(A1:J{LAST_ROW})"), "1"),
        ("J", &format!("=LCM(A1:J{LAST_ROW})"), "0"),
        ("J", "=MAX(A:J)", "1"),
        ("J", "=COUNT(A:J)", "1"),
        ("J", "=SUM(FREQUENCY(A:J,1))", "1"),
        ("J", "=AND(A:J)", "TRUE"),
        ("J", "=MEDIAN(A:J)", "1"),
        ("J", "=SUMSQ(A:J)", "1"),
        ("J", "=SUBTOTAL(9,A:J)", "1"),
        ("J", "=AGGREGATE(9,6,A:J)", "1"),
        ("J", "=COUNTA(1:1048576)", "1"),
        ("J", "=COUNTBLANK(A:J)", "10485759"),
        ("J", "=SUM(Sheet1:Sheet2!A:J)", "2"),
        ("J", "=SUM(BYROW(SEQUENCE(100),LAMBDA(r,SUM(A:J))))", "100"),
    ];
    for (column, formula, expected) in cases {
        let mut model = new_empty_model();
        model.new_sheet();
        model.new_sheet();
        model._set(&format!("Sheet1!{column}{LAST_ROW}"), "1");
        model._set(&format!("Sheet2!{column}{LAST_ROW}"), "1");
        // The formula sits on Sheet3, outside the ranges it reads on Sheet1.
        let formula = if formula.contains('!') {
            formula.to_string()
        } else {
            formula
                .replace("A:", "Sheet1!A:")
                .replace("A1:", "Sheet1!A1:")
                .replace("(1:", "(Sheet1!1:")
        };
        let (value, reads) = evaluate_counting_reads(&mut model, "Sheet3!A1", &formula);
        assert_eq!(value, expected, "{formula}");
        assert!(reads < 1000, "{formula} read {reads} cells");
    }
}

#[test]
fn a_3d_textjoin_counts_the_blanks_it_does_not_read() {
    // TEXTJOIN's 3-D path reads a blank cell as an empty value with ignore_empty FALSE: each adds
    // a delimiter. A run of them is counted, not read, so each sheet costs its two cells.
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_sheet();
    for sheet in ["Sheet1", "Sheet2"] {
        model._set(&format!("{sheet}!B1"), "1");
        model._set(&format!("{sheet}!J{LAST_ROW}"), "1");
    }
    let cases = [
        ("=LEN(TEXTJOIN(\"\",FALSE,Sheet1:Sheet2!A:J))", "4"),
        ("=TEXTJOIN(\",\",TRUE,Sheet1:Sheet2!A:J)", "1,1,1,1"),
        (
            &format!("=TEXTJOIN(\",\",TRUE,Sheet1:Sheet2!A1:J{LAST_ROW})") as &str,
            "1,1,1,1",
        ),
        // 2 x 10 x 1048576 values make the join far longer than 32,767 characters.
        ("=TEXTJOIN(\",\",FALSE,Sheet1:Sheet2!A:J)", "#CALC!"),
    ];
    for (formula, expected) in cases {
        let (value, reads) = evaluate_counting_reads(&mut model, "Sheet3!A1", formula);
        assert_eq!(value, expected, "{formula}");
        assert!(reads < 1000, "{formula} read {reads} cells");
    }
}

#[test]
fn the_whole_range_walk_hands_blank_runs_on_in_row_major_order() {
    // On a sheet holding A1, C1, A2 and J1048576, `A:J` is "a", one blank, "c", seven blanks,
    // "d", then blanks up to the last cell, each run handed on as its count.
    let mut model = new_empty_model();
    model._set("A1", "a");
    model._set("C1", "c");
    model._set("A2", "d");
    model._set(&format!("J{LAST_ROW}"), "z");
    model.evaluate();
    let corner = |row, column| CellReferenceIndex {
        sheet: 0,
        row,
        column,
    };
    let range = Range {
        left: corner(1, 1),
        right: corner(LAST_ROW, 10),
    };
    let mut items = Vec::new();
    let before = model.evaluation.cell_reads;
    let walked = model.for_each_cell_or_error_in_whole_range(&range, corner(1, 20), |_, item| {
        items.push(match item {
            RangeItem::Cell(CalcResult::String(text)) => text.into_lossy_string(),
            RangeItem::Cell(_) => "?".to_string(),
            RangeItem::Blanks(count) => count.to_string(),
        });
        Ok(())
    });
    assert!(walked.is_ok());
    // The rest of row 2, rows 3 to 1048575, and the last row up to J.
    let rest = 9 + (i64::from(LAST_ROW) - 3) * 10 + 9;
    assert_eq!(
        items,
        ["a", "1", "c", "7", "d", &rest.to_string(), "z"].map(String::from)
    );
    assert!(model.evaluation.cell_reads - before < 10);
}
