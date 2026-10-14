#![allow(clippy::unwrap_used)]

use super::results_independent_of_used_area;
use crate::test::util::new_empty_model;

#[test]
fn test_lookup_range_vector_only() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "=LOOKUP(20,A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "20");
}

#[test]
fn test_lookup_range_with_result_vector() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("C1", "a");
    model._set("C2", "b");
    model._set("C3", "c");
    model._set("B1", "=LOOKUP(20,A1:A3,C1:C3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "b");
}

#[test]
fn test_lookup_range_result_vector_different_orientation() {
    let mut model = new_empty_model();
    // lookup vector is a column, result vector is a row of the same length
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("C1", "a");
    model._set("D1", "b");
    model._set("E1", "c");
    model._set("B5", "=LOOKUP(30,A1:A3,C1:E1)");
    model.evaluate();
    assert_eq!(model._get_text("B5"), "c");
}

#[test]
fn test_lookup_array_vector_only() {
    let mut model = new_empty_model();
    model._set("A1", "=LOOKUP(20,{10,20,30})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "20");
}

#[test]
fn test_lookup_array_with_result_array() {
    let mut model = new_empty_model();
    model._set("A1", r#"=LOOKUP(20,{10,20,30},{"a","b","c"})"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_lookup_array_column_vectors() {
    let mut model = new_empty_model();
    model._set("A1", r#"=LOOKUP(30,{10;20;30},{"a";"b";"c"})"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "c");
}

#[test]
fn test_lookup_array_largest_smaller() {
    let mut model = new_empty_model();
    // 25 is absent, returns the value matching the largest entry <= 25, i.e. 20 -> "b"
    model._set("A1", r#"=LOOKUP(25,{10,20,30},{"a","b","c"})"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_lookup_array_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "=LOOKUP(5,{10,20,30})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

#[test]
fn test_lookup_array_form_wide_searches_first_row() {
    let mut model = new_empty_model();
    // 2 rows x 3 columns: wider than tall, so search the first row for "b"
    // (index 1) and return from the last row at that index -> 2.
    model._set("A1", r#"=LOOKUP("b",{"a","b","c";1,2,3})"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_lookup_array_form_tall_searches_first_column() {
    let mut model = new_empty_model();
    // 3 rows x 2 columns: taller than wide, so search the first column for 20
    // (index 1) and return from the last column at that index -> "b".
    model._set("A1", r#"=LOOKUP(20,{10,"a";20,"b";30,"c"})"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_lookup_array_form_square_searches_first_column() {
    let mut model = new_empty_model();
    // 2x2 square: not wider than tall, so search first column [1;3] for 1
    // (index 0) and return from the last column at that row -> 2.
    model._set("A1", "=LOOKUP(1,{1,2;3,4})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_lookup_array_form_range_wide() {
    let mut model = new_empty_model();
    // Same as the wide array-form case, but with a range reference.
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    model._set("A2", "x");
    model._set("B2", "y");
    model._set("C2", "z");
    model._set("E1", "=LOOKUP(20,A1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "y");
}

// Whole-column and whole-row searches give the results of a full walk. LOOKUP
// always searches sorted, and how Excel's sorted search treats the empty tail
// is not yet oracle-probed (T-0126), so no value is pinned.
#[test]
fn test_lookup_whole_row_and_column_independent_of_used_area() {
    let data = [
        ("A1", "10"),
        ("A2", "20"),
        ("A3", "30"),
        ("B1", "a"),
        ("B2", "b"),
        ("B3", "c"),
    ];
    results_independent_of_used_area(
        &data,
        "H3",
        "Z1048576",
        &[
            "=LOOKUP(20,A:A,B:B)",
            "=LOOKUP(99,A:A,B:B)",
            "=LOOKUP(20,A:B)",
            "=LOOKUP(30,A1:A3,E:E)",
            "=LOOKUP(1,A100:A200,B100:B200)",
            "=LOOKUP(20,A1:A3,B:B)",
        ],
    );
    results_independent_of_used_area(
        &data,
        "H3",
        "XFD5",
        &["=LOOKUP(\"b\",1:1,2:2)", "=LOOKUP(10,1:2)"],
    );
}

// Every cell of the result vector is evaluated, so a result vector holding the
// formula's own cell is circular even past the position returned.
#[test]
fn test_lookup_result_vector_holding_the_formula_is_circular() {
    for formula in ["=LOOKUP(20,A1:A3,E:E)", "=LOOKUP(20,A1:A3,E1:E10)"] {
        let mut model = new_empty_model();
        for (cell, value) in [
            ("A1", "10"),
            ("A2", "20"),
            ("A3", "30"),
            ("E1", "a"),
            ("E2", "b"),
            ("E3", "c"),
        ] {
            model._set(cell, value);
        }
        model._set("E5", formula);
        model.evaluate();
        assert_eq!(model._get_text("E5"), "#CIRC!", "{formula}");
    }
}

#[test]
fn test_lookup_2d_lookup_vector_and_short_result() {
    // probes/lookup-2d-vector and -wide (excel-graph): a 2-D lookup_vector is searched along its
    // first column (or first row when wide), a 2-D result_vector is #N/A, and a result range
    // shorter than the match position reads on past its end.
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("B1", "2"), ("A2", "3"), ("B2", "4")] {
        model._set(cell, value);
    }
    for (cell, value) in [("F1", "10"), ("F2", "20"), ("F3", "30"), ("G1", "100")] {
        model._set(cell, value);
    }
    model._set("H1", "=LOOKUP(2,A1:B2,F1:F2)");
    model._set("H2", "=LOOKUP(2,A1:B1,F1:G2)");
    model._set("H3", "=LOOKUP(2,A1:B1,F1)");
    model._set("H4", "=LOOKUP(99,A1:A2,F1:F1)");
    model._set("H5", "=LOOKUP(99,A1:B2,{10})");
    model._set("H6", "=LOOKUP(2,{1,2,5;3,4,6},{7,8,9})");
    model._set("A3", "5");
    model._set("H7", "=LOOKUP(99,A1:A3,F1048575:F1048576)");
    model._set("H8", "=LOOKUP(99,A1:B1,XFD1)");
    model._set("H9", "=LOOKUP(0,A1:B2,F1:F2)");
    // Wide: A1:C2 has more columns than rows, so A1:C1 is searched.
    model._set("C1", "5");
    model._set("C2", "6");
    model._set("H10", "=LOOKUP(5,A1:C2,F1:F3)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "10");
    assert_eq!(model._get_text("H2"), "#N/A");
    assert_eq!(model._get_text("H3"), "100");
    assert_eq!(model._get_text("H4"), "100");
    assert_eq!(model._get_text("H5"), "#N/A");
    assert_eq!(model._get_text("H6"), "8");
    // Past the sheet's edge reads a blank: 0 (probes/lookup-vector-edges A15, A17).
    assert_eq!(model._get_text("H7"), "0");
    assert_eq!(model._get_text("H8"), "0");
    assert_eq!(model._get_text("H9"), "#N/A");
    assert_eq!(model._get_text("H10"), "30");
}

#[test]
fn test_lookup_single_cell_vector_and_scalar_result() {
    // probes/lookup-vector-edges A1-A8 (excel-graph): a single cell is a one-element lookup_vector
    // and a scalar result_vector is a one-element array, which is not read past.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "3");
    model._set("C1", "2");
    model._set("D1", "10");
    model._set("B1", "=LOOKUP(2,C1,D1)");
    model._set("B2", "=LOOKUP(3,C1,D1)");
    model._set("B3", "=LOOKUP(0,C1,D1)");
    model._set("B4", "=LOOKUP(2,A1:A2,5)");
    model._set("B5", "=LOOKUP(99,A1:A2,5)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "10");
    assert_eq!(model._get_text("B2"), "10");
    assert_eq!(model._get_text("B3"), "#N/A");
    assert_eq!(model._get_text("B4"), "5");
    assert_eq!(model._get_text("B5"), "#N/A");
}

#[test]
fn test_lookup_result_vector_at_take_keeps_the_reference() {
    // probes/anchorarray-lookup-implicit-xlookup-take (excel-graph, T-1527): `@TAKE`/`@XLOOKUP`
    // as result_vector is the one cell they pick, read on past to the right like a plain cell;
    // an `@` whose intersection fails is `#VALUE!`.
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("A2", "2"),
        ("A3", "3"),
        ("F1", "5"),
        ("F2", "6"),
    ] {
        model._set(cell, value);
    }
    model._set("H1", "7");
    model._set("B1", "=LOOKUP(1,A1:A3,@TAKE(F1:F3,1))");
    model._set("B2", "=LOOKUP(3,A1:A3,@TAKE(F1:F3,1))");
    model._set("B10", "=LOOKUP(3,A1:A3,@TAKE(F1:F3,2))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "5");
    assert_eq!(model._get_text("B2"), "7");
    assert_eq!(model._get_text("B10"), "#VALUE!");
}
