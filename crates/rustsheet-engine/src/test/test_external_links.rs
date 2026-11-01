#![allow(clippy::unwrap_used)]

// References into a linked workbook read the values Excel cached for it in the externalLink part
// (poi/ref-56737.xlsx, ref2-56737.xlsx Refs!E3, G3, I3; docs/excel-semantics/references.md,
// "External reference with no cached data is `#BLOCKED!`"; T-1382).

use std::collections::HashMap;

use crate::expressions::token::Error;
use crate::test::util::new_empty_model;
use crate::types::{DefinedName, ExternalBook, SpillValue};
use crate::Model;

// Book `[1]` caches what poi/ref-56737.xlsx externalLink1.xml does, plus a boolean and an error.
fn linked() -> Model {
    let mut model = new_empty_model();
    let cells = HashMap::from([
        ((0, 1, 1), SpillValue::Text("Hello!".into())),
        ((0, 2, 1), SpillValue::Boolean(true)),
        ((0, 3, 1), SpillValue::Error(Error::DIV)),
        ((1, 1, 1), SpillValue::Text("Test A1".into())),
        ((1, 2, 2), SpillValue::Number(142.0)),
    ]);
    let name = |name: &str, formula: &str, sheet_id| DefinedName {
        name: name.to_string(),
        formula: formula.to_string(),
        sheet_id,
    };
    model.workbook.external_books.push(ExternalBook {
        sheet_names: vec!["Uses".to_string(), "Defines".to_string()],
        defined_names: vec![
            name("NR_Global_B2", "='Defines'!$B$2", None),
            name("NR_To_A1", "='Defines'!$A$1", Some(1)),
            name("Span", "='Defines'!$A$1:$B$2", None),
        ],
        cells,
        refresh_error_sheets: Vec::new(),
    });
    model
}

fn eval(model: &mut Model, formula: &str) -> String {
    model._set("C1", formula);
    model.evaluate();
    model._get_text("C1")
}

#[test]
fn cell_references_read_the_cache() {
    let mut model = linked();
    assert_eq!(eval(&mut model, "=[1]Uses!$A$1"), "Hello!");
    assert_eq!(eval(&mut model, "=[1]uses!A2"), "TRUE");
    assert_eq!(eval(&mut model, "=[1]Uses!$A$3"), "#DIV/0!");
    assert_eq!(eval(&mut model, "=[1]Defines!B2*2"), "284");
    assert_eq!(eval(&mut model, "='[1]Defines'!$B$2"), "142");
    // A cell the cache does not hold is empty.
    assert_eq!(eval(&mut model, "=[1]Uses!$Z$9"), "0");
    // A sheet or book it holds no cache for is #BLOCKED!, as with no link at all.
    assert_eq!(eval(&mut model, "=[1]Nope!$A$1"), "#BLOCKED!");
    assert_eq!(eval(&mut model, "=[2]Uses!$A$1"), "#BLOCKED!");
    assert_eq!(eval(&mut model, "=[other.xlsx]Uses!$A$1"), "#BLOCKED!");
    // Still a reference of the written shape (T-1381).
    assert_eq!(eval(&mut model, "=ISREF([1]Uses!$A$1)"), "TRUE");
    // Aggregates and COUNTA read the cached cell as a reference cell (T-1460).
    assert_eq!(eval(&mut model, "=SUM([1]Defines!$B$2)"), "142");
    assert_eq!(eval(&mut model, "=SUM(([1]Defines!$B$2))"), "142");
    assert_eq!(eval(&mut model, "=AVERAGE([1]Defines!$B$2)"), "142");
    assert_eq!(eval(&mut model, "=COUNT([1]Defines!$B$2)"), "1");
    assert_eq!(eval(&mut model, "=COUNTA([1]Uses!$A$1)"), "1");
    assert_eq!(eval(&mut model, "=SUM([1]Uses!$A$1)"), "0");
    assert_eq!(eval(&mut model, "=SUM([1]Uses!$A$2)"), "0");
    // COUNTBLANK of another workbook is #VALUE!, cached or not.
    assert_eq!(eval(&mut model, "=COUNTBLANK([1]Uses!$Z$9)"), "#VALUE!");
    assert_eq!(eval(&mut model, "=COUNTBLANK([1]Uses!$A$1)"), "#VALUE!");
}

// A range of a cached sheet reads its cached cells as a range of this workbook would
// (fixtures/external-link-cached-ranges, excel-graph; T-1460).
#[test]
fn cached_ranges_read_as_references() {
    let mut model = linked();
    model.workbook.external_books[0].cells.extend([
        ((0, 1, 2), SpillValue::Number(10.0)),
        ((0, 2, 2), SpillValue::Number(5.0)),
    ]);
    assert_eq!(eval(&mut model, "=SUM([1]Uses!$A$1:$B$2)"), "15");
    assert_eq!(eval(&mut model, "=COUNT([1]Uses!$A$1:$B$2)"), "2");
    assert_eq!(eval(&mut model, "=COUNTA([1]Uses!$A$1:$B$2)"), "4");
    assert_eq!(eval(&mut model, "=COUNTA([1]Uses!$A$1:$B$9)"), "5");
    assert_eq!(eval(&mut model, "=SUM([1]Uses!$B:$B)"), "15");
    assert_eq!(
        eval(&mut model, "=SUM(([1]Uses!$A$1,[1]Uses!$B$1:$B$2))"),
        "15"
    );
    assert_eq!(eval(&mut model, "=SUM([1]Uses!$C$1:$D$4)"), "0");
    // An area past the cached cells keeps its blanks; only whole columns and rows are cut.
    assert_eq!(
        eval(&mut model, "=TEXTJOIN(\"|\",FALSE,[1]Uses!$B$1:$B$5)"),
        "10|5|||"
    );
    // An error in the range is the aggregate's error.
    assert_eq!(eval(&mut model, "=SUM([1]Uses!$A$1:$B$3)"), "#DIV/0!");
    // Lookups and INDEX read the cached values.
    assert_eq!(eval(&mut model, "=MATCH(5,[1]Uses!$B$1:$B$2,0)"), "2");
    assert_eq!(
        eval(&mut model, "=VLOOKUP(TRUE,[1]Uses!$A$1:$B$2,2,FALSE)"),
        "5"
    );
    assert_eq!(eval(&mut model, "=INDEX([1]Uses!$A$1:$B$2,1,1)"), "Hello!");
    assert_eq!(
        eval(&mut model, "=SUMPRODUCT(--ISTEXT([1]Uses!$A$1:$B$2))"),
        "1"
    );
    // Still a reference of the written shape.
    assert_eq!(eval(&mut model, "=ISREF([1]Uses!$A$1:$B$2)"), "TRUE");
    assert_eq!(eval(&mut model, "=ROWS([1]Uses!$A$1:$B$3)"), "3");
    // A sheet the cache does not hold stays #BLOCKED!.
    assert_eq!(eval(&mut model, "=SUM([1]Nope!$A$1:$B$2)"), "#BLOCKED!");
    assert_eq!(eval(&mut model, "=COUNTA([1]Nope!$A$1:$B$2)"), "0");
}

// A structured reference qualified by a workbook is `#REF!` whatever the link caches
// (corpus-bench/58994 F8; docs/excel-semantics/references.md; T-0482).
#[test]
fn structured_reference_into_a_linked_workbook_is_ref() {
    let mut model = linked();
    for (cell, formula) in [
        ("A1", "=ROWS([1]!Table3[#Data])"),
        ("A2", "=VLOOKUP(1,[1]!Table3[#Data],2,FALSE)"),
        ("A3", "=ROWS([1]Defines!Table3[Col])"),
    ] {
        model
            .set_loaded_user_input(0, cell[1..].parse().unwrap(), 1, formula.to_string())
            .unwrap();
    }
    model.evaluate();
    for row in 1..=3 {
        assert_eq!(model.get_formatted_cell_value(0, row, 1).unwrap(), "#REF!");
    }
    // A typed formula stays a parse error; same-workbook `Sales[[#Data],[Val]]` resolving to its
    // data area is covered by test_structured_references.
    assert_eq!(eval(&mut model, "=ROWS([1]!Table3[#Data])"), "#ERROR!");
}

#[test]
fn defined_names_read_the_cache() {
    let mut model = linked();
    assert_eq!(eval(&mut model, "=[1]!NR_Global_B2"), "142");
    assert_eq!(eval(&mut model, "=[1]!nr_global_b2+1"), "143");
    assert_eq!(eval(&mut model, "=[1]Defines!NR_To_A1"), "Test A1");
    // A sheet-scoped name is not a workbook name, and the reverse.
    assert_eq!(eval(&mut model, "=[1]!NR_To_A1"), "#BLOCKED!");
    assert_eq!(eval(&mut model, "=[1]Defines!NR_Global_B2"), "#BLOCKED!");
    assert_eq!(eval(&mut model, "=[1]!Missing"), "#BLOCKED!");
    assert_eq!(eval(&mut model, "=[3]!NR_Global_B2"), "#BLOCKED!");
    // A workbook name of several cells is that cached range (T-1460).
    assert_eq!(eval(&mut model, "=SUM([1]!Span)"), "142");
    assert_eq!(eval(&mut model, "=COUNTA([1]!Span)"), "2");
    assert_eq!(eval(&mut model, "=ROWS([1]!Span)"), "2");
    assert_eq!(eval(&mut model, "=ISREF([1]!Span)"), "TRUE");
    assert_eq!(eval(&mut model, "=INDEX([1]!Span,2,2)"), "142");
    // The formula text keeps the external prefix.
    model._set("C1", "=[1]Defines!NR_To_A1&[1]!NR_Global_B2");
    assert_eq!(
        model._get_formula("C1"),
        "=[1]Defines!NR_To_A1&[1]!NR_Global_B2"
    );
}

// `[0]` is this workbook (poi/56737.xlsx Uses!D3, D5, C5, Excel-cached).
#[test]
fn book_zero_is_this_workbook() {
    let mut model = linked();
    model._set("A1", "42");
    model
        .new_defined_name("Answer", None, "Sheet1!$A$1")
        .unwrap();
    model
        .new_defined_name("Local", Some(0), "Sheet1!$A$1")
        .unwrap();
    assert_eq!(eval(&mut model, "=[0]!Answer"), "42");
    assert_eq!(eval(&mut model, "=[0]Sheet1!Local+1"), "43");
    assert_eq!(eval(&mut model, "=[0]!Nope"), "#NAME?");
    assert_eq!(eval(&mut model, "=[0]Nope!Local"), "#REF!");
}

// `Sheet!LocalName` is the name scoped to that sheet of this workbook (poi/56737.xlsx Uses!D2,
// Excel-cached "Test A1"); `Sheet1!A1` is still a reference (T-1459).
#[test]
fn sheet_qualified_defined_name() {
    let mut model = new_empty_model();
    model.add_sheet("Sheet 2").unwrap();
    model._set("B1", "7");
    model
        .new_defined_name("Local", Some(1), "Sheet1!$B$1")
        .unwrap();
    model._set("A1", "=Sheet1!B1");
    model._set("A2", "='Sheet 2'!Local+1");
    model._set("A3", "=ISREF('Sheet 2'!Local)");
    model._set("A4", "=Sheet1!Nope");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
    assert_eq!(model._get_formula("A2"), "='Sheet 2'!Local+1");
    assert_eq!(model._get_text("A2"), "8");
    assert_eq!(model._get_text("A3"), "TRUE");
    assert_eq!(model._get_text("A4"), "#NAME?");
    // A rename keeps the prefix as typed.
    model
        .update_defined_name("Local", Some(1), "Renamed", Some(1), "Sheet1!$B$1")
        .unwrap();
    assert_eq!(model._get_formula("A2"), "='Sheet 2'!Renamed+1");
}

// `[0]!Name` resolves at parse time like `Name`, so ISREF and a rename see it (T-1459); it shows
// the workbook's file name as its prefix (probes/workbook-name-own-book-prefix, T-2834).
#[test]
fn book_zero_name_is_resolved_at_parse_time() {
    let mut model = new_empty_model();
    model._set("B1", "7");
    model
        .new_defined_name("CellName", None, "Sheet1!$B$1")
        .unwrap();
    model._set("A1", "=ISREF([0]!CellName)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
    model
        .update_defined_name("CellName", None, "Other", None, "Sheet1!$B$1")
        .unwrap();
    assert_eq!(model._get_formula("A1"), "=ISREF(model.xlsx!Other)");
}

// A local name never answers for a linked workbook's name of the same spelling (T-1459).
#[test]
fn local_name_does_not_shadow_a_linked_name() {
    let mut model = linked();
    model._set("A1", "42");
    model
        .new_defined_name("NR_Global_B2", None, "Sheet1!$A$1")
        .unwrap();
    model
        .new_defined_name("Missing", None, "Sheet1!$A$1")
        .unwrap();
    assert_eq!(eval(&mut model, "=SUM([1]!NR_Global_B2)"), "142");
    assert_eq!(eval(&mut model, "=ISREF([1]!Missing)"), "FALSE");
    assert_eq!(eval(&mut model, "=SUM([1]!Missing)"), "#BLOCKED!");
    model._set("C1", "=[1]!Missing");
    model
        .update_defined_name("Missing", None, "Renamed", None, "Sheet1!$A$1")
        .unwrap();
    assert_eq!(model._get_formula("C1"), "=[1]!Missing");
}

// A sparse cache (A1 and the last row) makes a whole column span the grid: a union of such
// columns must stay within the array bound, not allocate a column per area (directive 5).
#[test]
fn union_of_sparse_whole_columns_is_bounded() {
    let mut model = linked();
    model.workbook.external_books[0]
        .cells
        .insert((0, 1_048_576, 1), SpillValue::Number(1.0));
    assert_eq!(eval(&mut model, "=COUNT([1]Uses!$A:$A)"), "1");
    let columns = ["[1]Uses!$A:$A"; 8].join(",");
    assert_eq!(eval(&mut model, &format!("=SUM(({columns}))")), "#VALUE!");
}

// A lifted INDEX reads each picked cell from the cache, the top-left cell of a whole row or column
// (probes/external-ref-no-link-ifchoose H1, I1 for the no-cache case; T-1537). The cached case is
// UNVERIFIED until T-1546 probes it.
#[test]
fn lifted_index_reads_each_picked_cell_from_the_cache() {
    let mut model = linked();
    model._set("C1", "=INDEX([1]Uses!$A$1:$B$2,{1,2},1)");
    model._set("C2", "=INDEX([1]Uses!$A$1:$B$2,{0,2},1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "Hello!");
    assert_eq!(model._get_text("D1"), "TRUE");
    assert_eq!(model._get_text("C2"), "Hello!");
    assert_eq!(model._get_text("D2"), "TRUE");
}

// Book `[1]` caches what fixtures/external-link-cached-ranges does: `Uses` A1 "Hello!", B1 10,
// C1 #N/A, A2 5, B2 the text "7", C2 1, A3 TRUE.
fn cached_ranges_fixture() -> Model {
    let mut model = new_empty_model();
    let cells = HashMap::from([
        ((0, 1, 1), SpillValue::Text("Hello!".into())),
        ((0, 1, 2), SpillValue::Number(10.0)),
        ((0, 1, 3), SpillValue::Error(Error::NA)),
        ((0, 2, 1), SpillValue::Number(5.0)),
        ((0, 2, 2), SpillValue::Text("7".into())),
        ((0, 2, 3), SpillValue::Number(1.0)),
        ((0, 3, 1), SpillValue::Boolean(true)),
    ]);
    model.workbook.external_books.push(ExternalBook {
        sheet_names: vec!["Uses".to_string()],
        defined_names: vec![DefinedName {
            name: "NR_Range".to_string(),
            formula: "='Uses'!$A$1:$B$3".to_string(),
            sheet_id: None,
        }],
        cells,
        refresh_error_sheets: Vec::new(),
    });
    model
}

// The functions that take a reference see a cached range as one (fixtures/external-link-cached-ranges
// Refs!B53, B59, B60; T-1526).
#[test]
fn reference_functions_take_a_cached_range() {
    let mut model = cached_ranges_fixture();
    assert_eq!(eval(&mut model, "=SUBTOTAL(9,[1]Uses!$A$1:$B$3)"), "15");
    assert_eq!(eval(&mut model, "=SUBTOTAL(109,[1]!NR_Range)"), "15");
    assert_eq!(eval(&mut model, "=SUBTOTAL(9,[1]Uses!$C$1:$C$2)"), "#N/A");
    assert_eq!(eval(&mut model, "=AGGREGATE(9,6,[1]Uses!$A$1:$C$2)"), "16");
    assert_eq!(
        eval(&mut model, "=AGGREGATE(9,4,[1]Uses!$A$1:$C$2)"),
        "#N/A"
    );
    assert_eq!(eval(&mut model, "=AVERAGEA([1]Uses!$A$1:$B$3)"), "3.2");
    assert_eq!(eval(&mut model, "=AVERAGEA([1]!NR_Range)"), "3.2");
    // A cached error ranks as a referenced error: a scalar direct error beats it either side
    // (T-1573; UNVERIFIED, mirrors probes/aggregate-picked-direct-a-variants D20-D23).
    let cached_na = "[1]Uses!$C$1:$C$2";
    assert_eq!(eval(&mut model, &format!("=MAXA({cached_na})")), "#N/A");
    assert_eq!(
        eval(&mut model, &format!("=MAXA({cached_na},1/0)")),
        "#DIV/0!"
    );
    assert_eq!(
        eval(&mut model, &format!("=MAXA(1/0,{cached_na})")),
        "#DIV/0!"
    );
    // A sheet the cache does not hold stays #BLOCKED!.
    assert_eq!(
        eval(&mut model, "=SUBTOTAL(9,[1]Nope!$A$1:$B$3)"),
        "#BLOCKED!"
    );
    assert_eq!(
        eval(&mut model, "=AVERAGEA([1]Nope!$A$1:$B$3)"),
        "#BLOCKED!"
    );
}

// An aggregate that skips blanks reads an area of any shape only where the cache holds cells, so a
// tall area adds its cached cells instead of tripping the array bound (arrays.md "Array results
// above 1,048,576 cells"; T-1550). A reader that sees blanks keeps the written shape.
#[test]
fn blank_skipping_aggregates_read_a_tall_area_from_the_cache() {
    let mut model = cached_ranges_fixture();
    let tall = "[1]Uses!$A$2:$B$1048576";
    assert_eq!(eval(&mut model, &format!("=SUM({tall})")), "5");
    assert_eq!(eval(&mut model, &format!("=COUNT({tall})")), "1");
    assert_eq!(eval(&mut model, &format!("=MAX({tall})")), "5");
    assert_eq!(eval(&mut model, &format!("=MIN({tall})")), "5");
    assert_eq!(eval(&mut model, &format!("=PRODUCT({tall})")), "5");
    assert_eq!(eval(&mut model, &format!("=AVERAGE({tall})")), "5");
    // "7" counts 0 and TRUE 1 (Refs!B53's AVERAGEA rule).
    assert_eq!(eval(&mut model, &format!("=AVERAGEA({tall})")), "2");
    assert_eq!(eval(&mut model, &format!("=SUM(IF(TRUE,{tall}))")), "5");
    assert_eq!(eval(&mut model, &format!("=SUBTOTAL(9,{tall})")), "5");
    assert_eq!(eval(&mut model, &format!("=RANK(5,{tall})")), "1");
    // An area wholly past the cache reads no cell.
    let outside = "[1]Uses!$Z$100:$Z$200";
    assert_eq!(eval(&mut model, &format!("=SUM({outside})")), "0");
    assert_eq!(eval(&mut model, &format!("=AVERAGE({outside})")), "#DIV/0!");
    assert_eq!(
        eval(&mut model, &format!("=AVERAGEA({outside})")),
        "#DIV/0!"
    );
    // The cut changes no position an INDEX picks.
    assert_eq!(eval(&mut model, &format!("=SUM(INDEX({tall},1,1))")), "5");
    // Blanks past the cache still count where the reader sees them.
    assert_eq!(eval(&mut model, "=ROWS(LET(x,[1]Uses!$A$1:$B$9,x))"), "9");
    assert_eq!(
        eval(&mut model, "=TEXTJOIN(\"|\",FALSE,[1]Uses!$B$1:$B$5)"),
        "10|7|||"
    );
}

/// `formula` set in `cell` as a legacy (pre-dynamic-array) formula, as xlsx import reads one.
fn eval_legacy(model: &mut Model, cell: &str, formula: &str) -> String {
    let reference = model._parse_reference(cell);
    let text = model
        .legacy_formula_text(reference.sheet, reference.row, reference.column, formula)
        .unwrap();
    model._set(cell, &text);
    model.evaluate();
    model._get_text(cell)
}

// A legacy formula implicitly intersects a cached range as it does a range of this workbook
// (fixtures/external-link-cached-ranges Refs!B38, B44, B56, B62; T-1526).
#[test]
fn legacy_formula_intersects_a_cached_range() {
    let mut model = cached_ranges_fixture();
    let sum = "=SUM(IFERROR([1]Uses!$A$1:$B$3*1,100))";
    assert_eq!(eval_legacy(&mut model, "D38", sum), "100");
    let sumproduct = "=SUMPRODUCT(IFERROR([1]Uses!$A$1:$B$3*1,100))";
    assert_eq!(eval_legacy(&mut model, "D56", sumproduct), "100");
    assert_eq!(
        eval_legacy(&mut model, "D44", "=SUM([1]Uses!$A$2:$A$3+0)"),
        "#VALUE!"
    );
    // In a row the range spans, the intersection reads the cached cell.
    assert_eq!(
        eval_legacy(&mut model, "D2", "=SUM([1]Uses!$A$1:$A$3+0)"),
        "5"
    );
    assert_eq!(
        eval_legacy(&mut model, "B9", "=[1]Uses!$B$1:$B$3"),
        "#VALUE!"
    );
    assert_eq!(eval_legacy(&mut model, "B9", "=[1]Uses!$A$1:$C$1"), "10");
    // A legacy TYPE intersects too (Refs!B62; probes/legacy-type-intersection).
    assert_eq!(
        eval_legacy(&mut model, "D62", "=TYPE([1]Uses!$A$1:$B$3)"),
        "16"
    );
    assert_eq!(
        eval_legacy(&mut model, "D2", "=TYPE([1]Uses!$A$1:$A$3)"),
        "1"
    );
    // The same formula typed today (dynamic) reads the whole range.
    assert_eq!(eval(&mut model, "=SUM([1]Uses!$A$1:$A$3+0)"), "#VALUE!");
}

/// A LET whose body picks a reference into another workbook is that reference, its names bound
/// while it is read (probes/external-ref-no-link-let, external-ref-no-link-ifchoose A26; T-1535).
#[test]
fn let_body_picking_an_external_reference_is_that_reference() {
    let mut model = new_empty_model();
    let ext = "'[other.xlsx]Sheet1'!";
    for (formula, expected) in [
        (format!("=ROWS(LET(c,TRUE,IF(c,{ext}A1:B3)))"), "3"),
        (
            format!("=ROWS(LET(c,TRUE,LET(d,1,CHOOSE(d,{ext}A1:B4))))"),
            "4",
        ),
        (format!("=ROWS(IF(TRUE,LET(c,1,{ext}A1:B3)))"), "3"),
        (format!("=ROW(LET(c,TRUE,IF(c,{ext}B3)))"), "3"),
        (format!("=ISREF(LET(c,TRUE,IF(c,{ext}A1)))"), "TRUE"),
        (format!("=COUNTA(LET(c,TRUE,IF(c,{ext}A1:B2)))"), "0"),
        (format!("=ROWS(LET(c,FALSE,IF(c,{ext}A1:B3)))"), "1"),
        (format!("=ROWS(LET(c,1/0,IF(c,{ext}A1:B3)))"), "#DIV/0!"),
        // A body that may pick a reference into this workbook is read as before.
        (format!("=ROWS(LET(c,FALSE,IF(c,{ext}A1:B3,D1:D2)))"), "2"),
    ] {
        assert_eq!(eval(&mut model, &formula), expected, "{formula}");
    }
}

/// A LET whose body may pick a reference into another workbook or another argument is the
/// external reference when it picks it (probes/external-ref-no-link-let-index A3:A7), and
/// otherwise the reference or value it picks, read while its names are bound (T-1721).
#[test]
fn let_body_picking_an_external_reference_or_another_argument() {
    let mut model = new_empty_model();
    model._set("D1", "1");
    model._set("D2", "x");
    let ext = "'[other.xlsx]Sheet1'!";
    for (formula, expected) in [
        (format!("=ISREF(OFFSET(LET(c,TRUE,{ext}A1),1,1))"), "FALSE"),
        (format!("=OFFSET(LET(c,TRUE,{ext}A1),1/0,1)"), "#DIV/0!"),
        (format!("=ROWS(LET(c,TRUE,IF(c,{ext}A1:B3,\"x\")))"), "3"),
        (format!("=ROWS(LET(c,TRUE,IF(c,{ext}A1:B3,D1:D2)))"), "3"),
        (format!("=ISREF(LET(c,TRUE,IF(c,{ext}A1,D1)))"), "TRUE"),
        (format!("=COUNTA(LET(c,TRUE,IF(c,{ext}A1:B2,1)))"), "0"),
        (format!("=ROW(LET(c,TRUE,IF(c,{ext}B3,D1)))"), "3"),
        (
            format!("=AREAS(LET(c,TRUE,IF(c,({ext}A1,{ext}B2),D1)))"),
            "2",
        ),
        // The other argument, a name bound by the LET included.
        (format!("=ISREF(LET(c,FALSE,IF(c,{ext}A1,D1)))"), "TRUE"),
        (format!("=ISREF(LET(c,FALSE,IF(c,{ext}A1,1)))"), "FALSE"),
        (
            format!("=ROWS(LET(r,D1:D2,c,FALSE,IF(c,{ext}A1:B3,r)))"),
            "2",
        ),
        (
            format!("=COUNTA(LET(r,D1:D3,c,FALSE,IF(c,{ext}A1:B2,r)))"),
            "2",
        ),
        (
            format!("=SUM(LET(r,D1:D3,c,FALSE,IF(c,{ext}A1:B2,r)))"),
            "1",
        ),
        (format!("=ROW(LET(r,D2,c,FALSE,IF(c,{ext}B3,r)))"), "2"),
        (format!("=ROW(LET(c,FALSE,IF(c,{ext}B3,1)))"), "#VALUE!"),
        (format!("=AREAS(LET(c,FALSE,IF(c,{ext}A1,(D1,E1))))"), "2"),
        (
            format!("=ROWS(LET(c,FALSE,IF(c,{ext}A1,(D1,E1))))"),
            "#REF!",
        ),
        (
            format!("=INDEX(LET(r,D1:D2,c,FALSE,IF(c,{ext}A1,r)),2)"),
            "x",
        ),
        (
            format!("=COUNTBLANK(LET(r,D1:D3,c,FALSE,IF(c,{ext}A1,r)))"),
            "1",
        ),
        (
            format!("=ISREF(OFFSET(LET(c,FALSE,IF(c,{ext}A1,D1)),1,1))"),
            "TRUE",
        ),
        (format!("=OFFSET(LET(c,FALSE,IF(c,{ext}A1,D1)),1,0)"), "x"),
    ] {
        assert_eq!(eval(&mut model, &formula), expected, "{formula}");
    }
}

// Book `[1]` caches what fixtures/external-link-cached-3d-span does: `Uses` as in
// [`cached_ranges_fixture`], `Defines` A1 "Test A1", B2 142, and an empty `56737`.
fn cached_span_fixture() -> Model {
    let mut model = cached_ranges_fixture();
    let book = &mut model.workbook.external_books[0];
    book.sheet_names
        .extend(["Defines".to_string(), "56737".to_string()]);
    book.cells
        .insert((1, 1, 1), SpillValue::Text("Test A1".into()));
    book.cells.insert((1, 2, 2), SpillValue::Number(142.0));
    model
}

// A 3-D span of the cached sheets aggregates over them, in either order; it is no reference to
// ISREF, `#REF!` as a value and `#VALUE!` where one sheet is needed, as a span of this workbook
// is (fixtures/external-link-cached-3d-span Refs!B2:B10, excel-graph; T-1654).
#[test]
fn span_of_cached_sheets_aggregates_over_them() {
    let mut model = cached_span_fixture();
    assert_eq!(eval(&mut model, "=SUM([1]Uses:Defines!$B$2)"), "142");
    assert_eq!(eval(&mut model, "=SUM([1]Uses:Defines!$A$1:$B$2)"), "157");
    assert_eq!(eval(&mut model, "=COUNT([1]Uses:Defines!$A$1:$C$3)"), "4");
    assert_eq!(eval(&mut model, "=COUNTA([1]Uses:Defines!$A$1:$C$3)"), "9");
    assert_eq!(eval(&mut model, "=MAX([1]Uses:Defines!$B$1:$B$2)"), "142");
    assert_eq!(eval(&mut model, "=ISREF([1]Uses:Defines!$A$1)"), "FALSE");
    assert_eq!(eval(&mut model, "=ERROR.TYPE([1]Uses:Defines!$A$1)"), "4");
    assert_eq!(eval(&mut model, "=SUM([1]Defines:Uses!$B$2)"), "142");
    assert_eq!(
        eval(&mut model, "=ROWS([1]Uses:Defines!$A$1:$B$3)"),
        "#VALUE!"
    );
}

#[test]
fn span_of_cached_sheets_round_trips() {
    let mut model = cached_span_fixture();
    model._set("C1", "=SUM([1]Uses:Defines!$A$1:$B$2)");
    assert_eq!(
        model._get_formula("C1"),
        "=SUM('[1]Uses:Defines'!$A$1:$B$2)"
    );
}

// A span over many cached sheets walks the cache once, not once per sheet (CLAUDE.md
// directive 5); an end the cache does not hold stays `#BLOCKED!`, and a malformed span prefix does
// not parse as one (T-1654).
#[test]
fn span_of_many_cached_sheets_is_one_walk() {
    let mut model = cached_span_fixture();
    let book = &mut model.workbook.external_books[0];
    book.sheet_names
        .extend((0..100_000).map(|index| format!("P{index}x")));
    book.cells.insert((100_001, 7, 1), SpillValue::Number(1.0));
    assert_eq!(eval(&mut model, "=SUM([1]Uses:P99999x!$A:$B)"), "158");
    assert_eq!(eval(&mut model, "=COUNTA([1]Uses:P99999x!$A:$C)"), "10");
    assert_eq!(eval(&mut model, "=SUM([1]Uses:Nope!$B$2)"), "#BLOCKED!");
    for formula in ["=[1]A:!B1", "=[1]:B!A1", "=[1]A:B:C!A1"] {
        model._set("C1", formula);
        model.evaluate();
        assert_eq!(model._get_text("C1"), "#ERROR!", "{formula}");
    }
}

// A sheet `sheetNames` lists but the cache holds no cell of reads `#BLOCKED!`, yet a lookup over
// it is `#N/A` (fixtures/external-link-uncached-lookups, excel-graph; T-0457).
#[test]
fn lookups_into_an_uncached_linked_sheet() {
    let mut model = linked();
    model.workbook.external_books[0]
        .sheet_names
        .push("Sort".to_string());
    assert_eq!(eval(&mut model, "=[1]Sort!$B$2"), "#BLOCKED!");
    assert_eq!(eval(&mut model, "=ISBLANK([1]Sort!$B$2)"), "FALSE");
    assert_eq!(eval(&mut model, "=SUM([1]Sort!$B$2:$C$3)"), "#BLOCKED!");
    assert_eq!(eval(&mut model, "=VLOOKUP(5,[1]Sort!$B$2,1,FALSE)"), "#N/A");
    assert_eq!(
        eval(&mut model, "=VLOOKUP(5,[1]Sort!$B:$Z,2,FALSE)"),
        "#N/A"
    );
    assert_eq!(
        eval(&mut model, "=VLOOKUP(5,[1]Sort!$B:$Z,30,FALSE)"),
        "#N/A"
    );
    assert_eq!(
        eval(&mut model, "=HLOOKUP(5,[1]Sort!$1:$3,2,FALSE)"),
        "#N/A"
    );
    assert_eq!(eval(&mut model, "=MATCH(5,[1]Sort!$B$2,0)"), "#N/A");
    assert_eq!(eval(&mut model, "=MATCH(5,[1]Sort!$B:$B)"), "#N/A");
    // XLOOKUP over one cell finds nothing, as over a range; COUNTA counts every blocked cell
    // (fixtures/external-link-uncached-lookups Refs!B13, B16, B44; T-0472).
    assert_eq!(
        eval(&mut model, "=XLOOKUP(5,[1]Sort!$B$2,[1]Sort!$C$2)"),
        "#N/A"
    );
    assert_eq!(
        eval(&mut model, "=XLOOKUP(5,[1]Sort!$B$2,[1]Sort!$C$2,\"none\")"),
        "none"
    );
    assert_eq!(eval(&mut model, "=COUNTA([1]Sort!$B:$B)"), "1048576");
    // A sheet marked refreshError reads #REF! where it reads no cell (Refs!B38); its lookups
    // stay #N/A (B32).
    model.workbook.external_books[0]
        .sheet_names
        .push("56737".to_string());
    model.workbook.external_books[0]
        .refresh_error_sheets
        .push(3);
    assert_eq!(eval(&mut model, "=ERROR.TYPE('[1]56737'!$B$2)"), "4");
    assert_eq!(
        eval(&mut model, "=VLOOKUP(5,'[1]56737'!$B:$Z,2,FALSE)"),
        "#N/A"
    );
}

// A criteria or sum range of a cached sheet is #VALUE!, as any argument that is not a range of
// this workbook is, cached or not; it is that error without its array being built (T-0477). One
// too large for an array keeps its bound's error.
#[test]
fn criteria_ranges_of_a_cached_sheet_are_value_errors() {
    let mut model = linked();
    for formula in [
        "=COUNTIF([1]Uses!$A$1:$A$3,\"Hello!\")",
        "=COUNTIFS([1]Uses!$A:$A,\"Hello!\")",
        "=COUNTIFS(A1:A3,1,[1]Uses!$A$1:$A$3,1)",
        "=SUMIF([1]Uses!$A$1:$A$3,1)",
        "=SUMIFS(A1:A3,[1]Uses!$A$1:$A$3,1)",
        "=SUMIFS([1]Uses!$A$1:$A$3,A1:A3,1)",
        "=AVERAGEIFS([1]Defines!$B:$B,[1]Defines!$A:$A,\"<>\")",
        "=MAXIFS(A1:A3,[1]Uses!$A$1:$A$3,1)",
        "=COUNTIF([1]Uses!$A$1,\"Hello!\")",
    ] {
        assert_eq!(eval(&mut model, formula), "#VALUE!", "{formula}");
    }
    assert_eq!(eval(&mut model, "=COUNTIF([1]Nope!$A$1:$A$3,1)"), "#VALUE!");
    // Two whole columns are past the array bound (docs/excel-semantics/arrays.md).
    assert_eq!(eval(&mut model, "=COUNTIF([1]Uses!$A:$B,1)"), "#ERROR!");
}

#[test]
fn only_multi_cell_external_ranges_are_external_arrays() {
    use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
    let mut model = linked();
    let cell = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 3,
    };
    let context = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 3,
    };
    // The predicate agrees with what evaluating the range builds.
    let mut is_array = |range: &str| {
        let node = model.parse_internal_formula(range, &context);
        let predicted = model.is_external_array(&node, cell);
        let built = matches!(
            model.evaluate_node_in_context(&node, cell),
            crate::calc_result::CalcResult::Array(_)
        );
        assert_eq!(predicted, built, "{range}");
        predicted
    };
    assert!(is_array("[1]Uses!$A$1:$A$3"));
    assert!(is_array("[1]Uses!$A:$A"));
    assert!(!is_array("[1]Uses!$A$1"));
    assert!(is_array("[1]Nope!$A$1:$A$3"));
    assert!(is_array("[1]Nope!$A:$A"));
    assert!(!is_array("[1]Uses!$A:$B"));
    assert!(!is_array("A1:A3"));
}

// A whole column or row of a cached linked sheet is searched at full length, as a local one is: an
// approximate search over unsorted data probes the same midpoints, so the cache's extent never
// moves the answer (T-0474; T-0457's review). Each lookup is checked against the same lookup over
// the same values in this workbook (H:J and rows 10:12; the rows are cached on [1]Defines).
#[test]
fn whole_column_lookups_keep_full_length() {
    let mut model = linked();
    let keys = [1.0, 5.0, 2.0, 9.0, 3.0];
    let book = &mut model.workbook.external_books[0];
    book.cells.clear();
    for (i, key) in (1..).zip(keys) {
        book.cells.extend([
            ((0, i, 1), SpillValue::Number(key)),
            ((0, i, 2), SpillValue::Number(key * 10.0)),
            ((1, 1, i), SpillValue::Number(key)),
            ((1, 2, i), SpillValue::Number(key * 10.0)),
        ]);
    }
    for (i, key) in (1..).zip(keys) {
        model._set(&format!("H{i}"), &key.to_string());
        model._set(&format!("I{i}"), &(key * 10.0).to_string());
    }
    for (column, key) in ["A", "B", "C", "D", "E"].into_iter().zip(keys) {
        model._set(&format!("{column}10"), &key.to_string());
        model._set(&format!("{column}11"), &(key * 10.0).to_string());
    }
    let pairs = [
        ("VLOOKUP(1,{t}!$A:$C,2,TRUE)", "VLOOKUP(1,$H:$J,2,TRUE)"),
        ("VLOOKUP(4,{t}!$A:$C,2,TRUE)", "VLOOKUP(4,$H:$J,2,TRUE)"),
        ("VLOOKUP(9,{t}!$A:$C,2)", "VLOOKUP(9,$H:$J,2)"),
        ("VLOOKUP(99,{t}!$A:$C,3,TRUE)", "VLOOKUP(99,$H:$J,3,TRUE)"),
        ("VLOOKUP(2,{t}!$A:$C,2,FALSE)", "VLOOKUP(2,$H:$J,2,FALSE)"),
        ("VLOOKUP(7,{t}!$A:$C,2,FALSE)", "VLOOKUP(7,$H:$J,2,FALSE)"),
        ("MATCH(4,{t}!$A:$A)", "MATCH(4,$H:$H)"),
        ("MATCH(1,{t}!$A:$A,1)", "MATCH(1,$H:$H,1)"),
        ("MATCH(4,{t}!$A:$A,-1)", "MATCH(4,$H:$H,-1)"),
        ("MATCH(9,{t}!$A:$A,0)", "MATCH(9,$H:$H,0)"),
        ("HLOOKUP(4,{r}!$1:$3,2,TRUE)", "HLOOKUP(4,$10:$12,2,TRUE)"),
        ("HLOOKUP(5,{r}!$1:$3,3,FALSE)", "HLOOKUP(5,$10:$12,3,FALSE)"),
        ("MATCH(4,{r}!$1:$1)", "MATCH(4,$10:$10)"),
    ];
    for (linked, local) in pairs {
        let linked = linked
            .replace("{t}", "[1]Uses")
            .replace("{r}", "[1]Defines");
        let expected = eval(&mut model, &format!("={local}"));
        assert_eq!(
            eval(&mut model, &format!("={linked}")),
            expected,
            "{linked}"
        );
    }
    // The answer full length gives over this unsorted column, not the 2 a cut vector would.
    assert_eq!(eval(&mut model, "=VLOOKUP(4,[1]Uses!$A:$C,1,TRUE)"), "1");
}

// Lookups of one pass into the same long cached column share one search vector, and lookups into
// a linked sheet share one index of where its cached cells end (search_cache.rs): each still finds
// its own key, exact or approximate, and an empty column of the sheet is all blank.
#[test]
fn whole_column_lookups_share_the_cached_vector() {
    let mut model = linked();
    let book = &mut model.workbook.external_books[0];
    book.cells.clear();
    book.cells
        .extend((1..=100).map(|row| ((0, row, 1), SpillValue::Number(f64::from(row) * 2.0))));
    model._set("C1", "=MATCH(40,[1]Uses!$A:$A,0)");
    model._set("C2", "=MATCH(41,[1]Uses!$A:$A)");
    model._set("C3", "=VLOOKUP(200,[1]Uses!$A:$B,1,FALSE)");
    model._set("C4", "=MATCH(1,[1]Uses!$A:$A,0)");
    model._set("C5", "=MATCH(1,[1]Uses!$C:$C,0)");
    model._set("C6", "=VLOOKUP(1,[1]Uses!$C:$D,2)");
    model._set("C7", "=MATCH(2,[1]Uses!$1:$1,0)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "20");
    assert_eq!(model._get_text("C2"), "20");
    assert_eq!(model._get_text("C3"), "200");
    assert_eq!(model._get_text("C4"), "#N/A");
    assert_eq!(model._get_text("C5"), "#N/A");
    assert_eq!(model._get_text("C6"), "#N/A");
    assert_eq!(model._get_text("C7"), "1");
}
