//! Writes the errors Excel keeps only as rich values (`#SPILL!`, `#CONNECT!`, `#BLOCKED!`,
//! `#FIELD!`, `#CALC!`) in Excel's own form (SPEC-0003 X2, T-1014): the cell stores `#VALUE!` with
//! a `vm` attribute, and `xl/metadata.xml` plus `xl/richData/*` say which error it is. Excel for the
//! web rejects these kinds as stored literals (T-0999, docs/excel-semantics/syntax.md).
//!
//! The layout copies Excel's (docs/excel-semantics/arrays.md "Cached rich errors";
//! `tests/calc_tests/LOOKUP_AND_REFERENCE/DROP_TAKE.xlsx` holds `XLDAPR` and `XLRICHVALUE` together):
//! one `_error` value per kind, whatever the number of cells, in a fixed order, so `vm="k"` is
//! the same kind in every exported workbook. `subType` (the reason) is 0: it does not change the
//! error, and Excel itself writes 0 for `#SPILL!` and `#CALC!` (ERROR.TYPE.xlsx).

use rustsheet_engine::{
    expressions::token::Error,
    types::{Cell, FormulaValue, SpillValue, Workbook},
};

use crate::import::rich_errors::{rich_error_name, RICH_VALUE_PART, RICH_VALUE_STRUCTURE_PART};

use super::xml_constants::XML_DECLARATION;

/// The rich-value types part Excel writes next to the values.
pub(crate) const RICH_VALUE_TYPES_PART: &str = "xl/richData/rdRichValueTypes.xml";

/// The `errorType` codes written, in `vm` order: `vm="k"` is the k-th code (1-based). Each
/// code's error is the importer's [`rich_error_name`], so export and import share one table.
/// Append only: reordering would change which kind a `vm` means between exported workbooks.
const RICH_ERROR_CODES: [&str; 5] = ["8", "9", "10", "12", "13"];

/// The `vm` index Excel needs for `error`, or `None` for a kind stored as a plain literal.
pub(crate) fn rich_error_vm(error: &Error) -> Option<usize> {
    let literal = error.stored_literal();
    RICH_ERROR_CODES
        .iter()
        .position(|code| rich_error_name(code) == Some(literal))
        .map(|k| k + 1)
}

/// The ` vm="k"` attribute of an error cell holding `error`, or nothing.
pub(crate) fn vm_attribute(error: &Error) -> String {
    rich_error_vm(error).map_or_else(String::new, |vm| format!(r#" vm="{vm}""#))
}

/// Whether some cell of `workbook` holds a rich-value error, so the package needs the
/// rich-value parts.
pub(crate) fn has_rich_errors(workbook: &Workbook) -> bool {
    workbook.worksheets.iter().any(|sheet| {
        sheet.sheet_data.cells().any(|(_, _, cell)| {
            let error = match cell {
                Cell::ErrorCell { ei, .. }
                | Cell::SpillCell {
                    v: SpillValue::Error(ei),
                    ..
                }
                | Cell::CellFormula {
                    v: FormulaValue::Error { ei, .. },
                    ..
                }
                | Cell::ArrayFormula {
                    v: FormulaValue::Error { ei, .. },
                    ..
                } => ei,
                _ => return false,
            };
            rich_error_vm(error).is_some()
        })
    })
}

const XLDAPR_TYPE: &str = r#"<metadataType name="XLDAPR" minSupportedVersion="120000" copy="1" pasteAll="1" pasteValues="1" merge="1" splitFirst="1" rowColShift="1" clearFormats="1" clearComments="1" assign="1" coerce="1" cellMeta="1"/>"#;
const XLRICHVALUE_TYPE: &str = r#"<metadataType name="XLRICHVALUE" minSupportedVersion="120000" copy="1" pasteAll="1" pasteValues="1" merge="1" splitFirst="1" rowColShift="1" clearFormats="1" clearComments="1" assign="1" coerce="1"/>"#;
const XLDAPR_BLOCKS: &str = r#"<futureMetadata name="XLDAPR" count="1"><bk><extLst><ext uri="{bdbb8cdc-fa1e-496e-a857-3c3f30c029c3}"><xda:dynamicArrayProperties fDynamic="1" fCollapsed="0"/></ext></extLst></bk></futureMetadata>"#;
/// `cm="1"` (a dynamic-array formula) is the first `XLDAPR` block.
const CELL_METADATA: &str = r#"<cellMetadata count="1"><bk><rc t="1" v="0"/></bk></cellMetadata>"#;

/// `xl/metadata.xml`: the dynamic-array cell metadata, and with `rich` the rich-value error
/// blocks (metadata type 2, `XLRICHVALUE`) that the `vm` attributes point at.
pub(crate) fn get_metadata_xml(rich: bool) -> String {
    let main = r#"xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main""#;
    let xda = r#"xmlns:xda="http://schemas.microsoft.com/office/spreadsheetml/2017/dynamicarray""#;
    if !rich {
        return format!(
            r#"{XML_DECLARATION}
<metadata {main} {xda}><metadataTypes count="1">{XLDAPR_TYPE}</metadataTypes>{XLDAPR_BLOCKS}{CELL_METADATA}</metadata>"#
        );
    }
    let count = RICH_ERROR_CODES.len();
    let mut rich_blocks = String::new();
    let mut value_blocks = String::new();
    for k in 0..count {
        rich_blocks.push_str(&format!(
            r#"<bk><extLst><ext uri="{{3e2802c4-a4d2-4d8b-9148-e3be6c30e623}}"><xlrd:rvb i="{k}"/></ext></extLst></bk>"#
        ));
        value_blocks.push_str(&format!(r#"<bk><rc t="2" v="{k}"/></bk>"#));
    }
    format!(
        r#"{XML_DECLARATION}
<metadata {main} xmlns:xlrd="http://schemas.microsoft.com/office/spreadsheetml/2017/richdata" {xda}><metadataTypes count="2">{XLDAPR_TYPE}{XLRICHVALUE_TYPE}</metadataTypes>{XLDAPR_BLOCKS}<futureMetadata name="XLRICHVALUE" count="{count}">{rich_blocks}</futureMetadata>{CELL_METADATA}<valueMetadata count="{count}">{value_blocks}</valueMetadata></metadata>"#
    )
}

/// The rich-value parts, as `(part name, content type, relationship type, xml)`.
pub(crate) fn rich_value_parts() -> [(&'static str, &'static str, &'static str, String); 3] {
    let values: String = RICH_ERROR_CODES
        .iter()
        .flat_map(|code| [r#"<rv s="0"><v>"#, code, "</v><v>0</v></rv>"])
        .collect();
    let richdata = "http://schemas.microsoft.com/office/spreadsheetml/2017/richdata";
    [
        (
            RICH_VALUE_PART,
            "application/vnd.ms-excel.rdrichvalue+xml",
            "http://schemas.microsoft.com/office/2017/06/relationships/rdRichValue",
            format!(
                r#"{XML_DECLARATION}
<rvData xmlns="{richdata}" count="{}">{values}</rvData>"#,
                RICH_ERROR_CODES.len()
            ),
        ),
        (
            RICH_VALUE_STRUCTURE_PART,
            "application/vnd.ms-excel.rdrichvaluestructure+xml",
            "http://schemas.microsoft.com/office/2017/06/relationships/rdRichValueStructure",
            format!(
                r#"{XML_DECLARATION}
<rvStructures xmlns="{richdata}" count="1"><s t="_error"><k n="errorType" t="i"/><k n="subType" t="i"/></s></rvStructures>"#
            ),
        ),
        (
            RICH_VALUE_TYPES_PART,
            "application/vnd.ms-excel.rdrichvaluetypes+xml",
            "http://schemas.microsoft.com/office/2017/06/relationships/rdRichValueTypes",
            format!(
                r#"{XML_DECLARATION}
<rvTypesInfo xmlns="{richdata}2" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" mc:Ignorable="x" xmlns:x="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><global><keyFlags><key name="_Self"><flag name="ExcludeFromFile" value="1"/><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_DisplayString"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Flags"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Format"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_SubLabel"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Attribution"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Icon"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Display"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_CanonicalPropertyNames"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_ClassificationId"><flag name="ExcludeFromCalcComparison" value="1"/></key></keyFlags></global></rvTypesInfo>"#
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rich_kind_has_a_vm_and_round_trips_through_the_decoder() {
        for (vm, error) in [
            (1, Error::SPILL),
            (2, Error::CONNECT),
            (3, Error::BLOCKED),
            (4, Error::FIELD),
            (5, Error::CALC),
        ] {
            assert_eq!(rich_error_vm(&error), Some(vm), "{error}");
        }
        for error in [Error::VALUE, Error::NA, Error::CIRC, Error::GETTINGDATA] {
            assert_eq!(rich_error_vm(&error), None, "{error}");
        }
        let [values, structures, _] = rich_value_parts().map(|(_, _, _, xml)| xml);
        let decoded = crate::import::rich_errors::value_metadata_errors(
            &get_metadata_xml(true),
            &values,
            Some(&structures),
        )
        .unwrap();
        assert_eq!(
            decoded,
            [
                Some("#SPILL!"),
                Some("#CONNECT!"),
                Some("#BLOCKED!"),
                Some("#FIELD!"),
                Some("#CALC!")
            ]
        );
    }
}
