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
//!   "-0    ").

use super::parser::{Fraction, NumberPart, TextToken};
use crate::decimal_round::{round_decimal, RoundMode};
use crate::number_format::to_precision;

/// The text of `value` under the number section `part`, which holds `fraction`.
pub(super) fn format_fraction(value: f64, part: &NumberPart, fraction: &Fraction) -> String {
    let negative = value < 0.0;
    let value = value.abs() * 100.0_f64.powi(part.percent) / 1000.0_f64.powi(part.comma);
    // The parser only builds a fraction from a run of ASCII digits starting with 1-9.
    let denominator = fraction.denominator.parse::<f64>().unwrap_or(1.0);
    let tokens = &part.tokens;
    let has_integer = tokens[..fraction.numerator_start]
        .iter()
        .any(|t| matches!(t, TextToken::Digit(_)));
    let mut integer = if has_integer { value.trunc() } else { 0.0 };
    let mut numerator = round_numerator((value - integer) * denominator);
    if has_integer && numerator >= denominator {
        integer += 1.0;
        numerator = 0.0;
    }
    let blank = has_integer && numerator == 0.0;
    let integer_digits = match (integer == 0.0, numerator == 0.0) {
        (true, true) => "0".to_owned(),
        (true, false) => String::new(),
        // Past 15 significant digits Excel prints zeros, as for any number format.
        (false, _) => format!("{}", to_precision(integer, 15)),
    };
    let numerator_digits = if numerator == 0.0 {
        "0".to_owned()
    } else {
        format!("{numerator}")
    };

    let mut text = String::new();
    if negative {
        text.push('-');
    }
    if let Some(c) = part.currency {
        text.push(c);
    }
    let integer_kinds = placeholder_kinds(&tokens[..fraction.numerator_start]);
    let numerator_kinds = placeholder_kinds(&tokens[fraction.numerator_start..fraction.slash]);
    let mut integer_fill = fill(&integer_digits, &integer_kinds).into_iter();
    let mut numerator_fill = fill(&numerator_digits, &numerator_kinds).into_iter();
    for (index, token) in tokens.iter().enumerate() {
        let in_fraction = (fraction.numerator_start..=fraction.slash).contains(&index);
        match token {
            TextToken::Digit(_) if index < fraction.numerator_start => {
                text.push_str(&integer_fill.next().unwrap_or_default());
            }
            _ if in_fraction && blank => {
                // Every placeholder and the `/` take one column.
                text.push(' ');
                if index == fraction.slash {
                    text.extend(fraction.denominator.chars().map(|_| ' '));
                }
            }
            TextToken::Digit(_) if in_fraction => {
                text.push_str(&numerator_fill.next().unwrap_or_default());
            }
            TextToken::Literal('/') if index == fraction.slash => {
                text.push('/');
                text.push_str(&fraction.denominator);
            }
            TextToken::Literal(c) => text.push(*c),
            TextToken::Text(t) => text.push_str(t),
            TextToken::Ghost(_) | TextToken::Spacer(_) => text.push(' '),
            _ => {}
        }
    }
    text
}

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
