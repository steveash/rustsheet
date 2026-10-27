//! Reading the text of a complex number (`3+4i`, `$1+2i`) for the IM* functions.

use super::Suffix;
use crate::expressions::{
    lexer::util::get_tokens,
    token::{OpSum, TokenType},
};

pub(super) fn parse_complex_number(s: &str) -> Result<(f64, f64, Suffix), String> {
    // Check for i, j, -i, -j
    // One leading `+` is read like a leading `-`: `IMABS("+i")` 1, `IMABS("+3+4i")` 5, while a second
    // sign (`"++i"`, `"+-i"`) is `#NUM!` (probes/eng-leading-plus-oct-bool A1:A8, A26).
    let (sign, s) = match s.strip_prefix('-') {
        Some(r) => (-1.0, r),
        None => (1.0, s.strip_prefix('+').unwrap_or(s)),
    };
    // A currency symbol is read before a number part, after a sign or not: `IMABS("$1+2i")`,
    // `"-$1+2i"`, `"1+$2i"`, `"€1+2i"`, `"$3i"` and `"$ 1+2i"` are all read, while `"$i"`, `"-$i"`
    // (no number) and `"$$1"` are `#NUM!` (probes/eng-inumber-reading A5,
    // probes/eng-inumber-currency-shapes A1:A11). A sign after the symbol stays unreadable
    // (`"$-5"`, probes/eng-inumber-date-boundary A4).
    let stripped = strip_currency_symbols(s);
    let s = stripped.as_str();
    if s.starts_with(['+', '-']) {
        return Err(format!("Not a complex number: {s}"));
    }
    match s {
        "i" => return Ok((0.0, sign * 1.0, Suffix::I)),
        "j" => return Ok((0.0, sign * 1.0, Suffix::J)),
        _ => {
            // Let it go
        }
    };

    // TODO: This is an overuse
    let tokens = get_tokens(s);

    // There has to be 1, 2 3, or 4 tokens
    // number
    // number suffix
    // number1+suffix
    // number1+number2 suffix

    match tokens.len() {
        1 => {
            // Real number
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            Ok((sign * number1, 0.0, Suffix::Real))
        }
        2 => {
            // number2 i
            let number2 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[1].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            Ok((0.0, sign * number2, suffix))
        }
        3 => {
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let operation = match &tokens[1].token {
                TokenType::Addition(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[2].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            let number2 = if matches!(operation, OpSum::Minus) {
                -1.0
            } else {
                1.0
            };
            Ok((sign * number1, number2, suffix))
        }
        4 => {
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let operation = match &tokens[1].token {
                TokenType::Addition(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let mut number2 = match tokens[2].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[3].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            if matches!(operation, OpSum::Minus) {
                number2 = -number2;
            }
            Ok((sign * number1, number2, suffix))
        }
        _ => Err(format!("Not a complex number: {s}")),
    }
}

/// `text` without its currency symbols (`$`, `€`) that stand before a number part: at the start
/// or right after a `+` or `-` that begins a part, where a digit or `.` follows (past spaces).
fn strip_currency_symbols(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at_part_start = true;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if at_part_start && matches!(c, '$' | '€') {
            let mut rest = chars.clone().skip_while(|&r| r == ' ');
            if matches!(rest.next(), Some('0'..='9' | '.')) {
                at_part_start = false;
                continue;
            }
        }
        at_part_start = matches!(c, '+' | '-') && !out.ends_with(['E', 'e']);
        out.push(c);
    }
    out
}

/// Whether `text`, past its leading spaces, has a `-` that is neither its first character nor an
/// exponent's sign (`1E-3`, an `E` after a digit or a `.`): Excel's complex reader does not hand
/// such text to VALUE.
pub(super) fn has_inner_minus(text: &str) -> bool {
    let bytes = text.trim_start_matches(' ').as_bytes();
    (1..bytes.len()).any(|i| {
        let exponent_sign = matches!(bytes.get(i - 1), Some(b'E' | b'e'))
            && matches!(
                i.checked_sub(2).and_then(|j| bytes.get(j)),
                Some(b'0'..=b'9' | b'.')
            );
        bytes.get(i) == Some(&b'-') && !exponent_sign
    })
}
