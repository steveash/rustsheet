//! Which section of a parsed format code a value takes, and what kind of section it is (split
//! from `parser.rs`, T-3649).

use super::lexer::Compare;
use super::parser::{ParsePart, TextToken};

/// How many of `parts` are number sections. A last section with `@` in a format of two or three
/// sections is the text section: numbers use the sections before it (`TEXT(-1,"0;x@")` is "-1",
/// `TEXT(-1,"0;0;x@")` "1"; probes/text-at-token-sections C10, C11, T-1656).
pub(crate) fn numeric_section_count(parts: &[ParsePart]) -> usize {
    match parts.last() {
        Some(last) if (2..4).contains(&parts.len()) && is_text_section(last) => parts.len() - 1,
        _ => parts.len(),
    }
}

/// The `[cond]` of a number or date section, if it has one.
pub(crate) fn section_condition(part: &ParsePart) -> Option<(Compare, f64)> {
    match part {
        ParsePart::Number(p) => p.condition,
        ParsePart::Date(p) => p.condition,
        ParsePart::General(_) | ParsePart::Error(_) => None,
    }
}

/// The section a value takes when the first section carries a condition, and whether the value
/// then prints without its sign. `None` leaves the choice to the sign rules.
///
/// Excel: the first section is used when its condition holds; otherwise a second condition
/// decides the next section and the last section takes the rest (`[<100]0.0;[>=1000]#,##0;0` prints
/// 999.9999 as "1000"; `[>=1]0.00;0.000` prints -0.5 as "-0.500"; gen-m10 039accabcfe1e6c7 F!Q1-Q20
/// and siblings, T-1734). The sign is dropped only by a condition that holds for negatives alone
/// (`[<=-1]0` prints -1.5 as "2"); `[<100]0.0` prints -0.5 as "-0.5" and `[<1]0.0000` prints
/// -0.5 as "-0.5000". `[<=0]` is unprobed.
pub(crate) fn conditional_section(
    parts: &[ParsePart],
    sections: usize,
    value: f64,
) -> Option<(usize, bool)> {
    if !(2..=4).contains(&sections) {
        return None;
    }
    let (first_cmp, first_bound) = section_condition(parts.first()?)?;
    let index = if first_cmp.holds(value, first_bound) {
        0
    } else if sections == 2 {
        1
    } else {
        // A third section is the "else"; a second without a condition keeps the sign rules.
        let (cmp, bound) = section_condition(parts.get(1)?)?;
        if cmp.holds(value, bound) {
            1
        } else {
            2
        }
    };
    let negatives_only = |(cmp, bound): (Compare, f64)| match cmp {
        Compare::LessThan => bound <= 0.0,
        Compare::LessOrEqualThan => bound < 0.0,
        _ => false,
    };
    let drop_sign = parts
        .get(index)
        .and_then(section_condition)
        .is_some_and(negatives_only);
    Some((index, drop_sign))
}

/// Whether `part` is a number section that holds a color or a condition and no token. Excel
/// prints the value in General there, a text value unchanged, where a section with nothing in it
/// (or only a locale tag) prints nothing: `TEXT(5,"[Red]")` is "5", `TEXT(-5,"0;[Red]")` "5",
/// `TEXT(5,"[>1];0")` "5", `TEXT("abc","0;;;[Red]")` "abc", `TEXT(5,"[$-409];0")` ""
/// (probes/text-value-kind-sweep B69, probes/text-value-kind-sweep-b rows 6-20; number-formats.md
/// "A section of only a color or a condition prints General").
pub(crate) fn is_bracket_only_section(part: &ParsePart) -> bool {
    match part {
        ParsePart::Number(p) => p.tokens.is_empty() && (p.color.is_some() || p.condition.is_some()),
        ParsePart::Date(_) | ParsePart::General(_) | ParsePart::Error(_) => false,
    }
}

/// Whether `part` holds `@`, which makes it a text section.
pub(crate) fn is_text_section(part: &ParsePart) -> bool {
    section_tokens(part).is_some_and(|t| t.iter().any(|t| matches!(t, TextToken::Raw)))
}

/// The tokens of a number or date section; `None` for General or an unparsable section.
pub(crate) fn section_tokens(part: &ParsePart) -> Option<&Vec<TextToken>> {
    match part {
        ParsePart::Number(p) => Some(&p.tokens),
        ParsePart::Date(p) => Some(&p.tokens),
        ParsePart::General(_) | ParsePart::Error(_) => None,
    }
}
