use std::io::Read;

use super::package::Package;

use super::xml::XmlNode;
use rustsheet_engine::types::{
    CustomFilter, FilterColumn, FilterCriterion, Table, TableColumn, TableStyleInfo,
};

use crate::error::XlsxError;

use super::util::{get_bool, get_bool_false};

// <table name="Table" displayName="Table" totalsRowCount ref="A1:D6">
//   <autoFilter ref="A1:D6">
//       <filterColumn colId="0">
//            <customFilters><customFilter operator="greaterThan" val=20></customFilter></customFilters>
//       </filterColumn>
//   </autoFilter>
//   <tableColumns count="5">
//      <tableColumn name="Monday" totalsRowFunction="sum" />
//      ...
//   </tableColumns>
//   <tableStyleInfo name="TableStyle5"/>
// </table>

/// Reads a table in an Excel workbook
pub(crate) fn load_table<R: Read + std::io::Seek>(
    archive: &mut Package<R>,
    path: &str,
    sheet_name: &str,
) -> Result<Table, XlsxError> {
    let file = archive.open(path)?;
    let document = XmlNode::parse(file)?;

    // table
    let table = &document;

    // Name and display name are normally the same and are unique in a workbook
    // They also need to be different from any defined name
    // A part written by LibreOffice can carry only displayName; Excel reaches the table through it
    // (docs/excel-semantics/xlsx-storage.md), so each attribute stands in for the other.
    let (name, display_name) = match (table.attribute("name"), table.attribute("displayName")) {
        (Some(n), Some(d)) => (n.to_string(), d.to_string()),
        (Some(n), None) => (n.to_string(), n.to_string()),
        (None, Some(d)) => (d.to_string(), d.to_string()),
        (None, None) => {
            return Err(XlsxError::Xml(
                "Corrupt XML structure: missing table name".to_string(),
            ))
        }
    };

    // Range of the table, including the totals if any and headers.
    let reference = table
        .attribute("ref")
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure: missing table ref".to_string()))?
        .to_string();

    // Either 0 or 1, indicates if the table has a formula for totals at the bottom of the table
    let totals_row_count = match table.attribute("totalsRowCount") {
        Some(s) => s.parse::<u32>().map_err(|_| {
            XlsxError::Xml("Corrupt XML structure: Invalid totalsRowCount".to_string())
        })?,
        None => 0,
    };

    // Either 0 or 1, indicates if the table has headers at the top of the table
    let header_row_count = match table.attribute("headerRowCount") {
        Some(s) => s.parse::<u32>().map_err(|_| {
            XlsxError::Xml("Corrupt XML structure: Invalid headerRowCount".to_string())
        })?,
        None => 1,
    };

    // style index of the header row of the table
    let header_row_dxf_id = if let Some(index_str) = table.attribute("headerRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // style index of the header row of the table
    let data_dxf_id = if let Some(index_str) = table.attribute("headerRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // style index of the totals row of the table
    let totals_row_dxf_id = if let Some(index_str) = table.attribute("totalsRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // Missing in Calc: styles can also be defined via a name:
    // headerRowCellStyle, dataCellStyle, totalsRowCellStyle

    // Missing in Calc: styles can also be applied to the borders:
    // headerRowBorderDxfId, tableBorderDxfId, totalsRowBorderDxfId

    // TODO: Conformant implementations should panic if header_row_dxf_id or data_dxf_id are out of bounds.

    // Note that filters are non dynamic
    // The only thing important for us is whether or not it has filters
    let auto_filter = table
        .descendants()
        .filter(|n| n.has_tag_name("autoFilter"))
        .collect::<Vec<&XmlNode>>();

    let has_filters = if let Some(filter) = auto_filter.first() {
        filter.children().count() > 0
    } else {
        false
    };
    let filter_columns = auto_filter
        .first()
        .map(|filter| {
            filter
                .children()
                .filter(|n| n.has_tag_name("filterColumn"))
                .filter_map(filter_column)
                .collect()
        })
        .unwrap_or_default();

    // tableColumn
    let table_column = table
        .descendants()
        .filter(|n| n.has_tag_name("tableColumn"))
        .collect::<Vec<&XmlNode>>();
    let mut columns = Vec::new();
    for table_column in table_column {
        let column_name = table_column.attribute("name").ok_or_else(|| {
            XlsxError::Xml("Corrupt XML structure: missing column name".to_string())
        })?;
        let id = table_column.attribute("id").ok_or_else(|| {
            XlsxError::Xml("Corrupt XML structure: missing column id".to_string())
        })?;
        let id = id
            .parse::<u32>()
            .map_err(|_| XlsxError::Xml("Corrupt XML structure: invalid id".to_string()))?;

        // style index of the header row of the table
        let header_row_dxf_id = if let Some(index_str) = table_column.attribute("headerRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // style index of the header row of the table column
        let data_dxf_id = if let Some(index_str) = table_column.attribute("headerRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // style index of the totals row of the table column
        let totals_row_dxf_id = if let Some(index_str) = table_column.attribute("totalsRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // NOTE: Same as before, we should panic if indices to differential formatting records are out of bounds
        // Missing in Calc: styles can also be defined via a name:
        // headerRowCellStyle, dataCellStyle, totalsRowCellStyle

        columns.push(TableColumn {
            id,
            name: column_name.to_string(),
            totals_row_label: None,
            header_row_dxf_id,
            data_dxf_id,
            totals_row_function: None,
            totals_row_dxf_id,
            // `calculatedColumnFormula` is not read yet: whether Excel honors it on load is
            // T-2969's probe.
            calculated_column_formula: None,
        });
    }

    // tableInfo
    let table_info = table
        .descendants()
        .filter(|n| n.has_tag_name("tableInfo"))
        .collect::<Vec<&XmlNode>>();
    let style_info = match table_info.first() {
        Some(node) => {
            let name = node.attribute("name").map(|s| s.to_string());
            TableStyleInfo {
                name,
                show_first_column: get_bool_false(node, "showFirstColumn"),
                show_last_column: get_bool_false(node, "showLastColumn"),
                show_row_stripes: get_bool(node, "showRowStripes"),
                show_column_stripes: get_bool_false(node, "showColumnStripes"),
            }
        }
        None => TableStyleInfo {
            name: None,
            show_first_column: false,
            show_last_column: false,
            show_row_stripes: true,
            show_column_stripes: false,
        },
    };
    Ok(Table {
        name,
        display_name,
        reference,
        totals_row_count,
        header_row_count,
        header_row_dxf_id,
        data_dxf_id,
        totals_row_dxf_id,
        columns,
        style_info,
        has_filters,
        filter_columns,
        sheet_name: sheet_name.to_string(),
    })
}

/// A `<filterColumn>`'s column and criterion; `None` for one without a valid `colId`, which
/// Excel's schema requires.
fn filter_column(node: &XmlNode) -> Option<FilterColumn> {
    let col_id = node.attribute("colId")?.parse::<u32>().ok()?;
    let criterion = if let Some(filters) = node.child("filters") {
        FilterCriterion::Values {
            values: filters
                .children()
                .filter(|n| n.has_tag_name("filter"))
                .filter_map(|n| n.attribute("val").map(str::to_string))
                .collect(),
            blank: get_bool_false(filters, "blank"),
        }
    } else if let Some(custom) = node.child("customFilters") {
        FilterCriterion::Custom {
            and: get_bool_false(custom, "and"),
            conditions: custom
                .children()
                .filter(|n| n.has_tag_name("customFilter"))
                .map(|n| CustomFilter {
                    operator: n.attribute("operator").map(str::to_string),
                    val: n.attribute("val").unwrap_or_default().to_string(),
                })
                .collect(),
        }
    } else {
        FilterCriterion::Other
    };
    Some(FilterColumn { col_id, criterion })
}
