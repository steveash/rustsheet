use std::borrow::Cow;

use crate::{
    cast::{array_node_to_text, MAX_TEXT_UNITS},
    expressions::{parser::ArrayNode, token::Error},
    text::Text,
};

/// `text` with each UTF-16 code unit as one `char`, so that `char` positions (and a regex `.`)
/// are the code-unit positions FIND and SEARCH count in (`SEARCH("?b","😀b")` is 2,
/// probes/text-surrogate-positions A22). A surrogate unit `u` becomes the private-use character
/// `U+F0000 + (u - 0xD800)`; no other character of the result is outside the BMP, so these never
/// collide with real text.
pub(crate) fn one_char_per_utf16_unit(text: &str) -> Cow<'_, str> {
    if text.chars().all(|ch| ch.len_utf16() == 1) {
        return Cow::Borrowed(text);
    }
    Cow::Owned(unit_chars(text.encode_utf16()))
}

/// [`one_char_per_utf16_unit`] for engine text, a lone surrogate unit kept as its own character
/// (probes/text-lone-surrogates A48-A57).
pub(crate) fn one_char_per_unit(text: &Text) -> String {
    unit_chars(text.units())
}

fn unit_chars(units: impl Iterator<Item = u16>) -> String {
    units
        .map(|unit| {
            let code = u32::from(unit);
            let code = if (0xD800..=0xDFFF).contains(&code) {
                0xF0000 + (code - 0xD800)
            } else {
                code
            };
            char::from_u32(code).unwrap_or(char::REPLACEMENT_CHARACTER)
        })
        .collect()
}

pub(crate) enum Case {
    Sensitive,
    Insensitive,
}

/// Finds the text after the occurrence instance of '`search_for`' in text
pub(crate) fn text_after(
    text: &str,
    delimiter: &str,
    instance_num: i32,
    match_mode: Case,
) -> Option<String> {
    if let Some((_, right)) = match_text(text, delimiter, instance_num, match_mode) {
        return Some(text[right..].to_string());
    };
    None
}

pub(crate) fn text_before(
    text: &str,
    delimiter: &str,
    instance_num: i32,
    match_mode: Case,
) -> Option<String> {
    if let Some((left, _)) = match_text(text, delimiter, instance_num, match_mode) {
        return Some(text[..left].to_string());
    };
    None
}

/// Character count of `SUBSTITUTE(text, old_text, new_text[, instance_num])` computed without
/// building the result, so a hostile replacement cannot allocate it (CLAUDE.md directive 5).
/// Excel: a result over 32767 characters is #VALUE! (probes/text-result-limit C1-C7).
fn substituted_chars(
    text: &str,
    text_chars: usize,
    old_text: &str,
    new_text: &str,
    instance_num: Option<i32>,
) -> usize {
    let new_chars = new_text.chars().count();
    if let Some(n) = instance_num {
        let upper_bound = text_chars + new_chars;
        if upper_bound <= MAX_TEXT_UNITS {
            return upper_bound;
        }
        // Only a matching instance is replaced; count the text left once it is removed.
        let removed = substitute(text, old_text, "", n);
        return if removed.len() == text.len() {
            text_chars
        } else {
            removed.chars().count() + new_chars
        };
    }
    let matches = text.matches(old_text).count();
    text_chars
        .saturating_add(matches.saturating_mul(new_chars))
        .saturating_sub(matches.saturating_mul(old_text.chars().count()))
}

/// Whether a SUBSTITUTE result would be over the cell text limit, so it is `#VALUE!`. A result
/// that does not grow passes even from over-limit input (text.md "Open").
pub(crate) fn substitute_exceeds_limit(
    text: &str,
    old_text: &str,
    new_text: &str,
    instance_num: Option<i32>,
) -> bool {
    let text_chars = text.chars().count();
    let result_chars = substituted_chars(text, text_chars, old_text, new_text, instance_num);
    result_chars > MAX_TEXT_UNITS && result_chars > text_chars
}

pub(crate) fn substitute(text: &str, old_text: &str, new_text: &str, instance_num: i32) -> String {
    if let Some((left, right)) = match_text(text, old_text, instance_num, Case::Sensitive) {
        return format!("{}{}{}", &text[..left], new_text, &text[right..]);
    };
    text.to_string()
}

fn match_text(
    text: &str,
    delimiter: &str,
    instance_num: i32,
    match_mode: Case,
) -> Option<(usize, usize)> {
    match match_mode {
        Case::Sensitive => {
            if instance_num > 0 {
                text_sensitive(text, delimiter, instance_num)
            } else {
                text_sensitive_reverse(text, delimiter, -instance_num)
            }
        }
        Case::Insensitive => {
            if instance_num > 0 {
                text_sensitive(
                    &text.to_lowercase(),
                    &delimiter.to_lowercase(),
                    instance_num,
                )
            } else {
                text_sensitive_reverse(
                    &text.to_lowercase(),
                    &delimiter.to_lowercase(),
                    -instance_num,
                )
            }
        }
    }
}

fn text_sensitive(text: &str, delimiter: &str, instance_num: i32) -> Option<(usize, usize)> {
    let mut byte_index = 0;
    let mut local_index = 1;
    // delimiter length in bytes
    let delimiter_len = delimiter.len();
    for c in text.chars() {
        if text[byte_index..].starts_with(delimiter) {
            if local_index == instance_num {
                return Some((byte_index, byte_index + delimiter_len));
            } else {
                local_index += 1;
            }
        }
        byte_index += c.len_utf8();
    }
    None
}

fn text_sensitive_reverse(
    text: &str,
    delimiter: &str,
    instance_num: i32,
) -> Option<(usize, usize)> {
    let text_len = text.len();
    let mut byte_index = text_len;
    let mut local_index = 1;
    let delimiter_len = delimiter.len();
    for c in text.chars().rev() {
        if text[byte_index..].starts_with(delimiter) {
            if local_index == instance_num {
                return Some((byte_index, byte_index + delimiter_len));
            } else {
                local_index += 1;
            }
        }

        byte_index -= c.len_utf8();
    }
    None
}

/// Cap on the total bytes of text one broadcast may produce. Excel has no such
/// limit that we have measured; this bounds allocation on untrusted input
/// (DEVIATION, decision 0014; docs/excel-semantics/text.md).
const MAX_BROADCAST_TEXT_BYTES: usize = 64 * 1024 * 1024;

/// Adds `len` to the running `total`; false once it exceeds the broadcast cap.
pub(crate) fn within_text_budget(total: &mut usize, len: usize) -> bool {
    *total = total.saturating_add(len);
    *total <= MAX_BROADCAST_TEXT_BYTES
}

#[cfg(test)]
mod tests {

    use super::{text_after, text_before, within_text_budget, Case};
    #[test]
    fn test_text_after_sensitive() {
        assert_eq!(
            text_after("One element", "ele", 1, Case::Sensitive),
            Some("ment".to_string())
        );
        assert_eq!(
            text_after("One element", "e", 1, Case::Sensitive),
            Some(" element".to_string())
        );
        assert_eq!(
            text_after("One element", "e", 4, Case::Sensitive),
            Some("nt".to_string())
        );
        assert_eq!(text_after("One element", "e", 5, Case::Sensitive), None);
        assert_eq!(
            text_after("長壽相等！", "相", 1, Case::Sensitive),
            Some("等！".to_string())
        );
    }
    #[test]
    fn test_text_before_sensitive() {
        assert_eq!(
            text_before("One element", "ele", 1, Case::Sensitive),
            Some("One ".to_string())
        );
        assert_eq!(
            text_before("One element", "e", 1, Case::Sensitive),
            Some("On".to_string())
        );
        assert_eq!(
            text_before("One element", "e", 4, Case::Sensitive),
            Some("One elem".to_string())
        );
        assert_eq!(text_before("One element", "e", 5, Case::Sensitive), None);
        assert_eq!(
            text_before("長壽相等！", "相", 1, Case::Sensitive),
            Some("長壽".to_string())
        );
    }
    #[test]
    fn test_text_after_insensitive() {
        assert_eq!(
            text_after("One element", "eLe", 1, Case::Insensitive),
            Some("ment".to_string())
        );
        assert_eq!(
            text_after("One element", "E", 1, Case::Insensitive),
            Some(" element".to_string())
        );
        assert_eq!(
            text_after("One element", "E", 4, Case::Insensitive),
            Some("nt".to_string())
        );
        assert_eq!(text_after("One element", "E", 5, Case::Insensitive), None);
        assert_eq!(
            text_after("長壽相等！", "相", 1, Case::Insensitive),
            Some("等！".to_string())
        );
    }
    #[test]
    fn test_text_before_insensitive() {
        assert_eq!(
            text_before("One element", "eLe", 1, Case::Insensitive),
            Some("One ".to_string())
        );
        assert_eq!(
            text_before("One element", "E", 1, Case::Insensitive),
            Some("On".to_string())
        );
        assert_eq!(
            text_before("One element", "E", 4, Case::Insensitive),
            Some("One elem".to_string())
        );
        assert_eq!(text_before("One element", "E", 5, Case::Insensitive), None);
        assert_eq!(
            text_before("長壽相等！", "相", 1, Case::Insensitive),
            Some("長壽".to_string())
        );
    }

    #[test]
    fn budget_allows_exactly_64_mib() {
        let mut total = 0;
        // 2048 results of 32 KiB make exactly 64 MiB; the next one crosses the cap.
        assert!((0..2048).all(|_| within_text_budget(&mut total, 32768)));
        assert!(!within_text_budget(&mut total, 1));
    }
}

/// Computes a single SUBSTITUTE result from one element of each argument.
/// Without an `instance_num` every instance is replaced.
pub(super) fn substitute_element(
    text: &ArrayNode,
    old_text: &ArrayNode,
    new_text: &ArrayNode,
    instance_num: Option<Result<f64, Error>>,
) -> Result<ArrayNode, Error> {
    let text = array_node_to_text(text)?;
    let old_text = array_node_to_text(old_text)?;
    let new_text = array_node_to_text(new_text)?;
    let instance_num = match instance_num {
        Some(n) => {
            let n = n?.floor();
            if n < 1.0 {
                return Err(Error::VALUE);
            }
            Some(n.min(i32::MAX as f64) as i32)
        }
        None => None,
    };
    if old_text.is_empty() {
        return Ok(ArrayNode::String(text.into_owned()));
    }
    if text.has_lone_surrogate() || old_text.has_lone_surrogate() || new_text.has_lone_surrogate() {
        return substitute_units(&text, &old_text, &new_text, instance_num).map(ArrayNode::String);
    }
    if substitute_exceeds_limit(&text, &old_text, &new_text, instance_num) {
        return Err(Error::VALUE);
    }
    Ok(ArrayNode::String(match instance_num {
        Some(n) => substitute(&text, &old_text, &new_text, n).into(),
        None => {
            let (text, old_text, new_text): (&str, &str, &str) = (&text, &old_text, &new_text);
            text.replace(old_text, new_text).into()
        }
    }))
}

/// SUBSTITUTE over exact UTF-16 units, used when any operand holds a lone surrogate: a lone half
/// as `old_text` matches inside a pair, unit for unit (probes/text-lone-surrogates A41-A47). The
/// result length is checked before it is built (CLAUDE.md directive 5), as for well-formed text.
fn substitute_units(
    text: &Text,
    old_text: &Text,
    new_text: &Text,
    instance_num: Option<i32>,
) -> Result<Text, Error> {
    let units: Vec<u16> = text.units().collect();
    let old: Vec<u16> = old_text.units().collect();
    let new: Vec<u16> = new_text.units().collect();
    let mut matches = Vec::new();
    let mut at = 0;
    while at + old.len() <= units.len() {
        if units[at..at + old.len()] == old[..] {
            matches.push(at);
            at += old.len();
        } else {
            at += 1;
        }
    }
    let replaced: Vec<usize> = match instance_num {
        Some(n) => matches.get(n as usize - 1).copied().into_iter().collect(),
        None => matches,
    };
    let result_len =
        (units.len() + replaced.len() * new.len()).saturating_sub(replaced.len() * old.len());
    if result_len > MAX_TEXT_UNITS && result_len > units.len() {
        return Err(Error::VALUE);
    }
    let mut result = Vec::with_capacity(result_len);
    let mut from = 0;
    for &start in &replaced {
        result.extend_from_slice(&units[from..start]);
        result.extend_from_slice(&new);
        from = start + old.len();
    }
    result.extend_from_slice(&units[from..]);
    Ok(Text::from_utf16(result))
}
