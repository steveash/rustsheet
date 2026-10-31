//! Unit tests of the worksheet importer.

use std::collections::HashMap;

use rustsheet_engine::types::Link;

use crate::import::worksheets::{load_hyperlinks, parse_range, parse_reference};
use crate::import::xml::XmlNode;
use rustsheet_engine::expressions::utils::number_to_column;

#[test]
fn test_parse_range() {
    assert!(parse_range("3Aw").is_err());
    assert_eq!(parse_range("A1"), Ok((1, 1, 1, 1)));
    assert_eq!(parse_range("B5:C6"), Ok((5, 2, 6, 3)));
    assert!(parse_range("A1:A2:A3").is_err());
    assert!(parse_range("A1:34").is_err());
    assert!(parse_range("A").is_err());
    assert!(parse_range("12").is_err());
}

#[test]
fn parse_reference_works() {
    let cell_reference = parse_reference("📈 Overview!B2");
    assert!(cell_reference.is_ok());
    let cell_reference = cell_reference.unwrap();
    assert_eq!(cell_reference.sheet, "📈 Overview");
}

#[test]
fn load_hyperlinks_skips_malformed_and_caps_huge_ranges() {
    let xml = r#"<worksheet xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
        <hyperlinks>
            <hyperlink ref="B2" location="Target!A1"/>
            <hyperlink ref="B3"/>
            <hyperlink ref="B4" location=""/>
            <hyperlink ref="D1:E2" location="Target!A1"/>
            <hyperlink ref="G1:XFD1048576" location="Target!A1"/>
            <hyperlink ref="F1" r:id="rId9"/>
        </hyperlinks>
    </worksheet>"#;
    let doc = XmlNode::parse_str(xml).unwrap();
    let ws = &doc;
    // no relationships: the r:id hyperlink is dangling
    let rels = HashMap::new();

    let links = load_hyperlinks(ws, &rels).unwrap();

    let internal = Link::Internal {
        location: "Target!A1".to_string(),
        tooltip: None,
    };
    // B2 plus the four cells of D1:E2 plus the top-left of the huge range.
    // The hyperlinks with no location, an empty location or a dangling
    // relationship are skipped.
    assert_eq!(links.len(), 6);
    assert_eq!(links.get(&(2, 2)), Some(&internal));
    for (row, column) in [(1, 4), (1, 5), (2, 4), (2, 5)] {
        assert_eq!(links.get(&(row, column)), Some(&internal));
    }
    // the huge range is not expanded: only its top-left cell gets the link
    assert_eq!(links.get(&(1, 7)), Some(&internal));
    assert_eq!(links.get(&(2, 7)), None);
    assert_eq!(links.get(&(3, 2)), None);
    assert_eq!(links.get(&(4, 2)), None);
    assert_eq!(links.get(&(1, 6)), None);
}

/// A workbook's sheet names and the formulas written in its worksheets.
struct WorkbookFormulas {
    /// The sheet names, in tab order.
    sheets: Vec<String>,
    /// Each `<f>` text with its cell (`B7`), from every worksheet.
    formulas: Vec<(String, String)>,
}

/// The sheet names and `<f>` texts of the workbook at `path`.
fn workbook_formulas(path: &std::path::Path) -> WorkbookFormulas {
    use std::io::Read;

    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut sheets = Vec::new();
    let mut formulas = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let name = file.name().unwrap().to_string();
        if name != "xl/workbook.xml" && !name.starts_with("xl/worksheets/sheet") {
            continue;
        }
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        let root = XmlNode::parse_str(&text).unwrap();
        for node in root.descendants() {
            if node.has_tag_name("sheet") {
                sheets.extend(node.attribute("name").map(str::to_string));
            }
            if node.has_tag_name("c") {
                let formula = node.child("f").and_then(XmlNode::text);
                if let (Some(cell), Some(formula)) = (node.attribute("r"), formula) {
                    formulas.push((cell.to_string(), formula.to_string()));
                }
            }
        }
    }
    WorkbookFormulas { sheets, formulas }
}

/// The `.xlsx` files under `dir`, recursively. build.rs's `collect_xlsx` does the same for the
/// generated calc tests, but a build script's items cannot be imported from tests.
fn xlsx_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            xlsx_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "xlsx") {
            out.push(path);
        }
    }
}

/// `from_a1_to_rc` prints the same R1C1 text through one cache shared by a sheet's formulas as
/// with none (T-3318), for every formula of the vendored workbooks (the upstream calc tests and
/// the fixtures suite), as written and copied one row down and one column right (the copy has
/// the same tree, so it is read from the cache), plain and as an array formula.
#[test]
fn rc_text_cache_prints_what_a_fresh_parse_prints() {
    use rustsheet_engine::expressions::parser::new_parser_english;
    use rustsheet_engine::expressions::parser::stringify::to_english_string;
    use rustsheet_engine::expressions::types::CellReferenceRC;

    use crate::import::worksheets::{from_a1_to_rc, RcTextCache};

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    xlsx_files(&root.join("tests/calc_tests"), &mut files);
    xlsx_files(&root.join("../../eval/suites/fixtures"), &mut files);
    assert!(files.len() > 200, "vendored workbooks not found");
    let (mut checked, mut copies) = (0, 0);
    for path in files {
        let WorkbookFormulas { sheets, formulas } = workbook_formulas(&path);
        let Some(sheet) = sheets.first().cloned() else {
            continue;
        };
        let mut copy_parser = new_parser_english(sheets.clone(), vec![], HashMap::new());
        let mut parser = new_parser_english(sheets, vec![], HashMap::new());
        parser.set_loaded(true);
        let mut shared = RcTextCache::default();
        let mut convert = |formula: &str, context: &str, array: bool, cache: &mut RcTextCache| {
            from_a1_to_rc(
                formula.to_string(),
                &mut parser,
                cache,
                context.to_string(),
                array,
            )
            .map_err(|e| e.to_string())
        };
        for (cell, formula) in &formulas {
            let context = format!("{sheet}!{cell}");
            let Ok(at) = parse_reference(&context) else {
                continue;
            };
            let moved = CellReferenceRC {
                sheet: sheet.clone(),
                row: at.row + 1,
                column: at.column + 1,
            };
            let Some(column) = number_to_column(moved.column) else {
                continue;
            };
            let copy = to_english_string(&copy_parser.parse(formula, &at), &moved);
            let copy_context = format!("{sheet}!{column}{}", moved.row);
            for array in [false, true] {
                for (text, context) in
                    [(formula.as_str(), &context), (copy.as_str(), &copy_context)]
                {
                    let fresh = convert(text, context, array, &mut RcTextCache::default());
                    assert_eq!(
                        convert(text, context, array, &mut shared),
                        fresh,
                        "{} {context}: {text}",
                        path.display()
                    );
                }
            }
            checked += 1;
            copies += usize::from(copy != *formula);
        }
    }
    assert!(
        checked > 1_000 && copies > 1_000,
        "{checked} formulas, {copies} copies"
    );
}

/// `{-0}` and `{0}` parse to trees the derived `PartialEq` holds equal but print differently, so
/// neither may take the other's text from the cache, in either order (T-3318).
#[test]
fn rc_text_cache_keeps_the_sign_of_zero() {
    use rustsheet_engine::expressions::parser::new_parser_english;

    use crate::import::worksheets::{from_a1_to_rc, RcTextCache};

    let mut parser = new_parser_english(vec!["Sheet1".to_string()], vec![], HashMap::new());
    parser.set_loaded(true);
    for order in [["SUM({-0})", "SUM({0})"], ["SUM({0})", "SUM({-0})"]] {
        let mut cache = RcTextCache::default();
        for (row, formula) in order.iter().enumerate() {
            let context = format!("Sheet1!A{}", row + 1);
            let text = from_a1_to_rc(formula.to_string(), &mut parser, &mut cache, context, false);
            let expected = if formula.contains('-') { "{-0}" } else { "{0}" };
            assert!(text.unwrap().contains(expected), "{formula}");
        }
    }
}

/// The relocation key (T-3326) at the places the A1 parse reads its cell: each formula is copied
/// over a grid of cells, every copy is read through one shared cache and through a fresh one, and
/// the two must print the same. Mixed-`$` range corners swap order with the cell; LET names,
/// dotted ranges, defined names and a callee spelled like a cell are never keyed.
#[test]
fn relocation_key_gives_equal_trees_for_equal_keys() {
    use rustsheet_engine::expressions::parser::new_parser_english;
    use rustsheet_engine::expressions::parser::stringify::to_english_string;
    use rustsheet_engine::expressions::types::CellReferenceRC;

    use crate::import::relocation_key::KeyLexer;
    use crate::import::worksheets::{from_a1_to_rc, RcTextCache};

    let sheets = vec!["Sheet1".to_string(), "Other".to_string()];
    let mut parser = new_parser_english(sheets.clone(), vec![], HashMap::new());
    parser.set_loaded(true);
    let mut copier = new_parser_english(sheets, vec![], HashMap::new());
    let mut shared = RcTextCache::default();
    let mut lexer = KeyLexer::default();
    let at = |row, column| CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    };
    for formula in [
        "COUNTIFS($A$2:$A$9,B3,C$1:C3,\">0\")",
        "SUM(C$3:$B5)",
        "SUM($C3:B$5)+Other!$A1",
        "A1 B1:C3",
        "SUM(A1:A3 B2:C2)",
        "IF(A1>0,\"x\",#N/A)*{1,2;3,4}",
        "_xlfn.XLOOKUP(A1,B:B,C:C)",
        "SUM(1:1)+ABS(-0)",
        "LET(x1,1,x1+A1)",
        "_xlfn.LET(_xlpm.x,A1,_xlpm.x)",
        "A1.B2",
        "rate*A1",
        "A1(2)",
        "SUM(A1",
    ] {
        let origin = at(3, 3);
        // As stored (the copies below are as the engine prints them, `A1.B2` as `A1:B2`).
        let keyed = lexer.key(formula, &origin).is_some();
        let expect = !["LET", "A1.B2", "rate", "A1(2)"]
            .iter()
            .any(|s| formula.contains(s));
        assert_eq!(keyed, expect, "{formula}");
        for (row, column) in [(3, 3), (4, 3), (3, 4), (6, 1), (1, 6), (9, 9)] {
            let cell = at(row, column);
            let Some(name) = number_to_column(column) else {
                continue;
            };
            let copy = to_english_string(&copier.parse(formula, &origin), &cell);
            let context = format!("Sheet1!{name}{row}");
            for array in [false, true] {
                let fresh = from_a1_to_rc(
                    copy.clone(),
                    &mut parser,
                    &mut RcTextCache::default(),
                    context.clone(),
                    array,
                )
                .map_err(|e| e.to_string());
                let cached = from_a1_to_rc(
                    copy.clone(),
                    &mut parser,
                    &mut shared,
                    context.clone(),
                    array,
                )
                .map_err(|e| e.to_string());
                assert_eq!(cached, fresh, "{context}: {copy}");
            }
        }
    }
}
