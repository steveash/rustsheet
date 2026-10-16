#![allow(clippy::unwrap_used)]

//! PIVOTBY (SPEC-0006 step 5, T-1326). Excel: probes/groupby-pivotby sheet P,
//! probes/pivotby-followup and the GROUPBY follow-ups (docs/excel-semantics/arrays.md "GROUPBY
//! and PIVOTBY").

use crate::test::util::new_empty_model;
use crate::Model;

/// The Region/Product/Year/Sales/Units records of probes/groupby-pivotby's Data sheet (A1:E11,
/// row 1 headers; A9 is blank and A6 is a lowercase `east`).
pub(super) fn records() -> Model {
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
        for (column, value) in ["A", "B", "C", "D", "E"].iter().zip(row) {
            if !value.is_empty() {
                model._set(&format!("{column}{}", i + 1), value);
            }
        }
    }
    model
}

const SUM: &str = "LAMBDA(x,SUM(x))";

/// The texts of row `row`, columns `first..=last` (letters).
pub(super) fn row(model: &Model, row: usize, first: char, last: char) -> Vec<String> {
    (first..=last)
        .map(|column| model._get_text(&format!("{column}{row}")))
        .collect()
}

/// Evaluates `formula` in G1 of the records and returns the texts of its rows, `width` columns
/// from G.
fn pivot(formula: &str, height: usize, width: u8) -> Vec<Vec<String>> {
    let mut model = records();
    model._set("G1", formula);
    model.evaluate();
    let last = char::from(b'G' + width - 1);
    (1..=height).map(|r| row(&model, r, 'G', last)).collect()
}

#[test]
fn grid_with_empty_text_for_missing_combinations_and_totals() {
    // P!A1: the corner and the East/2025 cell are "", West/Apple/2023 sums to 0 (P!A97).
    let grid = pivot(&format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM})"), 8, 6);
    assert_eq!(grid[0], ["", "2023", "2024", "2025", "Total", ""]);
    assert_eq!(grid[1], ["East", "15", "30", "", "45", ""]);
    assert_eq!(grid[2], ["North", "", "7", "", "7", ""]);
    assert_eq!(grid[5], ["0", "", "9", "", "9", ""]);
    assert_eq!(grid[6], ["Total", "35", "86", "9.5", "130.5", ""]);
    assert_eq!(grid[7], ["", "", "", "", "", ""]);
    let mut model = records();
    model._set("G1", &format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM})"));
    model._set("G20", "=ISTEXT(J2)");
    model.evaluate();
    assert_eq!(model._get_text("G20"), "TRUE");
}

#[test]
fn shown_and_generated_headers() {
    // P!M1: the column field name above the first column key, then the row field name and the
    // value name under every column.
    let grid = pivot(&format!("=PIVOTBY(A1:A11,C1:C11,D1:D11,{SUM},3)"), 4, 5);
    assert_eq!(grid[0], ["", "Year", "", "", ""]);
    assert_eq!(grid[1], ["", "2023", "2024", "2025", "Total"]);
    assert_eq!(grid[2], ["Region", "Sales", "Sales", "Sales", "Sales"]);
    assert_eq!(grid[3], ["East", "15", "30", "", "45"]);
    // probes/pivotby-followup P!A81: generated names, the column fields joined with ", ".
    let grid = pivot(&format!("=PIVOTBY(A1:B11,B1:C11,D1:D11,{SUM},2)"), 4, 3);
    assert_eq!(grid[0], ["", "", "Column Field 1, Column Field 2"]);
    assert_eq!(grid[3], ["Row Field 1", "Row Field 2", "Value 1"]);
    // P!A421: field_headers 3 over records consumes the first one; its year is a text name.
    let mut model = records();
    model._set("G1", &format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM},3)"));
    model._set("G20", "=ISTEXT(H1)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "2023");
    assert_eq!(model._get_text("G20"), "TRUE");
    assert_eq!(row(&model, 3, 'G', 'K'), ["East", "10", "10", "10", "10"]);
}

#[test]
fn totals_off_and_first() {
    // P!Y1 and P!A25: no total row, no total column.
    let grid = pivot(&format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM},,0,,0)"), 7, 5);
    assert_eq!(grid[0], ["", "2023", "2024", "2025", ""]);
    assert_eq!(grid[5], ["0", "", "9", "", ""]);
    assert_eq!(grid[6], ["", "", "", "", ""]);
    // probes/pivotby-followup P!A101: -1 puts both totals first.
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM},,-1,,-1)"),
        3,
        5,
    );
    assert_eq!(grid[0], ["", "Total", "2023", "2024", "2025"]);
    assert_eq!(grid[1], ["Total", "130.5", "35", "86", "9.5"]);
    assert_eq!(grid[2], ["East", "45", "15", "30", ""]);
}

#[test]
fn several_value_columns_have_empty_total_columns() {
    // P!Y97: values interleave per column key; both Total columns are "" in every row.
    let grid = pivot(&format!("=PIVOTBY(A2:A11,C2:C11,D2:E11,{SUM})"), 7, 9);
    assert_eq!(
        grid[0],
        ["", "2023", "2023", "2024", "2024", "2025", "2025", "Total", "Total"]
    );
    assert_eq!(grid[1], ["East", "15", "6", "30", "3", "", "", "", ""]);
    assert_eq!(
        grid[6],
        ["Total", "35", "15", "86", "21", "9.5", "19", "", ""]
    );
}

#[test]
fn key_and_value_sorts() {
    // P!M25: both axes descending, the blank key still last.
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM},,,-1,,-1)"),
        7,
        5,
    );
    assert_eq!(grid[0], ["", "2025", "2024", "2023", "Total"]);
    let keys: Vec<&str> = grid.iter().map(|r| r[0].as_str()).collect();
    assert_eq!(keys, ["", "West", "South", "North", "East", "0", "Total"]);
    // P!M121: rows by the total column ascending, columns by the Total row descending.
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM},,,2,,-2)"),
        7,
        5,
    );
    assert_eq!(grid[0], ["", "2024", "2023", "2025", "Total"]);
    let keys: Vec<&str> = grid.iter().map(|r| r[0].as_str()).collect();
    assert_eq!(keys, ["", "South", "North", "0", "East", "West", "Total"]);
    // probes/pivotby-followup P!A781: without a total column, rows keep the key order...
    let grid = pivot(&format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM},,,2,0)"), 3, 4);
    assert_eq!(grid[1][0], "East");
    // ...but columns sort by value without a total row (P!A821).
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM},,0,,,2)"),
        1,
        4,
    );
    assert_eq!(grid[0], ["", "2025", "2023", "2024"]);
    // P!A861: two row fields, parents by their aggregate and children by value.
    let grid = pivot(&format!("=PIVOTBY(A2:B11,C2:C11,D2:D11,{SUM},,1,3)"), 10, 2);
    let keys: Vec<String> = grid.iter().map(|r| r.join("/")).collect();
    assert_eq!(
        keys,
        [
            "/",
            "South/Banana",
            "North/Pear",
            "0/Apple",
            "East/Apple",
            "East/Pear",
            "West/Banana",
            "West/Pear",
            "West/Apple",
            "Total/"
        ]
    );
    // P!A1161: two column fields, only the children sort by value (descending); the parents keep
    // their key order.
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,B2:C11,D2:D11,{SUM},,,,0,-3)"),
        2,
        6,
    );
    assert_eq!(grid[0], ["", "Apple", "Apple", "Banana", "Pear", "Pear"]);
    assert_eq!(grid[1], ["", "2024", "2023", "2025", "2024", "2023"]);
}

#[test]
fn filter_and_argument_errors() {
    // P!Y73: filtering out 2025 drops that column and South.
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,{SUM},,,,,,C2:C11<>2025)"),
        2,
        4,
    );
    assert_eq!(grid[0], ["", "2023", "2024", "Total"]);
    // P!A1501: with a header row the filter may be as tall as the records.
    let grid = pivot(
        &format!("=PIVOTBY(A1:A11,C1:C11,D1:D11,{SUM},,,,,,C2:C11<>2025)"),
        6,
        4,
    );
    assert_eq!(grid[5], ["Total", "35", "86", "121"]);
    let mut model = records();
    let cases = [
        // All rows filtered out, a filter of another height (P!A281), short col_fields (P!A301),
        // sort index 0 (P!A141) and past the last column (P!A161), a non-function (P!A761),
        // row_total_depth 2 with one row field (P!A401).
        (
            "G1",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,,,,,C2:C11=1999)",
            "#VALUE!",
        ),
        (
            "G2",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,,,,,C2:C10=2023)",
            "#VALUE!",
        ),
        (
            "G3",
            "=PIVOTBY(A2:A11,C2:C10,D2:D11,LAMBDA(x,SUM(x)))",
            "#VALUE!",
        ),
        (
            "G4",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,,0)",
            "#VALUE!",
        ),
        (
            "G5",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,,,,3)",
            "#VALUE!",
        ),
        ("G6", "=PIVOTBY(A2:A11,C2:C11,D2:D11,5)", "#VALUE!"),
        (
            "G7",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,2)",
            "#VALUE!",
        ),
        ("G8", "=PIVOTBY(A2:A11,C2:C11,D2:D11,NA())", "#N/A"),
        // Deferred: column subtotals over several values (T-1699), relative_to (T-1327),
        // function arrays (T-1325). 3 over two row fields is #VALUE! (probes/pivotby-followup
        // P!A381).
        (
            "G9",
            "=PIVOTBY(A2:A11,B2:C11,D2:E11,LAMBDA(x,SUM(x)),,,,2)",
            "#N/IMPL!",
        ),
        (
            "G10",
            "=PIVOTBY(A2:B11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,3)",
            "#VALUE!",
        ),
        (
            "G11",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,,,,,,1)",
            "#N/IMPL!",
        ),
        (
            "G12",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,HSTACK(SUM,MAX))",
            "#N/IMPL!",
        ),
        // probes/pivotby-followup P!A1401-A1461: an error option beats an invalid one before or
        // after it and a non-function; P!A1481, P!A1341: an error in the filter and a key next to
        // a value column in a sort array are #VALUE!.
        (
            "G13",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),4,,,,NA())",
            "#N/A",
        ),
        ("G14", "=PIVOTBY(A2:A11,C2:C11,D2:D11,5,NA())", "#N/A"),
        (
            "G15",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,,,,,IF(C2:C11=2023,NA(),TRUE))",
            "#VALUE!",
        ),
        (
            "G16",
            "=PIVOTBY(A2:A11,C2:C11,D2:E11,LAMBDA(x,SUM(x)),,,{1,3})",
            "#VALUE!",
        ),
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
fn lambda_array_results_are_value_errors_per_cell() {
    // probes/pivotby-followup P!A341.
    let grid = pivot("=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,x))", 2, 5);
    assert_eq!(grid[1], ["East", "#VALUE!", "#VALUE!", "", "#VALUE!"]);
}

#[test]
fn cross_product_is_size_checked_before_any_call() {
    // SPEC-0006 Design step 6: 20000 row keys by 20000 column keys would be a 4E8-cell grid; it is
    // refused while grouping the columns, before the grid or any LAMBDA call is allocated. A
    // smaller cross product is computed.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=PIVOTBY(SEQUENCE(20000),SEQUENCE(20000),SEQUENCE(20000),LAMBDA(x,SUM(x)))",
    );
    model._set(
        "B1",
        "=ROWS(PIVOTBY(SEQUENCE(1048575),SEQUENCE(1048575,,0,0),SEQUENCE(1048575),LAMBDA(x,SUM(x))))",
    );
    model._set(
        "C1",
        "=LET(p,PIVOTBY(SEQUENCE(60),MOD(SEQUENCE(60),40),SEQUENCE(60),LAMBDA(x,SUM(x))),ROWS(p)&\"x\"&COLUMNS(p)&\" \"&INDEX(p,62,42))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    // One column group, but 1048575 row groups of three columns exceed the array bound; the row
    // grouping stops at the first group past it.
    assert_eq!(model._get_text("B1"), "#ERROR!");
    assert_eq!(model._get_text("C1"), "62x42 1830");
}
