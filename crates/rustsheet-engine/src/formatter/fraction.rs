//! Fraction sections with a fixed denominator (`# ?/4`, `# ??/16`, `?/100`).
//!
//! Rules from Excel's displayed text (docs/excel-semantics/number-formats.md "Fraction formats";
//! eval/suites/gen-m10 `F!` cells, excel-graph):
//! - the numerator is the fraction part times the denominator, rounded half away from zero at 15
//!   significant digits and never reduced (`# ?/4` on 1.5 is "1 2/4", `?/100` on 12.345 is
//!   "1235/100"); with no integer part the whole value goes into the numerator ("150/100");
//! - a numerator that rounds up to the denominator carries into the integer (`# ?/2` on 1.75 is
//!   "2    "), and a zero numerator blanks the fraction to spaces of its width, the integer then
//!   printing even under `#` (`# ?/4` on 1E-5 is "0    ");
//! - a zero integer part prints as its placeholders do with no digit (`#` nothing, `?` a space,
//!   `0` a zero: `# ?/4` on 0.5 is " 2/4");
//! - the minus leads the whole text, also when the value rounds to zero (`# ?/2` on -1/7 is
//!   "-0    "), except with no integer part, where a zero numerator takes no minus (`?/100` on
//!   -0.001 is "0/100", probes/fraction-format-features F9, F13);
//! - a comma between integer placeholders groups the integer digits (`#,##0 ?/4` on 1234.5 is
//!   "1,234 2/4", probes/fraction-format-features I10);
//! - with no integer part a numerator past Excel's bound cannot be shown ([`format_fraction`]
//!   returns `None`);
//! - a `,` right after the denominator scales the value up ([`fraction_scale`]), and a fixed
//!   denominator past 32768 is replaced by a picked one ([`LARGE_FIXED_MAX_DENOMINATOR`]).

use super::parser::{Denominator, Fraction, NumberPart, TextToken};
use crate::decimal_round::{round_decimal, RoundMode};
use crate::number_format::{group_thousands, to_precision};

/// The text of `value` under the number section `part`, which holds `fraction`, or `None` when
/// Excel cannot show it: with no integer part, a fixed denominator's numerator past 32767 and a
/// placeholder denominator's value from 2147483647 on, either sign (`?/100` on 327.67 is
/// "32767/100" and on 327.68 255 `#`, `TEXT` `#VALUE!`; `?/?` on 2147483646.9 is "2147483647/1"
/// and on 2147483647 255 `#`; probes/fraction-numerator-overflow, -overflow2). A whole part takes
/// the rest, so `# ?/100` and `# ?/?` show 1E15.
pub(super) fn format_fraction(
    value: f64,
    part: &NumberPart,
    fraction: &Fraction,
    group_separator: &str,
) -> Option<String> {
    let negative = value < 0.0;
    let mut value = value.abs() * 100.0_f64.powi(part.percent);
    if part.comma > 0 {
        // The scaled value is cut to 15 significant digits: `?/?,,` on 1E-251 is "0/1", where the
        // double product (0.10000000000000002) would pick 1/9 (probes/fraction-overflow-scaled A65).
        value = to_precision(value * fraction_scale(part.comma), 15);
    }
    // A scaled value past f64's range would print past 255 characters, which Excel cannot show.
    if !value.is_finite() {
        return None;
    }
    let tokens = &part.tokens;
    let has_integer = tokens[..fraction.numerator_start]
        .iter()
        .any(|t| matches!(t, TextToken::Digit(_)));
    let mut integer = if has_integer { value.trunc() } else { 0.0 };
    let numerator_kinds =
        placeholder_kinds(&tokens[fraction.numerator_start..fraction.numerator_end]);
    // The parser only builds a fixed denominator from ASCII digits starting with 1-9.
    let fixed = match &fraction.denominator {
        Denominator::Fixed(digits) => digits
            .parse::<f64>()
            .ok()
            .filter(|d| *d <= MAX_FIXED_DENOMINATOR),
        Denominator::Placeholders(_) => None,
    };
    // A picked denominator's placeholder kinds. A fixed one past 32768 prints its digits as
    // placeholders that pad with spaces under a `?` numerator and with nothing otherwise
    // (`?/40000` on 0.5 is "1/4    ", `0/40000` and `#/40000` "1/4", `??/40000` " 1/4    ").
    let denominator_kinds: Vec<char> = match &fraction.denominator {
        Denominator::Fixed(_) if fixed.is_some() => Vec::new(),
        Denominator::Fixed(digits) => {
            let kind = if numerator_kinds.last() == Some(&'?') {
                '?'
            } else {
                '#'
            };
            vec![kind; digits.len()]
        }
        Denominator::Placeholders(kinds) => kinds.clone(),
    };
    let (mut numerator, denominator) = match fixed {
        Some(denominator) => (
            round_numerator((value - integer) * denominator),
            denominator,
        ),
        None => {
            let max_denominator = match &fraction.denominator {
                Denominator::Fixed(_) => LARGE_FIXED_MAX_DENOMINATOR,
                // Past 15 digits 10^n - 1 is no longer exact in a double; no such bound can bind.
                Denominator::Placeholders(kinds) => {
                    let digits = i32::try_from(kinds.len()).unwrap_or(i32::MAX).min(15);
                    10.0_f64.powi(digits) - 1.0
                }
            };
            last_convergent(value - integer, max_denominator)
        }
    };
    let overflows = match fixed {
        Some(_) => numerator > MAX_FIXED_NUMERATOR,
        None => value >= MAX_PLACEHOLDER_VALUE,
    };
    if !has_integer && overflows {
        return None;
    }
    if has_integer && numerator >= denominator {
        integer += 1.0;
        numerator = 0.0;
    }
    let integer_kinds = placeholder_kinds(&tokens[..fraction.numerator_start]);
    // A zero numerator blanks the fraction unless a `0` placeholder prints it (`# 00/16` on 0 is
    // "00/16"; `#\:0=/=#` on 1 "1:0=/=1", poi/NumberFormatTests A258).
    let blank = has_integer && numerator == 0.0 && !numerator_kinds.contains(&'0');
    let sharp_numerator = numerator_kinds.first() == Some(&'#');
    let last_integer_kind = integer_kinds.last().copied();
    // A blank fraction prints as spaces of its width, or as nothing under a `#` numerator with
    // neither a `?` integer nor a `?` in the denominator (`#\:#=/=?` and `?\:#=/=#` on 1 are
    // "1      ", `#\:#=/=#`, `#\:#=/=0` and `0\:#=/=#` "1"; poi/NumberFormatTests A253, A261,
    // A252, A254, A270).
    let blank_spaces =
        !sharp_numerator || denominator_kinds.contains(&'?') || last_integer_kind == Some('?');
    let integer_digits = match (integer == 0.0, numerator == 0.0) {
        // A blank fraction leaves the integer printing even under `#`; a printed zero numerator
        // drops a zero integer under `#` and keeps it under `?` (`#\:0=/=#` on 0 is "0=/=1",
        // `?\:0=/=#` "0:0=/=1"; poi/NumberFormatTests A204, A213).
        (true, true) if blank || last_integer_kind != Some('#') => "0".to_owned(),
        (true, _) => String::new(),
        // Past 15 significant digits Excel prints zeros, as for any number format.
        (false, _) => {
            let digits = format!("{}", to_precision(integer, 15));
            if part.use_thousands {
                group_thousands(&digits, group_separator)
            } else {
                digits
            }
        }
    };
    let numerator_digits = if numerator == 0.0 {
        "0".to_owned()
    } else {
        format!("{numerator}")
    };
    let denominator_digits = format!("{denominator}");
    let denominator_text = match &fraction.denominator {
        Denominator::Fixed(digits) if fixed.is_some() => digits.clone(),
        // The code's own digits print, cut to the picked denominator's length: 512/625 under
        // `?/40000` is "512/400  ", 4095937/5000000 "4095937/40000".
        Denominator::Fixed(digits) => {
            let shown: String = digits.chars().take(denominator_digits.len()).collect();
            fill_left(&shown, &denominator_kinds)
        }
        Denominator::Placeholders(_) => fill_left(&denominator_digits, &denominator_kinds),
    };

    let separator = separator_kind(
        blank.then_some(blank_spaces),
        !integer_digits.is_empty(),
        last_integer_kind,
        numerator_kinds.first().copied(),
    );
    let mut text = String::new();
    if negative && (has_integer || numerator != 0.0) {
        text.push('-');
    }
    if let Some(c) = part.currency {
        text.push(c);
    }
    let mut integer_fill = fill(&integer_digits, &integer_kinds).into_iter();
    let mut numerator_fill = fill(&numerator_digits, &numerator_kinds).into_iter();
    let integer_end = tokens[..fraction.numerator_start]
        .iter()
        .rposition(|t| matches!(t, TextToken::Digit(_)))
        .map_or(0, |i| i + 1);
    for (index, token) in tokens.iter().enumerate() {
        if index == fraction.denominator_at {
            push_denominator(&mut text, &denominator_text, blank, blank_spaces);
        }
        let in_fraction = (fraction.numerator_start..fraction.denominator_at).contains(&index);
        let in_separator = has_integer && (integer_end..fraction.numerator_start).contains(&index);
        match token {
            _ if in_separator && separator == Separator::Dropped => {}
            _ if in_separator && separator == Separator::Spaces => push_spaces(&mut text, token),
            TextToken::Digit(_) if index < fraction.numerator_start => {
                text.push_str(&integer_fill.next().unwrap_or_default());
            }
            _ if in_fraction && blank && !blank_spaces => {}
            // Every placeholder and literal of a blank fraction takes its width in spaces.
            _ if in_fraction && blank => push_spaces(&mut text, token),
            TextToken::Digit(_) if in_fraction => {
                text.push_str(&numerator_fill.next().unwrap_or_default());
            }
            TextToken::Literal(c) => text.push(*c),
            TextToken::Text(t) => text.push_str(t),
            TextToken::Ghost(_) | TextToken::Spacer(_) => text.push(' '),
            _ => {}
        }
    }
    if fraction.denominator_at == tokens.len() {
        push_denominator(&mut text, &denominator_text, blank, blank_spaces);
    }
    Some(text)
}

/// How the separator (the literals between the integer part and the numerator) prints, given
/// whether the fraction is blank and blanks to spaces (`blank`), whether the integer prints digits,
/// and the kinds of the last integer and the first numerator placeholders. A blank fraction takes
/// its separator with it. Where the integer prints no digit the separator prints under a `0`
/// integer, turns into spaces under a `?` integer or before a `?` numerator, and is dropped
/// otherwise (`# #/#` on 0.5 is "1/2", gen-m10 039accab K8; `#\:?=/=#` on 0.75 " 3=/=4",
/// `?\:#=/=#` "  3=/=4", `0\:#=/=#` "0:3=/=4", poi/NumberFormatTests A228, A234, A243).
fn separator_kind(
    blank: Option<bool>,
    integer_prints: bool,
    last_integer_kind: Option<char>,
    first_numerator_kind: Option<char>,
) -> Separator {
    match blank {
        Some(true) => Separator::Spaces,
        Some(false) => Separator::Dropped,
        None if integer_prints || last_integer_kind == Some('0') => Separator::Printed,
        None if last_integer_kind == Some('?') || first_numerator_kind == Some('?') => {
            Separator::Spaces
        }
        None => Separator::Dropped,
    }
}

/// How the literals between a fraction's integer part and its numerator print.
#[derive(PartialEq)]
enum Separator {
    Printed,
    Spaces,
    Dropped,
}

/// Pushes the denominator, or its width in spaces (or nothing) for a blank fraction.
fn push_denominator(text: &mut String, denominator: &str, blank: bool, blank_spaces: bool) {
    match (blank, blank_spaces) {
        (false, _) => text.push_str(denominator),
        (true, true) => text.extend(denominator.chars().map(|_| ' ')),
        (true, false) => {}
    }
}

/// Pushes one space per column `token` would take.
fn push_spaces(text: &mut String, token: &TextToken) {
    let width = match token {
        TextToken::Text(t) => t.chars().count(),
        _ => 1,
    };
    text.extend(std::iter::repeat_n(' ', width));
}

/// The last continued-fraction convergent `(p, q)` of `x >= 0` whose denominator is at most
/// `max_denominator`, which is Excel's choice for a placeholder denominator (not the closest
/// fraction: `# ??/??` on `PI()` is "3  1/7 "; number-formats.md "Fraction formats").
///
/// Excel also stops where a partial quotient times the last numerator passes 2^31 - 1, although
/// the numerator itself (that product plus the one before) may pass it: `?/?` on 2147483646.6 is
/// "4294967293/2" (1 x 2147483647, then 2 x 4294967293 stops). When that stop comes before any
/// fractional term, the value is rounded to an integer: 1073741824.5 is "1073741825/1" (2 x
/// 1073741824 stops), 1073741823.5 "2147483647/2" (probes/fraction-placeholder-past-2-30;
/// number-formats.md "Placeholder fractions past 2^30").
fn last_convergent(x: f64, max_denominator: f64) -> (f64, f64) {
    // Cell values are finite; this only keeps a NaN out of the loop.
    if !x.is_finite() {
        return (0.0, 1.0);
    }
    let (mut p0, mut q0, mut p1, mut q1) = (0.0, 1.0, 1.0, 0.0);
    let mut rest = x;
    // A double's continued fraction ends within ~45 terms; the bound only guards the loop.
    for _ in 0..64 {
        let a = rest.floor();
        if a * p1 > MAX_PLACEHOLDER_VALUE {
            if q1 <= 1.0 {
                return (x.round(), 1.0);
            }
            break;
        }
        let (p, q) = (a * p1 + p0, a * q1 + q0);
        if q > max_denominator {
            break;
        }
        (p0, q0, p1, q1) = (p1, q1, p, q);
        let fract = rest - a;
        if fract < CONVERGENT_EPSILON {
            break;
        }
        rest = 1.0 / fract;
    }
    if q1 == 0.0 {
        (x.round(), 1.0)
    } else {
        (p1, q1)
    }
}

/// The factor `n` scaling commas right after a fraction's denominator apply:
/// 10^((-3n) mod 256), as if Excel kept the exponent -3n in an unsigned byte. `?/?,` on 1E-250 is
/// "1000/1" (10^253), `?/?,,` "1/1" (10^250), `?/?,,,` on 1E-238 "1000000000/1" (10^247); a
/// nonzero value that scales past the bounds is 255 `#` and `#VALUE!` from TEXT (`?/?,` on 0.5;
/// probes/fraction-overflow-scaled, number-formats.md "Scaling commas in fraction formats").
fn fraction_scale(commas: i32) -> f64 {
    10.0_f64.powi(comma_exponent(commas, false))
}

/// The exponent of ten that `commas` scaling commas apply: -3n, which Excel keeps in a byte,
/// unsigned in a fraction section ([`fraction_scale`]) and signed elsewhere (`signed`), where 252
/// commas on 5 print "5000000000000" (-756 wraps to 12; probes/t2842-dollar-run-contexts K1:K3,
/// number-formats.md "Many scaling commas wrap").
pub(super) fn comma_exponent(commas: i32, signed: bool) -> i32 {
    let exponent = commas.saturating_mul(-3).rem_euclid(256);
    if signed && exponent > 127 {
        exponent - 256
    } else {
        exponent
    }
}

/// The largest fixed denominator Excel uses as written (2^15); past it Excel picks the
/// denominator as for placeholders, up to [`LARGE_FIXED_MAX_DENOMINATOR`] (`?/32768` on 0.5 is
/// "16384/32768", `?/32769` "1/3    "; probes/fraction-large-fixed-denominator).
const MAX_FIXED_DENOMINATOR: f64 = 32768.0;

/// The largest denominator Excel picks for a fixed denominator past 32768 (10^7 - 1), whatever
/// its digit count: `?/40000` on 1/9999999 is "1/40000" and on 1E-7 "0/4    ", and `?/32769` alike
/// (probes/fraction-large-fixed-denominator).
const LARGE_FIXED_MAX_DENOMINATOR: f64 = 9_999_999.0;

/// The largest numerator a fixed denominator shows with no integer part (2^15 - 1).
const MAX_FIXED_NUMERATOR: f64 = 32767.0;

/// The magnitude from which a placeholder denominator with no integer part cannot show a value
/// (2^31 - 1).
const MAX_PLACEHOLDER_VALUE: f64 = 2_147_483_647.0;

/// A remainder below this ends the continued fraction.
const CONVERGENT_EPSILON: f64 = 1e-12;

/// `x` rounded half away from zero to an integer at 15 significant digits, as ROUND does.
fn round_numerator(x: f64) -> f64 {
    round_decimal(x, 0.0, RoundMode::HalfAwayFromZero).unwrap_or(x)
}

/// The kinds (`#`, `?` or `0`) of the digit placeholders among `tokens`, in order.
fn placeholder_kinds(tokens: &[TextToken]) -> Vec<char> {
    tokens
        .iter()
        .filter_map(|t| match t {
            TextToken::Digit(d) => Some(d.kind),
            _ => None,
        })
        .collect()
}

/// What each placeholder of `kinds` prints for `digits`, right-aligned: the first one takes any
/// digits beyond the placeholder count, and one with no digit prints nothing (`#`), a space (`?`)
/// or a zero (`0`).
fn fill(digits: &str, kinds: &[char]) -> Vec<String> {
    let digits: Vec<char> = digits.chars().collect();
    let pad = kinds.len().saturating_sub(digits.len());
    let extra = digits.len().saturating_sub(kinds.len());
    kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            if i < pad {
                match kind {
                    '?' => " ".to_owned(),
                    '0' => "0".to_owned(),
                    _ => String::new(),
                }
            } else if i == 0 {
                digits[..=extra].iter().collect()
            } else {
                digits[i - pad + extra].to_string()
            }
        })
        .collect()
}

/// Like [`fill`] for a denominator: each placeholder past `digits` prints nothing (`#`), a space
/// after the digits (`?`) or a zero before them (`0`): `# ???/???` on 1.5 is "1   1/2  ", `?/00`
/// on 1.5 "3/02" (number-formats.md "Fraction formats", probes/fraction-format-features B13) and
/// `#\:? ?0#/000` on 23.75 "2:3  03/004" (poi/NumberFormatTests A160).
fn fill_left(digits: &str, kinds: &[char]) -> String {
    let mut zeros = String::new();
    let mut spaces = String::new();
    for kind in kinds.iter().skip(digits.chars().count()) {
        match kind {
            '?' => spaces.push(' '),
            '0' => zeros.push('0'),
            _ => {}
        }
    }
    format!("{zeros}{digits}{spaces}")
}

#[cfg(test)]
mod comma_tests {
    use super::comma_exponent;

    #[test]
    fn comma_exponents_wrap_in_a_byte() {
        assert_eq!(comma_exponent(1, true), -3);
        assert_eq!(comma_exponent(42, true), -126);
        assert_eq!(comma_exponent(252, true), 12);
        assert_eq!(comma_exponent(254, true), 6);
        assert_eq!(comma_exponent(1, false), 253);
    }
}
