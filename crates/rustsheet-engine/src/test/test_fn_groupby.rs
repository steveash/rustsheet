#![allow(clippy::unwrap_used)]

//! GROUPBY (SPEC-0006 step 1, T-1322). Excel: probes/groupby-pivotby sheet G and its follow-ups
//! (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY").

use crate::test::util::new_empty_model;
use crate::Model;

/// The Region/Sales records of probes/groupby-pivotby's Data sheet (A1:B11, row 1 headers; A9 is
/// blank and A6 is a lowercase `east`).
fn region_sales() -> Model {
    let mut model = new_empty_model();
    let rows = [
        ("Region", "Sales"),
        ("East", "10"),
        ("West", "20"),
        ("East", "30"),
        ("West", "40"),
        ("east", "5"),
        ("North", "7"),
        ("West", "0"),
        ("", "9"),
        ("South", "-3"),
        ("West", "12.5"),
    ];
    for (i, (region, sales)) in rows.iter().enumerate() {
        if !region.is_empty() {
            model._set(&format!("A{}", i + 1), region);
        }
        model._set(&format!("B{}", i + 1), sales);
    }
    model
}

/// The texts of column `column`, rows `first..=last`.
fn column(model: &Model, column: &str, first: usize, last: usize) -> Vec<String> {
    (first..=last)
        .map(|row| model._get_text(&format!("{column}{row}")))
        .collect()
}

#[test]
fn groups_sorted_case_folded_with_blank_last_and_total() {
    // G!A1: East and east are one group shown with the first spelling; the blank key is its own
    // group, shown as 0 and sorted last; the total covers every value.
    let mut model = region_sales();
    model._set("D1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)))");
    model.evaluate();
    assert_eq!(
        column(&model, "D", 1, 7),
        ["East", "North", "South", "West", "0", "Total", ""]
    );
    assert_eq!(
        column(&model, "E", 1, 7),
        ["45", "7", "-3", "72.5", "9", "130.5", ""]
    );
    assert_eq!(model._get_text("F1"), "");
}

#[test]
fn function_inputs_keep_source_order_and_total_uses_all_values() {
    // G!S73 (AVERAGE total 13.05 = 130.5/10, not the mean of the group means), G!A121 (ROWS),
    // G!AB97 (ARRAYTOTEXT keeps source order).
    let mut model = region_sales();
    model._set("D1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,AVERAGE(x)))");
    model._set("G1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,ROWS(x)))");
    model._set("J1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,ARRAYTOTEXT(x)))");
    model.evaluate();
    assert_eq!(
        column(&model, "E", 1, 6),
        ["15", "7", "-3", "18.125", "9", "13.05"]
    );
    assert_eq!(column(&model, "H", 1, 6), ["3", "1", "1", "4", "1", "10"]);
    assert_eq!(model._get_text("K1"), "10, 30, 5");
}

#[test]
fn mixed_key_types_sort_numbers_text_booleans_blank() {
    // G!A193: numbers < text < FALSE < TRUE < blank (shown as 0); `b` and `B` are one group.
    let mut model = new_empty_model();
    let keys = ["2", "b", "TRUE", "1", "a", "FALSE", "2", "", "1.5", "B"];
    let values = ["10", "20", "30", "40", "5", "7", "0", "9", "-3", "12.5"];
    for (i, (key, value)) in keys.iter().zip(values).enumerate() {
        if !key.is_empty() {
            model._set(&format!("A{}", i + 1), key);
        }
        model._set(&format!("B{}", i + 1), value);
    }
    model._set("D1", "=GROUPBY(A1:A10,B1:B10,LAMBDA(x,SUM(x)))");
    model.evaluate();
    assert_eq!(
        column(&model, "D", 1, 9),
        ["1", "1.5", "2", "a", "b", "FALSE", "TRUE", "0", "Total"]
    );
    assert_eq!(
        column(&model, "E", 1, 9),
        ["40", "-3", "10", "5", "32.5", "7", "30", "9", "130.5"]
    );
}

#[test]
fn number_keys_compare_exactly() {
    // computed -0 are one group (G!A781); 1E-17 is not 0; 1 and 1+1E-15 are two groups.
    // -0 are one group; 1E-17 is not 0; 1 and 1+1E-15 are two groups.
    let mut model = new_empty_model();
    let keys = ["=0.1+0.2", "0.3", "0", "=-0", "=1E-17", "1", "=1+1E-15"];
    for (i, key) in keys.iter().enumerate() {
        model._set(&format!("A{}", i + 1), key);
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }
    model._set("D1", "=GROUPBY(A1:A7,B1:B7,LAMBDA(x,SUM(x)))");
    model._set("G1", "=INDEX(D1#,4,1)=0.1+0.2");
    model.evaluate();
    assert_eq!(
        column(&model, "E", 1, 7),
        ["7", "5", "2", "1", "6", "7", "28"]
    );
    assert_eq!(model._get_text("D7"), "Total");
    assert_eq!(model._get_text("G1"), "TRUE");
}

#[test]
fn total_row_fillers_are_empty_text_and_blank_key_is_zero() {
    // G!AB25 and G!AC33: the second key cell of the Total row is "" (text), the blank key group
    // shows the number 0.
    let mut model = region_sales();
    for row in 2..=11 {
        model._set(&format!("C{row}"), "k");
    }
    model._set(
        "E20",
        "=GROUPBY(HSTACK(A2:A11,C2:C11),B2:B11,LAMBDA(x,SUM(x)))",
    );
    model._set("J1", "=ISTEXT(INDEX(E20#,6,2))");
    model._set("J2", "=INDEX(E20#,6,2)");
    model._set("J3", "=ISNUMBER(INDEX(E20#,5,1))");
    model._set("J4", "=INDEX(E20#,5,1)");
    model.evaluate();
    assert_eq!(
        column(&model, "E", 20, 25),
        ["East", "North", "South", "West", "0", "Total"]
    );
    assert_eq!(model._get_text("F25"), "");
    assert_eq!(column(&model, "J", 1, 4), ["TRUE", "", "TRUE", "0"]);
}

#[test]
fn several_key_and_value_columns() {
    // G!J73: value columns aggregate side by side.
    let mut model = region_sales();
    for row in 2..=11 {
        model._set(&format!("C{row}"), "1");
    }
    model._set("E1", "=GROUPBY(A2:A11,B2:C11,LAMBDA(x,SUM(x)))");
    model.evaluate();
    assert_eq!(column(&model, "G", 1, 6), ["3", "1", "1", "4", "1", "10"]);
}

#[test]
fn field_headers() {
    let mut model = region_sales();
    // Omitted: row 1 is detected as headers and hidden (G!J1).
    model._set("D1", "=GROUPBY(A1:A11,B1:B11,LAMBDA(x,SUM(x)))");
    // 0: no headers, "Region" is a group summing the text "Sales" to 0 (G!S1).
    model._set("G1", "=GROUPBY(A1:A11,B1:B11,LAMBDA(x,SUM(x)),0)");
    // 1: headers, not shown (G!AB1).
    model._set("J1", "=GROUPBY(A1:A11,B1:B11,LAMBDA(x,SUM(x)),1)");
    // 2: no headers, generated ones shown (G!A25).
    model._set("M1", "=GROUPBY(A1:A11,B1:B11,LAMBDA(x,SUM(x)),2)");
    // 3: headers shown (G!J25); over data without headers it consumes the first record (G!S25).
    model._set("P1", "=GROUPBY(A1:A11,B1:B11,LAMBDA(x,SUM(x)),3)");
    model._set("S1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)),3)");
    // 4 is #VALUE! (G!A217).
    model._set("V1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)),4)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "East");
    assert_eq!(model._get_text("E6"), "130.5");
    assert_eq!(
        column(&model, "G", 1, 7),
        ["East", "North", "Region", "South", "West", "0", "Total"]
    );
    assert_eq!(model._get_text("H3"), "0");
    assert_eq!(model._get_text("J1"), "East");
    assert_eq!(model._get_text("M1"), "Row Field 1");
    assert_eq!(model._get_text("N1"), "Value 1");
    assert_eq!(model._get_text("M4"), "Region");
    assert_eq!(column(&model, "P", 1, 2), ["Region", "East"]);
    assert_eq!(model._get_text("Q1"), "Sales");
    assert_eq!(column(&model, "S", 1, 2), ["East", "East"]);
    assert_eq!(
        column(&model, "T", 1, 7),
        ["10", "35", "7", "-3", "72.5", "9", "120.5"]
    );
    assert_eq!(model._get_text("V1"), "#VALUE!");
}

#[test]
fn header_auto_detect_looks_at_the_first_values_only() {
    // probes/groupby-pivotby-followup3 G!A145-A421: text over non-text is a header, all text is
    // data, text over mixed rows is #VALUE!.
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=GROUPBY({"k";"a";"b";"a"},{"v";1;2;3},LAMBDA(x,SUM(x)))"#,
    );
    model._set(
        "D1",
        r#"=GROUPBY({"k";"a";"b";"a"},{5;1;2;3},LAMBDA(x,SUM(x)))"#,
    );
    model._set(
        "G1",
        r#"=GROUPBY({"k";"a";"b";"a"},{"v";"1";"2";"3"},LAMBDA(x,SUM(x)))"#,
    );
    model._set(
        "J1",
        r#"=GROUPBY({"k";"a";"b";"a"},{"v";1;2;"3"},LAMBDA(x,SUM(x)))"#,
    );
    model.evaluate();
    assert_eq!(column(&model, "A", 1, 3), ["a", "b", "Total"]);
    assert_eq!(column(&model, "B", 1, 3), ["4", "2", "6"]);
    assert_eq!(column(&model, "D", 1, 4), ["a", "b", "k", "Total"]);
    assert_eq!(column(&model, "E", 1, 4), ["4", "2", "5", "11"]);
    assert_eq!(column(&model, "G", 1, 4), ["a", "b", "k", "Total"]);
    assert_eq!(column(&model, "H", 1, 4), ["0", "0", "0", "0"]);
    assert_eq!(model._get_text("J1"), "#VALUE!");
}

#[test]
fn total_depth() {
    let mut model = region_sales();
    // 0: no totals (G!A49); -1: the total on top (G!AB49); 3 is #VALUE!, so is 2 with a single key
    // field (probes/groupby-pivotby-followup G!A101, G!A161).
    model._set("D1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)),,0)");
    model._set("G1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)),,-1)");
    model._set("J1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)),,3)");
    model._set("J2", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)),,2)");
    model.evaluate();
    assert_eq!(column(&model, "D", 5, 6), ["0", ""]);
    assert_eq!(column(&model, "G", 1, 2), ["Total", "East"]);
    assert_eq!(model._get_text("H1"), "130.5");
    assert_eq!(column(&model, "J", 1, 2), ["#VALUE!", "#VALUE!"]);
}

#[test]
fn array_results_errors_and_mismatched_heights() {
    let mut model = region_sales();
    model._set("B3", "=1/0");
    // A LAMBDA returning an array is #VALUE! in every cell, one-record groups too (G!J121).
    model._set("D1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,x))");
    // An error value is its group's result and the total's (G!J193).
    model._set("G1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)))");
    // Values one row shorter than the keys (G!AB193, followup G!A1).
    model._set("J1", "=GROUPBY(A2:A11,B2:B10,LAMBDA(x,SUM(x)))");
    model.evaluate();
    assert_eq!(column(&model, "E", 1, 6), vec!["#VALUE!"; 6]);
    assert_eq!(
        column(&model, "H", 1, 6),
        ["45", "7", "-3", "#DIV/0!", "9", "#DIV/0!"]
    );
    assert_eq!(model._get_text("J1"), "#VALUE!");
}

#[test]
fn many_distinct_keys_are_grouped_or_refused_by_the_array_size_check() {
    // SPEC-0006 Design step 6 (CLAUDE.md directive 5): about 1e6 distinct keys need an output of
    // 1e6 + 1 rows by 2 columns, over the array size limit, so GROUPBY is refused before any
    // LAMBDA call or output allocation. 20,000 distinct keys are grouped (no quadratic search).
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=GROUPBY(SEQUENCE(1000000),SEQUENCE(1000000),LAMBDA(x,SUM(x)))",
    );
    model._set(
        "B1",
        "=ROWS(GROUPBY(SEQUENCE(20000),SEQUENCE(20000),LAMBDA(x,SUM(x))))",
    );
    model._set(
        "C1",
        "=INDEX(GROUPBY(-SEQUENCE(20000),SEQUENCE(20000),LAMBDA(x,SUM(x))),1,2)",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("B1"), "20001");
    assert_eq!(model._get_text("C1"), "20000");
}

#[test]
fn lambda_gets_values_with_blanks_and_another_arity_is_value_error() {
    // probes/groupby-lambda-forms G!S217 (ISREF is FALSE), G!A241 (COUNTA counts a blank value),
    // G!J217 (a LAMBDA of another arity is #VALUE! in each cell),
    // probes/groupby-blank-values G!S25 (ISBLANK is TRUE).
    let mut model = region_sales();
    model._set("B9", "");
    model._set("D1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,ISREF(x)))");
    model._set("G1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,COUNTA(x)))");
    model._set(
        "J1",
        "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,ISBLANK(INDEX(x,1,1))))",
    );
    model._set("P1", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,y,x))");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "FALSE");
    assert_eq!(column(&model, "H", 1, 6), ["3", "1", "1", "4", "1", "10"]);
    assert_eq!(column(&model, "K", 4, 5), ["FALSE", "TRUE"]);
    assert_eq!(column(&model, "Q", 1, 6), vec!["#VALUE!"; 6]);
}

#[test]
fn argument_corners_are_value_errors() {
    // probes/groupby-lambda-forms: a header row without records (G!S193), a non-function
    // (G!A217), fractional total_depth and field_headers (G!J241, G!S241).
    let mut model = region_sales();
    model._set("D1", "=GROUPBY(A1,B1,LAMBDA(x,SUM(x)),1)");
    model._set("D2", "=GROUPBY(A2:A11,B2:B11,5)");
    model._set("D3", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)),,0.5)");
    model._set("D4", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,SUM(x)),2.5)");
    model.evaluate();
    assert_eq!(column(&model, "D", 1, 4), vec!["#VALUE!"; 4]);
}

#[test]
fn generated_headers_over_two_keys_and_shown_headers_above_a_top_total() {
    // probes/groupby-lambda-forms G!J169 and G!S169.
    let mut model = region_sales();
    for row in 1..=11 {
        model._set(&format!("C{row}"), if row == 1 { "Units" } else { "1" });
    }
    model._set(
        "E1",
        "=GROUPBY(HSTACK(A1:A11,A1:A11),B1:C11,LAMBDA(x,SUM(x)),2)",
    );
    model._set("E20", "=GROUPBY(A1:A11,B1:B11,LAMBDA(x,SUM(x)),3,-1)");
    model.evaluate();
    let header: Vec<String> = ["E1", "F1", "G1", "H1"]
        .iter()
        .map(|c| model._get_text(c))
        .collect();
    assert_eq!(header, ["Row Field 1", "Row Field 2", "Value 1", "Value 2"]);
    assert_eq!(column(&model, "E", 20, 22), ["Region", "Total", "East"]);
    assert_eq!(column(&model, "F", 20, 22), ["Sales", "130.5", "45"]);
}
