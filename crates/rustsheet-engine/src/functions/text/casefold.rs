//! Case folding shared by SEARCH, TEXTSPLIT, TEXTBEFORE and TEXTAFTER. Every fold maps one char
//! to exactly one char, so char positions in the folded text are char positions in the original.

use super::string_format::sigma_is_final;

/// Lowercases `text` one character at a time (see [`fold_char`]). Excel's SEARCH never changes
/// the length of the text it counts in: `SEARCH("x","İx")` is 2 and `SEARCH("i","İx")` is
/// #VALUE!, whereas Rust's `to_lowercase` turns U+0130 into "i" + U+0307
/// (probes/text-search-casefold-length A1, A2, A8, A12, A13).
pub(super) fn fold_case(text: &str) -> String {
    text.chars().map(fold_char).collect()
}

/// [`fold_case`] for one character: always exactly one character out.
fn fold_char(c: char) -> char {
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
}

/// The folding `match_mode` 1 of TEXTSPLIT, TEXTBEFORE and TEXTAFTER share (TEXTBEFORE and
/// TEXTAFTER handle sigma themselves; TEXTSPLIT's final sigma follows SEARCH through
/// [`fold_char`]): U+0130 matches `i`, unlike SEARCH; the Kelvin sign does not match `k`, again
/// unlike SEARCH; `ß` and `ẞ` do not match `ss` (probes/textsplit-casefold-length A2, A6, A7 and
/// probes/textbefore-casefold-length A1-A12; docs/excel-semantics/text.md).
fn fold_match_char(c: char) -> char {
    match c {
        '\u{130}' => 'i',
        '\u{212A}' => c,
        _ => fold_char(c),
    }
}

/// TEXTSPLIT's `match_mode` 1 folding (final sigma follows SEARCH; UNVERIFIED for TEXTSPLIT).
pub(super) fn fold_for_split(text: &str) -> String {
    text.chars().map(fold_match_char).collect()
}

/// TEXTBEFORE and TEXTAFTER's `match_mode` 1 folding: [`fold_match_char`], except that sigma
/// folds by position, with LOWER's final-sigma rule ([`sigma_is_final`]): a `Σ` before a letter
/// or digit is `σ` and any other `Σ` is `ς`, and `ς` stays `ς`, so `ς` matches only `ς` and a
/// final `Σ` (probes/textbefore-casefold-length A17-A25, probes/textbefore-sigma-contexts: a
/// digit gives `σ`, a space or punctuation `ς`, and a lone `Σ` is `ς` even against a text `Σ`
/// that is `σ`). `ẞ` does not fold to `ß`. That a combining mark, `’` or zero-width space
/// also gives `σ` is UNVERIFIED here (it is LOWER's rule).
pub(super) fn fold_for_before_after(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        folded.push(match c {
            'Σ' if !sigma_is_final(chars.peek().copied()) => 'σ',
            'Σ' | 'ς' => 'ς',
            // The delimiter `ß` does not match `ẞ` in the text (A9), although `ẞ` lowercases to `ß`.
            '\u{1E9E}' => c,
            _ => fold_match_char(c),
        });
    }
    folded
}

/// Byte offset of every char of `s`, plus `s.len()` for the end.
pub(super) fn char_offsets(s: &str) -> Vec<usize> {
    s.char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(s.len()))
        .collect()
}
