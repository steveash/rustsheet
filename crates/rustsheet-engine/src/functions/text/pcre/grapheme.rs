//! `\X`: one extended grapheme cluster as Excel's PCRE2 reads it (T-2287). That is UAX #29
//! before Unicode 15.1 (no `GB9c`: a Devanagari conjunct is two clusters) with two quirks of
//! PCRE2's join table: a regional indicator joins only a second regional indicator, so an
//! Extend, ZWJ or `SpacingMark` after it starts a new cluster, and a Prepend does not join an
//! `Extended_Pictographic`. Excel: see docs/excel-semantics/text.md "`\R` and `\X` are atomic"
//! (probes/t2287-regex-r-x-spans A11-A33, B1-B24).
//!
//! The break property of a character comes from regex-syntax's Unicode tables, built once.

use std::sync::OnceLock;

use regex_syntax::hir::ClassUnicodeRange;

use super::class::class_of;

/// PCRE2's grapheme break values: UAX #29's, with `Extended_Pictographic` as one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Break {
    Cr,
    Lf,
    Control,
    Extend,
    Prepend,
    SpacingMark,
    L,
    V,
    T,
    Lv,
    Lvt,
    RegionalIndicator,
    Other,
    Zwj,
    Pictographic,
}

/// Each break value but `Other`, as the regex-syntax class that holds its characters.
/// `Extended_Pictographic` comes last: its characters are all `Other` in UAX #29.
const PROPERTIES: [(&str, Break); 14] = [
    (r"\p{gcb=CR}", Break::Cr),
    (r"\p{gcb=LF}", Break::Lf),
    (r"\p{gcb=Control}", Break::Control),
    (r"\p{gcb=Extend}", Break::Extend),
    (r"\p{gcb=Prepend}", Break::Prepend),
    (r"\p{gcb=SpacingMark}", Break::SpacingMark),
    (r"\p{gcb=L}", Break::L),
    (r"\p{gcb=V}", Break::V),
    (r"\p{gcb=T}", Break::T),
    (r"\p{gcb=LV}", Break::Lv),
    (r"\p{gcb=LVT}", Break::Lvt),
    (r"\p{gcb=Regional_Indicator}", Break::RegionalIndicator),
    (r"\p{gcb=ZWJ}", Break::Zwj),
    (r"\p{Extended_Pictographic}", Break::Pictographic),
];

/// Every character with a break value but `Other`, by range, sorted and disjoint.
fn table() -> &'static [(ClassUnicodeRange, Break)] {
    static TABLE: OnceLock<Vec<(ClassUnicodeRange, Break)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table: Vec<(ClassUnicodeRange, Break)> = Vec::new();
        for (text, value) in PROPERTIES {
            // The spellings are fixed and known to regex-syntax; a failure leaves the value
            // empty (its characters read as `Other`) rather than panicking.
            if let Ok(class) = class_of(text) {
                table.extend(class.ranges().iter().map(|range| (*range, value)));
            }
        }
        table.sort_by_key(|(range, _)| range.start());
        table
    })
}

fn break_of(c: char) -> Break {
    let table = table();
    let at = table.partition_point(|(range, _)| range.end() < c);
    match table.get(at) {
        Some((range, value)) if range.start() <= c => *value,
        _ => Break::Other,
    }
}

const EXTEND_SPACING_ZWJ: [Break; 3] = [Break::Extend, Break::SpacingMark, Break::Zwj];

/// Whether `right` may follow `left` in one cluster, by PCRE2's join table; the ZWJ and
/// regional-indicator pair rules are [`cluster_end`]'s.
fn joins(left: Break, right: Break) -> bool {
    use Break::{
        Control, Cr, Extend, Lf, Lv, Lvt, Other, Pictographic, Prepend, RegionalIndicator,
        SpacingMark, Zwj, L, T, V,
    };
    match left {
        Cr => right == Lf,
        Lf | Control => false,
        RegionalIndicator => right == RegionalIndicator,
        Extend | SpacingMark | Other | Pictographic => EXTEND_SPACING_ZWJ.contains(&right),
        Zwj => EXTEND_SPACING_ZWJ.contains(&right) || right == Pictographic,
        Prepend => !matches!(right, Cr | Lf | Control | Pictographic),
        L => EXTEND_SPACING_ZWJ.contains(&right) || matches!(right, L | V | Lv | Lvt),
        V | Lv => EXTEND_SPACING_ZWJ.contains(&right) || matches!(right, V | T),
        T | Lvt => EXTEND_SPACING_ZWJ.contains(&right) || right == T,
    }
}

/// Where the cluster starting at byte `pos` of `text` ends, and how many characters it holds;
/// `None` at the end of the text. A cluster is read from `pos` as if one started there.
pub(super) fn cluster_end(text: &str, pos: usize) -> Option<(usize, u64)> {
    let mut chars = text.get(pos..)?.char_indices();
    let (_, first) = chars.next()?;
    let mut left = break_of(first);
    let mut end = pos + first.len_utf8();
    let mut count = 1;
    // Whether the text so far ends in Extended_Pictographic Extend* (GB11), and whether the
    // ZWJ just read followed one.
    let mut pictographic = left == Break::Pictographic;
    let mut zwj_after_pictographic = false;
    // Regional indicators read in a row: a cluster holds at most one pair (GB12, GB13).
    let mut indicators = u32::from(left == Break::RegionalIndicator);
    for (offset, c) in chars {
        let right = break_of(c);
        let pair_rule_breaks = match (left, right) {
            (Break::Zwj, Break::Pictographic) => !zwj_after_pictographic,
            (Break::RegionalIndicator, Break::RegionalIndicator) => indicators > 1,
            _ => false,
        };
        if !joins(left, right) || pair_rule_breaks {
            break;
        }
        zwj_after_pictographic = right == Break::Zwj && pictographic;
        indicators = if right == Break::RegionalIndicator {
            indicators + 1
        } else {
            0
        };
        pictographic = match right {
            Break::Pictographic => true,
            Break::Extend => pictographic,
            _ => false,
        };
        left = right;
        end = pos + offset + c.len_utf8();
        count += 1;
    }
    Some((end, count))
}

#[cfg(test)]
mod tests {
    use super::{class_of, PROPERTIES};

    /// Every spelling regex-syntax must know: one it rejects would read as `Other` silently.
    #[test]
    fn every_break_property_has_characters() {
        for (text, _) in PROPERTIES {
            assert!(
                class_of(text).is_ok_and(|class| !class.ranges().is_empty()),
                "{text}"
            );
        }
    }
}
