#![allow(clippy::unwrap_used)]

//! Excel's type-skipping approximate search (T-0212). Expected values are
//! Excel's: docs/excel-semantics/lookup.md "Approximate-match lookups".

use crate::test::util::new_empty_model;

/// Sets `column` rows 1.. to `values` ("" leaves the cell blank), evaluates
/// each formula in a free column and returns the results.
fn eval_over(column: char, values: &[&str], formulas: &[&str]) -> Vec<String> {
    let mut model = new_empty_model();
    for (row, value) in values.iter().enumerate() {
        if !value.is_empty() {
            model._set(&format!("{column}{}", row + 1), value);
        }
    }
    for (row, formula) in formulas.iter().enumerate() {
        model._set(&format!("Z{}", row + 1), formula);
    }
    model.evaluate();
    (1..=formulas.len())
        .map(|row| model._get_text(&format!("Z{row}")))
        .collect()
}

#[test]
fn blanks_never_match() {
    // gen-m2/04cfd82f54dbc2b2 A1: 42 in {"abc", blank}.
    assert_eq!(
        eval_over('C', &["abc", ""], &["=VLOOKUP(42,C1:C2,1)"]),
        ["#N/A"]
    );
    // probes/lookup-approximate-search N17 (and T-0257's masked HLOOKUP).
    assert_eq!(
        eval_over('K', &["", "-1"], &["=VLOOKUP(TRUE,K1:K2,1,TRUE)"]),
        ["#N/A"]
    );
}

#[test]
fn no_cross_type_match() {
    // gen-m2/d0c29dd5c530e28c A67, 2cfe279fe77a000d A132.
    let results = eval_over(
        'D',
        &["1", "2", "42"],
        &[r##"=VLOOKUP("#N/A",D1:D3,1)"##, r#"=MATCH("TRUE",D1:D3)"#],
    );
    assert_eq!(results, ["#N/A", "#N/A"]);
}

#[test]
fn skips_upwards_past_other_types() {
    // gen-m2/fa437879253827be A146: top row {1,"ü",blank,-2.5} has -2.5 at 4.
    let mut model = new_empty_model();
    model._set("B3", "1");
    model._set("C3", "ü");
    model._set("E3", "-2.5");
    model._set("E4", "found");
    model._set("A1", "=HLOOKUP(-2.5,B3:E4,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "found");
}

#[test]
fn greater_value_moves_the_upper_bound() {
    // gen-m2/cbdabe32a32a4012 A87.
    let values = [
        "1",
        "123456789.123",
        "2",
        "",
        "'1",
        "'1.5",
        "abc",
        "100",
        "x",
        "FALSE",
    ];
    assert_eq!(eval_over('A', &values, &["=MATCH(3.14159,A1:A10)"]), ["1"]);
}

#[test]
fn equal_value_ends_at_the_edge_of_its_run() {
    // probes/lookup-approximate-search N1, N3, N4, N7; MATCH.xlsx C7.
    assert_eq!(
        eval_over('A', &["1", "2", "2", "2", "3", "4"], &["=MATCH(2,A1:A6)"]),
        ["4"]
    );
    assert_eq!(
        eval_over(
            'C',
            &["0", "0", "0", "2", "x", "2", "9"],
            &["=MATCH(2,C1:C7)"]
        ),
        ["4"]
    );
    assert_eq!(
        eval_over('D', &["9", "5", "5", "5", "1"], &["=MATCH(5,D1:D5,-1)"]),
        ["2"]
    );
    assert_eq!(
        eval_over('E', &["a", "b", "B", "b", "c"], &[r#"=MATCH("b",E1:E5)"#]),
        ["4"]
    );
    // gen-m2/a6d4377c7b1b92cd A33: 0.5 in {0.5, 1E+15} descending.
    assert_eq!(
        eval_over('B', &["0.5", "1E+15"], &["=MATCH(0.5,B1:B2,-1)"]),
        ["1"]
    );
}

#[test]
fn descending_search_on_mixed_data() {
    // gen-m2/ce315bb699e1be6f A395 shape: no number above the key is probed.
    assert_eq!(
        eval_over(
            'B',
            &["1", "abc", "2", "", "TRUE", "0.5"],
            &["=MATCH(3.14159,B1:B6,-1)"]
        ),
        ["#N/A"]
    );
}

#[test]
fn match_type_sign_selects_the_direction() {
    // probes/lookup-approximate-search N20, N21.
    let results = eval_over(
        'D',
        &["9", "5", "5", "5", "1"],
        &["=MATCH(5,D1:D5,-2)", "=MATCH(1,D1:D5,-7)"],
    );
    assert_eq!(results, ["2", "5"]);
    assert_eq!(
        eval_over('A', &["1", "2", "2", "2", "3", "4"], &["=MATCH(2,A1:A6,2)"]),
        ["4"]
    );
}

#[test]
fn blank_key_is_zero_in_an_approximate_search() {
    // probes/lookup-approximate-search N12, N13.
    let results = eval_over(
        'F',
        &["1", "", "0", "", "2"],
        &["=VLOOKUP(G1,F1:F5,1)", "=MATCH(G1,F1:F5)"],
    );
    assert_eq!(results, ["0", "3"]);
}

#[test]
fn exact_match_never_matches_a_blank() {
    // probes/lookup-approximate-search N9, N10, N23.
    let results = eval_over(
        'F',
        &["1", "", "0", "", "2"],
        &[
            "=MATCH(0,G1:G3,0)",
            "=VLOOKUP(0,G1:G3,1,FALSE)",
            "=VLOOKUP(0,F1:F5,1,FALSE)",
        ],
    );
    assert_eq!(results, ["#N/A", "#N/A", "0"]);
}

#[test]
fn text_search_over_mixed_data() {
    // gen-m2/f9d4df4efdfbcbd5 A8 shape: the only text cell is the match.
    assert_eq!(
        eval_over(
            'B',
            &["0.2", "1", "", "ABC", "TRUE", "5"],
            &[r#"=VLOOKUP("abc",B1:B6,1)"#]
        ),
        ["ABC"]
    );
}

#[test]
fn exact_match_blank_key_is_zero_and_numbers_are_exact() {
    // probes/lookup-approximate-search N32, N37, N38, N41.
    let results = eval_over(
        'F',
        &["1", "", "0", "", "2"],
        &[
            "=MATCH(G1,F1:F5,0)",
            "=MATCH(0.1+0.2,{0.3},0)",
            "=VLOOKUP(0.1+0.2,{0.3},1,FALSE)",
            r#"=VLOOKUP("",G1:G3,1,FALSE)"#,
        ],
    );
    assert_eq!(results, ["3", "#N/A", "#N/A", "#N/A"]);
}

#[test]
fn descending_match_passes_over_other_types() {
    // docs/excel-semantics/lookup.md "MATCH descending search scans from the top"
    // (probes/match-descending-argcheck J1-J8): Excel's values.
    assert_eq!(
        eval_over('A', &["0.1", "TRUE", "3.14159"], &["=MATCH(0.5,A1:A3,-1)"]),
        ["#N/A"]
    );
    assert_eq!(
        eval_over('B', &["3", "TRUE", "0.1"], &["=MATCH(0.5,B1:B3,-1)"]),
        ["1"]
    );
    assert_eq!(
        eval_over('C', &["5", "x", "3", "1"], &["=MATCH(2,C1:C4,-1)"]),
        ["3"]
    );
    // Nothing of the key's type at or below the midpoint: the search goes on
    // forward (gen-m2 b4392a6514a36ade Calc!A277).
    assert_eq!(
        eval_over('D', &["FALSE", "0.5"], &["=MATCH(-14,D1:D2,-1)"]),
        ["2"]
    );
    // An all-blank column has no cell of the key's type.
    assert_eq!(eval_over('E', &["", ""], &["=MATCH(1,E1:E2,-1)"]), ["#N/A"]);
}

#[test]
fn descending_match_scans_from_the_top() {
    // probes/lookup-match-descending-blank (excel-graph, T-2278): over an unsorted
    // vector the search stops before the first value below the key.
    let values = [
        "0.5", "0.3", "0", "-2.5", "2.5", "-1", "1", "0.1", "100", "",
    ];
    assert_eq!(
        eval_over(
            'E',
            &values,
            &[
                "=MATCH(J1,E1:E9,-1)",
                "=MATCH(0,E1:E9,-2.2)",
                "=MATCH(0.0001,E1:E9,-1)",
                "=MATCH(-0.0001,E1:E9,-1)",
                "=MATCH(\"\",E1:E9,-1)",
                "=MATCH(0,E1:E5,-1)",
            ],
        ),
        ["3", "3", "2", "3", "#N/A", "3"]
    );
    // An equal value ends the scan at the first of a run; a first value below the key is #N/A.
    assert_eq!(
        eval_over(
            'E',
            &["5", "3", "3", "1"],
            &["=MATCH(3,E1:E4,-1)", "=MATCH(9,E1:E4,-1)"]
        ),
        ["2", "#N/A"]
    );
}

#[test]
fn matrix_of_a_lookup_landing_past_a_short_result_vector_is_value() {
    // probes/lookup-short-result-vector A20, A23 (excel-graph, T-2278): LOOKUP reads on
    // past the one-cell E11 to blank E19, an empty value that shows as 0 but is a blank matrix.
    assert_eq!(
        eval_over(
            'B',
            &["", "FALSE", "3", "42", "-1", "ABC", "-0.0001", "-0.0001", "TRUE"],
            &[
                "=LOOKUP(TRUE,B2:B10,E11:E11)",
                "=MMULT(LOOKUP(TRUE,B2:B10,E11:E11),2.5)",
            ],
        ),
        ["0", "#VALUE!"]
    );
}

#[test]
fn approximate_searches_compare_numbers_as_exact_doubles() {
    // probes/t3073-lookup-near-equal-approximate A17, A30, A31, D30, D31 (excel-graph, T-3073):
    // a number a few ULP from the key is above or below it, never equal (T-3099).
    assert_eq!(
        eval_over(
            'E',
            &[
                "1",
                "=1+2^-52",
                "=1+2*2^-52",
                "2", // ascending
                "2",
                "=0.1+0.2",
                "0.5", // unsorted
                "2",
                "=1+2*2^-52",
                "=1+2^-52",
                "1",
                "0.5", // descending
            ],
            &[
                "=MATCH(0.3,E5:E7,1)",
                "=MATCH(1,E1:E4,1)",
                "=MATCH(1+2^-52,E1:E4,1)",
                "=MATCH(1,E8:E12,-1)",
                "=MATCH(1+2^-52,E8:E12,-1)",
                "=MATCH(1,E10:E12,-1)",
            ],
        ),
        ["#N/A", "1", "2", "4", "3", "2"]
    );
}
