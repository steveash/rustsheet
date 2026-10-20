//! The result of formatting a value under a number format.

use super::format::MAX_TEXT_FORMAT_CHARS;

/// A formatted value: its text (`#VALUE!` when TEXT fails), the section's color and why it failed.
pub struct Formatted {
    pub color: Option<i32>,
    pub text: String,
    pub error: Option<String>,
    /// The value is a number the format cannot show as a date (a negative serial, or one past
    /// 9999-12-31): TEXT is `#VALUE!` and the cell displays 255 `#` ([`Formatted::displayed`]).
    pub unshowable: bool,
}

impl Formatted {
    /// The text a cell shows under the format, Excel's `Range.text`. Excel displays a date it
    /// cannot show as 255 `#`, not `#VALUE!`, whatever the column width (probes/displayed-text-formats
    /// S!B4; gen-m10 "Displayed text"; docs/excel-semantics/number-formats.md "Displayed text
    /// (`Range.text`) of a formatted cell").
    pub fn displayed(self) -> String {
        if self.unshowable {
            "#".repeat(MAX_TEXT_FORMAT_CHARS)
        } else {
            self.text
        }
    }
}
