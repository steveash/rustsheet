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
    model._set("D7", "=GROUPBY(A2:A11,B2:B11,_xleta.SUM)");
    model.evaluate();
    assert_eq!(column(&model, "D", 1, 4), vec!["#VALUE!"; 4]);
    assert_eq!(model._get_text("E7"), "45");
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

/// probes/groupby-pivotby's Data!A1:E11 (Region, Product, Year, Sales, Units; A9 is blank).
fn records() -> Model {
    let mut model = new_empty_model();
    let rows = [
        ["Region", "Product", "Year", "Sales", "Units"],
        ["East", "Apple", "2023", "10", "1"],
        ["West", "Pear", "2023", "20", "2"],
        ["East", "Pear", "2024", "30", "3"],
        ["West", "Apple", "2024", "40", "4"],
        ["east", "Apple", "2023", "5", "5"],
        ["North", "Pear", "2024", "7", "6"],
        ["West", "Apple", "2023", "0", "7"],
        ["", "Apple", "2024", "9", "8"],
        ["South", "Banana", "2025", "-3", "9"],
        ["West", "Banana", "2025", "12.5", "10"],
    ];
    for (i, row) in rows.iter().enumerate() {
        for (cell, column) in row.iter().zip(["A", "B", "C", "D", "E"]) {
            if !cell.is_empty() {
                model._set(&format!("{column}{}", i + 1), cell);
            }
        }
    }
    model
}

/// The texts of row `row`, columns `first..=last` (letters).
fn row(model: &Model, row: usize, first: char, last: char) -> Vec<String> {
    (first..=last)
        .map(|column| model._get_text(&format!("{column}{row}")))
        .collect()
}

#[test]
fn sort_order_by_keys_and_by_value_columns() {
    let mut model = records();
    // -1: keys descending, blank and Total last (G!A145); 2 / -2: by the value, the blank key not
    // pinned (G!J145, G!S145).
    model._set("H1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,-1)");
    model._set("J1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,2)");
    model._set("L1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,-2)");
    // {2,-1}: Region descending, then Product ascending, whatever the array order (G!AB145,
    // probes/groupby-pivotby-followup G!A341-A501); a vertical array too (groupby-sort-filter-args B15).
    model._set("N1", "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,,{2,-1})");
    model._set("R1", "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,0,{2;-1})");
    model.evaluate();
    assert_eq!(
        column(&model, "H", 1, 6),
        ["West", "South", "North", "East", "0", "Total"]
    );
    assert_eq!(
        column(&model, "J", 1, 6),
        ["South", "North", "0", "East", "West", "Total"]
    );
    assert_eq!(
        column(&model, "K", 1, 6),
        ["-3", "7", "9", "45", "72.5", "130.5"]
    );
    assert_eq!(
        column(&model, "L", 1, 6),
        ["West", "East", "0", "North", "South", "Total"]
    );
    let regions = [
        "West", "West", "West", "South", "North", "East", "East", "0",
    ];
    let products = [
        "Apple", "Banana", "Pear", "Banana", "Pear", "Apple", "Pear", "Apple",
    ];
    assert_eq!(column(&model, "N", 1, 9)[..8], regions);
    assert_eq!(column(&model, "O", 1, 8), products);
    assert_eq!(row(&model, 9, 'N', 'P'), ["Total", "", "130.5"]);
    assert_eq!(column(&model, "R", 1, 8), regions);
    assert_eq!(column(&model, "S", 1, 8), products);
}

#[test]
fn value_sort_over_two_keys_orders_parents_by_their_aggregate() {
    // probes/groupby-pivotby-followup2 G!A261 (3), groupby-sort-filter-args B21-B22 ({-3}, -3).
    let mut model = records();
    model._set("H1", "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,0,3)");
    model._set("L1", "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,0,{-3})");
    model.evaluate();
    assert_eq!(
        column(&model, "H", 1, 8),
        ["South", "North", "0", "East", "East", "West", "West", "West"]
    );
    assert_eq!(
        column(&model, "J", 1, 8),
        ["-3", "7", "9", "15", "30", "12.5", "20", "40"]
    );
    assert_eq!(
        column(&model, "L", 1, 8),
        ["West", "West", "West", "East", "East", "0", "North", "South"]
    );
    assert_eq!(
        column(&model, "N", 1, 8),
        ["40", "20", "12.5", "30", "15", "9", "7", "-3"]
    );
}

#[test]
fn value_sort_ties_keep_key_order_and_mixed_values_sort_like_sort() {
    // probes/groupby-sort-filter-args B27-B28 (the second value column), B32-B33 (COUNT ties
    // stay in ascending key order both ways), B35 (numbers before text), B36-B37 (an error last
    // ascending, first descending), B38-B39 (headers and a top total stay in place).
    let mut model = records();
    model._set("H1", "=GROUPBY(A2:A11,D2:E11,_xleta.SUM,,0,3)");
    model._set("K1", "=GROUPBY(A2:A11,D2:D11,_xleta.COUNT,,0,2)");
    model._set("M1", "=GROUPBY(A2:A11,D2:D11,_xleta.COUNT,,0,-2)");
    model._set(
        "O1",
        r#"=GROUPBY(A2:A11,D2:D11,LAMBDA(x,IF(SUM(x)>20,"t",SUM(x))),,0,2)"#,
    );
    model._set(
        "Q1",
        "=GROUPBY(A2:A11,IF(A2:A11=\"West\",NA(),E2:E11),_xleta.SUM,,0,-2)",
    );
    model._set("S1", "=GROUPBY(A1:A11,D1:D11,_xleta.SUM,3,,-2)");
    model._set("U1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,-1,2)");
    model.evaluate();
    assert_eq!(
        column(&model, "H", 1, 5),
        ["North", "0", "East", "South", "West"]
    );
    assert_eq!(
        column(&model, "K", 1, 5),
        ["North", "South", "0", "East", "West"]
    );
    assert_eq!(
        column(&model, "M", 1, 5),
        ["West", "East", "North", "South", "0"]
    );
    assert_eq!(
        column(&model, "O", 1, 5),
        ["South", "North", "0", "East", "West"]
    );
    assert_eq!(column(&model, "Q", 1, 2), ["West", "East"]);
    assert_eq!(model._get_text("R1"), "#N/A");
    assert_eq!(column(&model, "S", 1, 3), ["Region", "West", "East"]);
    assert_eq!(model._get_text("S7"), "Total");
    assert_eq!(column(&model, "U", 1, 3), ["Total", "South", "North"]);
}

#[test]
fn invalid_sort_orders_are_value_errors() {
    // G!S217 (past the last column), probes/groupby-pivotby-followup G!A461/A541 ({1,-1}, {0,1}),
    // groupby-sort-filter-args B1-B31: 0, -3, fractions, text, TRUE, a blank cell, key and value
    // columns mixed, two value columns, an error or text element. NA() itself is #N/A (B12).
    let mut model = records();
    let orders = [
        "3", "0", "-3", "1.5", "-0.5", "\"2\"", "TRUE", "Z1", "\"\"", "{1,-1}", "{0,1}",
    ];
    for (i, order) in orders.iter().enumerate() {
        model._set(
            &format!("H{}", i + 1),
            &format!("=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,{order})"),
        );
    }
    let two_keys = [
        "{1,3}",
        "{3,1}",
        "{-2,-3}",
        "{1.5,2}",
        "{1,#N/A}",
        "{1,\"2\"}",
        "{1,2,3}",
    ];
    for (i, order) in two_keys.iter().enumerate() {
        model._set(
            &format!("I{}", i + 1),
            &format!("=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,,{order})"),
        );
    }
    model._set("J1", "=GROUPBY(A2:A11,D2:E11,_xleta.SUM,,,{2,3})");
    model._set("J2", "=GROUPBY(A2:A11,D2:E11,_xleta.SUM,,,4)");
    model._set("J3", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,NA())");
    model.evaluate();
    assert_eq!(
        column(&model, "H", 1, orders.len()),
        vec!["#VALUE!"; orders.len()]
    );
    assert_eq!(
        column(&model, "I", 1, two_keys.len()),
        vec!["#VALUE!"; two_keys.len()]
    );
    assert_eq!(column(&model, "J", 1, 3), ["#VALUE!", "#VALUE!", "#N/A"]);
}

#[test]
fn filter_array_drops_rows_before_grouping() {
    let mut model = records();
    // G!A169: groups without kept rows vanish and the total covers the kept rows.
    model._set("H1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,C2:C11=2023)");
    // groupby-sort-filter-args B42 (numbers: 0 drops), B44 ("TRUE"/"FALSE" text), B52 (blank drops),
    // B55 and followup3 G!A85 (a scalar keeps or drops every row).
    model._set("J1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,C2:C11-2023)");
    model._set(
        "L1",
        r#"=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,IF(C2:C11=2023,"TRUE","FALSE"))"#,
    );
    model._set(
        "N1",
        "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,IF(C2:C11=2023,TRUE,Z2:Z11))",
    );
    model._set("P1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,2)");
    // With headers the filter may cover the header row or only the records (B56-B59).
    model._set("R1", "=GROUPBY(A1:A11,D1:D11,_xleta.SUM,1,,,C1:C11=2023)");
    model._set("T1", "=GROUPBY(A1:A11,D1:D11,_xleta.SUM,1,,,C2:C11=2023)");
    model._set("V1", "=GROUPBY(A1:A11,D1:D11,_xleta.SUM,3,,,C1:C11<>2024)");
    // B60: filtered, then sorted by value.
    model._set("X1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,-2,C2:C11<>2023)");
    model.evaluate();
    assert_eq!(column(&model, "H", 1, 3), ["East", "West", "Total"]);
    assert_eq!(column(&model, "I", 1, 3), ["15", "20", "35"]);
    assert_eq!(
        column(&model, "K", 1, 6),
        ["30", "7", "-3", "52.5", "9", "95.5"]
    );
    assert_eq!(column(&model, "M", 1, 3), ["15", "20", "35"]);
    assert_eq!(column(&model, "O", 1, 3), ["15", "20", "35"]);
    assert_eq!(model._get_text("Q6"), "130.5");
    assert_eq!(column(&model, "S", 1, 3), ["15", "20", "35"]);
    assert_eq!(column(&model, "U", 1, 3), ["15", "20", "35"]);
    assert_eq!(
        column(&model, "V", 1, 5),
        ["Region", "East", "South", "West", "Total"]
    );
    assert_eq!(
        column(&model, "W", 1, 5),
        ["Sales", "15", "-3", "32.5", "44.5"]
    );
    assert_eq!(column(&model, "X", 1, 3), ["West", "East", "0"]);
    assert_eq!(column(&model, "Y", 1, 3), ["52.5", "30", "9"]);
}

#[test]
fn invalid_filters_are_value_errors() {
    // G!AB217 (shorter), G!A241 and followup3 G!A1-A109 (all FALSE, scalar FALSE, 0, {FALSE}, two
    // keys), groupby-sort-filter-args B43-B54: text, an error element (also on a dropped row), a
    // row, two columns, longer, all blank, "", a text scalar. A scalar NA() is #N/A (B47).
    let mut model = records();
    let filters = [
        "C2:C10=2023",
        "C2:C11=1999",
        "FALSE",
        "0",
        "{FALSE}",
        "B2:B11",
        "IF(C2:C11=2024,NA(),TRUE)",
        "IF(C2:C11=2024,NA(),C2:C11=2023)",
        "TRANSPOSE(C2:C11=2023)",
        "C2:D11>2023",
        "C2:C12=2023",
        "Z2:Z11",
        "IF(C2:C11=2023,TRUE,\"\")",
        "\"x\"",
    ];
    for (i, filter) in filters.iter().enumerate() {
        model._set(
            &format!("H{}", i + 1),
            &format!("=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,{filter})"),
        );
    }
    model._set("I1", "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,,,C2:C11=1999)");
    model._set("I2", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,NA())");
    model._set("I3", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,0,,,C1:C11=2023)");
    model.evaluate();
    assert_eq!(
        column(&model, "H", 1, filters.len()),
        vec!["#VALUE!"; filters.len()]
    );
    assert_eq!(column(&model, "I", 1, 3), ["#VALUE!", "#N/A", "#VALUE!"]);
}

#[test]
fn error_values_of_arguments_come_first_in_argument_order() {
    // groupby-sort-filter-args B61-B78: an error value in any argument wins over an invalid other
    // argument, earlier or later, and the first error in argument order wins.
    let mut model = records();
    let cases = [
        ("_xleta.SUM,,,NA(),C2:C10=2023", "#N/A"),
        ("_xleta.SUM,,,9,NA()", "#N/A"),
        ("_xleta.SUM,,,NA(),1/0", "#N/A"),
        ("_xleta.SUM,1/0,,NA()", "#DIV/0!"),
        ("_xleta.SUM,,1/0,NA()", "#DIV/0!"),
        ("_xleta.SUM,,3,NA()", "#N/A"),
        ("_xleta.SUM,4,,,NA()", "#N/A"),
        ("5,,,NA()", "#N/A"),
        ("NA(),,,1/0", "#N/A"),
        ("_xleta.SUM,,,NA(),C2:C11=1999", "#N/A"),
        ("_xleta.SUM,,,,C2:C10=2023,NA()", "#N/A"),
        ("_xleta.SUM,,,NA(),,1/0", "#N/A"),
        ("_xleta.SUM,,,9,C2:C10=2023", "#VALUE!"),
    ];
    for (i, (args, _)) in cases.iter().enumerate() {
        model._set(
            &format!("H{}", i + 1),
            &format!("=GROUPBY(A2:A11,D2:D11,{args})"),
        );
    }
    model._set("I1", "=GROUPBY(A2:A11,D2:D10,_xleta.SUM,,,NA())");
    model._set("I2", "=GROUPBY(A2:A11,D2:D10,_xleta.SUM,,,,NA())");
    model.evaluate();
    let expected: Vec<&str> = cases.iter().map(|(_, error)| *error).collect();
    assert_eq!(column(&model, "H", 1, cases.len()), expected);
    assert_eq!(column(&model, "I", 1, 2), ["#N/A", "#N/A"]);
}

#[test]
fn field_relationship_scalars() {
    // G!J241 (2), groupby-sort-filter-args B79-B83 (3, -1, 0.5 are #VALUE!, NA() is #N/A, "1"
    // with one total is the default layout), followup G!A941/A961 (FALSE and 0 act as 0).
    let mut model = records();
    let invalid = ["2", "3", "-1", "0.5"];
    for (i, relationship) in invalid.iter().enumerate() {
        model._set(
            &format!("H{}", i + 1),
            &format!("=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,{relationship})"),
        );
    }
    model._set("H5", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,NA())");
    model._set("J1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,\"1\")");
    model._set("L1", "=GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,FALSE)");
    model._set("N1", "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,,,,0)");
    model.evaluate();
    assert_eq!(
        column(&model, "H", 1, 5),
        ["#VALUE!", "#VALUE!", "#VALUE!", "#VALUE!", "#N/A"]
    );
    assert_eq!(
        column(&model, "K", 1, 6),
        ["45", "7", "-3", "72.5", "9", "130.5"]
    );
    assert_eq!(
        column(&model, "M", 1, 6),
        ["45", "7", "-3", "72.5", "9", "130.5"]
    );
    assert_eq!(row(&model, 1, 'N', 'P'), ["East", "Apple", "15"]);
}

#[test]
fn value_sort_over_many_groups_and_two_keys_is_bounded() {
    // CLAUDE.md directive 5: a value sort over 20,000 two-key groups makes one call per group and
    // parent (no quadratic search) and keeps the output size check.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=INDEX(GROUPBY(HSTACK(MOD(SEQUENCE(20000),100),SEQUENCE(20000)),SEQUENCE(20000),LAMBDA(x,SUM(x)),,0,-3),1,2)",
    );
    model._set(
        "B1",
        "=GROUPBY(SEQUENCE(1000000),SEQUENCE(1000000),LAMBDA(x,SUM(x)),,,-2)",
    );
    model.evaluate();
    // Parent 0 (100, 200, ..., 20000) has the largest sum; its largest child is 20000.
    assert_eq!(model._get_text("A1"), "20000");
    assert_eq!(model._get_text("B1"), "#ERROR!");
}

#[test]
fn filter_elements_beside_headers_are_ignored_and_text_must_be_true_or_false() {
    // probes/groupby-sort-filter-args2: text, "" or an error beside the header row is ignored
    // (B1, B5, B29-B30) but not beside a record (B31); "true"/"True" count (B6-B7), "1" does
    // not (B8-B9); {#N/A} is #VALUE! as a filter and a sort_order (B11-B12); a one-cell
    // reference to an error is that error, before total_depth 3 (B16-B18).
    let mut model = records();
    model._set("Z5", "=1/0");
    let cases = [
        ("1,,,IF(ROW(C1:C11)=1,NA(),C1:C11=2023)", "35"),
        ("1,,,IF(ROW(C1:C11)=1,\"x\",C1:C11=2023)", "35"),
        ("1,,,IF(ROW(C1:C11)=1,\"\",C1:C11=2023)", "35"),
        ("1,,,IF(ROW(C1:C11)=3,NA(),C1:C11=2023)", "#VALUE!"),
        ("1,,,IF(ROW(C1:C11)=1,NA(),C1:C11=1999)", "#VALUE!"),
        ("0,,,IF(ROW(C1:C11)=1,NA(),C1:C11=2023)", "#VALUE!"),
        (",,,IF(C1:C11=2023,\"true\",\"false\")", "35"),
        (",,,IF(C1:C11=2023,\"True\",\"False\")", "35"),
        (",,,IF(C1:C11=2023,\"1\",\"0\")", "#VALUE!"),
        (",,,\"1\"", "#VALUE!"),
        (",,,{#N/A}", "#VALUE!"),
        (",,{#N/A}", "#VALUE!"),
        (",,Z5", "#DIV/0!"),
        (",,,Z5", "#DIV/0!"),
        (",3,Z5", "#DIV/0!"),
    ];
    for (i, (args, _)) in cases.iter().enumerate() {
        model._set(
            &format!("H{}", i + 1),
            &format!("=INDEX(GROUPBY(A1:A11,D1:D11,_xleta.SUM,{args}),ROWS(GROUPBY(A1:A11,D1:D11,_xleta.SUM,{args})),2)"),
        );
    }
    model.evaluate();
    let expected: Vec<&str> = cases.iter().map(|(_, total)| *total).collect();
    assert_eq!(column(&model, "H", 1, cases.len()), expected);
}

#[test]
fn value_sort_parents_use_the_function_of_all_their_rows() {
    // probes/groupby-sort-filter-args2 B22 (AVERAGE: East 15 < West 18.125) and B24
    // (100-SUM: West 27.5 < East 55, where the sum of the children would put East first).
    let mut model = records();
    model._set("H1", "=GROUPBY(A2:B11,D2:D11,_xleta.AVERAGE,,0,3)");
    model._set("L1", "=GROUPBY(A2:B11,D2:D11,LAMBDA(x,100-SUM(x)),,0,3)");
    model.evaluate();
    assert_eq!(
        column(&model, "H", 1, 8),
        ["South", "North", "0", "East", "East", "West", "West", "West"]
    );
    assert_eq!(column(&model, "I", 6, 8), ["Banana", "Apple", "Pear"]);
    assert_eq!(
        column(&model, "L", 1, 8),
        ["West", "West", "West", "East", "East", "0", "North", "South"]
    );
    assert_eq!(column(&model, "N", 1, 3), ["60", "80", "87.5"]);
}

/// The texts of `formula`'s result in D1 of the Region/Sales records, `height` rows by `width`
/// columns, with Units in C (1 to 10).
fn grouped(formula: &str, height: usize, width: u8) -> Vec<Vec<String>> {
    let mut model = region_sales();
    model._set("C1", "Units");
    for row in 2..=11 {
        model._set(&format!("C{row}"), &(row - 1).to_string());
    }
    model._set("D1", formula);
    model.evaluate();
    let last = char::from(b'D' + width - 1);
    (1..=height).map(|r| row(&model, r, 'D', last)).collect()
}

#[test]
fn a_row_of_functions_adds_columns_under_their_names() {
    // probes/groupby-pivotby G!S121: a row of function names above the shown headers.
    let grid = grouped(
        "=GROUPBY(A1:A11,B1:B11,HSTACK(_xleta.SUM,_xleta.AVERAGE),3)",
        8,
        3,
    );
    assert_eq!(grid[0], ["", "SUM", "AVERAGE"]);
    assert_eq!(grid[1], ["Region", "Sales", "Sales"]);
    assert_eq!(grid[2], ["East", "45", "15"]);
    assert_eq!(grid[7], ["Total", "130.5", "13.05"]);
    // probes/groupby-pivotby-functions G!A1 (names without headers), G!A181 (a LAMBDA is
    // "CUSTOM"), functions2 G!A641 (detected headers are shown with a row of functions).
    let grid = grouped(
        "=GROUPBY(A2:A11,B2:B11,HSTACK(_xleta.SUM,LAMBDA(x,ROWS(x))))",
        2,
        3,
    );
    assert_eq!(grid, [["", "SUM", "CUSTOM"], ["East", "45", "3"]]);
    let grid = grouped(
        "=GROUPBY(A1:A11,B1:B11,HSTACK(_xleta.SUM,_xleta.MAX))",
        3,
        3,
    );
    assert_eq!(grid[1], ["Region", "Sales", "Sales"]);
    assert_eq!(grid[2], ["East", "45", "30"]);
    // One function in HSTACK is that function (G!A601).
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,HSTACK(_xleta.SUM))", 1, 2);
    assert_eq!(grid[0], ["East", "45"]);
}

#[test]
fn a_column_of_functions_repeats_each_group_after_a_label_column() {
    // probes/groupby-pivotby G!AB121; functions G!A61 (shown headers: "" over the labels).
    let grid = grouped(
        "=GROUPBY(A2:A11,B2:B11,VSTACK(_xleta.SUM,_xleta.MAX))",
        12,
        3,
    );
    assert_eq!(grid[0], ["East", "SUM", "45"]);
    assert_eq!(grid[1], ["East", "MAX", "30"]);
    assert_eq!(grid[10], ["Total", "SUM", "130.5"]);
    assert_eq!(grid[11], ["Total", "MAX", "40"]);
    let grid = grouped(
        "=GROUPBY(A1:A11,B1:B11,VSTACK(_xleta.SUM,_xleta.MAX),3,-1)",
        3,
        3,
    );
    assert_eq!(grid[0], ["Region", "", "Sales"]);
    assert_eq!(grid[1], ["Total", "SUM", "130.5"]);
    assert_eq!(grid[2], ["Total", "MAX", "40"]);
}

#[test]
fn labels_beside_or_below_the_functions() {
    // functions G!A221, functions2 G!A121 and G!A161: values next to a column of functions label
    // its rows, values below a row of functions label its columns.
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,HSTACK(_xleta.SUM,\"s\"))", 1, 3);
    assert_eq!(grid[0], ["East", "s", "45"]);
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,VSTACK(_xleta.SUM,5))", 2, 2);
    assert_eq!(grid, [["", "5"], ["East", "45"]]);
}

#[test]
fn functions_pair_with_value_columns() {
    // functions G!A41, G!A81: two functions over two value columns pair up; functions2
    // G!A1-A61: other counts are #VALUE!.
    let grid = grouped(
        "=GROUPBY(A2:A11,B2:C11,HSTACK(_xleta.SUM,_xleta.MAX))",
        2,
        3,
    );
    assert_eq!(grid, [["", "SUM", "MAX"], ["East", "45", "5"]]);
    let grid = grouped(
        "=GROUPBY(A2:A11,B2:C11,VSTACK(_xleta.SUM,_xleta.MAX))",
        2,
        3,
    );
    assert_eq!(grid, [["East", "SUM", "45"], ["East", "MAX", "5"]]);
    let grid = grouped(
        "=GROUPBY(A2:A11,B2:C11,HSTACK(_xleta.SUM,_xleta.MAX,_xleta.MIN))",
        1,
        1,
    );
    assert_eq!(grid[0], ["#VALUE!"]);
}

#[test]
fn invalid_function_arrays() {
    // functions G!A201 (2-D), G!A261 (an array constant), G!A781 (an error element), functions2
    // G!A101 (a value first), G!A381 and G!A421 (functions that take three or no arguments).
    let mut model = region_sales();
    let cases = [
        (
            "D1",
            "=GROUPBY(A2:A11,B2:B11,HSTACK(VSTACK(_xleta.SUM,_xleta.MAX),VSTACK(_xleta.MIN,_xleta.COUNT)))",
            "#VALUE!",
        ),
        ("D2", "=GROUPBY(A2:A11,B2:B11,{1,2})", "#VALUE!"),
        ("D3", "=GROUPBY(A2:A11,B2:B11,HSTACK(_xleta.SUM,NA()))", "#N/A"),
        ("D4", "=GROUPBY(A2:A11,B2:B11,HSTACK(5,_xleta.SUM))", "#VALUE!"),
        ("D5", "=GROUPBY(A2:A11,B2:B11,_xleta.TEXTJOIN)", "#VALUE!"),
        ("D6", "=GROUPBY(A2:A11,B2:B11,_xleta.NOW)", "#VALUE!"),
        ("D7", "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,y,z,1))", "#VALUE!"),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cases {
        assert_eq!(model._get_text(cell), expected, "{formula}");
    }
}

#[test]
fn value_sorts_name_output_columns() {
    // functions2 G!A221 (3 is the second function of a row), G!A201 (2 is the first function of
    // a column of them).
    let grid = grouped(
        "=GROUPBY(A2:A11,B2:B11,HSTACK(_xleta.SUM,LAMBDA(x,-SUM(x))),,,3)",
        2,
        3,
    );
    assert_eq!(grid[1], ["West", "72.5", "-72.5"]);
    let grid = grouped(
        "=GROUPBY(A2:A11,B2:B11,VSTACK(LAMBDA(x,-SUM(x)),_xleta.SUM),,,2)",
        2,
        3,
    );
    assert_eq!(grid, [["West", "CUSTOM", "-72.5"], ["West", "SUM", "72.5"]]);
}

#[test]
fn two_argument_functions_get_all_rows() {
    // probes/groupby-pivotby G!S97: PERCENTOF is each group's share and 1 in the total;
    // functions G!A301-A321 (the relative set is every kept row), G!A381 (an optional second
    // parameter is not passed).
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,_xleta.PERCENTOF)", 6, 2);
    assert_eq!(grid[1], ["North", "0.053639847"]);
    assert_eq!(grid[5], ["Total", "1"]);
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,LAMBDA(x,y,ROWS(y)))", 6, 2);
    assert_eq!(grid[0], ["East", "10"]);
    assert_eq!(grid[5], ["Total", "10"]);
    let grid = grouped(
        "=GROUPBY(A2:A11,B2:B11,LAMBDA(x,y,SUM(y)),,,,B2:B11>5)",
        1,
        2,
    );
    assert_eq!(grid[0], ["East", "128.5"]);
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,LAMBDA(x,[y],ISOMITTED(y)))", 1, 2);
    assert_eq!(grid[0], ["East", "TRUE"]);
}

#[test]
fn a_reference_result_is_value_error() {
    // probes/groupby-lambda-forms G!J193; functions G!A561, G!A581 (picked by IF, from INDEX).
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,LAMBDA(x,B2))", 1, 2);
    assert_eq!(grid[0], ["East", "#VALUE!"]);
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,LAMBDA(x,IF(TRUE,B2)))", 1, 2);
    assert_eq!(grid[0], ["East", "#VALUE!"]);
    let grid = grouped("=GROUPBY(A2:A11,B2:B11,LAMBDA(x,INDEX(B2:B3,1)))", 1, 2);
    assert_eq!(grid[0], ["East", "#VALUE!"]);
}

#[test]
fn two_parameter_lambdas_are_charged_against_the_visit_cap() {
    // SPEC-0006 Design step 3, DEVIATION (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY"):
    // 20000 groups each given all 20000 rows visit 4E8 cells, past the cap, so the result is
    // #ERROR!, while the built-in PERCENTOF sums the relative set once (O(N)) and a one-parameter
    // LAMBDA visits each row once.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=GROUPBY(SEQUENCE(20000),SEQUENCE(20000),LAMBDA(x,y,ROWS(y)))",
    );
    model._set(
        "B1",
        "=INDEX(GROUPBY(SEQUENCE(20000),SEQUENCE(20000),_xleta.PERCENTOF),20001,2)",
    );
    model._set(
        "C1",
        "=INDEX(GROUPBY(SEQUENCE(20000),SEQUENCE(20000),LAMBDA(x,ROWS(x))),20001,2)",
    );
    model._set(
        "D1",
        "=INDEX(GROUPBY(SEQUENCE(2000),SEQUENCE(2000),LAMBDA(x,y,ROWS(y))),2001,2)",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("C1"), "20000");
    assert_eq!(model._get_text("D1"), "2000");
}
