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
fn column_subtotals_over_several_values_are_placed_by_counted_lines() {
    // probes/groupby-subtotals G!A781 (T-1700): each subtotal writes its values at its first
    // line's column plus the number of lines it covers. Apple and Pear (two years each) land on
    // their own years; Banana (one year) puts its Units in its first subtotal column; the Grand
    // Total (8 lines) its Units in Banana's second, Banana's own write winning over its Sales.
    let formula = format!("=PIVOTBY(A2:A11,B2:C11,D2:E11,{SUM},,,,2)");
    let grid = pivot(&formula, 8, 19);
    let east = [
        "East", "15", "6", "", "", "", "", "", "", "", "9", "", "", "30", "3", "", "", "", "",
    ];
    assert_eq!(grid[2], east);
    let south = [
        "South", "", "", "", "", "", "", "-3", "9", "9", "9", "", "", "", "", "", "", "", "",
    ];
    assert_eq!(grid[4], south);
    let total = [
        "Total", "15", "13", "49", "12", "", "", "9.5", "19", "19", "55", "20", "2", "37", "9", "",
        "", "", "",
    ];
    assert_eq!(grid[7], total);
    // G!A1141: with -2 the subtotal columns go first but show the same cells.
    let formula = format!("=PIVOTBY(A2:A11,B2:C11,D2:E11,{SUM},,,,-2)");
    let grid = pivot(&formula, 8, 19);
    let total = [
        "Total", "", "", "", "", "15", "13", "49", "12", "19", "55", "9.5", "19", "", "", "20",
        "2", "37", "9",
    ];
    assert_eq!(grid[7], total);
}

#[test]
fn one_column_group_shows_the_last_values_total_first() {
    // probes/pivotby-total-columns-one-group S!A49: with one column group the Total's writes
    // land past it, so the first Total column shows the Units totals and the second is "".
    let formula = format!("=PIVOTBY(A2:A11,B2:B11<>\"zz\",D2:E11,{SUM})");
    let grid = pivot(&formula, 7, 5);
    assert_eq!(grid[0], ["", "TRUE", "TRUE", "Total", "Total"]);
    assert_eq!(grid[1], ["East", "45", "9", "9", ""]);
    assert_eq!(grid[6], ["Total", "130.5", "55", "55", ""]);
}

#[test]
fn paired_functions_subtotal_columns_in_place() {
    // probes/pivotby-col-subtotals-values3 S!A1: two functions paired with two value columns
    // subtotal and total in place.
    let formula = format!("=PIVOTBY(A2:A11,B2:C11,D2:E11,HSTACK({SUM},LAMBDA(x,COUNT(x))),,,,2)");
    let grid = pivot(&formula, 9, 19);
    let total = [
        "Total", "15", "3", "49", "2", "64", "5", "9.5", "2", "9.5", "2", "20", "1", "37", "2",
        "57", "3", "130.5", "10",
    ];
    assert_eq!(grid[8], total);
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
        // 3 over two row fields is #VALUE! (probes/pivotby-followup P!A381).
        (
            "G10",
            "=PIVOTBY(A2:B11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,3)",
            "#VALUE!",
        ),
        // A one-argument function ignores a valid relative_to and refuses an invalid one
        // (probes/groupby-pivotby-functions P!A561, P!A581).
        (
            "G11",
            "=INDEX(PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,,,,,,1),2,2)",
            "15",
        ),
        (
            "G12",
            "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(x,SUM(x)),,,,,,,5)",
            "#VALUE!",
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

#[test]
fn a_row_of_functions_interleaves_per_column_group() {
    // probes/groupby-pivotby P!A121: a row of function names under the column keys, the Total
    // columns filled.
    let grid = pivot(
        "=PIVOTBY(A2:A11,C2:C11,D2:D11,HSTACK(_xleta.SUM,_xleta.MAX))",
        8,
        9,
    );
    assert_eq!(
        grid[0],
        ["", "2023", "2023", "2024", "2024", "2025", "2025", "Total", "Total"]
    );
    assert_eq!(
        grid[1],
        ["", "SUM", "MAX", "SUM", "MAX", "SUM", "MAX", "SUM", "MAX"]
    );
    assert_eq!(
        grid[2],
        ["East", "15", "10", "30", "30", "", "", "45", "30"]
    );
    assert_eq!(
        grid[7],
        ["Total", "35", "20", "86", "40", "9.5", "12.5", "130.5", "40"]
    );
    // probes/groupby-pivotby-functions2 P!A1: two functions over two value columns pair up, and
    // the Total columns are filled.
    let grid = pivot(
        "=PIVOTBY(A2:A11,C2:C11,D2:E11,HSTACK(_xleta.SUM,_xleta.MAX))",
        3,
        9,
    );
    assert_eq!(grid[2], ["East", "15", "5", "30", "3", "", "", "45", "5"]);
    // probes/groupby-pivotby-functions P!A621: shown headers, the labels after the column keys.
    let grid = pivot(
        "=PIVOTBY(A1:A11,C1:C11,D1:D11,HSTACK(_xleta.SUM,_xleta.MAX),3)",
        4,
        3,
    );
    assert_eq!(grid[0], ["", "Year", ""]);
    assert_eq!(grid[2], ["", "SUM", "MAX"]);
    assert_eq!(grid[3], ["Region", "Sales", "Sales"]);
}

#[test]
fn a_column_of_functions_repeats_each_row_group() {
    // probes/groupby-pivotby-functions P!A601, functions2 P!A41 and P!A61.
    let grid = pivot(
        "=PIVOTBY(A2:A11,C2:C11,D2:D11,VSTACK(_xleta.SUM,_xleta.MAX))",
        13,
        6,
    );
    assert_eq!(grid[0], ["", "", "2023", "2024", "2025", "Total"]);
    assert_eq!(grid[1], ["East", "SUM", "15", "30", "", "45"]);
    assert_eq!(grid[2], ["East", "MAX", "10", "30", "", "30"]);
    assert_eq!(grid[12], ["Total", "MAX", "20", "40", "12.5", "40"]);
    let grid = pivot(
        "=PIVOTBY(A1:A11,C1:C11,D1:D11,VSTACK(_xleta.SUM,_xleta.MAX),3,-1,,0)",
        5,
        5,
    );
    assert_eq!(grid[0], ["", "", "Year", "", ""]);
    assert_eq!(grid[2], ["Region", "", "Sales", "Sales", "Sales"]);
    assert_eq!(grid[3], ["Total", "SUM", "35", "86", "9.5"]);
    assert_eq!(grid[4], ["Total", "MAX", "20", "40", "12.5"]);
}

const SET: &str = "LAMBDA(x,y,SUM(y))";

#[test]
fn relative_to_gives_percentof_its_reference_set() {
    // probes/groupby-pivotby P!Y25 (omitted: the column total), P!A49 (1: the row total), P!M49
    // (2: the grand total), P!Y49 and P!A73 (3 and 4 with one field per axis: 1's and 0's).
    let percentof = |relative_to: &str| {
        pivot(
            &format!("=PIVOTBY(A2:A11,C2:C11,D2:D11,_xleta.PERCENTOF,,,,,,,{relative_to})"),
            7,
            5,
        )
    };
    let column = percentof("");
    assert_eq!(
        column[1],
        ["East", "0.428571429", "0.348837209", "", "0.344827586"]
    );
    assert_eq!(column[6], ["Total", "1", "1", "1", "1"]);
    let row = percentof("1");
    assert_eq!(row[1], ["East", "0.333333333", "0.666666667", "", "1"]);
    assert_eq!(
        row[6],
        ["Total", "0.268199234", "0.659003831", "0.072796935", "1"]
    );
    let grand = percentof("2");
    assert_eq!(
        grand[1],
        ["East", "0.114942529", "0.229885057", "", "0.344827586"]
    );
    assert_eq!(percentof("3"), row);
    assert_eq!(percentof("4"), column);
    // "1" and TRUE read as 1; 5, -1 and 0.5 are #VALUE! (functions P!A461-A541).
    assert_eq!(percentof("\"1\""), row);
    assert_eq!(percentof("TRUE"), row);
    for invalid in ["5", "-1", "0.5"] {
        assert_eq!(percentof(invalid)[0][0], "#VALUE!", "{invalid}");
    }
}

#[test]
fn parent_totals_over_two_fields() {
    // probes/groupby-pivotby-functions P!A181: with two row fields, 4 is the cell's column under
    // its parent row (East/Apple in 2023 is East's 15, its Total cell East's 45); the Total row
    // is the column's.
    let grid = pivot(
        &format!("=PIVOTBY(A2:B11,C2:C11,D2:D11,{SET},,1,,,,,4)"),
        10,
        6,
    );
    assert_eq!(grid[1], ["East", "Apple", "15", "", "", "45"]);
    assert_eq!(grid[5], ["West", "Apple", "20", "40", "", "72.5"]);
    assert_eq!(grid[9], ["Total", "", "35", "86", "9.5", "130.5"]);
    // P!A261: with two column fields, 3 is the cell's row under its parent column.
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,B2:C11,D2:D11,{SET},,,,1,,,3)"),
        8,
        7,
    );
    assert_eq!(grid[2], ["East", "15", "", "", "", "30", "45"]);
    assert_eq!(grid[7], ["Total", "64", "64", "9.5", "57", "57", "130.5"]);
    // P!A321: both: West/Apple in Apple 2023 is the West records of Apple 2023, 0.
    let grid = pivot(
        &format!("=PIVOTBY(A2:B11,B2:C11,D2:D11,{SET},,1,,1,,,4)"),
        7,
        8,
    );
    assert_eq!(grid[6], ["West", "Apple", "0", "40", "", "", "", "72.5"]);
}

#[test]
fn value_sorts_by_a_two_argument_function() {
    // probes/groupby-pivotby-functions3 P!A91: rows sort by their Total cell, 4 the parent's:
    // the Region parents tie at the grand total and keep their key order.
    let grid = pivot(
        &format!("=PIVOTBY(A2:B11,C2:C11,D2:D11,{SET},,1,3,,,,4)"),
        10,
        2,
    );
    let keys: Vec<&str> = grid.iter().skip(1).map(|row| row[0].as_str()).collect();
    assert_eq!(
        keys,
        ["East", "East", "North", "South", "West", "West", "West", "0", "Total"]
    );
    // probes/groupby-pivotby-functions4 P!A1, P!A21: columns sort by their Total row cell too: 3
    // ties the years at their product's total (2023 before 2024), 0 by their own records.
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,B2:C11,D2:D11,{SET},,,,1,-3,,3)"),
        2,
        7,
    );
    assert_eq!(grid[1], ["", "2023", "2024", "2025", "2023", "2024", ""]);
    assert_eq!(
        pivot(
            &format!("=PIVOTBY(A2:A11,B2:C11,D2:D11,{SET},,,,1,-3,,3)"),
            8,
            7
        )[7],
        ["Total", "64", "64", "9.5", "57", "57", "130.5"]
    );
    let grid = pivot(
        &format!("=PIVOTBY(A2:A11,B2:C11,D2:D11,{SET},,,,1,-3,,0)"),
        2,
        7,
    );
    assert_eq!(grid[1], ["", "2024", "2023", "2025", "2024", "2023", ""]);
}

#[test]
fn an_array_relative_to_is_lifted_per_element() {
    // probes/pivotby-followup P!A681, probes/groupby-pivotby-functions P!A721, P!A741: each
    // element is the top-left `""` of its PIVOTBY, or the element's error.
    let grid = pivot(
        "=PIVOTBY(A2:A11,C2:C11,D2:D11,_xleta.PERCENTOF,,,,,,,{5;1})",
        3,
        2,
    );
    assert_eq!(grid, [["#VALUE!", ""], ["", ""], ["", ""]]);
}

#[test]
fn two_parameter_lambdas_are_charged_against_the_visit_cap() {
    // DEVIATION (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY: two-parameter LAMBDAs and
    // the cell-visit cap"): 20000 row groups in one column, each cell given the 20000-record
    // column, visit 4E8 cells; the built-in PERCENTOF sums each relative set once.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=PIVOTBY(SEQUENCE(20000),SEQUENCE(20000,,0,0),SEQUENCE(20000),LAMBDA(x,y,ROWS(y)))",
    );
    model._set(
        "B1",
        "=INDEX(PIVOTBY(SEQUENCE(20000),SEQUENCE(20000,,0,0),SEQUENCE(20000),_xleta.PERCENTOF),20002,2)",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("B1"), "1");
}

#[test]
fn relative_sets_are_capped_before_they_are_built() {
    // DEVIATION (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY: two-parameter LAMBDAs and
    // the cell-visit cap"): 16 row and 16 column fields at depth 16 with relative_to 3 use 17 x 16
    // level pairs, 272 x 65535 set entries, past 2^24: #ERROR! before any set is built; depth 2
    // uses 3 x 2 and computes.
    let mut model = new_empty_model();
    let fields = "SEQUENCE(65535,16,1,0)";
    model._set(
        "A1",
        &format!("=PIVOTBY({fields},{fields},SEQUENCE(65535),{SET},,16,,16,,,3)"),
    );
    model._set(
        "B1",
        &format!("=ISERROR(PIVOTBY({fields},{fields},SEQUENCE(65535),{SET},,2,,2,,,3))"),
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("B1"), "FALSE");
}
