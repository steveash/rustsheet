use std::collections::HashMap;
use std::io::Read;

use super::package::{ImportLimits, Package};
use super::xml::XmlNode;

use crate::error::XlsxError;

/// The workbook's shared strings plus a string -> first-index map, so interning an inline or
/// formula string during import is O(1) instead of a linear scan (untrusted sheets can hold
/// many distinct inline strings). The first occurrence of a duplicate still wins, so indices
/// match the linear search this replaces.
#[derive(Debug, Default)]
pub(crate) struct SharedStringTable {
    strings: Vec<String>,
    first_index: HashMap<String, usize>,
}

impl SharedStringTable {
    fn from_strings(strings: Vec<String>) -> Self {
        let mut first_index = HashMap::with_capacity(strings.len());
        for (i, s) in strings.iter().enumerate() {
            first_index.entry(s.clone()).or_insert(i);
        }
        SharedStringTable {
            strings,
            first_index,
        }
    }

    /// Index of `s`, appending it when it is not in the table yet.
    pub(crate) fn intern(&mut self, s: &str) -> usize {
        if let Some(&i) = self.first_index.get(s) {
            return i;
        }
        let i = self.strings.len();
        self.strings.push(s.to_string());
        self.first_index.insert(s.to_string(), i);
        i
    }

    pub(crate) fn into_strings(self) -> Vec<String> {
        self.strings
    }
}

/// Reads the list of shared strings in an Excel workbook
/// Note than in IronCalc we lose _internal_ styling of a string
/// See Section 18.4
pub(crate) fn read_shared_strings<R: Read + std::io::Seek>(
    archive: &mut Package<R>,
) -> Result<SharedStringTable, XlsxError> {
    let limits = *archive.limits();
    let strings = match archive.open_optional("xl/sharedStrings.xml")? {
        Some(file) => read_shared_strings_from_xml(XmlNode::parse(file)?, &limits)?,
        None => Vec::new(),
    };
    Ok(SharedStringTable::from_strings(strings))
}

#[cfg(test)]
fn read_shared_strings_from_string(text: &str) -> Result<Vec<String>, XlsxError> {
    read_shared_strings_from_xml(XmlNode::parse(text.as_bytes())?, &ImportLimits::default())
}

fn read_shared_strings_from_xml(
    doc: XmlNode,
    limits: &ImportLimits,
) -> Result<Vec<String>, XlsxError> {
    let mut shared_strings = Vec::new();
    let nodes: Vec<&XmlNode> = doc.descendants().filter(|n| n.has_tag_name("si")).collect();
    for node in nodes {
        // Only the plain `<t>` and the `<r><t>` rich-text runs are the cell text; phonetic
        // (furigana) runs `<rPh><t>` are not (ECMA-376 §18.4.6; the eval snapshot reader skips
        // them too).
        let text = node
            .children()
            .flat_map(|child| match child.tag_name() {
                "t" => vec![child],
                "r" => child.children().filter(|n| n.has_tag_name("t")).collect(),
                _ => Vec::new(),
            })
            .map(|n| decode_xlsx_escapes(n.text().unwrap_or("")))
            .collect::<Vec<String>>()
            .join("");
        limits.check_text(&text)?;
        shared_strings.push(text);
    }
    Ok(shared_strings)
}

/// Decodes Excel's `_xXXXX_` escape sequences for characters that are invalid in XML 1.0.
/// For example, `_x0001_` → U+0001 (SOH). Also used by the eval snapshot reader for expected values.
pub fn decode_xlsx_escapes(s: &str) -> String {
    if !s.contains("_x") {
        return s.to_string();
    }

    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Look for _xHHHH_ pattern (7 bytes: '_', 'x', 4 hex digits, '_')
        if i + 6 < len && bytes[i] == b'_' && bytes[i + 1] == b'x' && bytes[i + 6] == b'_' {
            let hex = &s[i + 2..i + 6];

            if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(code) = u32::from_str_radix(hex, 16) {
                    if let Some(c) = char::from_u32(code) {
                        result.push(c);
                        i += 7;
                        continue;
                    }
                }
            }
        }

        if let Some(c) = s[i..].chars().next() {
            result.push(c);
            i += c.len_utf8();
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn intern_keeps_first_index_and_appends_new_strings() {
        let mut table = SharedStringTable::from_strings(vec!["a".into(), "b".into(), "a".into()]);
        assert_eq!(table.intern("a"), 0);
        assert_eq!(table.intern("b"), 1);
        assert_eq!(table.intern("c"), 3);
        assert_eq!(table.intern("c"), 3);
        assert_eq!(table.into_strings(), ["a", "b", "a", "c"]);
    }

    #[test]
    fn intern_many_distinct_strings_is_not_quadratic() {
        let mut table = SharedStringTable::default();
        let n = 200_000;
        for i in 0..n {
            assert_eq!(table.intern(&format!("inline-{i}")), i);
        }
        assert_eq!(table.into_strings().len(), n);
    }

    #[test]
    fn test_shared_strings() {
        let xml_string = r#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="3" uniqueCount="3">
    <si>
        <t>A string</t>
    </si>
    <si>
        <t>A second String</t>
    </si>
    <si>
        <r>
            <t>Hello</t>
        </r>
            <r>
            <rPr>
                <b/>
                <sz val="11"/>
                <color rgb="FFFF0000"/>
                <rFont val="Inter"/>
                <family val="2"/>
                <scheme val="minor"/>
            </rPr>
            <t xml:space="preserve"> World</t>
        </r>
    </si>
</sst>"#;
        let shared_strings = read_shared_strings_from_string(xml_string.trim()).unwrap();
        assert_eq!(
            shared_strings,
            [
                "A string".to_string(),
                "A second String".to_string(),
                "Hello World".to_string()
            ]
        );
    }

    #[test]
    fn test_phonetic_runs_are_not_cell_text() {
        let xml_string = r#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="2" uniqueCount="2">
    <si>
        <r><t>東</t></r>
        <r><rPr><b/></rPr><t>京</t></r>
        <rPh sb="0" eb="1"><t>トウ</t></rPh>
        <rPh sb="1" eb="2"><t>キョウ</t></rPh>
        <phoneticPr fontId="1"/>
    </si>
    <si>
        <t>漢字</t>
        <rPh sb="0" eb="2"><t>カンジ</t></rPh>
    </si>
</sst>"#;
        let shared_strings = read_shared_strings_from_string(xml_string.trim()).unwrap();
        assert_eq!(shared_strings, ["東京".to_string(), "漢字".to_string()]);
    }

    #[test]
    fn test_decode_xlsx_escapes_control_chars() {
        assert_eq!(decode_xlsx_escapes("_x0001_"), "\x01");
        assert_eq!(decode_xlsx_escapes("_x001F_"), "\x1F");
        assert_eq!(decode_xlsx_escapes("_x000B_"), "\x0B");
        assert_eq!(decode_xlsx_escapes("hello_x0001_world"), "hello\x01world");
    }

    #[test]
    fn test_decode_xlsx_escapes_literal_underscore() {
        // _x005F_ decodes to underscore; the rest is literal
        assert_eq!(decode_xlsx_escapes("_x005F_x0001_"), "_x0001_");
        assert_eq!(decode_xlsx_escapes("_x005F_x005F_"), "_x005F_");
    }

    #[test]
    fn test_decode_xlsx_escapes_non_patterns() {
        // Strings that look similar but are not valid _xHHHH_ patterns
        assert_eq!(decode_xlsx_escapes("_xGGGG_"), "_xGGGG_"); // non-hex digits
        assert_eq!(decode_xlsx_escapes("_x001"), "_x001"); // too short
        assert_eq!(decode_xlsx_escapes("_x00001_"), "_x00001_"); // 5 hex digits
        assert_eq!(decode_xlsx_escapes("plain text"), "plain text");
        assert_eq!(decode_xlsx_escapes("_hello"), "_hello");
    }

    #[test]
    fn test_decode_xlsx_escapes_nul() {
        assert_eq!(decode_xlsx_escapes("_x0000_"), "\x00");
    }
}
