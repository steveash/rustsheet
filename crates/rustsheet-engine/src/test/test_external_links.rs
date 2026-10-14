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
        ((0, 1, 1), SpillValue::Text("Hello!".to_string())),
        ((0, 2, 1), SpillValue::Boolean(true)),
        ((0, 3, 1), SpillValue::Error(Error::DIV)),
        ((1, 1, 1), SpillValue::Text("Test A1".to_string())),
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
    // Only a name for one cell is read.
    assert_eq!(eval(&mut model, "=[1]!Span"), "#BLOCKED!");
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

// `[0]!Name` resolves at parse time like `Name`, so ISREF and a rename see it (T-1459).
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
    assert_eq!(model._get_formula("A1"), "=ISREF([0]!Other)");
}
