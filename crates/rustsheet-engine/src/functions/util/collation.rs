//! Excel's text order for `<`, `>`, `=` between texts, sorting, approximate lookups and the
//! text criteria (docs/excel-semantics/coercion.md "Text comparison uses Excel's collation, not
//! code-point order"; probes/text-collation-order, probes/text-order-nbsp-space, excel-graph,
//! T-1469, T-0244).
//!
//! Excel compares texts in three passes, like a word sort, and ignores case at every pass:
//! 1. **Letters** (primary weights): the empty string, the space, CHAR(160), punctuation
//!    ``! " # $ % & ( ) * , . / : ; ? @ [ \ ] ^ _ ` { | } ~``, then `+ < = >`, digits, letters.
//!    An accented letter is its base letter (`"ü"` sorts with `"u"`, before `"x"`); `"æ"` is
//!    `"ae"` and `"ß"` is `"ss"`. Hyphen and apostrophe are skipped (`"-1" > "0"`).
//! 2. **Accents**, letter by letter: none, acute, grave, circumflex, diaeresis, ring (`"a" < "á"
//!    < "à" < "â" < "ä" < "å"`), then `"ø"` after `"ö"`.
//! 3. **Hyphens and apostrophes**: by position, a later one first, then apostrophe before hyphen
//!    (`"ab" < "a'b" < "a-b"`, `"a" < "a'" < "a-" < "'a" < "-a"`, `"1" < "'1" < "-1"`).
//!
//! Characters outside ASCII and the Latin-1 letters are unprobed: they keep code-point order,
//! after the letters (control characters before the space), as before T-0220.

use std::cmp::{Ordering, Reverse};

/// Orders two texts the way Excel's `<`/`>`/`=` and the `">"` criteria do (module docs).
pub(crate) fn cmp_text_excel(left: &str, right: &str) -> Ordering {
    if left.is_ascii() && right.is_ascii() {
        // ASCII has no accents, so the second pass always ties.
        return ascii_primaries(left)
            .cmp(ascii_primaries(right))
            .then_with(|| skipped(left).cmp(skipped(right)));
    }
    primaries(left)
        .cmp(primaries(right))
        .then_with(|| accents(left).cmp(accents(right)))
        .then_with(|| skipped(left).cmp(skipped(right)))
}

/// The letter weight of each ASCII byte (0 for hyphen and apostrophe), as [`element`] gives it.
const ASCII_PRIMARY: [u32; 128] = ascii_primary_table();

const fn ascii_primary_table() -> [u32; 128] {
    let mut table = [0; 128];
    let mut byte = 0;
    while byte < 128 {
        table[byte] = match byte as u8 {
            b'\'' | b'-' => 0,
            b' ' => SPACE,
            b'0'..=b'9' => DIGIT_BASE + (byte as u32 - b'0' as u32),
            b'a'..=b'z' => LETTER_BASE + (byte as u32 - b'a' as u32),
            b'A'..=b'Z' => LETTER_BASE + (byte as u32 - b'A' as u32),
            0..=0x1F => CONTROL + byte as u32,
            _ => OTHER_BASE + byte as u32,
        };
        byte += 1;
    }
    let punctuation = PUNCTUATION.as_bytes();
    let mut index = 0;
    while index < punctuation.len() {
        table[punctuation[index] as usize] = PUNCTUATION_BASE + index as u32;
        index += 1;
    }
    table
}

/// The first pass over ASCII text, through [`ASCII_PRIMARY`].
fn ascii_primaries(text: &str) -> impl Iterator<Item = u32> + '_ {
    text.bytes()
        .map(|byte| ASCII_PRIMARY[usize::from(byte & 0x7F)])
        .filter(|&weight| weight != 0)
}

/// Primary weights: controls, then the space and CHAR(160), punctuation, digits, letters, and
/// every other character after the letters by code point.
const CONTROL: u32 = 1;
const SPACE: u32 = 0x40;
const NBSP: u32 = SPACE + 1;
const PUNCTUATION_BASE: u32 = SPACE + 2;
const DIGIT_BASE: u32 = 0x80;
const LETTER_BASE: u32 = 0xA0;
const OTHER_BASE: u32 = 0x100;

/// Punctuation in Excel's order (probes/text-collation-order ranks 7-36).
const PUNCTUATION: &str = "!\"#$%&()*,./:;?@[\\]^_`{|}~+<=>";

/// What one case-folded character contributes to each pass.
#[derive(Clone, Copy)]
enum Element {
    /// Hyphen or apostrophe: no letter weight, only a third-pass weight.
    Skipped(u8),
    /// One or two letter weights (two for `æ`, `ß`) and the accent of the character.
    Weights { primary: [u32; 2], accent: u8 },
}

/// Accent weights, in Excel's order (probes/text-collation-order: `á < à < â < ä < å`, `ö < ø`).
/// Tilde and cedilla are placed by Windows' sort-key order, not probed.
const NO_ACCENT: u8 = 0;
const ACUTE: u8 = 1;
const GRAVE: u8 = 2;
const CIRCUMFLEX: u8 = 3;
const DIAERESIS: u8 = 4;
const RING: u8 = 5;
const TILDE: u8 = 6;
const CEDILLA: u8 = 7;
const STROKE: u8 = 8;

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

/// `c` with case removed: ASCII and Latin-1 letters directly, others through `to_lowercase`.
fn fold(c: char) -> char {
    match c {
        'A'..='Z' => c.to_ascii_lowercase(),
        'À'..='Þ' if c != '×' => char::from_u32(u32::from(c) + 0x20).unwrap_or(c),
        _ if c.is_ascii() => c,
        _ => c.to_lowercase().next().unwrap_or(c),
    }
}

fn element(c: char) -> Element {
    let c = fold(c);
    match c {
        '\'' => Element::Skipped(1),
        '-' => Element::Skipped(2),
        _ if c.is_ascii() => single(ASCII_PRIMARY[c as usize]),
        '\u{a0}' => single(NBSP),
        'á' => accented('a', ACUTE),
        'à' => accented('a', GRAVE),
        'â' => accented('a', CIRCUMFLEX),
        'ä' => accented('a', DIAERESIS),
        'å' => accented('a', RING),
        'ã' => accented('a', TILDE),
        'ç' => accented('c', CEDILLA),
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
        'æ' => Element::Weights {
            primary: [letter('a'), letter('e')],
            accent: NO_ACCENT,
        },
        'ß' => Element::Weights {
            primary: [letter('s'), letter('s')],
            accent: NO_ACCENT,
        },
        _ => single(OTHER_BASE + u32::from(c)),
    }
}

/// The first pass: letter weights, hyphens and apostrophes skipped.
fn primaries(text: &str) -> impl Iterator<Item = u32> + '_ {
    text.chars()
        .flat_map(|c| match element(c) {
            Element::Skipped(_) => [0, 0],
            Element::Weights { primary, .. } => primary,
        })
        .filter(|&weight| weight != 0)
}

/// The second pass: one accent weight per letter weight of the first pass.
fn accents(text: &str) -> impl Iterator<Item = u8> + '_ {
    text.chars()
        .flat_map(|c| match element(c) {
            Element::Skipped(_) => [None, None],
            Element::Weights { primary, accent } => {
                [Some(accent), (primary[1] != 0).then_some(NO_ACCENT)]
            }
        })
        .flatten()
}

/// The third pass: each hyphen or apostrophe as (position, weight), a later position first.
fn skipped(text: &str) -> impl Iterator<Item = (Reverse<usize>, u8)> + '_ {
    text.chars()
        .enumerate()
        .filter_map(|(position, c)| match element(c) {
            Element::Skipped(weight) => Some((Reverse(position), weight)),
            Element::Weights { .. } => None,
        })
}
