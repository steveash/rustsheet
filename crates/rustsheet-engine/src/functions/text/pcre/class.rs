//! The fixed character classes of the PCRE2 dialect: POSIX classes, `\h`, and the classes
//! regex-syntax's Unicode tables build for `\d`, `\w`, `\p{..}` and the like.

use regex_syntax::hir::{Class, ClassUnicode, ClassUnicodeRange, HirKind};

use super::parse::Invalid;

/// PCRE2's `\h` and `[:blank:]`: Zs, tab and U+180E; LF, VT, NEL and U+2028 are not blank.
pub(super) const BLANK: &str = r"[\t\p{Zs}\x{180E}]";

/// The Unicode class each POSIX class matches, as regex-syntax text (probes/regex-posix-classes;
/// the characters it does not probe follow PCRE2's UCP definitions).
pub(super) fn posix_class_text(name: &str) -> Option<&'static str> {
    Some(match name {
        "alnum" => r"[\p{L}\p{N}]",
        "alpha" => r"\p{L}",
        "ascii" => r"[\x00-\x7F]",
        "blank" => BLANK,
        "cntrl" => r"\p{Cc}",
        "digit" => r"\p{Nd}",
        // L, M, N, P, S and Cf (soft hyphen, ZWSP and BOM are graphic), but no separator.
        "graph" => r"[[\p{L}\p{M}\p{N}\p{P}\p{S}\p{Cf}]--[\x{61C}\x{180E}\x{2066}-\x{2069}]]",
        "lower" => r"\p{Ll}",
        // Graph plus the Zs spaces (not tab, LF or U+2028).
        "print" => r"[[\p{L}\p{M}\p{N}\p{P}\p{S}\p{Cf}\p{Zs}]--[\x{61C}\x{180E}\x{2066}-\x{2069}]]",
        // Every P, plus the ASCII symbols (`$+<=>^`|~`); `×` and `€` are not punctuation.
        "punct" => r"[\p{P}[\x00-\x7F&&\p{S}]]",
        // White_Space plus U+180E (probes/regex-posix-classes-c A29): NEL, NBSP, U+2028 and
        // U+3000 are spaces, ZWSP is not.
        "space" => r"[\p{White_Space}\x{180E}]",
        "upper" => r"\p{Lu}",
        // Letters, all numbers (`²`, `½`, `Ⅳ`), non-spacing marks and connectors.
        "word" => r"[\p{L}\p{N}\p{Mn}\p{Pc}]",
        "xdigit" => r"[0-9A-Fa-f\x{FF10}-\x{FF19}\x{FF21}-\x{FF26}\x{FF41}-\x{FF46}]",
        _ => return None,
    })
}

/// The Unicode class regex-syntax builds for `text` (a fixed spelling of ours, or `\p{name}`
/// with a name from the pattern, which regex-syntax rejects when unknown).
pub(super) fn class_of(text: &str) -> Result<ClassUnicode, Invalid> {
    let hir = regex_syntax::Parser::new()
        .parse(text)
        .map_err(|_| Invalid)?;
    match hir.kind() {
        HirKind::Class(Class::Unicode(class)) => Ok(class.clone()),
        HirKind::Literal(literal) => {
            let c = std::str::from_utf8(&literal.0)
                .ok()
                .and_then(|s| s.chars().next())
                .ok_or(Invalid)?;
            Ok(ClassUnicode::new([ClassUnicodeRange::new(c, c)]))
        }
        _ => Err(Invalid),
    }
}

/// PCRE2's extended-mode white space (`(?x)`).
pub(super) fn is_extended_space(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'
            | '\n'
            | '\u{B}'
            | '\u{C}'
            | '\r'
            | '\u{85}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{2028}'
            | '\u{2029}'
    )
}
