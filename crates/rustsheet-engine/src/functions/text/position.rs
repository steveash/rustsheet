//! FIND and SEARCH positions, counted in UTF-16 code units (docs/excel-semantics/text.md "Text
//! positions count UTF-16 code units, except in LEFT and RIGHT").

use std::borrow::Cow;

use crate::{
    cast::array_node_to_text,
    expressions::{parser::ArrayNode, token::Error},
    functions::{
        criteria::from_wildcard_to_regex,
        text::util::{one_char_per_unit, one_char_per_utf16_unit},
    },
    text::Text,
};

/// Computes a single FIND or SEARCH result from one element of each argument.
/// SEARCH is the one that is not case sensitive and understands wildcards.
pub(super) fn find_element(
    find_text: &ArrayNode,
    within_text: &ArrayNode,
    start_num: Result<f64, Error>,
    is_search: bool,
) -> Result<ArrayNode, Error> {
    let find_text = array_node_to_text(find_text)?;
    let within_text = array_node_to_text(within_text)?;
    let start_num = start_num?.floor();
    if start_num < 1.0 {
        return Err(Error::VALUE);
    }
    let start_num = start_num as usize;
    // Excel: positions count UTF-16 code units, and `?` matches one unit
    // (probes/text-surrogate-positions A14-A26, B11-B15).
    // A lone surrogate stays its own unit (probes/text-lone-surrogates A48-A57), also in SEARCH's
    // case folding, which only touches the well-formed runs.
    fn search_units(text: &Text, is_search: bool) -> Cow<'_, str> {
        let plain: &str = text;
        match (text.has_lone_surrogate(), is_search) {
            (false, false) => one_char_per_utf16_unit(plain),
            (false, true) => {
                let folded = fold_case(plain);
                match one_char_per_utf16_unit(&folded) {
                    Cow::Borrowed(_) => Cow::Owned(folded),
                    Cow::Owned(units) => Cow::Owned(units),
                }
            }
            (true, false) => Cow::Owned(one_char_per_unit(text)),
            (true, true) => Cow::Owned(one_char_per_unit(&text.map_runs(fold_case))),
        }
    }
    let (find_text, within_text) = (
        search_units(&find_text, is_search),
        search_units(&within_text, is_search),
    );
    let length = within_text.chars().count();
    // Excel: FIND accepts a start one past the end, where only "" is found (`FIND("","abc",4)` is
    // 4); SEARCH does not (`SEARCH("","abc",4)` is #VALUE!; probes/text-surrogate-positions C1-C9).
    let last_start = if is_search { length } else { length + 1 };
    if start_num > last_start {
        return Err(Error::VALUE);
    }
    let position = if is_search {
        search(&find_text, &within_text, start_num)
    } else if find_text.is_empty() {
        Some(start_num as i32)
    } else {
        find(&find_text, &within_text, start_num)
    };
    match position {
        Some(p) => Ok(ArrayNode::Number(p as f64)),
        // Text not found
        None => Err(Error::VALUE),
    }
}

/// Lowercases `text` one character at a time, leaving a character whose lowercase form is longer
/// than one character as it is. Excel's SEARCH never changes the length of the text it counts in:
/// `SEARCH("x","İx")` is 2 and `SEARCH("i","İx")` is #VALUE!, whereas Rust's `to_lowercase` turns
/// U+0130 into "i" + U+0307 (probes/text-search-casefold-length A1, A2, A8, A12, A13).
fn fold_case(text: &str) -> String {
    text.chars()
        .map(|c| {
            // Final sigma matches sigma in either case (`SEARCH("ς","ΟΣ")` is 2), while the other
            // simple-fold pairs do not: ſ/s, µ/μ and ϐ/β are #VALUE! (A14-A24).
            if c == 'ς' {
                return 'σ';
            }
            let mut lower = c.to_lowercase();
            match (lower.next(), lower.next()) {
                (Some(single), None) => single,
                _ => c,
            }
        })
        .collect()
}

/// Finds the first instance of '`search_for`' in text starting at char index start
fn find(search_for: &str, text: &str, start: usize) -> Option<i32> {
    let ch = text.chars();
    let mut byte_index = 0;
    for (char_index, c) in ch.enumerate() {
        if char_index + 1 >= start && text[byte_index..].starts_with(search_for) {
            return Some((char_index + 1) as i32);
        }
        byte_index += c.len_utf8();
    }
    None
}

/// You can use the wildcard characters — the question mark (?) and asterisk (*) — in the `find_text` argument.
/// * A question mark matches any single character.
/// * An asterisk matches any sequence of characters.
/// * If you want to find an actual question mark or asterisk, type a tilde (~) before the character.
fn search(search_for: &str, text: &str, start: usize) -> Option<i32> {
    let re = match from_wildcard_to_regex(search_for, false) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let ch = text.chars();
    let mut byte_index = 0;
    for (char_index, c) in ch.enumerate() {
        if char_index + 1 >= start {
            {
                let m = re.find(&text[byte_index..])?;
                return Some((text[0..(m.start() + byte_index)].chars().count() as i32) + 1);
            }
        }
        byte_index += c.len_utf8();
    }
    None
}
