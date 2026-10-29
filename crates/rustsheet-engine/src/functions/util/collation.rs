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
//!    circumflex, diaeresis, tilde (`"a" < "ª" < "á" < "à" < "â" < "ä" < "ã" < "å"`), breve, dot
//!    above, caron, macron, ring, ogonek, double acute, cedilla, then the stroke (`"ö" < "ø"`, `"d" < "đ"`, `"l" < "ł"`) and last the letter variants
//!    `ð ı` (`"đ" < "ð"`). `þ` is not an accent: it is the letters `th`.
//! 3. **Skipped characters**: by position, a later one first, then control characters (by code
//!    point) before apostrophe, hyphen, en dash, em dash (`"ab" < "a'b" < "a-b" < "a–b" <
//!    "a—b"`, `"a" < "a'" < "a-" < "'a" < "-a"`, `"1" < "'1" < "-1"`, `"" < CHAR(1) < "–"`).
//!
//! The non-ASCII characters placed here are those of probes/text-collation-unprobed (T-1590,
//! T-1595); where they were probed as single characters only, the module places them by the
//! smallest rule that fits (docs/excel-semantics/coercion.md "Collation equality in lookups and
//! criteria; unprobed characters"). Latin Extended-A, Greek and Cyrillic letters with a canonical
//! decomposition are their base letter plus an accent, and a combining mark (U+0300 to U+036F) is
//! the accent of the letter before it (probes/t3114-collation-beyond-latin1; coercion.md "Letters
//! beyond Latin-1 and combining marks"). Every other character keeps code-point order after the
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

/// What [`cmp_text`] compares: two texts are equal under it exactly when their keys are equal.
/// The exact-match lookup index of XMATCH and XLOOKUP hashes it (T-0448).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct CollationKey {
    primaries: Vec<u32>,
    accents: Vec<u8>,
    skipped: Vec<(usize, u8)>,
}

/// The [`CollationKey`] of `text`: its three passes, lone surrogates kept as [`cmp_text`] keeps them.
pub(crate) fn collation_key(text: &Text) -> CollationKey {
    let chars = || text.chars_lossless();
    CollationKey {
        primaries: primaries(chars()).collect(),
        accents: accents(chars()).collect(),
        skipped: skipped(chars())
            .map(|(Reverse(at), weight)| (at, weight))
            .collect(),
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
    /// A combining mark: no letter weight; it sets the accent of the letter before it.
    Mark(u8),
}

/// Accent weights, in Excel's order (probes/text-collation-order: `á < à < â < ä < å`, `ö < ø`;
/// probes/text-collation-unprobed: `a < ª < á`, `ä < ã < å`, `o < º < ö`, `đ < ð`;
/// probes/t3114-collation-beyond-latin1: `á < ă < ā < ą`, `é < ė < ě < ē < ę`, `ğ < ģ`,
/// `ň < ņ`, `ö < ő`, `ū < ů < ų < ű`, `ί < ϊ`). Only accents on one base letter were ranked, so
/// the order between the Latin-1 accents (grave to tilde) and breve to double acute is a fit;
/// the superscript is only ever compared with no accent.
const NO_ACCENT: u8 = 0;
const ORDINAL: u8 = 1;
const ACUTE: u8 = 2;
const GRAVE: u8 = 3;
const CIRCUMFLEX: u8 = 4;
const DIAERESIS: u8 = 5;
const TILDE: u8 = 6;
const BREVE: u8 = 7;
const DOT_ABOVE: u8 = 8;
const CARON: u8 = 9;
const MACRON: u8 = 10;
const RING: u8 = 11;
const OGONEK: u8 = 12;
const DOUBLE_ACUTE: u8 = 13;
const CEDILLA: u8 = 14;
const SUPERSCRIPT: u8 = 15;
const STROKE: u8 = 16;
/// `ð`, `ı`: probed as single characters only, each right after its base letter; kept a
/// tiebreak accent, like `ø`, rather than a letter of its own.
const VARIANT: u8 = 17;
/// A combining mark (U+0300 to U+036F) none of the probes ranks: after every accent.
const OTHER_MARK: u8 = 18;

/// The accent of combining mark `c` (U+0300 to U+036F). Excel equates a base letter plus a mark
/// with the precomposed letter (`"a"&UNICHAR(769) = "á"`, `"c"&UNICHAR(780)&"a" = "ča"`;
/// probes/t3114-collation-beyond-latin1 Calc!D7:E9, B141:B148).
fn mark(c: char) -> Option<u8> {
    Some(match c {
        '\u{300}' => GRAVE,
        '\u{301}' => ACUTE,
        '\u{302}' => CIRCUMFLEX,
        '\u{303}' => TILDE,
        '\u{304}' => MACRON,
        '\u{306}' => BREVE,
        '\u{307}' => DOT_ABOVE,
        '\u{308}' => DIAERESIS,
        '\u{30a}' => RING,
        '\u{30b}' => DOUBLE_ACUTE,
        '\u{30c}' => CARON,
        '\u{327}' => CEDILLA,
        '\u{328}' => OGONEK,
        '\u{300}'..='\u{36f}' => OTHER_MARK,
        _ => return None,
    })
}

/// The canonical decompositions of the lower-case Latin Extended-A, Greek and Cyrillic letters
/// that have one (`unicodedata.normalize("NFD", c)`): letter, base, mark, second mark or `'\0'`.
/// Excel ranks each as its base letter plus the mark's accent (`"ča"` between `"ca"` and `"cb"`,
/// `ё` between `е` and `ж`; probes/t3114-collation-beyond-latin1). Sorted by letter.
const DECOMPOSED: [(char, char, char, char); 72] = [
    ('ā', 'a', '\u{0304}', '\0'),
    ('ă', 'a', '\u{0306}', '\0'),
    ('ą', 'a', '\u{0328}', '\0'),
    ('ć', 'c', '\u{0301}', '\0'),
    ('ĉ', 'c', '\u{0302}', '\0'),
    ('ċ', 'c', '\u{0307}', '\0'),
    ('č', 'c', '\u{030c}', '\0'),
    ('ď', 'd', '\u{030c}', '\0'),
    ('ē', 'e', '\u{0304}', '\0'),
    ('ĕ', 'e', '\u{0306}', '\0'),
    ('ė', 'e', '\u{0307}', '\0'),
    ('ę', 'e', '\u{0328}', '\0'),
    ('ě', 'e', '\u{030c}', '\0'),
    ('ĝ', 'g', '\u{0302}', '\0'),
    ('ğ', 'g', '\u{0306}', '\0'),
    ('ġ', 'g', '\u{0307}', '\0'),
    ('ģ', 'g', '\u{0327}', '\0'),
    ('ĥ', 'h', '\u{0302}', '\0'),
    ('ĩ', 'i', '\u{0303}', '\0'),
    ('ī', 'i', '\u{0304}', '\0'),
    ('ĭ', 'i', '\u{0306}', '\0'),
    ('į', 'i', '\u{0328}', '\0'),
    ('ĵ', 'j', '\u{0302}', '\0'),
    ('ķ', 'k', '\u{0327}', '\0'),
    ('ĺ', 'l', '\u{0301}', '\0'),
    ('ļ', 'l', '\u{0327}', '\0'),
    ('ľ', 'l', '\u{030c}', '\0'),
    ('ń', 'n', '\u{0301}', '\0'),
    ('ņ', 'n', '\u{0327}', '\0'),
    ('ň', 'n', '\u{030c}', '\0'),
    ('ō', 'o', '\u{0304}', '\0'),
    ('ŏ', 'o', '\u{0306}', '\0'),
    ('ő', 'o', '\u{030b}', '\0'),
    ('ŕ', 'r', '\u{0301}', '\0'),
    ('ŗ', 'r', '\u{0327}', '\0'),
    ('ř', 'r', '\u{030c}', '\0'),
    ('ś', 's', '\u{0301}', '\0'),
    ('ŝ', 's', '\u{0302}', '\0'),
    ('ş', 's', '\u{0327}', '\0'),
    ('š', 's', '\u{030c}', '\0'),
    ('ţ', 't', '\u{0327}', '\0'),
    ('ť', 't', '\u{030c}', '\0'),
    ('ũ', 'u', '\u{0303}', '\0'),
    ('ū', 'u', '\u{0304}', '\0'),
    ('ŭ', 'u', '\u{0306}', '\0'),
    ('ů', 'u', '\u{030a}', '\0'),
    ('ű', 'u', '\u{030b}', '\0'),
    ('ų', 'u', '\u{0328}', '\0'),
    ('ŵ', 'w', '\u{0302}', '\0'),
    ('ŷ', 'y', '\u{0302}', '\0'),
    ('ź', 'z', '\u{0301}', '\0'),
    ('ż', 'z', '\u{0307}', '\0'),
    ('ž', 'z', '\u{030c}', '\0'),
    ('ΐ', 'ι', '\u{0308}', '\u{0301}'),
    ('ά', 'α', '\u{0301}', '\0'),
    ('έ', 'ε', '\u{0301}', '\0'),
    ('ή', 'η', '\u{0301}', '\0'),
    ('ί', 'ι', '\u{0301}', '\0'),
    ('ΰ', 'υ', '\u{0308}', '\u{0301}'),
    ('ϊ', 'ι', '\u{0308}', '\0'),
    ('ϋ', 'υ', '\u{0308}', '\0'),
    ('ό', 'ο', '\u{0301}', '\0'),
    ('ύ', 'υ', '\u{0301}', '\0'),
    ('ώ', 'ω', '\u{0301}', '\0'),
    ('й', 'и', '\u{0306}', '\0'),
    ('ѐ', 'е', '\u{0300}', '\0'),
    ('ё', 'е', '\u{0308}', '\0'),
    ('ѓ', 'г', '\u{0301}', '\0'),
    ('ї', 'і', '\u{0308}', '\0'),
    ('ќ', 'к', '\u{0301}', '\0'),
    ('ѝ', 'и', '\u{0300}', '\0'),
    ('ў', 'у', '\u{0306}', '\0'),
];

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

/// The primary weight of a punctuation mark or symbol of [`PUNCTUATION`]. Every mark is at or
/// below `€`, so text in other scripts skips the scan.
fn punctuation(c: char) -> Option<u32> {
    if c > '€' {
        return None;
    }
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
        // Excel: `Σ = σ = ς` (probes/t3114-collation-beyond-latin1 Calc!B168, B190, B216, D20:E20).
        'ς' => 'σ',
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
        '\u{300}'..='\u{36f}' => Element::Mark(mark(c).unwrap_or(OTHER_MARK)),
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
        // Excel: þ is the expansion `th` (probes/text-collation-longer Calc!A74-A76).
        'þ' => expansion('t', 'h'),
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

/// `c` case-folded and canonically decomposed through [`DECOMPOSED`]: one to three characters.
fn decompose(c: Char) -> impl Iterator<Item = Char> {
    let folded = match c {
        Char::Scalar(c) => fold(c),
        Char::Lone(_) => return [c, c, c].into_iter().take(1),
    };
    let entry = if ('\u{100}'..='\u{45f}').contains(&folded) {
        DECOMPOSED
            .binary_search_by_key(&folded, |&(letter, ..)| letter)
            .ok()
            .and_then(|index| DECOMPOSED.get(index))
    } else {
        None
    };
    match entry {
        Some(&(_, base, first, second)) => {
            let units = [
                Char::Scalar(base),
                Char::Scalar(first),
                Char::Scalar(second),
            ];
            units.into_iter().take(if second == '\0' { 2 } else { 3 })
        }
        None => [Char::Scalar(folded); 3].into_iter().take(1),
    }
}

/// The [`Element`] of each decomposed character of `text`.
fn elements(text: impl Iterator<Item = Char>) -> impl Iterator<Item = Element> {
    text.flat_map(decompose).map(element)
}

/// The first pass: letter weights, hyphens, apostrophes and combining marks skipped.
fn primaries(text: impl Iterator<Item = Char>) -> impl Iterator<Item = u32> {
    elements(text)
        .flat_map(|element| match element {
            Element::Skipped(_) | Element::Mark(_) => [0, 0],
            Element::Weights { primary, .. } => primary,
        })
        .filter(|&weight| weight != 0)
}

/// The second pass: one accent weight per letter weight of the first pass. A combining mark
/// gives an unaccented letter before it its accent (`"a"&UNICHAR(769)` is `"á"`) and otherwise
/// adds its own accent weight (`ΐ`, ι with dialytika and tonos, after `ϊ`;
/// probes/t3114-collation-beyond-latin1 Calc!A200, A204, A205). Other marks on an accented
/// letter are unprobed.
fn accents(text: impl Iterator<Item = Char>) -> impl Iterator<Item = u8> {
    Accents {
        elements: elements(text),
        ready: None,
        last: None,
    }
}

/// The iterator of [`accents`]: `last` is held back so that a combining mark after it can change it.
struct Accents<I> {
    elements: I,
    ready: Option<u8>,
    last: Option<u8>,
}

impl<I: Iterator<Item = Element>> Iterator for Accents<I> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if let Some(accent) = self.ready.take() {
            return Some(accent);
        }
        loop {
            match self.elements.next() {
                None => return self.last.take(),
                Some(Element::Skipped(_)) => {}
                Some(Element::Mark(accent)) => match self.last.replace(accent) {
                    None | Some(NO_ACCENT) => {}
                    Some(before) => return Some(before),
                },
                Some(Element::Weights { primary, accent }) => {
                    // An expansion (`æ`) weighs two letters: its accent, then an unaccented one.
                    let (first, held) = if primary[1] == 0 {
                        (None, accent)
                    } else {
                        (Some(accent), NO_ACCENT)
                    };
                    let before = self.last.replace(held);
                    match (before, first) {
                        (Some(before), first) => {
                            self.ready = first;
                            return Some(before);
                        }
                        (None, Some(first)) => return Some(first),
                        (None, None) => {}
                    }
                }
            }
        }
    }
}

/// The third pass: each hyphen or apostrophe as (position, weight), a later position first.
/// Positions count letters and skipped characters, not combining marks, so a decomposed letter
/// keeps the position of its precomposed form.
fn skipped(text: impl Iterator<Item = Char>) -> impl Iterator<Item = (Reverse<usize>, u8)> {
    elements(text)
        .filter(|element| !matches!(element, Element::Mark(_)))
        .enumerate()
        .filter_map(|(position, element)| match element {
            Element::Skipped(weight) => Some((Reverse(position), weight)),
            Element::Weights { .. } | Element::Mark(_) => None,
        })
}
