//! How an IM* argument's text is read before it is parsed as a complex number.

/// Whether the `-` or `+` at `i` of `chars` is an exponent's sign: an `E` after a digit or a `.` (`1E-3`).
fn is_exponent_sign(chars: &[char], i: usize) -> bool {
    matches!(i.checked_sub(1).and_then(|j| chars.get(j)), Some('E' | 'e'))
        && matches!(
            i.checked_sub(2).and_then(|j| chars.get(j)),
            Some('0'..='9' | '.')
        )
}

/// Whether `text`, past its leading spaces, has a `-` that is neither its first character nor an
/// exponent's sign (`1E-3`): Excel's complex reader does not hand such text to VALUE.
pub(super) fn has_inner_minus(text: &str) -> bool {
    let chars: Vec<char> = text.trim_start_matches(' ').chars().collect();
    (1..chars.len()).any(|i| chars[i] == '-' && !is_exponent_sign(&chars, i))
}

/// `text` without the `$` and `€` signs that start the text or a part after a `+` or `-` that is not
/// an exponent's (spaces allowed around the sign) and stand before a digit or a `.`: Excel reads
/// `IMABS("$1+2i")`, `"-$1+2i"`, `"1-$2i"`, `"1+ $2i"`, `"$1+$2i"`, `"$ 1+2i"` and `"€1+2i"` as 1+2i
/// (2.23606797749979) and `"$.5"` as 0.5, but `"$i"`, `"$$1"`, `"$ -5"` and `"$-5"` are `#NUM!`, and so is
/// a sign inside a number (`"1$2"`, `"1.$5"`, `"1E-$3"`) or `£`/`¥` (probes/eng-inumber-reading A5,
/// A20:A26, probes/eng-inumber-currency-shapes A1:A11, A27:A40).
pub(super) fn strip_currency_symbols(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        let starts_part = match chars[..i].iter().rposition(|&p| p != ' ') {
            None => true,
            Some(j) => matches!(chars[j], '+' | '-') && !is_exponent_sign(&chars, j),
        };
        let before_digit = chars[i + 1..]
            .iter()
            .find(|&&n| n != ' ')
            .is_some_and(|&n| n.is_ascii_digit() || n == '.');
        if !(matches!(c, '$' | '€') && starts_part && before_digit) {
            out.push(c);
        }
    }
    out
}
