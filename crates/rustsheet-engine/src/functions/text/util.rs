use std::borrow::Cow;

use super::casefold::{char_offsets, fold_for_before_after};

use crate::{
    calc_result::CalcResult,
    cast::{calc_result_to_text, MAX_TEXT_UNITS},
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
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

/// The inverse of [`one_char_per_unit`]: the UTF-16 units of a string it produced.
pub(crate) fn unit_chars_to_units(text: &str) -> Vec<u16> {
    text.chars()
        .map(|c| {
            let code = u32::from(c);
            let code = if (0xF0000..0xF0800).contains(&code) {
                code - 0xF0000 + 0xD800
            } else {
                code
            };
            code as u16
        })
        .collect()
}

#[derive(Clone, Copy)]
pub(crate) enum Case {
    Sensitive,
    Insensitive,
}

impl Case {
    /// The `match_mode` of TEXTSPLIT, TEXTBEFORE and TEXTAFTER: cut toward zero, then 0 is
    /// case-sensitive, 1 case-insensitive and anything else `#VALUE!` (probes/
    /// t2314-textbefore-instance-bound A1-A12, t2581-textsplit-textbefore A17-A23; the text.md
    /// entry on TEXTBEFORE and TEXTAFTER instance numbers past the text).
    pub(crate) fn from_match_mode(mode: f64, cell: CellReferenceIndex) -> Result<Case, CalcResult> {
        match mode.trunc() {
            0.0 => Ok(Case::Sensitive),
            1.0 => Ok(Case::Insensitive),
            _ => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "match_mode must be 0 or 1".to_string(),
            )),
        }
    }
}

/// The delimiter list of one TEXTSPLIT, TEXTBEFORE or TEXTAFTER call, folded for `match_mode` 1
/// once per call rather than once per lifted element (T-1272 performance review).
pub(crate) struct Delimiters {
    exact: Vec<String>,
    folded: Vec<String>,
}

impl Delimiters {
    /// TEXTBEFORE and TEXTAFTER's list, folded by [`fold_for_before_after`].
    pub(crate) fn new(exact: Vec<String>) -> Self {
        Self::folded_with(exact, fold_for_before_after)
    }

    /// A list folded by `fold` (TEXTSPLIT's is `fold_for_split`).
    pub(crate) fn folded_with(exact: Vec<String>, fold: fn(&str) -> String) -> Self {
        let folded = exact.iter().map(|d| fold(d)).collect();
        Delimiters { exact, folded }
    }

    /// The delimiters as written.
    pub(crate) fn exact(&self) -> &[String] {
        &self.exact
    }

    /// The delimiters a text is matched with under `case`.
    pub(crate) fn for_case(&self, case: Case) -> &[String] {
        match case {
            Case::Sensitive => &self.exact,
            Case::Insensitive => &self.folded,
        }
    }
}

/// Finds the text after the occurrence instance of any of the `delimiters` in text; with
/// `match_end` the end of the text (the start, counting from the end) is one more occurrence.
pub(crate) fn text_after(
    text: &str,
    delimiters: &Delimiters,
    instance_num: i32,
    match_mode: Case,
    match_end: bool,
) -> Option<String> {
    let (_, right) = match_text(
        text,
        delimiters.for_case(match_mode),
        instance_num,
        match_mode,
        Instances::Disjoint { match_end },
    )?;
    text.get(right..).map(str::to_string)
}

/// Finds the text before the occurrence instance of any of the `delimiters` in text, as
/// [`text_after`].
pub(crate) fn text_before(
    text: &str,
    delimiters: &Delimiters,
    instance_num: i32,
    match_mode: Case,
    match_end: bool,
) -> Option<String> {
    let (left, _) = match_text(
        text,
        delimiters.for_case(match_mode),
        instance_num,
        match_mode,
        Instances::Disjoint { match_end },
    )?;
    text.get(..left).map(str::to_string)
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

/// Whether a SUBSTITUTE result would be over the cell text limit, so it is `#VALUE!`, even when
/// it does not grow from over-limit input; a result that shrinks to the limit passes
/// (probes/regexreplace-long-result-d A2, text-long-input-e).
pub(crate) fn substitute_exceeds_limit(
    text: &str,
    old_text: &str,
    new_text: &str,
    instance_num: Option<i32>,
) -> bool {
    let text_chars = text.chars().count();
    let result_chars = substituted_chars(text, text_chars, old_text, new_text, instance_num);
    result_chars > MAX_TEXT_UNITS
}

pub(crate) fn substitute(text: &str, old_text: &str, new_text: &str, instance_num: i32) -> String {
    if let Some((left, right)) = match_text(
        text,
        &[old_text],
        instance_num,
        Case::Sensitive,
        Instances::Overlapping,
    ) {
        if let (Some(head), Some(tail)) = (text.get(..left), text.get(right..)) {
            return format!("{head}{new_text}{tail}");
        }
    };
    text.to_string()
}

/// How matches are counted towards an instance number.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Instances {
    /// Every position where a match starts counts, inside an earlier match too: SUBSTITUTE's
    /// `instance_num` (docs/excel-semantics/text.md "SUBSTITUTE instances overlap").
    Overlapping,
    /// A match inside one already counted does not count: TEXTBEFORE and TEXTAFTER
    /// (docs/excel-semantics/text.md "TEXTBEFORE and TEXTAFTER: overlapping delimiter instances").
    /// With `match_end` the end of the text (its start, counting from the end) is one more match,
    /// after the last one (probes/t3335-textbefore-overlapping-instances A27, A48, A49).
    Disjoint { match_end: bool },
}

/// Byte range of the `instance_num`-th match of any of `delimiters` (counted from the end when
/// negative), counted as `instances` says. Every position where some delimiter starts counts
/// once; the longest delimiter that matches there decides the match length (docs/excel-semantics/text.md "TEXTSPLIT,
/// TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar arguments"; T-1824). With
/// `Case::Insensitive` the delimiters are already folded ([`Delimiters`]); only the text is
/// folded here.
fn match_text<S: AsRef<str>>(
    text: &str,
    delimiters: &[S],
    instance_num: i32,
    match_mode: Case,
    instances: Instances,
) -> Option<(usize, usize)> {
    // The callers saturate a huge instance number to i32::MIN, whose negation overflows; saturate
    // it to i32::MAX instead, the same count a huge positive instance number gets.
    let reverse_num = instance_num.saturating_neg();
    match match_mode {
        Case::Sensitive => {
            if instance_num > 0 {
                text_sensitive(text, delimiters, instance_num, instances)
            } else {
                text_sensitive_reverse(text, delimiters, reverse_num, instances)
            }
        }
        Case::Insensitive => {
            // Fold char by char (one char in, one char out) and map the match back to byte
            // offsets of the original; offsets found in a lowercased copy are not valid there
            // (T-1796, probes/textbefore-casefold-length).
            let folded_text = fold_for_before_after(text);
            let (left, right) = if instance_num > 0 {
                text_sensitive(&folded_text, delimiters, instance_num, instances)
            } else {
                text_sensitive_reverse(&folded_text, delimiters, reverse_num, instances)
            }?;
            let original = char_offsets(text);
            let char_index = |byte: usize| folded_text.get(..byte).map(|s| s.chars().count());
            Some((
                *original.get(char_index(left)?)?,
                *original.get(char_index(right)?)?,
            ))
        }
    }
}

/// Length in bytes of the longest delimiter that `rest` starts with (Excel: the longest wins when
/// several match at one position, probes/textsplit-lift-guesses A1, A2, A9).
fn delimiter_at<S: AsRef<str>>(rest: &str, delimiters: &[S]) -> Option<usize> {
    delimiters
        .iter()
        .map(AsRef::as_ref)
        .filter(|d| rest.starts_with(d))
        .map(str::len)
        .max()
}

fn text_sensitive<S: AsRef<str>>(
    text: &str,
    delimiters: &[S],
    instance_num: i32,
    instances: Instances,
) -> Option<(usize, usize)> {
    let mut byte_index = 0;
    let mut local_index = 1;
    // A disjoint scan resumes after the end of the match it counted.
    let mut next_start = 0;
    for c in text.chars() {
        if byte_index >= next_start {
            if let Some(len) = text
                .get(byte_index..)
                .and_then(|rest| delimiter_at(rest, delimiters))
            {
                if local_index == instance_num {
                    return Some((byte_index, byte_index + len));
                }
                local_index += 1;
                if instances != Instances::Overlapping {
                    next_start = byte_index + len;
                }
            }
        }
        byte_index += c.len_utf8();
    }
    (instances == Instances::Disjoint { match_end: true } && local_index == instance_num)
        .then_some((text.len(), text.len()))
}

/// The `instance_num`-th match counted from the end, scanning start positions from the last one
/// back. A disjoint scan then steps back by the length of the match it counted, in UTF-16 units,
/// and goes on from there; a longer delimiter starting there may still run into the counted match
/// (`TEXTAFTER("3.141590.2-0.5x~*",{..,"0.2",..,"2",..},-2)` counts "2" then "0.2",
/// gen-m7/a470dc9e165bdb20 Calc!A1441), but the same delimiter cannot (probes/
/// t3335-textbefore-overlapping-instances A21, A22, A33, A42, A55).
fn text_sensitive_reverse<S: AsRef<str>>(
    text: &str,
    delimiters: &[S],
    instance_num: i32,
    instances: Instances,
) -> Option<(usize, usize)> {
    let mut byte_index = text.len();
    let mut local_index = 1;
    // UTF-16 units still to step back over before a position is checked again.
    let mut skip = 0;
    let mut chars = text.chars().rev();
    loop {
        if let Some(len) = text
            .get(byte_index..)
            .and_then(|rest| delimiter_at(rest, delimiters))
            // The empty end of the text matches no delimiter; the start of the text does
            // (probes/textsplit-lift-guesses A9).
            .filter(|_| skip == 0 && byte_index < text.len())
        {
            if local_index == instance_num {
                return Some((byte_index, byte_index + len));
            }
            local_index += 1;
            if instances != Instances::Overlapping {
                skip = text
                    .get(byte_index..byte_index + len)
                    .map_or(0, |m| m.encode_utf16().count());
            }
        }
        let Some(c) = chars.next() else {
            return (instances == Instances::Disjoint { match_end: true }
                && local_index == instance_num)
                .then_some((0, 0));
        };
        byte_index -= c.len_utf8();
        skip = skip.saturating_sub(c.len_utf16());
    }
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

/// Largest `instance_num` SUBSTITUTE accepts (2^31 - 2; text.md, T-2279).
const MAX_SUBSTITUTE_INSTANCE: f64 = 2_147_483_646.0;

/// Computes a single SUBSTITUTE result from one element of each argument.
/// Without an `instance_num` every instance is replaced.
pub(super) fn substitute_element(
    text: &CalcResult,
    old_text: &CalcResult,
    new_text: &CalcResult,
    instance_num: Option<Result<f64, Error>>,
) -> Result<ArrayNode, Error> {
    let text = calc_result_to_text(text)?;
    let old_text = calc_result_to_text(old_text)?;
    let new_text = calc_result_to_text(new_text)?;
    let instance_num = match instance_num {
        Some(n) => {
            // Excel accepts 1..=2^31-2 after flooring: 2147483646.9 is fine, 2147483647 and
            // above (1E+15, 1E+300) are #VALUE!, even with no match and before an empty
            // old_text short-circuits (probes/substitute-instance-num-bound, T-2279).
            let n = n?.floor();
            if !(1.0..=MAX_SUBSTITUTE_INSTANCE).contains(&n) {
                return Err(Error::VALUE);
            }
            Some(n as i32)
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
/// as `old_text` matches inside a pair, unit for unit (probes/text-lone-surrogates A41-A47).
/// Replace-all maps each unit to one `char` (`one_char_per_unit`, reversible) so `str` matching,
/// which is linear, finds the non-overlapping instances. With an `instance_num` the instances
/// overlap, as in the well-formed path (`text_sensitive`; unprobed, text.md), and a linear KMP
/// scan finds the nth. The result length is checked before it is built (CLAUDE.md directive 5).
fn substitute_units(
    text: &Text,
    old_text: &Text,
    new_text: &Text,
    instance_num: Option<i32>,
) -> Result<Text, Error> {
    let units: Vec<u16> = text.units().collect();
    let old: Vec<u16> = old_text.units().collect();
    let new: Vec<u16> = new_text.units().collect();
    let at = instance_num.and_then(|n| nth_overlapping_match(&units, &old, n as usize));
    let replaced = match instance_num {
        Some(_) => usize::from(at.is_some()),
        None => one_char_per_unit(text)
            .matches(one_char_per_unit(old_text).as_str())
            .count(),
    };
    let result_len = (units.len() + replaced * new.len()).saturating_sub(replaced * old.len());
    if result_len > MAX_TEXT_UNITS {
        return Err(Error::VALUE);
    }
    let result = match (instance_num, at) {
        (Some(_), None) => units,
        (Some(_), Some(at)) => [&units[..at], &new[..], &units[at + old.len()..]].concat(),
        (None, _) => unit_chars_to_units(&one_char_per_unit(text).replace(
            one_char_per_unit(old_text).as_str(),
            &one_char_per_unit(new_text),
        )),
    };
    Ok(Text::from_utf16(result))
}

/// The start of the `n`th (1-based) possibly overlapping occurrence of `pattern` in `text`,
/// found by KMP in linear time.
fn nth_overlapping_match(text: &[u16], pattern: &[u16], n: usize) -> Option<usize> {
    let mut fail = vec![0usize; pattern.len()];
    let mut k = 0;
    for i in 1..pattern.len() {
        while k > 0 && pattern[i] != pattern[k] {
            k = fail[k - 1];
        }
        if pattern[i] == pattern[k] {
            k += 1;
        }
        fail[i] = k;
    }
    let (mut matched, mut found) = (0, 0);
    for (i, unit) in text.iter().enumerate() {
        while matched > 0 && *unit != pattern[matched] {
            matched = fail[matched - 1];
        }
        if *unit == pattern[matched] {
            matched += 1;
        }
        if matched == pattern.len() {
            found += 1;
            if found == n {
                return Some(i + 1 - pattern.len());
            }
            matched = fail[matched - 1];
        }
    }
    None
}

#[cfg(test)]
mod tests {

    use super::{within_text_budget, Case, Delimiters};

    fn text_after(text: &str, delimiters: &[&str], n: i32, case: Case) -> Option<String> {
        let delimiters = Delimiters::new(delimiters.iter().map(|d| (*d).to_string()).collect());
        super::text_after(text, &delimiters, n, case, false)
    }

    fn text_before(text: &str, delimiters: &[&str], n: i32, case: Case) -> Option<String> {
        let delimiters = Delimiters::new(delimiters.iter().map(|d| (*d).to_string()).collect());
        super::text_before(text, &delimiters, n, case, false)
    }
    #[test]
    fn test_text_after_sensitive() {
        assert_eq!(
            text_after("One element", &["ele"], 1, Case::Sensitive),
            Some("ment".to_string())
        );
        assert_eq!(
            text_after("One element", &["e"], 1, Case::Sensitive),
            Some(" element".to_string())
        );
        assert_eq!(
            text_after("One element", &["e"], 4, Case::Sensitive),
            Some("nt".to_string())
        );
        assert_eq!(text_after("One element", &["e"], 5, Case::Sensitive), None);
        assert_eq!(
            text_after("長壽相等！", &["相"], 1, Case::Sensitive),
            Some("等！".to_string())
        );
    }
    #[test]
    fn test_text_before_sensitive() {
        assert_eq!(
            text_before("One element", &["ele"], 1, Case::Sensitive),
            Some("One ".to_string())
        );
        assert_eq!(
            text_before("One element", &["e"], 1, Case::Sensitive),
            Some("On".to_string())
        );
        assert_eq!(
            text_before("One element", &["e"], 4, Case::Sensitive),
            Some("One elem".to_string())
        );
        assert_eq!(text_before("One element", &["e"], 5, Case::Sensitive), None);
        assert_eq!(
            text_before("長壽相等！", &["相"], 1, Case::Sensitive),
            Some("長壽".to_string())
        );
    }
    #[test]
    fn test_text_after_insensitive() {
        assert_eq!(
            text_after("One element", &["eLe"], 1, Case::Insensitive),
            Some("ment".to_string())
        );
        assert_eq!(
            text_after("One element", &["E"], 1, Case::Insensitive),
            Some(" element".to_string())
        );
        assert_eq!(
            text_after("One element", &["E"], 4, Case::Insensitive),
            Some("nt".to_string())
        );
        assert_eq!(
            text_after("One element", &["E"], 5, Case::Insensitive),
            None
        );
        assert_eq!(
            text_after("長壽相等！", &["相"], 1, Case::Insensitive),
            Some("等！".to_string())
        );
    }
    #[test]
    fn test_text_before_insensitive() {
        assert_eq!(
            text_before("One element", &["eLe"], 1, Case::Insensitive),
            Some("One ".to_string())
        );
        assert_eq!(
            text_before("One element", &["E"], 1, Case::Insensitive),
            Some("On".to_string())
        );
        assert_eq!(
            text_before("One element", &["E"], 4, Case::Insensitive),
            Some("One elem".to_string())
        );
        assert_eq!(
            text_before("One element", &["E"], 5, Case::Insensitive),
            None
        );
        assert_eq!(
            text_before("長壽相等！", &["相"], 1, Case::Insensitive),
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

    // T-1272: a list of delimiters matches at the earliest position of any entry.
    #[test]
    fn delimiter_lists() {
        let list = [",", "-"];
        assert_eq!(
            text_after("a,b-c", &list, 2, Case::Sensitive),
            Some("c".to_string())
        );
        assert_eq!(
            text_before("a,b-c", &list, -1, Case::Sensitive),
            Some("a,b".to_string())
        );
        assert_eq!(text_before("a,b-c", &list, 3, Case::Sensitive), None);
        // The longest entry sets the length where two match at one position (T-1824).
        assert_eq!(
            text_after("xabc", &["ab", "a"], 1, Case::Sensitive),
            Some("c".to_string())
        );
        assert_eq!(
            text_after("xabc", &["a", "ab"], 1, Case::Sensitive),
            Some("c".to_string())
        );
        assert_eq!(
            text_before("xAbC", &["B", "x"], 1, Case::Insensitive),
            Some("".to_string())
        );
        assert_eq!(text_before("abc", &[], 1, Case::Sensitive), None);
    }

    #[test]
    fn instances_are_disjoint_but_substitute_overlaps() {
        // probes/t3335-textbefore-overlapping-instances A17, A18, A21, A22, A42, A55.
        assert_eq!(text_before("aaa", &["aa"], 2, Case::Sensitive), None);
        assert_eq!(
            text_before("aaaa", &["aa"], 2, Case::Sensitive),
            Some("aa".to_string())
        );
        assert_eq!(
            text_before("aaa", &["aa"], -1, Case::Sensitive),
            Some("a".to_string())
        );
        assert_eq!(text_before("aaa", &["aa"], -2, Case::Sensitive), None);
        assert_eq!(
            text_before("xabcx", &["abc", "b"], -1, Case::Sensitive),
            Some("xa".to_string())
        );
        assert_eq!(
            text_before("abcd", &["ab", "bc", "cd"], -2, Case::Sensitive),
            Some(String::new())
        );
        assert_eq!(text_before("AAA", &["aa"], 2, Case::Insensitive), None);
        // Backwards, a longer delimiter may run into the match counted before it
        // (gen-m7/a470dc9e165bdb20 Calc!A1441).
        assert_eq!(
            text_after("3.141590.2-0.5", &["0.2", "2"], -2, Case::Sensitive),
            Some("-0.5".to_string())
        );
        // SUBSTITUTE still counts the overlapping second "aa" (probes/substitute-overlapping-instances).
        assert_eq!(super::substitute("aaa", "aa", "x", 2), "ax");
    }

    #[test]
    fn match_end_is_one_more_instance() {
        // probes/t3335-textbefore-overlapping-instances A27, A48, A49.
        let aa = Delimiters::new(vec!["aa".to_string()]);
        let before = |n| super::text_before("aaa", &aa, n, Case::Sensitive, true);
        assert_eq!(before(2), Some("aaa".to_string()));
        assert_eq!(before(-2), Some(String::new()));
        assert_eq!(before(3), None);
        assert_eq!(
            super::text_after("aaa", &aa, 2, Case::Insensitive, true),
            Some(String::new())
        );
    }
}
