#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── INDEX with inline arrays ──────────────────────────────────────────────────

#[test]
fn test_index_array_row_vector() {
    let mut model = new_empty_model();
    // A 1×4 row vector: the single index is a column index -> scalar.
    model._set("A1", "=INDEX({1,2,3,4},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_index_array_column_vector() {
    let mut model = new_empty_model();
    // A 4×1 column vector: the single index is a row index -> scalar.
    model._set("A1", "=INDEX({1;2;3;4},3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_index_array_whole_row() {
    let mut model = new_empty_model();
    // A 2×2 array: a single index selects an entire row -> array (spills).
    model._set("A1", "=INDEX({1,2;3,4},1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "2");
}

#[test]
fn test_index_array_whole_row_second() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({1,2;3,4},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("B1"), "4");
}

#[test]
fn test_index_array_row_and_col() {
    let mut model = new_empty_model();
    // Both row_num and col_num given -> single cell.
    model._set("A1", "=INDEX({1,2;3,4},1,1)");
    model._set("A2", "=INDEX({1,2;3,4},2,1)");
    model._set("A3", "=INDEX({1,2;3,4},2,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "4");
}

#[test]
fn test_index_array_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({1,2,3,4},9)");
    model._set("A2", "=INDEX({1,2;3,4},3,1)");
    model._set("A3", "=INDEX({1,2;3,4},1,5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#REF!");
    assert_eq!(model._get_text("A2"), "#REF!");
    assert_eq!(model._get_text("A3"), "#REF!");
}

#[test]
fn test_index_array_strings_and_booleans() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({\"a\",\"b\";TRUE,FALSE},2,1)");
    model._set("A2", "=INDEX({\"a\",\"b\";TRUE,FALSE},1,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
    assert_eq!(model._get_text("A2"), "b");
}

// ── Zero / missing indices select whole rows, columns or the entire array ─────

#[test]
fn test_index_missing_both_returns_whole_array() {
    let mut model = new_empty_model();
    // Missing row and column -> the whole array spills.
    model._set("A1", "=INDEX({1,2,3,4},,)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("C1"), "3");
    assert_eq!(model._get_text("D1"), "4");
}

#[test]
fn test_index_missing_row_returns_whole_column() {
    let mut model = new_empty_model();
    // Missing row, column 1 -> the whole first column spills down.
    model._set("A1", "=INDEX({1,2;3,4},,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "3");
}

#[test]
fn test_index_zero_row_returns_whole_column() {
    let mut model = new_empty_model();
    // A row index of 0 behaves like a missing row.
    model._set("A1", "=INDEX({1,2,3;4,5,6},0,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "4");
}

#[test]
fn test_index_zero_column_returns_whole_row() {
    let mut model = new_empty_model();
    // A column index of 0 (with the area argument) selects the whole row.
    model._set("A1", "=INDEX({1,2,3},1,0,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("C1"), "3");
}

#[test]
fn test_index_negative_row_is_value_error() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({1,2,3;4,5,6},-1,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

#[test]
fn test_index_area_out_of_range_is_ref_error() {
    let mut model = new_empty_model();
    // Only one area exists, so area_num = 2 is #REF!.
    model._set("A1", "=INDEX({1,2,3},1,0,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#REF!");
}

#[test]
fn test_index_range_missing_row_spills_column() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("B1", "30");
    model._set("B2", "40");
    // Missing row, column 2 of A1:B2 -> the whole second column spills.
    model._set("D1", "=INDEX(A1:B2,,2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "30");
    assert_eq!(model._get_text("D2"), "40");
}

// One index into a 2-D range is #REF!, also for 0 and fractions; vectors keep
// selecting along the vector (T-0213; docs/excel-semantics/lookup.md).
#[test]
fn test_index_single_index_into_2d_range_is_ref_error() {
    let mut model = new_empty_model();
    for (i, cell) in ["A1", "B1", "A2", "B2"].iter().enumerate() {
        model._set(cell, &(i + 1).to_string());
    }
    model._set("D1", "=INDEX(A1:B2,1)");
    model._set("D2", "=INDEX(A1:B2,0)");
    model._set("D3", "=INDEX(A1:B2,1E-15)");
    model._set("D4", "=INDEX(A1:B2,1/0)");
    model._set("D5", "=INDEX(A1:B2,1,2)");
    model._set("D6", "=INDEX(A1:A2,2)");
    model._set("D7", "=INDEX(A1:B1,2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"#REF!");
    assert_eq!(model._get_text("D2"), *"#REF!");
    assert_eq!(model._get_text("D3"), *"#REF!");
    assert_eq!(model._get_text("D4"), *"#DIV/0!");
    assert_eq!(model._get_text("D5"), *"2");
    assert_eq!(model._get_text("D6"), *"3");
    assert_eq!(model._get_text("D7"), *"2");
}

// ── INDEX lifted over array indices (T-1260, probes/index-array-indices) ──────

fn index_column_model() -> crate::Model {
    let mut model = new_empty_model();
    for (i, v) in [10, 20, 30, 40, 50].iter().enumerate() {
        model._set(&format!("A{}", i + 1), &v.to_string());
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }
    model
}

#[test]
fn index_lifts_over_an_array_or_range_index() {
    let mut model = index_column_model();
    model._set("D1", "=INDEX(A1:A5,{1,3})");
    model._set("D3", "=SUM(INDEX(A1:A5,{1,3}))");
    model._set("D4", "=ROWS(INDEX(A1:A5,{1,3}))");
    model._set("D5", "=INDEX(A1:A5,SEQUENCE(2))");
    model._set("J1", "1");
    model._set("J2", "3");
    model._set("F1", "=INDEX(A1:A5,J1:J2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("E1"), "30");
    assert_eq!(model._get_text("D3"), "40");
    assert_eq!(model._get_text("D4"), "1");
    assert_eq!(model._get_text("D5"), "10");
    assert_eq!(model._get_text("D6"), "20");
    assert_eq!(model._get_text("F1"), "10");
    assert_eq!(model._get_text("F2"), "30");
}

#[test]
fn index_indices_broadcast_and_whole_selections_keep_their_top_left() {
    let mut model = index_column_model();
    // A column of rows against a row of columns gives every pair.
    model._set("D1", "=INDEX(A1:B5,{1;2;3},{1,2})");
    // Row 0 selects the whole column; a lifted element keeps its first cell.
    model._set("D5", "=INDEX(A1:A5,{0,2})");
    model._set("D6", "=SUM(INDEX(A1:B5,0,{1,2}))");
    // One index into an array literal picks the row, whose first value is the element.
    model._set("D7", "=INDEX({1,2;3,4},{1,2})");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("E1"), "1");
    assert_eq!(model._get_text("D3"), "30");
    assert_eq!(model._get_text("E3"), "3");
    assert_eq!(model._get_text("D5"), "10");
    assert_eq!(model._get_text("E5"), "20");
    assert_eq!(model._get_text("D6"), "11");
    assert_eq!(model._get_text("D7"), "1");
    assert_eq!(model._get_text("E7"), "3");
}

#[test]
fn lifted_index_elements_fail_on_their_own() {
    let mut model = index_column_model();
    model._set("D1", "=INDEX(A1:A5,{1,#N/A})");
    model._set("D2", "=INDEX(A1:A5,{1,9})");
    model._set("D3", "=INDEX(A1:A5,{\"2\",\"x\"})");
    // One index into a 2-D range is #REF!, element by element.
    model._set("D4", "=INDEX(A1:B5,{1,2})");
    model._set("D5", "=INDEX(A1:A5,{1.9,2.5})");
    // An area_num array lifts too.
    model._set("D6", "=INDEX((A1:A5,B1:B5),{1,2},1,{1,2})");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("E1"), "#N/A");
    assert_eq!(model._get_text("E2"), "#REF!");
    assert_eq!(model._get_text("D3"), "20");
    assert_eq!(model._get_text("E3"), "#VALUE!");
    assert_eq!(model._get_text("D4"), "#REF!");
    assert_eq!(model._get_text("E4"), "#REF!");
    assert_eq!(model._get_text("D5"), "10");
    assert_eq!(model._get_text("E5"), "20");
    assert_eq!(model._get_text("D6"), "10");
    assert_eq!(model._get_text("E6"), "2");
}

#[test]
fn lifted_index_over_a_large_array_reads_one_value_per_element() {
    let mut model = new_empty_model();
    // Every element selects the whole 100000-row array and keeps its first value. Copying the
    // selection per element would be 1e10 node copies; reading the top-left one is linear.
    model._set("A1", "=SUM(INDEX(SEQUENCE(100000),SEQUENCE(100000)*0))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "100000");
}

#[test]
fn lifted_index_broadcast_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    // 1048576 rows against 16384 columns would be 1.7e10 elements: past the shared array bound.
    model._set(
        "B1",
        "=INDEX(A1:A5,SEQUENCE(1048576)*0+1,SEQUENCE(1,16384)*0+1)",
    );
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#ERROR!");
}

// probes/index-tiny-negative and probes/index-snap-tolerance (T-1407): a negative index within
// 2^-23 + 2^-34 of 0 is 0 (the whole row), and an index within 2^-22 + 2^-33 below an integer
// is that integer; gen-m9/1466c66c61189a83 reads `INDEX(Data!A1:C3,3,-1e-15)` as row A3:C3.
#[test]
fn index_snaps_near_integer_indexes() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("C1", "1"),
        ("D1", "2"),
        ("E1", "3"),
        ("C3", "0.5"),
        ("D3", "8"),
        ("E3", "9"),
    ] {
        model._set(cell, value);
    }
    model._set("A1", "=SUM(INDEX(C1:E3,3,-1e-15))");
    model._set("A2", "=COLUMNS(INDEX(C1:E3,3,-1.19E-7))");
    model._set("A3", "=COLUMNS(INDEX(C1:E3,3,-1.193E-7))");
    model._set("A4", "=COLUMNS(INDEX(C1:E3,3,1-2.3853E-7))");
    model._set("A5", "=COLUMNS(INDEX(C1:E3,3,1-2.3854E-7))");
    model._set("A6", "=ROWS(INDEX(C1:E3,-1E-7,1))");
    model._set("A7", "=INDEX(C1:E3,3,-0.5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "17.5");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "1");
    assert_eq!(model._get_text("A5"), "3");
    assert_eq!(model._get_text("A6"), "3");
    assert_eq!(model._get_text("A7"), "#VALUE!");
}
