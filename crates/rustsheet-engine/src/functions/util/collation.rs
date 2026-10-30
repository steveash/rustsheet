//! Excel's text order for `<`, `>`, `=` between texts, sorting, approximate lookups and the
//! text criteria (docs/excel-semantics/coercion.md "Text comparison uses Excel's collation, not
//! code-point order"; probes/text-collation-order, probes/text-order-nbsp-space, excel-graph,
//! T-1469, T-0244).
//!
//! Excel compares texts in four passes, like a word sort, and ignores case at every pass:
//! 1. **Letters** (primary weights): the empty string, the space, CHAR(160), CHAR(9) to
//!    CHAR(13), punctuation ``! " # $ % & ( ) * , . / : ; ? @ [ \ ] ^ _ ` { | } ~``, the Latin-1
//!    marks, quotes and currency signs, `+ < = >`, the other Latin-1 symbols, digits (`¼ ½ ¾`
//!    between `0` and `1`, `¹ ² ³` with `1 2 3`), letters.
//!    An accented letter is its base letter (`"ü"` sorts with `"u"`, before `"x"`); `"æ"` is
//!    `"ae"`, `"ß"` is `"ss"`, `"ĳ"` is `"ij"` and `"œ"` is `"oe"`. Hyphens, apostrophes, the
//!    en and em dash and the control characters other than CHAR(9) to CHAR(13) are skipped
//!    (`"-1" > "0"`).
//! 2. **Accents**, letter by letter: the diacritic weights of Microsoft's Windows sorting weight
//!    table, a letter's own plus those of the combining marks after it, in one wrapping byte
//!    (`"a" < "ª" < "á" < "à" < "â" < "ä" < "ǎ" < "ă" < "ā" < "ã" < "å"`, `"ö" < "ø"`,
//!    `"d" < "đ" < "ð"`; [`MARK_WEIGHT`]). `þ` is not an accent: it is the letters `th`.
//! 3. **Variants**: the superscripts `ª º ¹ ² ³` after the plain letters and digits
//!    (`"1" < "¹"`).
//! 4. **Skipped characters**: by position, a later one first, then control characters (by code
//!    point) before apostrophe, hyphen, en dash, em dash (`"ab" < "a'b" < "a-b" < "a–b" <
//!    "a—b"`, `"a" < "a'" < "a-" < "'a" < "-a"`, `"1" < "'1" < "-1"`, `"" < CHAR(1) < "–"`).
//!
//! The non-ASCII characters placed here are those of probes/text-collation-unprobed (T-1590,
//! T-1595); where they were probed as single characters only, the module places them by the
//! smallest rule that fits (docs/excel-semantics/coercion.md "Collation equality in lookups and
//! criteria; unprobed characters"). Latin Extended-A and -B, Greek, Cyrillic and Latin Extended
//! Additional letters with a canonical decomposition are their base letter plus their marks, and a
//! combining mark (U+0300 to U+0362) adds its weight to the letter before it
//! (probes/t3114-collation-beyond-latin1, probes/t3154-collation-accents; coercion.md "Letters
//! beyond Latin-1 and combining marks"). Every other character keeps code-point order after the
//! letters, as before T-0220.
//!
//! **Lone surrogates** (`cmp_text`): a lone surrogate unit weighs `UNIT_BASE` plus its value and an
//! astral character weighs its two UTF-16 units, so lone units sort above every BMP character
//! (U+FFFD included) and a lone high half sorts before a pair it starts, a lone low half after it
//! (docs/excel-semantics/text.md "Lone surrogates through CODE, comparisons, ...").

use std::cmp::{Ordering, Reverse};

use super::collation_tables::{COMBINING_LETTERS, DECOMPOSED, MARK_WEIGHT};
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
    /// The positions of the nonzero [`variants`] weights (all are [`SUPERSCRIPT`]); equality
    /// needs no more.
    superscripts: Vec<usize>,
    skipped: Vec<(usize, u8)>,
}

/// The [`CollationKey`] of `text`: its four passes, lone surrogates kept as [`cmp_text`] keeps them.
pub(crate) fn collation_key(text: &Text) -> CollationKey {
    let chars = || text.chars_lossless();
    CollationKey {
        primaries: primaries(chars()).collect(),
        accents: accents(chars()).collect(),
        superscripts: variants(chars())
            .enumerate()
            .filter_map(|(at, weight)| (weight != 0).then_some(at))
            .collect(),
        skipped: skipped(chars())
            .map(|(Reverse(at), weight)| (at, weight))
            .collect(),
    }
}

/// The four passes over two character sequences.
fn cmp_chars<L, R>(left: L, right: R) -> Ordering
where
    L: Iterator<Item = Char> + Clone,
    R: Iterator<Item = Char> + Clone,
{
    primaries(left.clone())
        .cmp(primaries(right.clone()))
        .then_with(|| accents(left.clone()).cmp(accents(right.clone())))
        .then_with(|| variants(left.clone()).cmp(variants(right.clone())))
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
    /// A skipped character (module docs): no letter weight, only a last-pass weight.
    Skipped(u8),
    /// One or two letter weights (two for `æ`, `ß`), the accent of the character and its
    /// variant-pass weight ([`SUPERSCRIPT`] or 0).
    Weights {
        primary: [u32; 2],
        accent: u8,
        variant: u8,
    },
    /// A combining mark: no letter weight; it adds its diacritic weight to the letter before it.
    Mark(u8),
}

/// Diacritic (accent) weights, from Microsoft's Windows sorting weight table (the DW column; see
/// [`MARK_WEIGHT`]): a letter without an accent weighs 2, an accented letter 2 plus its marks'
/// weights. So `"a" < "á" < "à" < "â" < "ä" < "ǎ" < "ă" < "ā" < "ã" < "å" < "ą"` (2, 14, 15, 18,
/// 19, 20, 21, 23, 25, 26, 27; probes/t3154-collation-accents Calc!A1:A12). The letters with a
/// weight of their own (ordinal indicators, stroke, eth, dotless i) take theirs from the table too:
/// `"a" < "ª" < "á"`, `"ö" < "ø"`, `"d" < "đ" < "ð"` (probes/text-collation-unprobed).
const NO_ACCENT: u8 = 2;
/// `ª` and `º` (they also differ from `a`, `o` in the variant pass, [`variants`]).
const ORDINAL: u8 = 3;
const ACUTE: u8 = accent('\u{301}');
const GRAVE: u8 = accent('\u{300}');
const CIRCUMFLEX: u8 = accent('\u{302}');
const DIAERESIS: u8 = accent('\u{308}');
const TILDE: u8 = accent('\u{303}');
const RING: u8 = accent('\u{30a}');
const CEDILLA: u8 = accent('\u{327}');
/// `ı`, `đ`, `ł`, `ø`, `ð`: Windows table weights 4, 30, 31, 33, 104.
const DOTLESS: u8 = 4;
const D_STROKE: u8 = 30;
const L_STROKE: u8 = 31;
const O_STROKE: u8 = 33;
const ETH: u8 = 104;
/// The weight of the letters U+0363 to U+036F ([`COMBINING_LETTERS`]).
const COMBINING_LETTER: u8 = 3;

/// The accent of a letter carrying combining mark `mark` alone.
const fn accent(mark: char) -> u8 {
    NO_ACCENT + MARK_WEIGHT[mark as usize - 0x300]
}

/// The diacritic weight combining mark `c` (U+0300 to U+0362) adds to the letter before it.
fn mark_weight(c: char) -> Option<u8> {
    let index = usize::try_from(u32::from(c).checked_sub(0x300)?).ok()?;
    MARK_WEIGHT.get(index).copied()
}

/// The variant (third) pass weight of the superscripts `ª º ¹ ² ³`. The Windows table gives them
/// the case weight 14 (superscript and compatibility bits) where a plain letter or digit has 2;
/// Excel ignores the upper-case bit but keeps this one: `"¹" > "1"` with equal letters and
/// accents (probes/text-collation-longer), and `"ª"&UNICHAR(769)` (3 + 12 = 15) sorts after `"à"`
/// (15) (probes/t3154-collation-accent-weights Calc!A111, A10). `ς` has case weight 10 in the
/// table but equals `σ` in Excel (probes/t3114-collation-beyond-latin1), so it keeps weight 0.
const SUPERSCRIPT: u8 = 1;

/// Last-pass weights of the skipped characters: control characters below the apostrophe (none
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
        variant: 0,
    }
}

fn single(primary: u32) -> Element {
    Element::Weights {
        primary: [primary, 0],
        accent: NO_ACCENT,
        variant: 0,
    }
}

/// A superscript (`ª º ¹ ² ³`): `primary` with `accent` and the [`SUPERSCRIPT`] variant weight.
fn superscript(primary: u32, accent: u8) -> Element {
    Element::Weights {
        primary: [primary, 0],
        accent,
        variant: SUPERSCRIPT,
    }
}

/// A character that sorts as the two letters `first`, `second` (`æ` = `ae`).
fn expansion(first: char, second: char) -> Element {
    Element::Weights {
        primary: [letter(first), letter(second)],
        accent: NO_ACCENT,
        variant: 0,
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

/// The final sigma `ς` as `σ`, every other character as is: Excel's `Σ = σ = ς` in comparisons
/// and in exact MATCH (probes/t3114-collation-beyond-latin1 Calc!B168, B192, B216, D20:E20, H10).
/// The collation key, `criteria::fold_run` and SEARCH's `text::casefold` all fold through it.
pub(crate) fn fold_final_sigma(c: char) -> char {
    if c == 'ς' {
        'σ'
    } else {
        c
    }
}

/// `c` with case removed: ASCII and Latin-1 letters directly, others through `to_lowercase`.
fn fold(c: char) -> char {
    match c {
        'A'..='Z' => c.to_ascii_lowercase(),
        'À'..='Þ' if c != '×' => char::from_u32(u32::from(c) + 0x20).unwrap_or(c),
        _ if c.is_ascii() => c,
        _ => fold_final_sigma(c.to_lowercase().next().unwrap_or(c)),
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
        '\u{363}'..='\u{36f}' => {
            let index = usize::try_from(u32::from(c) - 0x363).unwrap_or(0);
            accented(COMBINING_LETTERS[index], COMBINING_LETTER)
        }
        '\u{300}'..='\u{362}' => Element::Mark(mark_weight(c).unwrap_or(0)),
        '¼' => single(digit(0) + 1),
        '½' => single(digit(0) + 2),
        '¾' => single(digit(0) + 3),
        '¹' => superscript(digit(1), NO_ACCENT),
        '²' => superscript(digit(2), NO_ACCENT),
        '³' => superscript(digit(3), NO_ACCENT),
        'ª' => superscript(letter('a'), ORDINAL),
        'º' => superscript(letter('o'), ORDINAL),
        'á' => accented('a', ACUTE),
        'à' => accented('a', GRAVE),
        'â' => accented('a', CIRCUMFLEX),
        'ä' => accented('a', DIAERESIS),
        'å' => accented('a', RING),
        'ã' => accented('a', TILDE),
        'ç' => accented('c', CEDILLA),
        'đ' => accented('d', D_STROKE),
        'ð' => accented('d', ETH),
        'ı' => accented('i', DOTLESS),
        'ł' => accented('l', L_STROKE),
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
        'ø' => accented('o', O_STROKE),
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
                variant: 0,
            }
        }
        _ => single(punctuation(c).unwrap_or(OTHER_BASE + u32::from(c))),
    }
}

/// `c` case-folded and canonically decomposed through [`DECOMPOSED`]: one to three characters.
fn decompose(c: Char) -> impl Iterator<Item = Char> {
    let folded = match c {
        Char::Scalar(c) => fold(c),
        // Every arm yields a `[Char; 3]` iterator so that the function has one return type.
        Char::Lone(_) => return [c, c, c].into_iter().take(1),
    };
    let entry = if ('\u{100}'..='\u{45f}').contains(&folded)
        || ('\u{1e00}'..='\u{1eff}').contains(&folded)
    {
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

/// The second pass: one accent weight per letter weight of the first pass. A combining mark adds
/// its diacritic weight to the letter before it, wrapping past 255 like the Windows sort key's
/// one-byte diacritic weight, so the order of the marks does not matter:
/// `"a"&UNICHAR(769)&UNICHAR(780)` = `"a"&UNICHAR(780)&UNICHAR(769)` = `"á"&UNICHAR(780)` =
/// `"ǎ"&UNICHAR(769)` (2 + 12 + 18), and three U+0323 (2 + 3 * 86 = 260, so 4) sort between `"a"`
/// and `"á"` (probes/t3154-collation-accent-weights, probes/t3154-collation-accent-pairs). A mark
/// with no letter before it weighs as an accent of its own (unprobed).
fn accents(text: impl Iterator<Item = Char>) -> impl Iterator<Item = u8> {
    Accents {
        elements: elements(text),
        ready: None,
        last: None,
    }
}

/// The iterator of [`accents`]: `last` is held back so that a combining mark after it can
/// add to it.
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
                Some(Element::Mark(weight)) => {
                    self.last = Some(self.last.unwrap_or(NO_ACCENT).wrapping_add(weight));
                }
                Some(Element::Weights {
                    primary, accent, ..
                }) => {
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

/// The variant pass, after the accents: one weight per letter weight of the first pass,
/// [`SUPERSCRIPT`] for `ª º ¹ ² ³` and 0 otherwise.
fn variants(text: impl Iterator<Item = Char>) -> impl Iterator<Item = u8> {
    elements(text)
        .flat_map(|element| match element {
            Element::Weights {
                primary, variant, ..
            } => [Some(variant), (primary[1] != 0).then_some(0)],
            Element::Skipped(_) | Element::Mark(_) => [None, None],
        })
        .flatten()
}

/// The last pass: each hyphen or apostrophe as (position, weight), a later position first.
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
