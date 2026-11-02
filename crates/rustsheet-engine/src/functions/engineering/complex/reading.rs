//! How an IM* argument's text is read: split into its parts, each part then read as VALUE reads it.

use super::parse::Suffix;

/// Whether a `+` or `-` that follows `before` is an exponent's sign: `before` ends in an `E` that
/// follows a digit or a `.` (`1E-3`, `1.E+2`).
fn is_exponent_sign(before: &str) -> bool {
    let mut back = before.chars().rev();
    matches!(back.next(), Some('E' | 'e')) && matches!(back.next(), Some('0'..='9' | '.'))
}

/// The characters a part may not start (past its leading spaces) or end with, where a space is
/// read: `IMABS` of tab&"5", " "&tab&"5", "5"&tab, "3+4i"&LF and NBSP&"3+4i" are `#NUM!`
/// (probes/eng-inumber-reading, probes/eng-inumber-currency-shapes A13:A26), and so are
/// "1+"&tab&"2i" and "1"&tab&"+2i" (probes/eng-inumber-grammar A32, A33).
const UNREAD_EDGE: [char; 5] = ['\t', '\u{3000}', '\u{a0}', '\n', '\r'];

/// Whether `part` starts (past its leading spaces) or ends with one of [`UNREAD_EDGE`].
pub(super) fn has_unread_edge(part: &str) -> bool {
    part.trim_start_matches(' ').starts_with(UNREAD_EDGE) || part.ends_with(UNREAD_EDGE)
}

/// The parts of inumber text, each still to be read as VALUE reads it.
#[derive(Debug, PartialEq)]
pub(super) enum Inumber<'a> {
    /// No `i`/`j` suffix: the whole text is a real number.
    Real(&'a str),
    /// A suffix: the real part's text, if any, and the imaginary coefficient, `sign` times the
    /// coefficient's text (empty text is a coefficient of 1).
    Complex {
        real: Option<&'a str>,
        sign: f64,
        coefficient: &'a str,
        suffix: Suffix,
    },
}

/// Splits `text` the way Excel's complex reader does (engineering.md "How an inumber is read";
/// probes/eng-inumber-grammar, probes/eng-inumber-grammar-parts):
/// - trailing spaces are dropped and a last `i` or `j` (lower case only) is the suffix;
/// - the first `+` or `-` that is neither the first character past leading spaces nor an
///   exponent's sign splits the rest into the real part and the imaginary coefficient, and that
///   sign is the coefficient's: "1--2i" is 1+2i and "1+-2i" 1-2i (the coefficient "-2" is read
///   as VALUE reads it), while "1---2i" is `#NUM!` because VALUE does not read "--2";
/// - with no split the rest is the coefficient, where a lone sign is ±1 ("-i"); after a split a
///   lone sign is not read ("1+-i" is `#NUM!`);
/// - text with a split and no suffix is not a number (`None`): "1+2", "2024-01-01", "1i+2".
pub(super) fn split_inumber(text: &str) -> Option<Inumber<'_>> {
    let core = text.trim_end_matches(' ');
    let (body, suffix) = if let Some(body) = core.strip_suffix('i') {
        (body, Some(Suffix::I))
    } else if let Some(body) = core.strip_suffix('j') {
        (body, Some(Suffix::J))
    } else {
        (text, None)
    };
    let lead = body.len() - body.trim_start_matches(' ').len();
    let split = body
        .char_indices()
        .skip_while(|&(k, _)| k <= lead)
        .find(|&(k, c)| matches!(c, '+' | '-') && !is_exponent_sign(&body[..k]))
        .map(|(k, _)| k);
    let Some(suffix) = suffix else {
        return split.is_none().then_some(Inumber::Real(text));
    };
    Some(match split {
        Some(k) => Inumber::Complex {
            real: Some(&body[..k]),
            sign: if body[k..].starts_with('-') {
                -1.0
            } else {
                1.0
            },
            coefficient: &body[k + 1..],
            suffix,
        },
        None => match body.trim_matches(' ') {
            "-" => Inumber::Complex {
                real: None,
                sign: -1.0,
                coefficient: "",
                suffix,
            },
            "+" => Inumber::Complex {
                real: None,
                sign: 1.0,
                coefficient: "",
                suffix,
            },
            _ => Inumber::Complex {
                real: None,
                sign: 1.0,
                coefficient: body,
                suffix,
            },
        },
    })
}

/// `text` without the `$` and `€` signs that start the text or a part after a `+` or `-` that is not
/// an exponent's (spaces allowed around the sign) and stand before a digit or a `.`: Excel reads
/// `IMABS("$1+2i")`, `"-$1+2i"`, `"1-$2i"`, `"1+ $2i"`, `"$1+$2i"`, `"$ 1+2i"` and `"€1+2i"` as 1+2i
/// (2.23606797749979) and `"$.5"` as 0.5, but `"$i"`, `"$$1"`, `"$ -5"` and `"$-5"` are `#NUM!`, and so is
/// a sign inside a number (`"1$2"`, `"1.$5"`, `"1E-$3"`) or `£`/`¥` (probes/eng-inumber-reading A5,
/// A20:A26, probes/eng-inumber-currency-shapes A1:A11, A27:A40). `complex_of` caps the text length first.
pub(super) fn strip_currency_symbols(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, c) in text.char_indices() {
        if !matches!(c, '$' | '€') {
            out.push(c);
            continue;
        }
        let before = text[..i].trim_end_matches(' ');
        let starts_part = match before.chars().next_back() {
            None => true,
            Some(sign @ ('+' | '-')) => {
                !is_exponent_sign(&before[..before.len() - sign.len_utf8()])
            }
            Some(_) => false,
        };
        let before_digit = text[i + c.len_utf8()..]
            .trim_start_matches(' ')
            .chars()
            .next()
            .is_some_and(|n| n.is_ascii_digit() || n == '.');
        if !(starts_part && before_digit) {
            out.push(c);
        }
    }
    out
}
