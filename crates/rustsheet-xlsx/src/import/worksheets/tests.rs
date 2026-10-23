//! Unit tests of the worksheet importer.

use std::collections::HashMap;

use rustsheet_engine::types::Link;

use crate::import::worksheets::{load_hyperlinks, parse_range, parse_reference};
use crate::import::xml::XmlNode;

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
