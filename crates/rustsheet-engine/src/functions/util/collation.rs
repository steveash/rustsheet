//! Excel's text order for `<`, `>`, `=` between texts, sorting, approximate lookups and the
//! text criteria (docs/excel-semantics/coercion.md "Text comparison uses Excel's collation, not
//! code-point order"; probes/text-collation-order, probes/text-order-nbsp-space, excel-graph,
//! T-1469, T-0244).
//!
//! Excel compares texts in three passes, like a word sort, and ignores case at every pass:
//! 1. **Letters** (primary weights): the empty string, the space, CHAR(160), CHAR(9) to
//!    CHAR(13), punctuation ``! " # $ % & ( ) * , . / : ; ? @ [ \ ] ^ _ ` { | } ~``, the Latin-1
//!    marks, quotes and currency signs, `+ < = >`, the other Latin-1 symbols, digits (`¼ ½ ¾`
//!    between `0` and `1`, `¹ ² ³` with `1 2 3`), letters.
//!    An accented letter is its base letter (`"ü"` sorts with `"u"`, before `"x"`); `"æ"` is
//!    `"ae"`, `"ß"` is `"ss"`, `"ĳ"` is `"ij"` and `"œ"` is `"oe"`. Hyphens, apostrophes, the
//!    en and em dash and the control characters other than CHAR(9) to CHAR(13) are skipped
//!    (`"-1" > "0"`).
//! 2. **Accents**, letter by letter: none, the ordinal indicators `ª º`, acute, grave,
//!    circumflex, diaeresis, tilde, ring (`"a" < "ª" < "á" < "à" < "â" < "ä" < "ã" < "å"`), then
//!    the stroke (`"ö" < "ø"`, `"d" < "đ"`, `"l" < "ł"`) and last the letter variants
//!    `ð ı þ` (`"đ" < "ð"`, `"t" < "þ" < "u"`).
//! 3. **Skipped characters**: by position, a later one first, then control characters (by code
//!    point) before apostrophe, hyphen, en dash, em dash (`"ab" < "a'b" < "a-b" < "a–b" <
//!    "a—b"`, `"a" < "a'" < "a-" < "'a" < "-a"`, `"1" < "'1" < "-1"`, `"" < CHAR(1) < "–"`).
//!
//! The non-ASCII characters placed here are those of probes/text-collation-unprobed (T-1590,
//! T-1595); where they were probed as single characters only, the module places them by the
//! smallest rule that fits (docs/excel-semantics/coercion.md "Collation equality in lookups and
//! criteria; unprobed characters"). Every other character keeps code-point order after the
//! letters, as before T-0220.
//!
//! **Lone surrogates** (`cmp_text`): a lone surrogate unit weighs `UNIT_BASE` plus its value and an
//! astral character weighs its two UTF-16 units, so lone units sort above every BMP character
//! (U+FFFD included) and a lone high half sorts before a pair it starts, a lone low half after it
//! (docs/excel-semantics/text.md "Lone surrogates through CODE, comparisons, ...").

use std::cmp::{Ordering, Reverse};

use crate::text::{Char, Text};

/// Orders two texts the way Excel's `<`/`>`/`=` and the `">"` criteria do (module docs).
pub(crate) fn cmp_text_excel(left: &str, right: &str) -> Ordering {
    if left.is_ascii() && right.is_ascii() {
        // ASCII has no accents, so the second pass always ties.
        return ascii_primaries(left)
            .cmp(ascii_primaries(right))
            .then_with(|| skipped(scalars(left)).cmp(skipped(scalars(right))));
    }
    cmp_chars(scalars(left), scalars(right))
}

/// The characters of well-formed text.
fn scalars(text: &str) -> impl Iterator<Item = Char> + Clone + '_ {
    text.chars().map(Char::Scalar)
}

/// [`cmp_text_excel`] for engine text, a lone surrogate kept as its own unit instead of the
/// lossy U+FFFD (module docs; probes/text-lone-surrogates A10-A24).
pub(crate) fn cmp_text(left: &Text, right: &Text) -> Ordering {
    if left.has_lone_surrogate() || right.has_lone_surrogate() {
        cmp_chars(left.chars_lossless(), right.chars_lossless())
    } else {
        cmp_text_excel(left, right)
    }
}

/// The three passes over two character sequences.
fn cmp_chars<L, R>(left: L, right: R) -> Ordering
where
    L: Iterator<Item = Char> + Clone,
    R: Iterator<Item = Char> + Clone,
{
    primaries(left.clone())
        .cmp(primaries(right.clone()))
        .then_with(|| accents(left.clone()).cmp(accents(right.clone())))
        .then_with(|| skipped(left).cmp(skipped(right)))
}

/// The letter weight of each ASCII byte (0 for hyphen and apostrophe), as [`element`] gives it.
const ASCII_PRIMARY: [u32; 128] = ascii_primary_table();

const fn ascii_primary_table() -> [u32; 128] {
    let mut table = [0; 128];
    let mut byte = 0;
    while byte < 128 {
        table[byte] = match byte as u8 {
            b' ' => SPACE,
            b'\t'..=b'\r' => WHITESPACE_BASE + (byte as u32 - b'\t' as u32),
            b'0'..=b'9' => digit(byte as u32 - b'0' as u32),
            b'a'..=b'z' => LETTER_BASE + (byte as u32 - b'a' as u32),
            b'A'..=b'Z' => LETTER_BASE + (byte as u32 - b'A' as u32),
            // Skipped: hyphen, apostrophe and the other control characters.
            _ => 0,
        };
        byte += 1;
    }
    let mut index = 0;
    while index < PUNCTUATION.len() {
        let c = PUNCTUATION[index] as usize;
        if c < 128 {
            table[c] = PUNCTUATION_BASE + index as u32;
        }
        index += 1;
    }
    table
}

/// The primary weight of digit `value`, leaving room for `¼ ½ ¾` between `0` and `1`.
const fn digit(value: u32) -> u32 {
    DIGIT_BASE + 4 * value
}

/// The first pass over ASCII text, through [`ASCII_PRIMARY`].
fn ascii_primaries(text: &str) -> impl Iterator<Item = u32> + '_ {
    text.bytes()
        .map(|byte| ASCII_PRIMARY[usize::from(byte & 0x7F)])
        .filter(|&weight| weight != 0)
}

/// Primary weights: the space, CHAR(160), CHAR(9) to CHAR(13), punctuation, digits, letters, and
/// every other character after the letters by code point.
const SPACE: u32 = 0x40;
const NBSP: u32 = SPACE + 1;
/// CHAR(9), CHAR(10) and CHAR(13) sort above CHAR(160) (probes/text-collation-unprobed); CHAR(11)
/// and CHAR(12), unprobed, are kept with them in code-point order.
const WHITESPACE_BASE: u32 = NBSP + 1;
const PUNCTUATION_BASE: u32 = WHITESPACE_BASE + 5;
const DIGIT_BASE: u32 = 0x90;
const LETTER_BASE: u32 = 0xC0;
const OTHER_BASE: u32 = 0x100;

/// Punctuation and symbols in Excel's order: the ASCII ones from probes/text-collation-order
/// ranks 7-36, the others from probes/text-collation-unprobed, which ranks them against `!`, `~`,
/// `+` and `0` only. So the Latin-1 marks, quotes and currency signs go between `~` and `+`, and
/// the symbols after `+`, placed after `>` (their order against `< = >` is unprobed).
const PUNCTUATION: [char; 59] = [
    '!', '"', '#', '$', '%', '&', '(', ')', '*', ',', '.', '/', ':', ';', '?', '@', '[', '\\', ']',
    '^', '_', '`', '{', '|', '}', '~', '¡', '¦', '¨', '¯', '´', '¸', '¿', '\u{2018}', '\u{2019}',
    '\u{201c}', '\u{201d}', '¢', '£', '¤', '¥', '€', '+', '<', '=', '>', '±', '«', '»', '×', '÷',
    '§', '©', '¬', '®', '°', 'µ', '…', '•',
];

/// Weight of the first UTF-16 unit of an astral character or of a lone surrogate: above every BMP
/// character. An astral character weighs its two units, so a lone high surrogate sorts before a
/// pair starting with it and a lone low one after it (`H<"😀"`, `"😀"<L`, `H<L` TRUE, `L<H` FALSE;
/// probes/text-lone-surrogates A18-A22). A lone surrogate is above U+FFFD (A23, A24 FALSE).
const UNIT_BASE: u32 = OTHER_BASE + 0x11_0000;

/// What one case-folded character contributes to each pass.
#[derive(Clone, Copy)]
enum Element {
    /// A skipped character (module docs): no letter weight, only a third-pass weight.
    Skipped(u8),
    /// One or two letter weights (two for `æ`, `ß`) and the accent of the character.
    Weights { primary: [u32; 2], accent: u8 },
}

/// Accent weights, in Excel's order (probes/text-collation-order: `á < à < â < ä < å`, `ö < ø`;
/// probes/text-collation-unprobed: `a < ª < á`, `ä < ã < å`, `o < º < ö`, `đ < ð`). The cedilla
/// and superscript are only ever compared with no accent, so their place is unprobed.
const NO_ACCENT: u8 = 0;
const ORDINAL: u8 = 1;
const ACUTE: u8 = 2;
const GRAVE: u8 = 3;
const CIRCUMFLEX: u8 = 4;
const DIAERESIS: u8 = 5;
const TILDE: u8 = 6;
const RING: u8 = 7;
const CEDILLA: u8 = 8;
const SUPERSCRIPT: u8 = 9;
const STROKE: u8 = 10;
/// `ð`, `ı`, `þ`: probed as single characters only, each right after its base letter (`þ` after
/// `t`); kept a tiebreak accent, like `ø`, rather than a letter of its own.
const VARIANT: u8 = 11;

/// Third-pass weights of the skipped characters: control characters below the apostrophe (none
/// probed against it; `CHAR(1) < CHAR(31) < CHAR(127) < "–"`), then `' - – —`.
const APOSTROPHE: u8 = 0x80;
const HYPHEN: u8 = APOSTROPHE + 1;
const EN_DASH: u8 = APOSTROPHE + 2;
const EM_DASH: u8 = APOSTROPHE + 3;

fn letter(base: char) -> u32 {
    LETTER_BASE + (u32::from(base) - u32::from('a'))
}

fn accented(base: char, accent: u8) -> Element {
    Element::Weights {
        primary: [letter(base), 0],
        accent,
    }
}

fn single(primary: u32) -> Element {
    Element::Weights {
        primary: [primary, 0],
        accent: NO_ACCENT,
    }
}

/// A character that sorts as the two letters `first`, `second` (`æ` = `ae`).
fn expansion(first: char, second: char) -> Element {
    Element::Weights {
        primary: [letter(first), letter(second)],
        accent: NO_ACCENT,
    }
}

/// The primary weight of a punctuation mark or symbol of [`PUNCTUATION`].
fn punctuation(c: char) -> Option<u32> {
    PUNCTUATION
        .iter()
        .position(|&mark| mark == c)
        .and_then(|index| u32::try_from(index).ok())
        .map(|index| PUNCTUATION_BASE + index)
}

/// `c` with case removed: ASCII and Latin-1 letters directly, others through `to_lowercase`.
fn fold(c: char) -> char {
    match c {
        'A'..='Z' => c.to_ascii_lowercase(),
        'À'..='Þ' if c != '×' => char::from_u32(u32::from(c) + 0x20).unwrap_or(c),
        _ if c.is_ascii() => c,
        _ => c.to_lowercase().next().unwrap_or(c),
    }
}

fn element(c: Char) -> Element {
    let c = match c {
        Char::Scalar(c) => fold(c),
        Char::Lone(unit) => return single(UNIT_BASE + u32::from(unit)),
    };
    match c {
        '\'' => Element::Skipped(APOSTROPHE),
        '-' => Element::Skipped(HYPHEN),
        '\u{2013}' => Element::Skipped(EN_DASH),
        '\u{2014}' => Element::Skipped(EM_DASH),
        // A control character other than CHAR(9) to CHAR(13): 0x00-0x1F, and DEL after them.
        _ if c.is_ascii() && ASCII_PRIMARY[c as usize] == 0 => {
            Element::Skipped(u8::try_from(c).unwrap_or(0x7F))
        }
        _ if c.is_ascii() => single(ASCII_PRIMARY[c as usize]),
        '\u{a0}' => single(NBSP),
        '¼' => single(digit(0) + 1),
        '½' => single(digit(0) + 2),
        '¾' => single(digit(0) + 3),
        '¹' => Element::Weights {
            primary: [digit(1), 0],
            accent: SUPERSCRIPT,
        },
        '²' => Element::Weights {
            primary: [digit(2), 0],
            accent: SUPERSCRIPT,
        },
        '³' => Element::Weights {
            primary: [digit(3), 0],
            accent: SUPERSCRIPT,
        },
        'ª' => accented('a', ORDINAL),
        'º' => accented('o', ORDINAL),
        'á' => accented('a', ACUTE),
        'à' => accented('a', GRAVE),
        'â' => accented('a', CIRCUMFLEX),
        'ä' => accented('a', DIAERESIS),
        'å' => accented('a', RING),
        'ã' => accented('a', TILDE),
        'ç' => accented('c', CEDILLA),
        'đ' => accented('d', STROKE),
        'ð' => accented('d', VARIANT),
        'ı' => accented('i', VARIANT),
        'ł' => accented('l', STROKE),
        'þ' => accented('t', VARIANT),
        'é' => accented('e', ACUTE),
        'è' => accented('e', GRAVE),
        'ê' => accented('e', CIRCUMFLEX),
        'ë' => accented('e', DIAERESIS),
        'í' => accented('i', ACUTE),
        'ì' => accented('i', GRAVE),
        'î' => accented('i', CIRCUMFLEX),
        'ï' => accented('i', DIAERESIS),
        'ñ' => accented('n', TILDE),
        'ó' => accented('o', ACUTE),
        'ò' => accented('o', GRAVE),
        'ô' => accented('o', CIRCUMFLEX),
        'ö' => accented('o', DIAERESIS),
        'õ' => accented('o', TILDE),
        'ø' => accented('o', STROKE),
        'ú' => accented('u', ACUTE),
        'ù' => accented('u', GRAVE),
        'û' => accented('u', CIRCUMFLEX),
        'ü' => accented('u', DIAERESIS),
        'ý' => accented('y', ACUTE),
        'ÿ' => accented('y', DIAERESIS),
        'æ' => expansion('a', 'e'),
        'ß' => expansion('s', 's'),
        'ĳ' => expansion('i', 'j'),
        // Between `ø` and `s`; whether it equals `oe` is unprobed, so it is kept with `æ`.
        'œ' => expansion('o', 'e'),
        _ if u32::from(c) > 0xFFFF => {
            let mut units = [0u16; 2];
            c.encode_utf16(&mut units);
            Element::Weights {
                primary: [
                    UNIT_BASE + u32::from(units[0]),
                    UNIT_BASE + u32::from(units[1]),
                ],
                accent: NO_ACCENT,
            }
        }
        _ => single(punctuation(c).unwrap_or(OTHER_BASE + u32::from(c))),
    }
}

/// The first pass: letter weights, hyphens and apostrophes skipped.
fn primaries(text: impl Iterator<Item = Char>) -> impl Iterator<Item = u32> {
    text.flat_map(|c| match element(c) {
        Element::Skipped(_) => [0, 0],
        Element::Weights { primary, .. } => primary,
    })
    .filter(|&weight| weight != 0)
}

/// The second pass: one accent weight per letter weight of the first pass.
fn accents(text: impl Iterator<Item = Char>) -> impl Iterator<Item = u8> {
    text.flat_map(|c| match element(c) {
        Element::Skipped(_) => [None, None],
        Element::Weights { primary, accent } => {
            [Some(accent), (primary[1] != 0).then_some(NO_ACCENT)]
        }
    })
    .flatten()
}

/// The third pass: each hyphen or apostrophe as (position, weight), a later position first.
fn skipped(text: impl Iterator<Item = Char>) -> impl Iterator<Item = (Reverse<usize>, u8)> {
    text.enumerate()
        .filter_map(|(position, c)| match element(c) {
            Element::Skipped(weight) => Some((Reverse(position), weight)),
            Element::Weights { .. } => None,
        })
}
