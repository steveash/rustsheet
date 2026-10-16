#![allow(clippy::unwrap_used)]

//! GROUPBY and PIVOTBY subtotals (`total_depth` beyond 1) and an array `field_relationship` (SPEC-0006
//! step 3, T-1324). Excel: probes/groupby-subtotals, probes/groupby-pivotby G!J49, G!S49, P!A97,
//! P!M97 and the follow-ups (docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY").

use super::test_fn_pivotby::{records, row};
use crate::test::util::new_empty_model;

/// Evaluates `formula` in G1 of the records and checks its spill: one `|`-joined text per row,
/// as wide as the first, and nothing below it.
fn assert_spill(formula: &str, expected: &[&str]) {
    let mut model = records();
    model._set("G1", formula);
    model.evaluate();
    let width = expected.first().map_or(1, |first| first.split('|').count());
    let last = char::from(b'G' + u8::try_from(width).unwrap() - 1);
    let actual: Vec<String> = (1..=expected.len() + 1)
        .map(|r| row(&model, r, 'G', last).join("|"))
        .collect();
    let mut wanted: Vec<String> = expected.iter().map(ToString::to_string).collect();
    wanted.push(vec![""; width].join("|"));
    assert_eq!(actual, wanted, "{formula}");
}

/// The texts of one-cell formulas, evaluated over the records.
fn values(formulas: &[&str]) -> Vec<String> {
    let mut model = records();
    for (i, formula) in formulas.iter().enumerate() {
        model._set(&format!("G{}", i + 1), formula);
    }
    model.evaluate();
    (1..=formulas.len())
        .map(|r| model._get_text(&format!("G{r}")))
        .collect()
}

#[test]
fn subtotals_below_and_above_their_groups() {
    // G!J49, G!S49: a subtotal per first key, `""` under the second; "Grand Total".
    assert_spill(
        "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,2)",
        &[
            "East|Apple|15",
            "East|Pear|30",
            "East||45",
            "North|Pear|7",
            "North||7",
            "South|Banana|-3",
            "South||-3",
            "West|Apple|40",
            "West|Banana|12.5",
            "West|Pear|20",
            "West||72.5",
            "0|Apple|9",
            "0||9",
            "Grand Total||130.5",
        ],
    );
    assert_spill(
        "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,-2)",
        &[
            "Grand Total||130.5",
            "East||45",
            "East|Apple|15",
            "East|Pear|30",
            "North||7",
            "North|Pear|7",
            "South||-3",
            "South|Banana|-3",
            "West||72.5",
            "West|Apple|40",
            "West|Banana|12.5",
            "West|Pear|20",
            "0||9",
            "0|Apple|9",
        ],
    );
}

#[test]
fn three_keys_subtotal_the_levels_the_depth_names() {
    // probes/groupby-subtotals G!A1 (depth 2: the first level only), G!A61, G!A91 (3 and -3:
    // both levels, the deepest first below and the shallowest first above).
    assert_spill(
        "=GROUPBY(A2:C11,D2:D11,_xleta.SUM,,3)",
        &[
            "East|Apple|2023|15",
            "East|Apple||15",
            "East|Pear|2024|30",
            "East|Pear||30",
            "East|||45",
            "North|Pear|2024|7",
            "North|Pear||7",
            "North|||7",
            "South|Banana|2025|-3",
            "South|Banana||-3",
            "South|||-3",
            "West|Apple|2023|0",
            "West|Apple|2024|40",
            "West|Apple||40",
            "West|Banana|2025|12.5",
            "West|Banana||12.5",
            "West|Pear|2023|20",
            "West|Pear||20",
            "West|||72.5",
            "0|Apple|2024|9",
            "0|Apple||9",
            "0|||9",
            "Grand Total|||130.5",
        ],
    );
    assert_spill(
        "=GROUPBY(A2:C11,D2:D11,_xleta.SUM,,-3)",
        &[
            "Grand Total|||130.5",
            "East|||45",
            "East|Apple||15",
            "East|Apple|2023|15",
            "East|Pear||30",
            "East|Pear|2024|30",
            "North|||7",
            "North|Pear||7",
            "North|Pear|2024|7",
            "South|||-3",
            "South|Banana||-3",
            "South|Banana|2025|-3",
            "West|||72.5",
            "West|Apple||40",
            "West|Apple|2023|0",
            "West|Apple|2024|40",
            "West|Banana||12.5",
            "West|Banana|2025|12.5",
            "West|Pear||20",
            "West|Pear|2023|20",
            "0|||9",
            "0|Apple||9",
            "0|Apple|2024|9",
        ],
    );
    // G!Z1, G!Z1231, G!AA1261: 15 rows at depth 2; four keys take depth 4 but not 5.
    assert_eq!(
        values(&[
            "=ROWS(GROUPBY(A2:C11,D2:D11,_xleta.SUM,,2))",
            "=ROWS(GROUPBY(HSTACK(A2:C11,E2:E11>5),D2:D11,_xleta.SUM,,4))",
            "=GROUPBY(HSTACK(A2:C11,E2:E11>5),D2:D11,_xleta.SUM,,5)",
            "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,3)",
        ]),
        ["15", "32", "#VALUE!", "#VALUE!"]
    );
}

#[test]
fn a_value_sort_with_the_depth_omitted_shows_every_subtotal() {
    // probes/pivotby-followup P!A921; three keys, both levels: groupby-subtotals G!A121 (23 rows);
    // an explicit depth keeps its layout (followup2 G!A261: 0).
    assert_spill(
        "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,,3)",
        &[
            "South|Banana|-3",
            "South||-3",
            "North|Pear|7",
            "North||7",
            "0|Apple|9",
            "0||9",
            "East|Apple|15",
            "East|Pear|30",
            "East||45",
            "West|Banana|12.5",
            "West|Pear|20",
            "West|Apple|40",
            "West||72.5",
            "Grand Total||130.5",
        ],
    );
    assert_eq!(
        values(&[
            "=ROWS(GROUPBY(A2:C11,D2:D11,_xleta.SUM,,,4))",
            "=INDEX(GROUPBY(A2:C11,D2:D11,_xleta.SUM,,,4),2,3)",
            "=ROWS(GROUPBY(A2:B11,D2:D11,_xleta.SUM,,0,3))",
        ]),
        ["23", "", "8"]
    );
}

#[test]
fn a_table_sorted_by_a_value_sorts_every_group_alone() {
    // probes/groupby-subtotals G!A391 (no subtotals with the depth omitted), G!A451, G!A1381.
    assert_spill(
        "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,,3,,1)",
        &[
            "South|Banana|-3",
            "North|Pear|7",
            "0|Apple|9",
            "West|Banana|12.5",
            "East|Apple|15",
            "West|Pear|20",
            "East|Pear|30",
            "West|Apple|40",
            "Total||130.5",
        ],
    );
    assert_spill(
        "=GROUPBY(A2:B11,D2:D11,_xleta.SUM,,0,-3,,1)",
        &[
            "West|Apple|40",
            "East|Pear|30",
            "West|Pear|20",
            "East|Apple|15",
            "West|Banana|12.5",
            "0|Apple|9",
            "North|Pear|7",
            "South|Banana|-3",
        ],
    );
    // Table subtotals stay #VALUE! (followup G!A181, G!A261).
    assert_eq!(
        values(&["=GROUPBY(A2:C11,D2:D11,_xleta.SUM,,-3,,,1)"]),
        ["#VALUE!"]
    );
}

#[test]
fn subtotals_get_their_rows_in_source_order_below_the_headers() {
    // probes/groupby-subtotals G!A331, G!A301.
    assert_spill(
        "=GROUPBY(A2:B11,D2:D11,LAMBDA(x,ARRAYTOTEXT(x)),,2)",
        &[
            "East|Apple|10, 5",
            "East|Pear|30",
            "East||10, 30, 5",
            "North|Pear|7",
            "North||7",
            "South|Banana|-3",
            "South||-3",
            "West|Apple|40, 0",
            "West|Banana|12.5",
            "West|Pear|20",
            "West||20, 40, 0, 12.5",
            "0|Apple|9",
            "0||9",
            "Grand Total||10, 20, 30, 40, 5, 7, 0, 9, -3, 12.5",
        ],
    );
    assert_spill(
        "=GROUPBY(A1:B11,D1:D11,_xleta.SUM,3,-2)",
        &[
            "Region|Product|Sales",
            "Grand Total||130.5",
            "East||45",
            "East|Apple|15",
            "East|Pear|30",
            "North||7",
            "North|Pear|7",
            "South||-3",
            "South|Banana|-3",
            "West||72.5",
            "West|Apple|40",
            "West|Banana|12.5",
            "West|Pear|20",
            "0||9",
            "0|Apple|9",
        ],
    );
}

#[test]
fn an_array_field_relationship_is_lifted_per_element() {
    // followup G!A861 and G!A981, followup2 G!A161 (text is #VALUE!, a blank 0), probes/
    // groupby-subtotals G!A511 (an invalid sort_order per element), G!A541 (an error element),
    // G!A571 (a row), G!A601 (the header cell), G!A631 (1 with subtotals), G!A1351 (an error
    // value in an earlier argument, per element).
    assert_eq!(
        values(&[
            "=ARRAYTOTEXT(GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,C2:C11=1999))",
            "=ARRAYTOTEXT(GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,C2:C11))",
            "=ARRAYTOTEXT(GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,A2:A11))",
            "=ARRAYTOTEXT(GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,9,,C2:C11=1999))",
            "=ARRAYTOTEXT(GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,IF(C2:C11=2025,NA(),0)))",
            "=ARRAYTOTEXT(GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,,,{0,1}),1)",
            "=ARRAYTOTEXT(GROUPBY(A1:A11,D1:D11,_xleta.SUM,3,,,,{0;1}))",
            "=ARRAYTOTEXT(GROUPBY(A2:B11,D2:D11,_xleta.SUM,,2,,,{0;1}))",
            "=ARRAYTOTEXT(GROUPBY(A2:A11,D2:D11,_xleta.SUM,,,NA(),,{0;1}))",
        ]),
        [
            "East, East, East, East, East, East, East, East, East, East",
            "#VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!",
            "#VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, East, #VALUE!, #VALUE!",
            "#VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!, #VALUE!",
            "East, East, East, East, East, East, East, East, #N/A, #N/A",
            "{\"East\",\"East\"}",
            "Region, Region",
            "East, #VALUE!",
            "#N/A, #N/A",
        ]
    );
}

#[test]
fn subtotal_rows_count_toward_the_output_size() {
    // CLAUDE.md directive 5: 200,000 two-key groups fit (600,000 cells), but their 200,000
    // subtotals would double the output past the array bound, so GROUPBY is refused before any
    // LAMBDA call; half as many fit with their subtotals.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=GROUPBY(HSTACK(SEQUENCE(200000),SEQUENCE(200000)),SEQUENCE(200000),LAMBDA(x,SUM(x)),,2)",
    );
    model._set(
        "B1",
        "=ROWS(GROUPBY(HSTACK(SEQUENCE(100000),SEQUENCE(100000)),SEQUENCE(100000),LAMBDA(x,SUM(x)),,2))",
    );
    model._set(
        "C1",
        "=PIVOTBY(HSTACK(SEQUENCE(200000),SEQUENCE(200000)),SEQUENCE(200000,,0,0),SEQUENCE(200000),LAMBDA(x,SUM(x)),,2,,0)",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("B1"), "200001");
    assert_eq!(model._get_text("C1"), "#ERROR!");
}

#[test]
fn pivot_row_subtotals() {
    // P!A97; probes/groupby-subtotals G!A661 (-2), G!A751 (two values: the Total columns stay
    // ""), G!A1021 (a value sort with col_total_depth 0 keeps the key order, but its omitted
    // row depth still shows subtotals); probes/pivotby-followup P!A201.
    assert_spill(
        "=PIVOTBY(A2:B11,C2:C11,D2:D11,_xleta.SUM,,2)",
        &[
            "||2023|2024|2025|Total",
            "East|Apple|15|||15",
            "East|Pear||30||30",
            "East||15|30||45",
            "North|Pear||7||7",
            "North|||7||7",
            "South|Banana|||-3|-3",
            "South||||-3|-3",
            "West|Apple|0|40||40",
            "West|Banana|||12.5|12.5",
            "West|Pear|20|||20",
            "West||20|40|12.5|72.5",
            "0|Apple||9||9",
            "0|||9||9",
            "Grand Total||35|86|9.5|130.5",
        ],
    );
    assert_spill(
        "=PIVOTBY(A2:B11,C2:C11,D2:D11,_xleta.SUM,,-2)",
        &[
            "||2023|2024|2025|Total",
            "Grand Total||35|86|9.5|130.5",
            "East||15|30||45",
            "East|Apple|15|||15",
            "East|Pear||30||30",
            "North|||7||7",
            "North|Pear||7||7",
            "South||||-3|-3",
            "South|Banana|||-3|-3",
            "West||20|40|12.5|72.5",
            "West|Apple|0|40||40",
            "West|Banana|||12.5|12.5",
            "West|Pear|20|||20",
            "0|||9||9",
            "0|Apple||9||9",
        ],
    );
    assert_spill(
        "=PIVOTBY(A2:B11,C2:C11,D2:E11,_xleta.SUM,,2)",
        &[
            "||2023|2023|2024|2024|2025|2025|Total|Total",
            "East|Apple|15|6||||||",
            "East|Pear|||30|3||||",
            "East||15|6|30|3||||",
            "North|Pear|||7|6||||",
            "North||||7|6||||",
            "South|Banana|||||-3|9||",
            "South||||||-3|9||",
            "West|Apple|0|7|40|4||||",
            "West|Banana|||||12.5|10||",
            "West|Pear|20|2||||||",
            "West||20|9|40|4|12.5|10||",
            "0|Apple|||9|8||||",
            "0||||9|8||||",
            "Grand Total||35|15|86|21|9.5|19||",
        ],
    );
    assert_spill(
        "=PIVOTBY(A2:B11,C2:C11,D2:D11,_xleta.SUM,,,3)",
        &[
            "||2023|2024|2025|Total",
            "South|Banana|||-3|-3",
            "South||||-3|-3",
            "North|Pear||7||7",
            "North|||7||7",
            "0|Apple||9||9",
            "0|||9||9",
            "East|Apple|15|||15",
            "East|Pear||30||30",
            "East||15|30||45",
            "West|Banana|||12.5|12.5",
            "West|Pear|20|||20",
            "West|Apple|0|40||40",
            "West||20|40|12.5|72.5",
            "Grand Total||35|86|9.5|130.5",
        ],
    );
    assert_spill(
        "=PIVOTBY(A2:B11,C2:C11,D2:D11,_xleta.SUM,,,3,0)",
        &[
            "||2023|2024|2025",
            "East|Apple|15||",
            "East|Pear||30|",
            "East||15|30|",
            "North|Pear||7|",
            "North|||7|",
            "South|Banana|||-3",
            "South||||-3",
            "West|Apple|0|40|",
            "West|Banana|||12.5",
            "West|Pear|20||",
            "West||20|40|12.5",
            "0|Apple||9|",
            "0|||9|",
            "Grand Total||35|86|9.5",
        ],
    );
}

#[test]
fn pivot_column_subtotals() {
    // P!M97; probes/groupby-subtotals G!A691 (-2), G!A871 (shown headers: the column names over
    // the first group, not a subtotal), G!A991 (a value sort orders the parents too), G!A1441
    // (source order), G!A721 (both axes).
    assert_spill(
        "=PIVOTBY(A2:A11,B2:C11,D2:D11,_xleta.SUM,,,,2)",
        &[
            "|Apple|Apple|Apple|Banana|Banana|Pear|Pear|Pear|Grand Total",
            "|2023|2024||2025||2023|2024||",
            "East|15||15||||30|30|45",
            "North|||||||7|7|7",
            "South||||-3|-3||||-3",
            "West|0|40|40|12.5|12.5|20||20|72.5",
            "0||9|9||||||9",
            "Total|15|49|64|9.5|9.5|20|37|57|130.5",
        ],
    );
    assert_spill(
        "=PIVOTBY(A2:A11,B2:C11,D2:D11,_xleta.SUM,,,,-2)",
        &[
            "|Grand Total|Apple|Apple|Apple|Banana|Banana|Pear|Pear|Pear",
            "|||2023|2024||2025||2023|2024",
            "East|45|15|15||||30||30",
            "North|7||||||7||7",
            "South|-3||||-3|-3|||",
            "West|72.5|40|0|40|12.5|12.5|20|20|",
            "0|9|9||9|||||",
            "Total|130.5|64|15|49|9.5|9.5|57|20|37",
        ],
    );
    assert_spill(
        "=PIVOTBY(A1:A11,B1:C11,D1:D11,_xleta.SUM,3,,,-2)",
        &[
            "|||Product, Year||||||",
            "|Grand Total|Apple|Apple|Apple|Banana|Banana|Pear|Pear|Pear",
            "|||2023|2024||2025||2023|2024",
            "Region|Sales|Sales|Sales|Sales|Sales|Sales|Sales|Sales|Sales",
            "East|45|15|15||||30||30",
            "North|7||||||7||7",
            "South|-3||||-3|-3|||",
            "West|72.5|40|0|40|12.5|12.5|20|20|",
            "0|9|9||9|||||",
            "Total|130.5|64|15|49|9.5|9.5|57|20|37",
        ],
    );
    assert_spill(
        "=PIVOTBY(A2:A11,B2:C11,D2:D11,_xleta.SUM,,,,-2,-3)",
        &[
            "|Grand Total|Apple|Apple|Apple|Pear|Pear|Pear|Banana|Banana",
            "|||2024|2023||2024|2023||2025",
            "East|45|15||15|30|30|||",
            "North|7||||7|7|||",
            "South|-3|||||||-3|-3",
            "West|72.5|40|40|0|20||20|12.5|12.5",
            "0|9|9|9||||||",
            "Total|130.5|64|49|15|57|37|20|9.5|9.5",
        ],
    );
    assert_spill(
        "=PIVOTBY(A2:A11,B2:C11,D2:D11,LAMBDA(x,ARRAYTOTEXT(x)),,,,2)",
        &[
            "|Apple|Apple|Apple|Banana|Banana|Pear|Pear|Pear|Grand Total",
            "|2023|2024||2025||2023|2024||",
            "East|10, 5||10, 5||||30|30|10, 30, 5",
            "North|||||||7|7|7",
            "South||||-3|-3||||-3",
            "West|0|40|40, 0|12.5|12.5|20||20|20, 40, 0, 12.5",
            "0||9|9||||||9",
            "Total|10, 5, 0|40, 9|10, 40, 5, 0, 9|-3, 12.5|-3, 12.5|20|30, 7|20, 30, 7|10, 20, 30, 40, 5, 7, 0, 9, -3, 12.5",
        ],
    );
    assert_spill(
        "=PIVOTBY(B2:C11,HSTACK(A2:A11,E2:E11>5),D2:D11,_xleta.SUM,,2,,2)",
        &[
            "||East|East|North|North|South|South|West|West|West|0|0|Grand Total",
            "||FALSE||TRUE||TRUE||FALSE|TRUE||TRUE||",
            "Apple|2023|15|15||||||0|0|||15",
            "Apple|2024|||||||40||40|9|9|49",
            "Apple||15|15|||||40|0|40|9|9|64",
            "Banana|2025|||||-3|-3||12.5|12.5|||9.5",
            "Banana||||||-3|-3||12.5|12.5|||9.5",
            "Pear|2023|||||||20||20|||20",
            "Pear|2024|30|30|7|7||||||||37",
            "Pear||30|30|7|7|||20||20|||57",
            "Grand Total||45|45|7|7|-3|-3|60|12.5|72.5|9|9|130.5",
        ],
    );
}
