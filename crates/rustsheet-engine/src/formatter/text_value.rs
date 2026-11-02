//! `TEXT` and cell display of a text or boolean value: the format's text section applies
//! (split from `format.rs`, T-3649).

use std::collections::BTreeSet;

use super::{
    format::{capped_text, format_error, text_too_long, Formatted, TEXT_TOO_LONG},
    formatted::{unshowable_text, MAX_TEXT_FORMAT_CHARS},
    lexer::last_section_is_bare_at,
    parser::{ParsePart, Parser, TextToken},
    sections::{is_bracket_only_section, is_text_section, numeric_section_count, section_tokens},
};
use crate::text::Text;

/// Formats a text value (a boolean's "TRUE"/"FALSE" included) the way `TEXT` does: only the
/// 4th section applies, `@` stands for the text and the section's other tokens print as literals
/// (poi `TextFormatTests` A3:A10, A12:A29; docs/excel-semantics/number-formats.md "Text and
/// booleans use the format's text section"). A last section with `@` is the text section too,
/// whatever the section count (`@" ""q"""` on "x" is "x q", probes/format-doubled-quotes C11;
/// `TEXT("abc","0;x@")` is "xabc", probes/text-at-token-sections A10, A11, T-1656). Any other
/// format with fewer than four sections returns the text unchanged; that case is unprobed. A
/// result over 255 characters is `#VALUE!` ([`MAX_TEXT_FORMAT_CHARS`]).
///
/// Excel checks the number sections first: a code with an unparsable number section is `#VALUE!`
/// for a text value too, even where the text section would apply (`TEXT("a","ab1")`,
/// `TEXT(TRUE,"n;n;n;@")`; probes/t2202-text-code-check-on-text; number-formats.md "TEXT checks
/// the code before it looks at a text or boolean value"). A bad section a number would not use
/// in a multi-section code rejects the text too; that case is unprobed.
pub fn format_text_value(text: &str, format: &str) -> Formatted {
    let mut parser = Parser::new(format);
    parser.parse();
    let parts = parser.parts.as_slice();
    if parts[..numeric_section_count(parts)]
        .iter()
        .any(ParsePart::is_error)
    {
        return format_error();
    }
    let section = match parts.last() {
        Some(last) if parts.len() < 4 && is_text_section(last) => parts.len() - 1,
        _ if parts.len() < 4 => return capped_text(text.to_string()),
        _ => 3,
    };
    let tokens = match parser.parts.get(section) {
        Some(ParsePart::Error(_)) => return format_error(),
        Some(part) if is_bracket_only_section(part) => return capped_text(text.to_string()),
        Some(part) => match section_tokens(part) {
            Some(tokens) => tokens,
            None => return capped_text(text.to_string()),
        },
        None => return capped_text(text.to_string()),
    };
    let mut out = String::new();
    let mut chars = 0;
    for token in tokens {
        let start = out.len();
        match token {
            TextToken::Literal(c) => out.push(*c),
            TextToken::Text(t) => out.push_str(t),
            // Excel: `_x` leaves a space but a `*x` fill prints nothing, and `AM/PM` prints "AM"
            // (`TEXT("abc","x@_x")` is "xabc ", `"x@*-"` "xabc", `"x@AM/PM"` "xabcAM";
            // probes/text-at-with-tokens A23, A37, A38, B37, B38, T-1656).
            TextToken::Ghost(_) => out.push(' '),
            TextToken::AMPM => out.push_str("AM"),
            // `A/P` prints its morning letter (probes/text-ap-locale-bracket A21-A23).
            TextToken::AP(am, _) => out.push(*am),
            TextToken::Raw => out.push_str(text),
            _ => {}
        }
        // Checked as it grows, so a format of many `@` cannot build a huge string.
        chars += out[start..].chars().count();
        if chars > MAX_TEXT_FORMAT_CHARS {
            return text_too_long();
        }
    }
    capped_text(out)
}

/// The text a cell shows for a text or boolean value under `format`. `TEXT()` is `#VALUE!` past 255
/// characters, where Excel shows the text whole when the format leaves it unchanged (no text
/// section, or a bare `@`) and 255 `#` when a text section changes it (`"pre "@`, `_(@_)`,
/// `[Red]@`; probes/displayed-text-long-text, -b; number-formats.md "Displayed text"). Any other
/// `TEXT()` error keeps the plain text: Excel refuses to open a workbook with an invalid code (T-3282).
#[must_use]
pub fn displayed_text(text: &str, format: &str) -> String {
    let formatted = format_text_value(text, format);
    match formatted.error.as_deref() {
        None => formatted.displayed(),
        Some(TEXT_TOO_LONG) if text_section_changes_text(format) => unshowable_text(),
        Some(_) => text.to_string(),
    }
}

/// Whether `format` has a text section other than a bare `@`.
fn text_section_changes_text(format: &str) -> bool {
    let mut parser = Parser::new(format);
    parser.parse();
    match parser.parts.last() {
        Some(last) if is_text_section(last) => !last_section_is_bare_at(format),
        _ => false,
    }
}

/// The private-use characters [`format_text_value_exact`] picks its stand-in from.
const STAND_IN_RANGE: std::ops::RangeInclusive<char> = '\u{E000}'..='\u{F8FF}';

/// [`format_text_value`] for exact text: a lone surrogate in `text` is kept where `@` prints it
/// (probes/text-lone-surrogates, TEXT(H,"@") has length 1). The format runs on a one-character
/// stand-in for the text, a private-use character the format does not hold, so the 255-character
/// limit and every other rule are the well-formed path's; the stand-ins in the result are then
/// replaced by the exact text. `Err` is the formatter's error message.
pub(crate) fn format_text_value_exact(text: &Text, format: &str) -> Result<Text, String> {
    if !text.has_lone_surrogate() {
        let f = format_text_value(text, format);
        return f.error.map_or(Ok(Text::from(f.text)), Err);
    }
    let in_format: BTreeSet<char> = format
        .chars()
        .filter(|c| STAND_IN_RANGE.contains(c))
        .collect();
    // The format holds at most one private-use character per character, so one of the first
    // `in_format.len() + 1` is free; the range holds 6,400.
    let stand_in = STAND_IN_RANGE
        .clone()
        .find(|c| !in_format.contains(c))
        .ok_or_else(|| "No free stand-in character".to_owned())?;
    let f = format_text_value(stand_in.encode_utf8(&mut [0; 4]), format);
    if let Some(error) = f.error {
        return Err(error);
    }
    // The stand-in counted as one character, so the limit is checked again on the real length
    // (this also bounds the result).
    let copies = f.text.matches(stand_in).count();
    let chars = f.text.chars().count() - copies + copies.saturating_mul(text.chars().count());
    if chars > MAX_TEXT_FORMAT_CHARS {
        return Err(TEXT_TOO_LONG.to_owned());
    }
    let mut out = Text::from("");
    let mut parts = f.text.split(stand_in);
    if let Some(first) = parts.next() {
        out.push(&Text::from(first));
    }
    for part in parts {
        out.push(text);
        out.push(&Text::from(part));
    }
    Ok(out)
}
