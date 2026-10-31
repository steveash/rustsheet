//! The parser's defined-name lookup (T-3195): one keyed probe gives what the scan of every name,
//! lower-casing both sides, gave.

use std::collections::HashMap;

use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::parser::DefinedNameS;

/// The lookup before T-3195: the sheet's names, then the workbook's, each compared lower-cased,
/// the first match in list order.
fn scanned(names: &[DefinedNameS], name: &str, sheet: u32) -> Option<(Option<u32>, String)> {
    let lower = name.to_lowercase();
    names
        .iter()
        .find(|(n, scope, _)| *scope == Some(sheet) && n.to_lowercase() == lower)
        .or_else(|| {
            names
                .iter()
                .find(|(n, scope, _)| scope.is_none() && n.to_lowercase() == lower)
        })
        .map(|(_, scope, formula)| (*scope, formula.clone()))
}

fn name(text: &str, scope: Option<u32>, formula: &str) -> DefinedNameS {
    (text.to_string(), scope, formula.to_string())
}

#[test]
fn lookups_match_the_scan_of_every_name() {
    let names = vec![
        name("Rate", None, "=1"),
        // A sheet name shadows the workbook name on its own sheet only.
        name("rate", Some(0), "=2"),
        // Two names folding to one key: the first in list order wins.
        name("Dup", None, "=3"),
        name("DUP", None, "=4"),
        name("dup", Some(1), "=5"),
        name("DuP", Some(1), "=6"),
        // Non-ASCII: `ß` has no one-letter upper case; `İ` lower-cases to `i` and a dot.
        name("Straße", None, "=7"),
        name("İndex", Some(0), "=8"),
        name("ǅx", None, "=9"),
    ];
    let parser = new_parser(
        vec!["Sheet1".to_string(), "Sheet2".to_string()],
        names.clone(),
        HashMap::new(),
    );
    let queries = [
        "rate",
        "RATE",
        "Rate",
        "dup",
        "DUP",
        "Dup",
        "straße",
        "STRAßE",
        "STRASSE",
        "strasse",
        "İndex",
        "i\u{307}ndex",
        "INDEX",
        "index",
        "ǆx",
        "Ǆx",
        "ǅX",
        "missing",
        "",
    ];
    for query in queries {
        for sheet in 0..3 {
            assert_eq!(
                parser.get_defined_name(query, sheet),
                scanned(&names, query, sheet),
                "{query:?} on sheet {sheet}"
            );
        }
    }
    // The cases the comparison stands for, spelled out.
    assert_eq!(
        parser.get_defined_name("RATE", 0),
        Some((Some(0), "=2".to_string()))
    );
    assert_eq!(
        parser.get_defined_name("RATE", 1),
        Some((None, "=1".to_string()))
    );
    assert_eq!(
        parser.get_defined_name("dup", 0),
        Some((None, "=3".to_string()))
    );
    assert_eq!(
        parser.get_defined_name("DUP", 1),
        Some((Some(1), "=5".to_string()))
    );
    assert_eq!(
        parser.get_defined_name("STRAßE", 0),
        Some((None, "=7".to_string()))
    );
    assert_eq!(parser.get_defined_name("STRASSE", 0), None);
}

/// Names set again later (a rename, a new sheet) are looked up through a new index.
#[test]
fn a_new_name_list_is_indexed_again() {
    let mut parser = new_parser(
        vec!["Sheet1".to_string()],
        vec![name("Old", None, "=1")],
        HashMap::new(),
    );
    assert!(parser.get_defined_name("old", 0).is_some());
    parser.set_worksheets_and_names(vec!["Sheet1".to_string()], vec![name("New", None, "=2")]);
    assert_eq!(parser.get_defined_name("old", 0), None);
    assert_eq!(
        parser.get_defined_name("NEW", 0),
        Some((None, "=2".to_string()))
    );
}
