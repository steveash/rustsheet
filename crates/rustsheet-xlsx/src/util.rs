use std::borrow::Cow;

use rustsheet_engine::Model;

/// workbook.xml spells Excel's built-in defined names with this prefix (`_xlnm.Print_Area`), while
/// formulas and the engine use the bare name (`=SUM(Print_Area)`); ECMA-376 Part 1, `definedName`.
const BUILT_IN_NAME_PREFIX: &str = "_xlnm.";

/// The built-in names ECMA-376 Part 1 (`definedName`) lists with the `_xlnm.` prefix.
const BUILT_IN_NAMES: [&str; 8] = [
    "Consolidate_Area",
    "Criteria",
    "Database",
    "Extract",
    "Print_Area",
    "Print_Titles",
    "Sheet_Title",
    "_FilterDatabase",
];

fn is_built_in_name(name: &str) -> bool {
    BUILT_IN_NAMES.iter().any(|b| b.eq_ignore_ascii_case(name))
}

/// A workbook.xml `definedName` name as the engine stores it: `_xlnm.Print_Area` -> `Print_Area`.
/// Other names, including an `_xlnm.` name outside the built-in list, are kept as written.
pub(crate) fn name_from_xlsx(name: &str) -> &str {
    match name.strip_prefix(BUILT_IN_NAME_PREFIX) {
        Some(bare) if is_built_in_name(bare) => bare,
        _ => name,
    }
}

/// The inverse of [`name_from_xlsx`]: a built-in name gets its `_xlnm.` prefix back.
pub(crate) fn name_to_xlsx(name: &str) -> Cow<'_, str> {
    if is_built_in_name(name) {
        Cow::Owned(format!("{BUILT_IN_NAME_PREFIX}{name}"))
    } else {
        Cow::Borrowed(name)
    }
}

pub fn get_metadata_sheet_index(model: &Model) -> Option<u32> {
    for (index, ws) in model.workbook.worksheets.iter().enumerate() {
        if ws.name.eq_ignore_ascii_case("METADATA") {
            return Some(index as u32);
        }
    }
    None
}

// Cheesy way to get the locale from the workbook metadata sheet
pub fn get_workbook_metadata(model: &Model) -> String {
    let metadata_sheet_index = get_metadata_sheet_index(model);
    let default_locale = "en".to_string();
    if let Some(sheet_index) = metadata_sheet_index {
        if let Ok(a1) = model.get_formatted_cell_value(sheet_index, 1, 1) {
            if a1 == "Locale" {
                match model.get_formatted_cell_value(sheet_index, 1, 2) {
                    Ok(v) if v == "en-GB" => {
                        return "en-GB".to_string();
                    }
                    _ => return default_locale,
                }
            }
        }
    }
    default_locale
}
