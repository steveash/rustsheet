//! Decodes the rich-value errors behind a cell's value metadata (`vm`).
//!
//! Excel caches the errors that older readers do not know (`#CALC!`, `#SPILL!`, `#CONNECT!`, ...)
//! as `#VALUE!` and points the cell's `vm` attribute at the real one:
//!
//! ```text
//! <c vm="2">  -> xl/metadata.xml  <valueMetadata> block 2, <rc t="T" v="V"/>
//!             -> <metadataTypes> entry T is XLRICHVALUE
//!             -> <futureMetadata name="XLRICHVALUE"> block V, <xlrd:rvb i="I"/>
//!             -> xl/richData/rdrichvalue.xml <rv> number I, its `errorType` field
//! ```
//!
//! The `errorType` field is found by key name in the value's structure
//! (`xl/richData/rdrichvaluestructure.xml`): Excel for the web writes `_error` keys alphabetically
//! (`colOffset, errorType, rwOffset, subType`), so it is not always the first field (T-0208,
//! gen-m2/78bc88199fc96650 Calc!A61). This is the one decoder shared by the importer and the eval
//! snapshot reader (T-0921). It is public only for that reader, not a stable API.

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use std::io::{Read, Seek};

use crate::error::XlsxError;

use super::package::Package;

/// The value-metadata part (`<valueMetadata>`, `<futureMetadata>`).
pub const METADATA_PART: &str = "xl/metadata.xml";
/// The rich values (`<rv>`) that `rvb i` indexes.
pub const RICH_VALUE_PART: &str = "xl/richData/rdrichvalue.xml";
/// The key names of each rich-value structure (`<s>`).
pub const RICH_VALUE_STRUCTURE_PART: &str = "xl/richData/rdrichvaluestructure.xml";

/// The metadata type whose `futureMetadata` blocks point at rich values.
const RICH_VALUE_TYPE: &str = "XLRICHVALUE";

/// The error a rich-value `errorType` code stands for. errorType is always ERROR.TYPE − 1, and
/// the `subType` (the reason) does not change the error. Evidence: probes/rich-errors-*
/// (excel-graph, 2026-10-05, T-0030): each cell's errorType from the cached oracle workbook, its
/// ERROR.TYPE and its VALUETOTEXT text; ironcalc-calc/ERROR.TYPE.xlsx for 8 and 13.
/// Codes Excel was not seen to write (`#BUSY!`, `#UNKNOWN!`, ...) are never guessed: they are
/// `None`, so callers cannot mistake them for the cached `#VALUE!`.
#[must_use]
pub fn rich_error_name(error_type: &str) -> Option<&'static str> {
    match error_type {
        "8" => Some("#SPILL!"),    // SEQUENCE blocked by a value
        "9" => Some("#CONNECT!"),  // STOCKHISTORY of an unknown ticker
        "10" => Some("#BLOCKED!"), // IMAGE of an unreachable URL; uncached external reference
        "12" => Some("#FIELD!"),   // FIELDVALUE of a plain number
        "13" => Some("#CALC!"),    // FILTER with no match
        _ => None,
    }
}

/// The error each value-metadata entry stands for: entry `k` is the cell attribute `vm="k+1"`.
/// `None` when the entry is not a rich value, its value has no `errorType`, or the code is one
/// [`rich_error_name`] does not know.
///
/// `metadata`, `rich_values` and `structures` are the texts of [`METADATA_PART`],
/// [`RICH_VALUE_PART`] and [`RICH_VALUE_STRUCTURE_PART`]; without a structures part the first
/// field of each value is its `errorType` (the ERROR.TYPE.xlsx layout).
pub fn value_metadata_errors(
    metadata: &str,
    rich_values: &str,
    structures: Option<&str>,
) -> Result<Vec<Option<&'static str>>, XlsxError> {
    let positions = structures.map(error_type_positions).transpose()?;
    let rv_errors = rich_value_errors(rich_values, positions.as_deref())?;
    let meta = parse_metadata(metadata)?;
    let rich_type = meta
        .types
        .iter()
        .position(|t| t == RICH_VALUE_TYPE)
        .map(|i| i + 1);
    Ok(meta
        .value_blocks
        .iter()
        .map(|records| {
            records.iter().find_map(|&(t, v)| {
                if Some(t) != rich_type {
                    return None;
                }
                let rv = meta.rich_value_blocks.get(v).copied().flatten()?;
                rv_errors.get(rv).copied().flatten()
            })
        })
        .collect())
}

/// The value-metadata errors of the package being imported (see [`value_metadata_errors`]);
/// empty when it has no metadata or rich values, or they do not parse, so every cell keeps the
/// error it caches. The importer is lenient on purpose (an untrusted workbook with broken rich
/// data still opens, as it would in a reader that ignores `vm`); the eval snapshot reader reports
/// the same failure as an error, because a golden it cannot decode must not be scored.
pub(super) fn load_value_metadata_errors<R: Read + Seek>(
    archive: &mut Package<R>,
) -> Result<Vec<Option<&'static str>>, XlsxError> {
    let (Some(metadata), Some(rich_values)) = (
        read_optional(archive, METADATA_PART)?,
        read_optional(archive, RICH_VALUE_PART)?,
    ) else {
        return Ok(Vec::new());
    };
    let structures = read_optional(archive, RICH_VALUE_STRUCTURE_PART)?;
    Ok(value_metadata_errors(&metadata, &rich_values, structures.as_deref()).unwrap_or_default())
}

/// The text of the part `name`, or `None` if the package has none or it is not UTF-8. Reads count against the
/// package's byte budget like every other part.
fn read_optional<R: Read + Seek>(
    archive: &mut Package<R>,
    name: &str,
) -> Result<Option<String>, XlsxError> {
    let Some(mut part) = archive.open_optional(name)? else {
        return Ok(None);
    };
    let mut bytes = Vec::new();
    part.read_to_end(&mut bytes)?;
    // Text that is not UTF-8 is treated like XML that does not parse: no rich errors, rather
    // than a failed import. Read errors (the byte budget among them) still propagate.
    Ok(String::from_utf8(bytes).ok())
}

fn xml_error(part: &str, error: impl std::fmt::Display) -> XlsxError {
    XlsxError::Xml(format!("{part}: {error}"))
}

fn attribute(element: &BytesStart<'_>, key: &str) -> Option<String> {
    element.attributes().flatten().find_map(|a| {
        (a.key.local_name().as_ref() == key)
            .then(|| {
                a.normalized_value(XmlVersion::Implicit1_0)
                    .ok()
                    .map(std::borrow::Cow::into_owned)
            })
            .flatten()
    })
}

fn index_attribute(element: &BytesStart<'_>, key: &str) -> Option<usize> {
    attribute(element, key).and_then(|v| v.trim().parse().ok())
}

/// What [`value_metadata_errors`] needs from `xl/metadata.xml`.
#[derive(Default)]
struct Metadata {
    /// `<metadataType name>` in order; `rc t` is 1-based into it.
    types: Vec<String>,
    /// The `rvb i` of each `<bk>` of `<futureMetadata name="XLRICHVALUE">`; `rc v` is 0-based
    /// into it.
    rich_value_blocks: Vec<Option<usize>>,
    /// The `(t, v)` records of each `<valueMetadata>` `<bk>`; `vm` is 1-based into it.
    value_blocks: Vec<Vec<(usize, usize)>>,
}

/// The section of `xl/metadata.xml` the reader is in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Other,
    RichValueFuture,
    Value,
}

fn parse_metadata(xml: &str) -> Result<Metadata, XlsxError> {
    let mut meta = Metadata::default();
    let mut section = Section::Other;
    let mut reader = Reader::from_str(xml);
    loop {
        let event = reader
            .read_event()
            .map_err(|e| xml_error(METADATA_PART, e))?;
        match &event {
            Event::Start(e) | Event::Empty(e) => match (e.local_name().as_ref(), section) {
                ("metadataType", _) => meta.types.push(attribute(e, "name").unwrap_or_default()),
                ("futureMetadata", _) if matches!(event, Event::Start(_)) => {
                    if attribute(e, "name").as_deref() == Some(RICH_VALUE_TYPE) {
                        section = Section::RichValueFuture;
                    }
                }
                ("valueMetadata", _) if matches!(event, Event::Start(_)) => {
                    section = Section::Value;
                }
                ("bk", Section::RichValueFuture) => meta.rich_value_blocks.push(None),
                ("bk", Section::Value) => meta.value_blocks.push(Vec::new()),
                ("rvb", Section::RichValueFuture) => {
                    if let Some(last) = meta.rich_value_blocks.last_mut() {
                        *last = index_attribute(e, "i");
                    }
                }
                ("rc", Section::Value) => {
                    if let (Some(last), Some(t), Some(v)) = (
                        meta.value_blocks.last_mut(),
                        index_attribute(e, "t"),
                        index_attribute(e, "v"),
                    ) {
                        last.push((t, v));
                    }
                }
                _ => {}
            },
            Event::End(e)
                if matches!(e.local_name().as_ref(), "futureMetadata" | "valueMetadata") =>
            {
                section = Section::Other;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(meta)
}

/// Position of the `errorType` key in each `<s>` of `rdrichvaluestructure.xml` (None when absent).
fn error_type_positions(xml: &str) -> Result<Vec<Option<usize>>, XlsxError> {
    let mut out = Vec::new();
    let mut reader = Reader::from_str(xml);
    let mut key = 0usize;
    loop {
        match reader
            .read_event()
            .map_err(|e| xml_error(RICH_VALUE_STRUCTURE_PART, e))?
        {
            Event::Start(e) if e.local_name().as_ref() == "s" => {
                out.push(None);
                key = 0;
            }
            Event::Empty(e) if e.local_name().as_ref() == "s" => out.push(None),
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == "k" => {
                if attribute(&e, "n").as_deref() == Some("errorType") {
                    if let Some(last) = out.last_mut() {
                        *last = Some(key);
                    }
                }
                key += 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(out)
}

/// The rich-value error of every `<rv>` in `rdrichvalue.xml`, reading the `errorType` field at
/// the position its structure gives (the first field without structures).
fn rich_value_errors(
    xml: &str,
    positions: Option<&[Option<usize>]>,
) -> Result<Vec<Option<&'static str>>, XlsxError> {
    let mut out: Vec<Option<&'static str>> = Vec::new();
    let mut reader = Reader::from_str(xml);
    // (field index of errorType in the current rv, current field index, inside that <v>)
    let (mut want, mut field, mut in_v) = (None::<usize>, 0usize, false);
    loop {
        match reader
            .read_event()
            .map_err(|e| xml_error(RICH_VALUE_PART, e))?
        {
            Event::Start(e) => match e.local_name().as_ref() {
                "rv" => {
                    out.push(None);
                    field = 0;
                    want = match positions {
                        None => Some(0),
                        Some(p) => {
                            index_attribute(&e, "s").and_then(|s| p.get(s).copied().flatten())
                        }
                    };
                }
                "v" => in_v = true,
                _ => {}
            },
            Event::Text(t) if in_v && want == Some(field) => {
                let code = t.xml_content(XmlVersion::Implicit1_0);
                if let Some(last) = out.last_mut() {
                    *last = rich_error_name(code.trim());
                }
            }
            Event::End(e) if e.local_name().as_ref() == "v" => {
                in_v = false;
                field += 1;
            }
            Event::Empty(e) if e.local_name().as_ref() == "v" => field += 1,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const META_TYPES: &str = r#"<metadataTypes count="2"><metadataType name="XLDAPR"/><metadataType name="XLRICHVALUE"/></metadataTypes><futureMetadata name="XLDAPR" count="1"><bk><extLst><ext uri="{bdbb8cdc-fa1e-496e-a857-3c3f30c029c3}"><xda:dynamicArrayProperties fDynamic="1" fCollapsed="0"/></ext></extLst></bk></futureMetadata>"#;

    fn rvb(i: usize) -> String {
        format!(
            r#"<bk><extLst><ext uri="{{3e2802c4-a4d2-4d8b-9148-e3be6c30e623}}"><xlrd:rvb i="{i}"/></ext></extLst></bk>"#
        )
    }

    /// The ERROR.TYPE.xlsx layout: one structure-less `rv` per error, `rc v` equal to `rvb i`.
    #[test]
    fn decodes_the_error_type_workbook_layout() {
        let meta = format!(
            r#"<metadata><metadataTypes count="1"><metadataType name="XLRICHVALUE"/></metadataTypes><futureMetadata name="XLRICHVALUE" count="2">{}{}</futureMetadata><valueMetadata count="2"><bk><rc t="1" v="0"/></bk><bk><rc t="1" v="1"/></bk></valueMetadata></metadata>"#,
            rvb(0),
            rvb(1)
        );
        let rich = r#"<rvData count="2"><rv s="0"><v>13</v><v>0</v></rv><rv s="0"><v>8</v><v>0</v></rv></rvData>"#;
        let got = value_metadata_errors(&meta, rich, None).unwrap_or_default();
        assert_eq!(got, vec![Some("#CALC!"), Some("#SPILL!")]);
    }

    /// `rc v` indexes the XLRICHVALUE futureMetadata blocks, whose `rvb i` indexes the values;
    /// `rc t` is 1-based into metadataTypes, so a non-rich-value record resolves to nothing.
    #[test]
    fn resolves_through_future_metadata() {
        let meta = format!(
            r#"<metadata>{META_TYPES}<futureMetadata name="XLRICHVALUE" count="2">{}{}</futureMetadata><cellMetadata count="1"><bk><rc t="1" v="0"/></bk></cellMetadata><valueMetadata count="4"><bk><rc t="2" v="0"/></bk><bk><rc t="2" v="1"/></bk><bk><rc t="1" v="0"/></bk><bk><rc t="2" v="7"/></bk></valueMetadata></metadata>"#,
            rvb(1),
            rvb(0)
        );
        let rich = r#"<rvData><rv s="0"><v>13</v></rv><rv s="0"><v>8</v></rv></rvData>"#;
        let got = value_metadata_errors(&meta, rich, None).unwrap_or_default();
        assert_eq!(got, vec![Some("#SPILL!"), Some("#CALC!"), None, None]);
    }

    /// Excel for the web's `_error` structure lists `colOffset` before `errorType`
    /// (gen-m2/78bc88199fc96650 Calc!A61: rv `0 8 5 1` is `#SPILL!`, not "no error").
    #[test]
    fn error_type_is_read_by_key_name() {
        let structures = r#"<rvStructures><s t="_error"><k n="colOffset" t="i"/><k n="errorType" t="i"/><k n="rwOffset" t="i"/><k n="subType" t="i"/></s><s t="_error"><k n="errorType" t="i"/></s></rvStructures>"#;
        let rich = r#"<rvData><rv s="0"><v>0</v><v>8</v><v>5</v><v>1</v></rv><rv s="1"><v>13</v></rv><rv s="0"><v>8</v><v>2</v><v>0</v><v>0</v></rv><rv s="1"><v>10</v></rv><rv s="0"><v>0</v></rv></rvData>"#;
        let positions = error_type_positions(structures).unwrap_or_default();
        let got = rich_value_errors(rich, Some(&positions)).unwrap_or_default();
        assert_eq!(
            got,
            vec![
                Some("#SPILL!"),
                Some("#CALC!"),
                None,
                Some("#BLOCKED!"),
                None
            ]
        );
    }

    /// probes/rich-errors-* (T-0030): errorType = ERROR.TYPE - 1, whatever the subType.
    #[test]
    fn error_codes_follow_the_probes() {
        let got: Vec<_> = ["8", "9", "10", "12", "13", "11", "x"]
            .into_iter()
            .map(rich_error_name)
            .collect();
        assert_eq!(
            got,
            [
                Some("#SPILL!"),
                Some("#CONNECT!"),
                Some("#BLOCKED!"),
                Some("#FIELD!"),
                Some("#CALC!"),
                None,
                None
            ]
        );
    }

    #[test]
    fn malformed_xml_is_an_error_not_a_panic() {
        assert!(value_metadata_errors("<metadata><a></b>", "<rvData/>", None).is_err());
    }
}
