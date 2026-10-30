//! The result of formatting a value under a number format.

/// Excel: TEXT of a text value is `#VALUE!` once the result is over 255 characters, also when the
/// format leaves the text unchanged (probes/text-format-text-limit A10, A11, A19-A21, A24, A25).
pub(super) const MAX_TEXT_FORMAT_CHARS: usize = 255;

/// What a cell shows for a value its format cannot show: 255 `#`, whatever the column width.
pub(super) fn unshowable_text() -> String {
    "#".repeat(MAX_TEXT_FORMAT_CHARS)
}

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
    /// A number the format cannot show: TEXT is `#VALUE!` and the cell displays 255 `#`.
    pub(super) fn unshowable(error: String) -> Self {
        Formatted {
            text: "#VALUE!".to_owned(),
            color: None,
            error: Some(error),
            unshowable: true,
        }
    }

    /// The text a cell shows under the format, Excel's `Range.text`. Excel displays a date it
    /// cannot show as 255 `#`, not `#VALUE!`, whatever the column width (probes/displayed-text-formats
    /// S!B4; gen-m10 "Displayed text"; docs/excel-semantics/number-formats.md "Displayed text
    /// (`Range.text`) of a formatted cell").
    #[must_use]
    pub fn displayed(self) -> String {
        if self.unshowable {
            unshowable_text()
        } else {
            self.text
        }
    }
}
