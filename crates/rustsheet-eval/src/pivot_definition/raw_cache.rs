//! The author's raw XML in a defined pivot's cache (T-4767): a grouped field's `groupItems`
//! slot (`_group_items` of `tools/src/rustsheet_tools/pivot_definition.py`), which replaces the
//! builder's `<groupItems>` in its `<fieldGroup>`, and `records_xml` (`_records_xml`), which
//! replaces one record value. They let a probe write group items of every type and attribute and
//! a record value with a tuple list (`<tpls>`), which the builder does not write itself. A spec is
//! untrusted input (directive 5): each is one element of an allowed tag (`raw_element`), and
//! `records_xml` is bounded like every definition list (`list`).

use std::collections::HashMap;

use serde_json::Value as Json;

use super::checks::raw_element;
use super::{DefinedPivot, Field};

/// The value elements a `records_xml` entry may write (`_RECORD_TAGS`, `CT_Record`'s children).
const RECORD_TAGS: [&str; 7] = ["m", "n", "b", "e", "s", "d", "x"];

/// `group` (a cache field's `<fieldGroup>`) with the field's `xml.groupItems` in place of the
/// builder's `<groupItems>`; `group` itself when the field has none.
pub(super) fn group_items(f: &Field<'_>, group: &str) -> Result<String, String> {
    let Some(raw) = (f.entry("xml").and_then(|x| x.get("groupItems"))).and_then(Json::as_str)
    else {
        return Ok(group.to_string());
    };
    let (Some(start), Some(end)) = (group.find("<groupItems"), group.find("</groupItems>")) else {
        return Err(format!(
            "field {:?} has no group for its xml groupItems",
            f.name
        ));
    };
    let items = raw_element(raw, "groupItems")?;
    let end = end + "</groupItems>".len();
    Ok(format!("{}{items}{}", &group[..start], &group[end..]))
}

impl DefinedPivot<'_> {
    /// `_records_xml`: the entries by (record, index among `database`, the fields with records).
    pub(super) fn records_xml(
        &self,
        database: &[&str],
    ) -> Result<HashMap<(usize, usize), String>, String> {
        let entries = self.list("records_xml")?;
        let mut out = HashMap::new();
        for e in entries {
            if (e.as_object().into_iter().flatten())
                .any(|(k, _)| !["record", "field", "xml"].contains(&k.as_str()))
            {
                return Err("records_xml entries are {record, field, xml}".to_string());
            }
            let record = e.get("record").and_then(Json::as_u64);
            let field = e.get("field").and_then(Json::as_str);
            let xml = e.get("xml").and_then(Json::as_str);
            let (Some(record), Some(field), Some(xml)) = (record, field, xml) else {
                return Err("records_xml entries are {record, field, xml}".to_string());
            };
            let r = usize::try_from(record).map_err(|_| "a records_xml record is too large")?;
            let k = (database.iter().position(|name| *name == field))
                .filter(|_| r < self.records)
                .ok_or_else(|| {
                    format!("records_xml: record {r} of {field:?} is not in the cache")
                })?;
            let tag = (xml.trim().strip_prefix('<'))
                .map(|rest| rest.split(|c: char| !c.is_ascii_lowercase()).next())
                .and_then(|tag| tag.filter(|t| RECORD_TAGS.contains(t)))
                .ok_or("a records_xml entry is one record value element")?;
            if out.insert((r, k), raw_element(xml, tag)?).is_some() {
                return Err("records_xml writes a record value twice".to_string());
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]
    use super::*;

    fn field(spec: &serde_json::Map<String, Json>) -> Field<'_> {
        let mut f = Field::new("Product2".to_string(), Vec::new(), None);
        f.spec = Some(spec);
        f
    }

    #[test]
    fn group_items_replace_the_builders_and_need_a_group() {
        let spec: Json = serde_json::from_str(
            r#"{"xml": {"groupItems": "<groupItems count=\"1\"><b v=\"1\"/></groupItems>"}}"#,
        )
        .unwrap();
        let f = field(spec.as_object().unwrap());
        let group =
            r#"<fieldGroup base="0"><groupItems count="1"><s v="X"/></groupItems></fieldGroup>"#;
        assert_eq!(
            group_items(&f, group).unwrap(),
            r#"<fieldGroup base="0"><groupItems count="1"><b v="1"/></groupItems></fieldGroup>"#
        );
        assert!(group_items(&f, "").is_err());
        let bad: Json = serde_json::from_str(r#"{"xml": {"groupItems": "<items/>"}}"#).unwrap();
        assert!(group_items(&field(bad.as_object().unwrap()), group).is_err());
    }

    /// The `records_xml` of a pivot over two records of `Region` and `Sales`.
    fn records_xml(entries: &str) -> Result<HashMap<(usize, usize), String>, String> {
        let d: Json = serde_json::from_str(&format!(r#"{{"records_xml": {entries}}}"#)).unwrap();
        let pivot = DefinedPivot {
            name: "P",
            d: d.as_object().unwrap(),
            fields: Vec::new(),
            records: 2,
            rows: Vec::new(),
            columns: Vec::new(),
            pages: Vec::new(),
            calculated_items: Vec::new(),
            data: Vec::new(),
            refresh_on_load: true,
        };
        pivot.records_xml(&["Region", "Sales"])
    }

    #[test]
    fn records_xml_keys_one_record_value_by_record_and_field() {
        let out = records_xml(
            r#"[{"record": 1, "field": "Sales", "xml": "<n v=\"3\"><tpls c=\"1\"/></n>"}]"#,
        )
        .unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[&(1, 1)], r#"<n v="3"><tpls c="1"/></n>"#);
    }

    #[test]
    fn records_xml_refusals() {
        for (entries, message) in [
            (
                r#"[{"record": 2, "field": "Sales", "xml": "<m/>"}]"#,
                "is not in the cache",
            ),
            (
                r#"[{"record": 0, "field": "Nope", "xml": "<m/>"}]"#,
                "is not in the cache",
            ),
            (
                r#"[{"record": true, "field": "Sales", "xml": "<m/>"}]"#,
                "entries are",
            ),
            (
                r#"[{"record": -1, "field": "Sales", "xml": "<m/>"}]"#,
                "entries are",
            ),
            (r#"[{"record": 0, "field": "Sales"}]"#, "entries are"),
            (
                r#"[{"record": 0, "field": "Sales", "xml": "<m/>", "x": 1}]"#,
                "entries are",
            ),
            (
                r#"[{"record": 0, "field": "Sales", "xml": "<r/>"}]"#,
                "one record value element",
            ),
            (
                r#"[{"record": 0, "field": "Sales", "xml": "<m/><m/>"}]"#,
                "<m>",
            ),
            (
                r#"[{"record": 0, "field": "Sales", "xml": "<m/>"},
                    {"record": 0, "field": "Sales", "xml": "<n v=\"1\"/>"}]"#,
                "twice",
            ),
        ] {
            let err = records_xml(entries).unwrap_err();
            assert!(err.contains(message), "{entries}: {err}");
        }
    }
}
