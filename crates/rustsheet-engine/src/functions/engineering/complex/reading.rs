//! How an IM* argument's text is read before it is parsed as a complex number.

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

/// `text` without the `$` and `€` signs that stand before a digit, past any spaces: Excel reads
/// `IMABS("$1+2i")`, `"$1-2i"`, `"$ 1+2i"`, `"1+$2i"` and `"€1+2i"` as 1+2i (2.23606797749979), but
/// not `"$i"` (`#NUM!`); a sign before a minus stays, so `"$-5"` is `#NUM!` (probes/eng-inumber-reading
/// A5, A20:A26).
pub(super) fn strip_currency_symbols(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        let before_digit = chars[i + 1..]
            .iter()
            .find(|&&n| n != ' ')
            .is_some_and(char::is_ascii_digit);
        if !(matches!(c, '$' | '€') && before_digit) {
            out.push(c);
        }
    }
    out
}
