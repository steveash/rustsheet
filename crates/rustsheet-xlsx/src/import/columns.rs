//! `<cols>` import (18.3.1.17).

use rustsheet_engine::{types::Col, COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH};

use super::{
    util::{get_attribute, get_bool_false},
    xml::XmlNode,
};
use crate::error::XlsxError;

/// Reads the `<cols>` of a worksheet. `width` is optional (ECMA-376 18.3.1.13): a `<col>` without
/// it takes `<sheetFormatPr defaultColWidth>`, else the engine default width.
pub(super) fn load_columns(ws: &XmlNode) -> Result<Vec<Col>, XlsxError> {
    // cols
    // <cols>
    //     <col min="5" max="5" width="38.26953125" customWidth="1"/>
    //     <col min="6" max="6" width="9.1796875" style="1"/>
    //     <col min="8" max="8" width="4" customWidth="1"/>
    // </cols>
    let mut cols = Vec::new();
    // A column without a width takes the sheet default: `<sheetFormatPr defaultColWidth>` when
    // present, else the engine's default column width (in Excel character units).
    let default_width = ws
        .children()
        .find(|n| n.has_tag_name("sheetFormatPr"))
        .and_then(|n| n.attribute("defaultColWidth"))
        .and_then(|w| w.parse::<f64>().ok())
        .unwrap_or(DEFAULT_COLUMN_WIDTH / COLUMN_WIDTH_FACTOR);
    let columns = ws
        .children()
        .filter(|n| n.has_tag_name("cols"))
        .collect::<Vec<&XmlNode>>();
    if columns.len() == 1 {
        for col in columns[0].children() {
            let min = get_attribute(col, "min")?;
            let min = min.parse::<i32>()?;
            let max = get_attribute(col, "max")?;
            let max = max.parse::<i32>()?;
            // `width` is optional (ECMA-376 18.3.1.13): producers that only set a style or
            // hide columns omit it. Port of ironcalc@9d5fa0dc.
            let width = match col.attribute("width") {
                Some(width) => width.parse::<f64>()?,
                None => default_width,
            };
            let custom_width = get_bool_false(col, "customWidth");
            let hidden = get_bool_false(col, "hidden");
            let style = col
                .attribute("style")
                .map(|s| s.parse::<i32>().unwrap_or(0));
            cols.push(Col {
                min,
                max,
                width,
                custom_width,
                style,
                hidden,
            });
        }
    }
    Ok(cols)
}
