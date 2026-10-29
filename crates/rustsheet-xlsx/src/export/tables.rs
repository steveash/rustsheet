//! Table parts (`xl/tables/tableN.xml`), so the structured references the exported formulas keep
//! name tables the file has (SPEC-0016 step 3, T-2196).
//!
//! ```xml
//! <table xmlns="..." id="1" name="Sales" displayName="Sales" ref="A1:B4" totalsRowCount="1">
//!   <tableColumns count="2">
//!     <tableColumn id="1" name="Item" totalsRowLabel="Total"/>
//!     <tableColumn id="2" name="Qty" totalsRowFunction="custom">
//!       <totalsRowFormula>SUBTOTAL(109,Sales[Qty])</totalsRowFormula>
//!     </tableColumn>
//!   </tableColumns>
//!   <tableStyleInfo name="TableStyleMedium2" showFirstColumn="0" ... />
//! </table>
//! ```

use rustsheet_engine::{
    expressions::{
        parser::{parse_range, stringify::to_excel_string},
        types::CellReferenceRC,
        utils::number_to_column,
    },
    types::{Cell, FilterColumn, FilterCriterion, Table, Workbook, Worksheet},
    ParsedFormula,
};

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

/// The content type of a table part.
pub(crate) const TABLE_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.table+xml";

/// The relationship type from a worksheet to one of its table parts.
pub(crate) const TABLE_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table";

/// The tables of each worksheet, by sheet index, in the order they are exported (by name). The
/// `n`-th table of the list flattened in sheet order is the part `xl/tables/table{n}.xml` with
/// `id="{n}"` (from 1). A table on no sheet of the workbook, or whose `ref` is not a range, is
/// not exported: formulas cannot resolve a reference to it, so the export writes them as `#REF!`.
pub(crate) fn tables_by_sheet(workbook: &Workbook) -> Vec<Vec<&Table>> {
    workbook
        .worksheets
        .iter()
        .map(|worksheet| {
            let mut tables: Vec<&Table> = workbook
                .tables
                .values()
                .filter(|t| t.is_on_sheet(&worksheet.name) && parse_range(&t.reference).is_ok())
                .collect();
            tables.sort_by(|a, b| a.name.cmp(&b.name));
            tables
        })
        .collect()
}

/// The `<tableParts>` section of a worksheet with `count` tables, whose relationships follow the
/// worksheet's `first_id - 1` external hyperlinks in its rels part, or an empty string.
pub(crate) fn get_table_parts_section(count: usize, first_id: usize) -> String {
    if count == 0 {
        return String::new();
    }
    let parts: Vec<String> = (first_id..first_id + count)
        .map(|id| format!("<tablePart r:id=\"rId{id}\"/>"))
        .collect();
    format!(
        "<tableParts count=\"{count}\">{}</tableParts>",
        parts.join("")
    )
}

/// A differential-format attribute, written only when the index is one of the workbook's `dxfs`
/// (an imported index is not checked, and Excel refuses a part pointing past the list).
fn dxf_attribute(name: &str, dxf_id: Option<u32>, dxf_count: usize) -> String {
    match dxf_id {
        Some(id) if usize::try_from(id).is_ok_and(|i| i < dxf_count) => {
            format!(" {name}=\"{id}\"")
        }
        _ => String::new(),
    }
}

/// What the totals row holds for a column, as the table part records it: a formula is written as
/// `totalsRowFunction="custom"` with its `<totalsRowFormula>`, a text cell as `totalsRowLabel`.
/// Excel refuses a table with a totals row whose cells lack it (xlsx-storage.md "Structured
/// references in the file"). Any other cell needs nothing.
fn totals_row_metadata(
    cell: Option<&Cell>,
    context: &CellReferenceRC,
    parsed_formulas: &[ParsedFormula],
    shared_strings: &[String],
) -> (String, String) {
    let Some(cell) = cell else {
        return (String::new(), String::new());
    };
    if let Some(f) = cell.get_formula() {
        let Some(parsed) = usize::try_from(f).ok().and_then(|i| parsed_formulas.get(i)) else {
            return (String::new(), String::new());
        };
        let formula = to_excel_string(&parsed.evaluated, context);
        return (
            " totalsRowFunction=\"custom\"".to_string(),
            format!(
                "<totalsRowFormula>{}</totalsRowFormula>",
                escape_xml(&formula)
            ),
        );
    }
    match cell {
        Cell::SharedString { si, .. } => match usize::try_from(*si)
            .ok()
            .and_then(|i| shared_strings.get(i))
        {
            Some(label) => (
                format!(" totalsRowLabel=\"{}\"", escape_xml(label)),
                String::new(),
            ),
            None => (String::new(), String::new()),
        },
        _ => (String::new(), String::new()),
    }
}

/// The `<autoFilter>` of a table with a header row, over its header and data rows, as Excel writes
/// it, with the imported filter columns written back (`Table::filter_columns`). A table that is
/// filtered with no kept criterion (`Txn::add_table` with `filtered`, which makes every hidden row
/// of the sheet filtered for SUBTOTAL) is written as a `<filterColumn>` with no criterion:
/// schema-valid, but how Excel reads it is unprobed (xlsx-storage.md "Table parts in the
/// export"; T-3081 probes it).
fn auto_filter(table: &Table, (c1, r1, c2, r2): (i32, i32, i32, i32)) -> String {
    if table.header_row_count == 0 {
        return String::new();
    }
    let last_row = if table.totals_row_count > 0 {
        r2 - 1
    } else {
        r2
    };
    let (Some(left), Some(right)) = (number_to_column(c1), number_to_column(c2)) else {
        return String::new();
    };
    let filter = if !table.filter_columns.is_empty() {
        table.filter_columns.iter().map(filter_column).collect()
    } else if table.has_filters {
        "<filterColumn colId=\"0\"/>".to_string()
    } else {
        String::new()
    };
    format!("<autoFilter ref=\"{left}{r1}:{right}{last_row}\">{filter}</autoFilter>")
}

/// A `<filterColumn>` as ECMA-376-1:2016 section 18.3.2.7 writes it.
fn filter_column(column: &FilterColumn) -> String {
    let col_id = column.col_id;
    let mut xml = format!("<filterColumn colId=\"{col_id}\">");
    match &column.criterion {
        FilterCriterion::Values { values, blank } => {
            xml.push_str(if *blank {
                "<filters blank=\"1\">"
            } else {
                "<filters>"
            });
            for value in values {
                xml.push_str("<filter val=\"");
                xml.push_str(&escape_xml(value));
                xml.push_str("\"/>");
            }
            xml.push_str("</filters>");
        }
        FilterCriterion::Custom { and, conditions } => {
            xml.push_str(if *and {
                "<customFilters and=\"1\">"
            } else {
                "<customFilters>"
            });
            for condition in conditions {
                xml.push_str("<customFilter");
                if let Some(operator) = &condition.operator {
                    xml.push_str(" operator=\"");
                    xml.push_str(&escape_xml(operator));
                    xml.push('"');
                }
                xml.push_str(" val=\"");
                xml.push_str(&escape_xml(&condition.val));
                xml.push_str("\"/>");
            }
            xml.push_str("</customFilters>");
        }
        FilterCriterion::Other => return format!("<filterColumn colId=\"{col_id}\"/>"),
    }
    xml.push_str("</filterColumn>");
    xml
}

/// The part `xl/tables/table{id}.xml` of `table`, which is on `worksheet`.
pub(crate) fn get_table_xml(
    table: &Table,
    id: usize,
    worksheet: &Worksheet,
    parsed_formulas: &[ParsedFormula],
    workbook: &Workbook,
) -> String {
    let dxf_count = workbook.styles.dxfs.len();
    let area = parse_range(&table.reference).unwrap_or((1, 1, 1, 1));
    let (first_column, _, _, last_row) = area;
    let totals_row = (table.totals_row_count > 0).then_some(last_row);
    let mut columns = String::new();
    for (index, column) in table.columns.iter().enumerate() {
        let (totals_attributes, totals_formula) = match totals_row {
            Some(row) => {
                let column_index = i32::try_from(index)
                    .ok()
                    .and_then(|i| first_column.checked_add(i))
                    .unwrap_or(first_column);
                let context = CellReferenceRC {
                    sheet: worksheet.get_name(),
                    row,
                    column: column_index,
                };
                totals_row_metadata(
                    worksheet.cell(row, column_index),
                    &context,
                    parsed_formulas,
                    &workbook.shared_strings,
                )
            }
            None => (String::new(), String::new()),
        };
        columns.push_str(&format!(
            "<tableColumn id=\"{}\" name=\"{}\"{totals_attributes}{}{}{}>{totals_formula}</tableColumn>",
            column.id,
            escape_xml(&column.name),
            dxf_attribute("headerRowDxfId", column.header_row_dxf_id, dxf_count),
            dxf_attribute("dataDxfId", column.data_dxf_id, dxf_count),
            dxf_attribute("totalsRowDxfId", column.totals_row_dxf_id, dxf_count),
        ));
    }
    let header_row_count = if table.header_row_count == 1 {
        String::new()
    } else {
        format!(" headerRowCount=\"{}\"", table.header_row_count)
    };
    let totals_row_count = if table.totals_row_count == 0 {
        String::new()
    } else {
        format!(" totalsRowCount=\"{}\"", table.totals_row_count)
    };
    let style = &table.style_info;
    let style_name = match &style.name {
        Some(name) => format!(" name=\"{}\"", escape_xml(name)),
        None => String::new(),
    };
    format!(
        "{XML_DECLARATION}\n<table xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
id=\"{id}\" name=\"{}\" displayName=\"{}\" ref=\"{}\"{header_row_count}{totals_row_count}{}{}{}>{}\
<tableColumns count=\"{}\">{columns}</tableColumns>\
<tableStyleInfo{style_name} showFirstColumn=\"{}\" showLastColumn=\"{}\" showRowStripes=\"{}\" showColumnStripes=\"{}\"/>\
</table>",
        escape_xml(&table.name),
        escape_xml(&table.display_name),
        escape_xml(&table.reference),
        dxf_attribute("headerRowDxfId", table.header_row_dxf_id, dxf_count),
        dxf_attribute("dataDxfId", table.data_dxf_id, dxf_count),
        dxf_attribute("totalsRowDxfId", table.totals_row_dxf_id, dxf_count),
        auto_filter(table, area),
        table.columns.len(),
        u8::from(style.show_first_column),
        u8::from(style.show_last_column),
        u8::from(style.show_row_stripes),
        u8::from(style.show_column_stripes),
    )
}
