//! Excel's text order for `<`, `>`, `=` between texts, sorting, approximate lookups and the
//! text criteria (docs/excel-semantics/coercion.md "Text comparison uses Excel's collation, not
//! code-point order"; probes/text-collation-order, probes/text-order-nbsp-space, excel-graph,
//! T-1469, T-0244).
//!
//! Excel compares texts in five passes, like a word sort, and ignores case at every pass:
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
//! 3. **Kana**: a small kana before its letter, the long-vowel mark `ー` and the iteration marks
//!    `ゝ ゞ` after the letter they repeat, `々` before it (`"かぁ" < "かあ" < "かー"`,
//!    `"人々" < "人人"`; [`kana_weights`]).
//! 4. **Variants**: the superscripts `ª º ¹ ² ³` after the plain letters and digits
//!    (`"1" < "¹"`), a full-width form after its ASCII form (`"A" < "Ａ"`), and the kana type
//!    (`"ｱ" < "ア" < "あ"`).
//! 5. **Skipped characters**: by position, a later one first, then control characters (by code
//!    point) before apostrophe, hyphen, en dash, em dash, the full-width apostrophe and hyphen
//!    after theirs (`"ab" < "a'b" < "a-b" < "a–b" <
//!    "a—b"`, `"a" < "a'" < "a-" < "'a" < "-a"`, `"1" < "'1" < "-1"`, `"" < CHAR(1) < "–"`).
//!
//! Kana of one syllable share a letter after Cyrillic, in syllabary order, voicing an accent;
//! the long-vowel and iteration marks weigh by the letter before them ([`resolve`];
//! probes/t3275-collation-width-kana, coercion.md "Full-width and half-width forms, kana, voicing
//! and the long-vowel mark").
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
use std::iter::Peekable;

use super::collation_tables::{COMBINING_LETTERS, DECOMPOSED, MARK_WEIGHT};
use crate::text::{Char, Text};

/// Orders two texts by the module's five passes, the accents by their wrapped sums: a total
/// order, the one sorts, the sorted lookups and the collation key use. Excel's own `<`, `=`, `>`
/// are [`cmp_text_pairwise`].
#[cfg(test)]
pub(crate) fn cmp_text_excel(left: &str, right: &str) -> Ordering {
    cmp_str(left, right, AccentOrder::Total)
}

/// How the accent pass compares two letters' accents.
#[derive(Clone, Copy)]
enum AccentOrder {
    /// By the wrapped sum of each letter's units: a total order.
    Total,
    /// Excel's comparison of two texts: the units both letters start with are skipped first
    /// ([`cmp_accent_units`]); not transitive.
    Pairwise,
}

/// [`cmp_text_excel`] or [`cmp_text_pairwise`] for well-formed text.
fn cmp_str(left: &str, right: &str, order: AccentOrder) -> Ordering {
    if left.is_ascii() && right.is_ascii() {
        // ASCII has no accents, so the second pass always ties.
        return ascii_primaries(left)
            .cmp(ascii_primaries(right))
            .then_with(|| {
                skipped(elements(scalars(left), None)).cmp(skipped(elements(scalars(right), None)))
            });
    }
    cmp_chars(scalars(left), scalars(right), order)
}

/// The characters of well-formed text.
fn scalars(text: &str) -> impl Iterator<Item = Char> + Clone + '_ {
    text.chars().map(Char::Scalar)
}

/// [`cmp_text_excel`] for engine text, a lone surrogate kept as its own unit instead of the
/// lossy U+FFFD (module docs; probes/text-lone-surrogates A10-A24).
pub(crate) fn cmp_text(left: &Text, right: &Text) -> Ordering {
    cmp_text_in(left, right, AccentOrder::Total)
}

/// Orders two texts the way Excel's `<`, `=`, `>` and the text criteria do: as [`cmp_text`],
/// except that the accents of a letter are compared after the units both texts share at its
/// start ([`cmp_accent_units`]). Excel's rule, which is not transitive
/// (docs/excel-semantics/coercion.md "Shared leading units are skipped before the sum",
/// probes/t3181-collation-prefix-callers), so nothing may sort or hash with it: texts it finds
/// equal always have equal [`CollationKey`]s, so a key lookup followed by this test finds them.
pub(crate) fn cmp_text_pairwise(left: &Text, right: &Text) -> Ordering {
    cmp_text_in(left, right, AccentOrder::Pairwise)
}

/// [`cmp_text`] or [`cmp_text_pairwise`].
fn cmp_text_in(left: &Text, right: &Text, order: AccentOrder) -> Ordering {
    if left.has_lone_surrogate() || right.has_lone_surrogate() {
        cmp_chars(left.chars_lossless(), right.chars_lossless(), order)
    } else {
        cmp_str(left, right, order)
    }
}

/// What [`cmp_text`] compares: two texts are equal under it exactly when their keys are equal.
/// The exact-match lookup index of XMATCH and XLOOKUP hashes it (T-0448).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct CollationKey {
    primaries: Vec<u32>,
    accents: Vec<u8>,
    /// The positions and weights of the [`kana_weights`] other than [`NORMAL_KANA`].
    kana: Vec<(usize, u8)>,
    /// The positions and weights of the nonzero [`variants`].
    variants: Vec<(usize, u8)>,
    skipped: Vec<(usize, u8)>,
}

/// The [`CollationKey`] of `text`: its five passes, lone surrogates kept as [`cmp_text`] keeps them.
pub(crate) fn collation_key(text: &Text) -> CollationKey {
    let elements = || elements(text.chars_lossless(), None);
    let sparse = |weights: &mut dyn Iterator<Item = u8>, default: u8| {
        weights
            .enumerate()
            .filter(|&(_, weight)| weight != default)
            .collect()
    };
    CollationKey {
        primaries: primaries(elements()).collect(),
        accents: accents(elements()).collect(),
        kana: sparse(&mut kana_weights(elements()), NORMAL_KANA),
        variants: sparse(&mut variants(elements()), 0),
        skipped: skipped(elements())
            .map(|(Reverse(at), weight)| (at, weight))
            .collect(),
    }
}

/// The passes over two character sequences.
fn cmp_chars<L, R>(left: L, right: R, order: AccentOrder) -> Ordering
where
    L: Iterator<Item = Char> + Clone,
    R: Iterator<Item = Char> + Clone,
{
    let difference = match order {
        AccentOrder::Total => None,
        AccentOrder::Pairwise => Some(first_difference(left.clone(), right.clone())),
    };
    let (left, right) = (elements(left, difference), elements(right, difference));
    primaries(left.clone())
        .cmp(primaries(right.clone()))
        .then_with(|| match order {
            AccentOrder::Total => accents(left.clone()).cmp(accents(right.clone())),
            AccentOrder::Pairwise => cmp_accent_units(left.clone(), right.clone()),
        })
        .then_with(|| kana_weights(left.clone()).cmp(kana_weights(right.clone())))
        .then_with(|| variants(left.clone()).cmp(variants(right.clone())))
        .then_with(|| skipped(left).cmp(skipped(right)))
}

/// The position of the first character at which two texts differ, ignoring case (the length of
/// the shorter when one starts with the other): where Excel's comparison of the two loses the
/// context of a long-vowel mark after a non-kana letter ([`resolve`]).
fn first_difference(left: impl Iterator<Item = Char>, right: impl Iterator<Item = Char>) -> usize {
    let folded = |c| match c {
        Char::Scalar(c) => Char::Scalar(fold(c)),
        Char::Lone(_) => c,
    };
    left.zip(right)
        .take_while(|&(l, r)| folded(l) == folded(r))
        .count()
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

/// The primary weight of a character after the letters: by code point, two apart so that a weight
/// fits between two neighbours ([`LONE_MARK`]).
fn other(c: char) -> u32 {
    OTHER_BASE + 2 * u32::from(c)
}

/// A long-vowel or iteration mark with no letter before it (`"ー"`, `"ゝ"`, `"ゞ"`, `"々"`, `"ーa"`):
/// after the ideographs probed (`"ー" > "人人"`), all four equal (probes/t3275-collation-width-kana
/// Calc!B119 = 4, probes/t3275-collation-width-kana-pairs AC25, AC37). Placed after the last
/// unified ideograph.
const LONE_MARK: u32 = OTHER_BASE + 2 * 0x9FFF + 1;

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
/// character ([`other`] stays below `OTHER_BASE + 0x2_0000`). An astral character weighs its two units, so a lone high surrogate sorts before a
/// pair starting with it and a lone low one after it (`H<"😀"`, `"😀"<L`, `H<L` TRUE, `L<H` FALSE;
/// probes/text-lone-surrogates A18-A22). A lone surrogate is above U+FFFD (A23, A24 FALSE).
const UNIT_BASE: u32 = OTHER_BASE + 0x11_0000;

/// What one case-folded character contributes to each pass.
#[derive(Clone, Copy)]
enum Element {
    /// A skipped character (module docs): no letter weight, only a last-pass weight.
    Skipped(u8),
    /// One or two letter weights (two for `æ`, `ß`), the accent of the character, its kana-pass
    /// weight ([`kana_weights`]) and its variant-pass weight ([`variants`]).
    Weights {
        primary: [u32; 2],
        accent: u8,
        kana: u8,
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

/// The letter that `c` (U+0363 to U+036F) sorts as ([`COMBINING_LETTERS`]).
fn combining_letter(c: char) -> Option<char> {
    let index = usize::try_from(u32::from(c).checked_sub(0x363)?).ok()?;
    COMBINING_LETTERS.get(index).copied()
}

/// The diacritic weight combining mark `c` (U+0300 to U+0362) adds to the letter before it.
fn mark_weight(c: char) -> Option<u8> {
    let index = usize::try_from(u32::from(c).checked_sub(0x300)?).ok()?;
    MARK_WEIGHT.get(index).copied()
}

/// The variant pass weight of the superscripts `ª º ¹ ² ³`. The Windows table gives them
/// the case weight 14 (superscript and compatibility bits) where a plain letter or digit has 2;
/// Excel ignores the upper-case bit but keeps this one: `"¹" > "1"` with equal letters and
/// accents (probes/text-collation-longer), and `"ª"&UNICHAR(769)` (3 + 12 = 15) sorts after `"à"`
/// (15) (probes/t3154-collation-accent-weights Calc!A114, A10). `ς` has case weight 10 in the
/// table but equals `σ` in Excel (probes/t3114-collation-beyond-latin1), so it keeps weight 0.
const SUPERSCRIPT: u8 = 1;

/// The variant pass weight of a full-width form (U+FF01 to U+FF5E, the ideographic space U+3000):
/// it weighs as its ASCII form, just after it and never equal (`"0" < "０" < "1"`,
/// `"Ａ"="A"` FALSE), after the accents (`"Ａ" < "á"`), left to right (`"AＢ" < "Ａb"`)
/// (probes/t3275-collation-width-kana Calc!G1, G3, G6; pairs H6). Its order against
/// [`SUPERSCRIPT`] is unprobed.
const FULL_WIDTH: u8 = 2;

/// The variant pass weights of the kana types: half-width katakana < katakana < hiragana, left to
/// right, after the kana pass (`"ｱ" < "ア" < "あ"`, `"ｱあ" < "アあ" < "あア" < "ああ"`;
/// probes/t3275-collation-width-kana Calc!G12:G14, ranks A90:A95).
const HALF_WIDTH_KANA: u8 = 0;
const KATAKANA: u8 = 1;
const HIRAGANA: u8 = 2;

/// The kana pass weights, after the accents and before the variants: a small kana before its
/// letter, the long-vowel mark `ー` and the kana iteration marks `ゝ ゞ` after the letter they
/// repeat, and `々` before it (`"かぁ" < "カア" < "かあ" < "カー" < "かー"`, `"かか" < "かゝ"`,
/// `"人々" < "人人"`; probes/t3275-collation-width-kana ranks A99, A101:A102, A120:A123, A130:A131,
/// pairs AB26, AC27). Every other letter weighs [`NORMAL_KANA`].
const SMALL_KANA: u8 = 1;
const NORMAL_KANA: u8 = 2;
const LONG_VOWEL: u8 = 3;
const KANA_REPEAT: u8 = 3;
const KANJI_REPEAT: u8 = 1;

/// The diacritic weights the voicing marks add (U+3099, U+309B, U+FF9E voiced; U+309A, U+309C,
/// U+FF9F semi-voiced), and that a voiced kana carries: an accent on its unvoiced letter
/// (`"か" < "が"`, `"は" < "ば" < "ぱ"`, `"があ" < "かい"`; `"か"&UNICHAR(12441)` = `"が"`;
/// probes/t3275-collation-width-kana Calc!G20:G24, B96:B97). The values follow the order; their
/// size against the Latin marks is unprobed.
const VOICED: u8 = 1;
const SEMI_VOICED: u8 = 2;

/// Last-pass weights of the skipped characters: control characters below the apostrophe (none
/// probed against it; `CHAR(1) < CHAR(31) < CHAR(127) < "–"`), then `' ＇ - － – —` (the
/// full-width forms just after theirs: `"'" < "＇" < "-" < "－"`, `"a＇b" < "a-b" < "a－b"`,
/// probes/t3275-collation-width-kana ranks A5, A6, A58:A62; their order against the dashes is
/// unprobed).
const APOSTROPHE: u8 = 0x80;
const FULL_WIDTH_APOSTROPHE: u8 = APOSTROPHE + 1;
const HYPHEN: u8 = APOSTROPHE + 2;
const FULL_WIDTH_HYPHEN: u8 = APOSTROPHE + 3;
const EN_DASH: u8 = APOSTROPHE + 4;
const EM_DASH: u8 = APOSTROPHE + 5;

fn letter(base: char) -> u32 {
    LETTER_BASE + (u32::from(base) - u32::from('a'))
}

fn accented(base: char, accent: u8) -> Element {
    Element::Weights {
        primary: [letter(base), 0],
        accent,
        kana: NORMAL_KANA,
        variant: 0,
    }
}

fn single(primary: u32) -> Element {
    Element::Weights {
        primary: [primary, 0],
        accent: NO_ACCENT,
        kana: NORMAL_KANA,
        variant: 0,
    }
}

/// A superscript (`ª º ¹ ² ³`): `primary` with `accent` and the [`SUPERSCRIPT`] variant weight.
fn superscript(primary: u32, accent: u8) -> Element {
    Element::Weights {
        primary: [primary, 0],
        accent,
        kana: NORMAL_KANA,
        variant: SUPERSCRIPT,
    }
}

/// A character that sorts as the two letters `first`, `second` (`æ` = `ae`).
fn expansion(first: char, second: char) -> Element {
    Element::Weights {
        primary: [letter(first), letter(second)],
        accent: NO_ACCENT,
        kana: NORMAL_KANA,
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
/// The collation key and [`fold_case`] both fold through it.
pub(crate) fn fold_final_sigma(c: char) -> char {
    if c == 'ς' {
        'σ'
    } else {
        c
    }
}

/// Lowercases `text` one character at a time (see [`fold_char`]), never changing its length:
/// the case fold of SEARCH, of exact-match lookup keys (MATCH, VLOOKUP and HLOOKUP with an exact
/// match, XMATCH in wildcard mode and, inferred, XLOOKUP) and of wildcard and D-function prefix
/// criteria.
/// Excel: `SEARCH("x","İx")` is 2 and `SEARCH("i","İx")` is #VALUE!, whereas Rust's
/// `to_lowercase` turns U+0130 into "i" + U+0307 (probes/text-search-casefold-length A1, A2, A8,
/// A12, A13); `MATCH("i",{"İ"},0)` is #N/A and `COUNTIF(r,"?")` counts `İ`
/// (probes/t3133-lookup-criteria-case-fold; docs/excel-semantics/criteria.md "Wildcard escapes,
/// case and trailing tilde").
pub(crate) fn fold_case(text: &str) -> String {
    text.chars().map(fold_char).collect()
}

/// [`fold_case`] plus the two long-vowel and iteration mark equalities of exact MATCH: a `ー` after
/// a letter or digit that is neither kana nor an ideograph is that character again, and a text
/// starting with `ゝ`, `ゞ`, `ヽ`, `ヾ` or `々` starts with `ー`. Excel: `MATCH("aー",r,0)` finds
/// `"aa"` and `MATCH` of each lone mark finds `"ー"`, though `=` says `"aー"` equals only itself
/// (probes/t3275-collation-width-kana Calc!B136, C127:C129, C136). The fold of the exact-match
/// lookups and of [`crate::functions::criteria::WildcardPattern`]: one character to one, like
/// [`fold_case`]; how the marks fold inside a wildcard pattern is unprobed.
pub(crate) fn fold_lookup(text: &str) -> String {
    let mut previous: Option<char> = None;
    text.chars()
        .map(|c| {
            let folded = match (fold_char(c), previous) {
                ('\u{30fc}', Some(letter))
                    if letter.is_alphanumeric()
                        && !('\u{3005}'..='\u{9fff}').contains(&letter)
                        && !('\u{ff66}'..='\u{ff9f}').contains(&letter) =>
                {
                    letter
                }
                ('\u{309d}' | '\u{309e}' | '\u{30fd}' | '\u{30fe}' | '\u{3005}', None) => {
                    '\u{30fc}'
                }
                (folded, _) => folded,
            };
            previous = Some(folded);
            folded
        })
        .collect()
}

/// [`fold_case`] for one character: always exactly one character out. A character whose full
/// lowercase is one character takes it (the Kelvin sign is `k`, `ẞ` is `ß`, `ǅ` and `Ǆ` are `ǆ`,
/// the Ångström sign is `å`, the Ohm sign is `ω`); one whose lowercase is longer (`İ`) stays as
/// it is; final sigma matches sigma in either case. `ſ`/`s`, `µ`/`μ` and `ϐ`/`β` stay apart: each
/// is its own lowercase (SEARCH: probes/text-search-casefold-length A14-A24; lookups and criteria:
/// probes/t3133-lookup-criteria-case-fold).
pub(crate) fn fold_char(c: char) -> char {
    let c = fold_final_sigma(c);
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(single), None) => single,
        _ => c,
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
        '\u{ff07}' => Element::Skipped(FULL_WIDTH_APOSTROPHE),
        '\u{ff0d}' => Element::Skipped(FULL_WIDTH_HYPHEN),
        '\u{3000}' => full_width(single(SPACE)),
        '\u{ff01}'..='\u{ff5e}' => char::from_u32(u32::from(c) - 0xFEE0).map_or_else(
            || single(other(c)),
            |ascii| full_width(element(Char::Scalar(ascii))),
        ),
        '\u{3099}' | '\u{309b}' | '\u{ff9e}' => Element::Mark(VOICED),
        '\u{309a}' | '\u{309c}' | '\u{ff9f}' => Element::Mark(SEMI_VOICED),
        // A control character other than CHAR(9) to CHAR(13): 0x00-0x1F, and DEL after them.
        _ if c.is_ascii() && ASCII_PRIMARY[c as usize] == 0 => {
            Element::Skipped(u8::try_from(c).unwrap_or(0x7F))
        }
        _ if c.is_ascii() => single(ASCII_PRIMARY[c as usize]),
        '\u{a0}' => single(NBSP),
        '\u{363}'..='\u{36f}' => combining_letter(c)
            .map_or_else(|| single(other(c)), |base| accented(base, COMBINING_LETTER)),
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
                kana: NORMAL_KANA,
                variant: 0,
            }
        }
        _ => kana(c).unwrap_or_else(|| single(punctuation(c).unwrap_or_else(|| other(c)))),
    }
}

/// `c` case-folded and canonically decomposed through [`DECOMPOSED`]: one to three characters.
fn decompose(c: Char) -> impl Iterator<Item = Char> + Clone {
    let folded = match c {
        // Excel: `İ` is `i` + U+0307 (its canonical decomposition, lower-cased), not `i` or `I`:
        // `XMATCH("i"&UNICHAR(775),{"İ"})` is 1 and `XMATCH("I",{"İ"})` is #N/A
        // (probes/t3133-lookup-criteria-case-fold Calc!J1, J3, J33).
        Char::Scalar('\u{130}') => {
            return [Char::Scalar('i'), Char::Scalar('\u{307}'), c]
                .into_iter()
                .take(2)
        }
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

/// The [`Element`] of each decomposed character of `text`, the long-vowel and iteration marks
/// resolved against the letter before them ([`resolve`]). `difference` is the position where a
/// pairwise comparison's two texts first differ ([`first_difference`]), `None` for the total order.
fn elements(
    text: impl Iterator<Item = Char> + Clone,
    difference: Option<usize>,
) -> impl Iterator<Item = Element> + Clone {
    text.enumerate()
        .flat_map(|(at, c)| decompose(c).map(move |c| (at, c)))
        .scan(None, move |previous, (at, c)| {
            let element = resolve(c, *previous, difference == Some(at));
            if let Element::Weights { primary, .. } = element {
                *previous = Some(if primary[1] == 0 {
                    primary[0]
                } else {
                    primary[1]
                });
            }
            Some(element)
        })
}

/// The [`Element`] of `c`, a long-vowel or iteration mark weighed by `previous`, the last letter
/// weight before it (probes/t3275-collation-width-kana, probes/t3275-collation-width-kana-pairs;
/// docs/excel-semantics/coercion.md "Full-width and half-width forms, kana, voicing and the
/// long-vowel mark"):
/// - `ー` (and the half-width `ｰ`) after a kana is that kana's vowel, just after the written vowel
///   (`"かあ" < "かー" < "かい"`, `"きい" < "きー"`).
/// - `ゝ ヽ` repeat the letter before them and `ゞ ヾ` repeat it voiced, just after the written
///   repeat (`"かか" < "かゝ" < "かが" < "かゞ"`); `々` repeats it just before
///   (`"人" < "人々" < "人人"`).
/// - `ー` after any other letter is that letter again (`"aー"` sorts as `"aa"`: below `"Ａa"`),
///   except at `lost`, the first character where the two compared texts differ: there it has no
///   letter before it (`"aー" > "ab"`, `"aー" > "AＢ"`). So Excel's `<` is not transitive there
///   (pairs E8, F4, H8; the collation key and the sorts keep `"aa"`).
/// - With no letter before them (or lost) the marks are [`LONE_MARK`].
fn resolve(c: Char, previous: Option<u32>, lost: bool) -> Element {
    let Char::Scalar(mark) = c else {
        return element(c);
    };
    let repeat = |primary, accent, kana, variant| Element::Weights {
        primary: [primary, 0],
        accent,
        kana,
        variant,
    };
    let lone = single(LONE_MARK);
    match (mark, previous) {
        ('\u{30fc}' | '\u{ff70}', Some(previous)) => {
            let variant = if mark == '\u{30fc}' {
                KATAKANA
            } else {
                HALF_WIDTH_KANA
            };
            match vowel(previous) {
                Some(vowel) => repeat(vowel, NO_ACCENT, LONG_VOWEL, variant),
                None if lost => lone,
                None => single(previous),
            }
        }
        ('\u{309d}', Some(previous)) => repeat(previous, NO_ACCENT, KANA_REPEAT, HIRAGANA),
        ('\u{309e}', Some(previous)) => repeat(previous, NO_ACCENT + VOICED, KANA_REPEAT, HIRAGANA),
        ('\u{30fd}', Some(previous)) => repeat(previous, NO_ACCENT, KANA_REPEAT, KATAKANA),
        ('\u{30fe}', Some(previous)) => repeat(previous, NO_ACCENT + VOICED, KANA_REPEAT, KATAKANA),
        ('\u{3005}', Some(previous)) => repeat(previous, NO_ACCENT, KANJI_REPEAT, 0),
        (
            '\u{30fc}' | '\u{ff70}' | '\u{309d}' | '\u{309e}' | '\u{30fd}' | '\u{30fe}'
            | '\u{3005}',
            None,
        ) => lone,
        _ => element(c),
    }
}

/// `element` as a full-width form: the same weights, [`FULL_WIDTH`] in the variant pass.
fn full_width(element: Element) -> Element {
    match element {
        Element::Weights {
            primary,
            accent,
            kana,
            ..
        } => Element::Weights {
            primary,
            accent,
            kana,
            variant: FULL_WIDTH,
        },
        other => other,
    }
}

/// The hiragana each half-width katakana U+FF66 to U+FF9D is (U+FF70, the half-width `ｰ`, is a
/// long-vowel mark, [`resolve`]).
const HALF_WIDTH_HIRAGANA: [char; 56] = [
    'を', 'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'ゃ', 'ゅ', 'ょ', 'っ', 'ー', 'あ', 'い', 'う', 'え', 'お',
    'か', 'き', 'く', 'け', 'こ', 'さ', 'し', 'す', 'せ', 'そ', 'た', 'ち', 'つ', 'て', 'と', 'な',
    'に', 'ぬ', 'ね', 'の', 'は', 'ひ', 'ふ', 'へ', 'ほ', 'ま', 'み', 'む', 'め', 'も', 'や', 'ゆ',
    'よ', 'ら', 'り', 'る', 'れ', 'ろ', 'わ', 'ん',
];

/// The weights of a kana letter: hiragana, katakana and half-width katakana of one syllable share
/// the primary weight of its plain hiragana, in syllabary order (code-point order of the plain
/// hiragana; `"ア" < "い"`); voicing is an accent ([`VOICED`]), a small kana is [`SMALL_KANA`] and
/// the kana type a variant (probes/t3275-collation-width-kana Calc!G12:G23, ranks A63:A117).
fn kana(c: char) -> Option<Element> {
    let (hiragana, variant) = match c {
        '\u{3041}'..='\u{3096}' => (c, HIRAGANA),
        '\u{30a1}'..='\u{30f6}' => (char::from_u32(u32::from(c) - 0x60)?, KATAKANA),
        // ヷ ヸ ヹ ヺ: わ ゐ ゑ を voiced.
        '\u{30f7}'..='\u{30fa}' => {
            let base = char::from_u32(u32::from(c) - 0x30f7 + 0x308f)?;
            return Some(Element::Weights {
                primary: [other(base), 0],
                accent: NO_ACCENT + VOICED,
                kana: NORMAL_KANA,
                variant: KATAKANA,
            });
        }
        '\u{ff66}'..='\u{ff9d}' => (
            *HALF_WIDTH_HIRAGANA.get(usize::try_from(u32::from(c) - 0xff66).ok()?)?,
            HALF_WIDTH_KANA,
        ),
        _ => return None,
    };
    let code = u32::from(hiragana);
    // (offset to the plain letter, small, voicing)
    let (offset, small, voicing) = match code {
        0x3041..=0x304a | 0x3083..=0x3088 => (code % 2, code % 2 == 1, 0),
        0x304b..=0x3062 => ((code - 0x304b) % 2, false, (code - 0x304b) % 2),
        0x3063 => (0, true, 0),
        0x3064..=0x3069 => ((code - 0x3064) % 2, false, (code - 0x3064) % 2),
        0x306f..=0x307d => ((code - 0x306f) % 3, false, (code - 0x306f) % 3),
        0x308e => (0, true, 0),
        _ => (0, false, 0),
    };
    let plain = match code {
        // ゔ is う voiced; ゕ ゖ are small か け.
        0x3094 => return Some(voiced_kana(0x3046, 1, variant)),
        0x3095 => 0x304b,
        0x3096 => 0x3051,
        0x3063 => 0x3064,
        0x308e => 0x308f,
        _ if small => code + 1,
        _ => code - offset,
    };
    let small = small || matches!(code, 0x3095 | 0x3096);
    let mut element = voiced_kana(plain, u8::try_from(voicing).ok()?, variant);
    if small {
        if let Element::Weights { kana, .. } = &mut element {
            *kana = SMALL_KANA;
        }
    }
    Some(element)
}

/// The plain hiragana `plain` with `voicing` (0 plain, 1 voiced, 2 semi-voiced) and the variant
/// weight of its kana type.
fn voiced_kana(plain: u32, voicing: u8, variant: u8) -> Element {
    Element::Weights {
        primary: [OTHER_BASE + 2 * plain, 0],
        accent: NO_ACCENT + voicing,
        kana: NORMAL_KANA,
        variant,
    }
}

/// The primary weight of the vowel of the kana weighing `primary` (`あ` for `か`, `い` for `き`),
/// `None` for a letter that is not a kana. A `ー` after `ん` repeats it (unprobed).
fn vowel(primary: u32) -> Option<u32> {
    const VOWELS: [u32; 5] = [0x3042, 0x3044, 0x3046, 0x3048, 0x304a];
    let code = primary.checked_sub(OTHER_BASE)? / 2;
    if primary % 2 != 0 || !(0x3042..=0x3093).contains(&code) {
        return None;
    }
    let index = match code {
        0x3042..=0x304a => (code - 0x3042) / 2,
        0x304b..=0x3062 => (code - 0x304b) / 2 % 5,
        0x3064..=0x3069 => 2 + (code - 0x3064) / 2,
        0x306a..=0x306e => code - 0x306a,
        0x306f..=0x307d => (code - 0x306f) / 3,
        0x307e..=0x3082 => code - 0x307e,
        0x3084..=0x3088 => code - 0x3084,
        0x3089..=0x308d => code - 0x3089,
        0x308f => 0,
        0x3090 => 1,
        0x3091 => 3,
        0x3092 => 4,
        _ => return Some(primary),
    };
    VOWELS
        .get(usize::try_from(index).ok()?)
        .map(|&vowel| OTHER_BASE + 2 * vowel)
}

/// The first pass: letter weights, hyphens, apostrophes and combining marks skipped.
fn primaries(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u32> {
    elements
        .flat_map(|element| match element {
            Element::Skipped(_) | Element::Mark(_) => [0, 0],
            Element::Weights { primary, .. } => primary,
        })
        .filter(|&weight| weight != 0)
}

/// One unit of the accent pass: a letter's own diacritic weight (`letter`), or the weight of a
/// combining mark after it.
#[derive(Clone, Copy)]
struct AccentUnit {
    weight: u8,
    letter: bool,
}

/// The accent units of `text`, letter by letter: one letter unit per letter weight of the first
/// pass (an expansion such as `æ` is two, its accent then an unaccented one), each followed by
/// the units of the combining marks after it. Marks with no letter before them follow an
/// unaccented letter unit of their own (unprobed).
fn accent_units(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = AccentUnit> {
    let letter = |weight| AccentUnit {
        weight,
        letter: true,
    };
    let mut started = false;
    elements
        .flat_map(move |element| match element {
            Element::Skipped(_) => [None, None],
            Element::Mark(weight) => {
                let mark = Some(AccentUnit {
                    weight,
                    letter: false,
                });
                if std::mem::replace(&mut started, true) {
                    [mark, None]
                } else {
                    [Some(letter(NO_ACCENT)), mark]
                }
            }
            Element::Weights {
                primary, accent, ..
            } => {
                started = true;
                let second = (primary[1] != 0).then(|| letter(NO_ACCENT));
                [Some(letter(accent)), second]
            }
        })
        .flatten()
}

/// The next unit of the letter whose units `units` is reading, if any is left.
fn next_mark(units: &mut Peekable<impl Iterator<Item = AccentUnit>>) -> Option<u8> {
    units.next_if(|unit| !unit.letter).map(|unit| unit.weight)
}

/// `first` plus the units left of the letter `units` is reading, in a wrapping byte.
fn letter_sum(first: u8, units: &mut Peekable<impl Iterator<Item = AccentUnit>>) -> u8 {
    std::iter::from_fn(|| next_mark(units)).fold(first, u8::wrapping_add)
}

/// The second pass: one accent weight per letter weight of the first pass, the sum of its
/// [`accent_units`] in one wrapping byte, like the Windows sort key's one-byte diacritic weight.
/// So the order of the marks does not matter:
/// `"a"&UNICHAR(769)&UNICHAR(780)` = `"a"&UNICHAR(780)&UNICHAR(769)` = `"á"&UNICHAR(780)` =
/// `"ǎ"&UNICHAR(769)` (2 + 12 + 18), and three U+0323 (2 + 3 * 86 = 260, so 4) sort between `"a"`
/// and `"á"` (probes/t3154-collation-accent-weights, probes/t3154-collation-accent-pairs).
fn accents(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u8> {
    let mut units = accent_units(elements).peekable();
    std::iter::from_fn(move || {
        let first = units.next()?.weight;
        Some(letter_sum(first, &mut units))
    })
}

/// Excel's accent pass between two texts (coercion.md "Shared leading units are skipped before
/// the sum", probes/t3201-collation-long-mark-runs, probes/t3201-collation-wrapped-run-pairs):
/// letter by letter, drop the leading units both letters share with equal weights; a letter
/// with no units left is less (so a text is below any text it is a proper prefix of,
/// probes/t3154-collation-accent-pairs Calc!S19, T18); otherwise the sums of the units left, each
/// in a wrapping byte, decide. Texts equal here have equal [`accents`].
fn cmp_accent_units(
    left: impl Iterator<Item = Element>,
    right: impl Iterator<Item = Element>,
) -> Ordering {
    let (mut left, mut right) = (
        accent_units(left).peekable(),
        accent_units(right).peekable(),
    );
    loop {
        let (mut l, mut r) = match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(l), Some(r)) => (Some(l.weight), Some(r.weight)),
        };
        while l.is_some() && l == r {
            (l, r) = (next_mark(&mut left), next_mark(&mut right));
        }
        let order = match (l, r) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(l), Some(r)) => letter_sum(l, &mut left).cmp(&letter_sum(r, &mut right)),
        };
        if order.is_ne() {
            return order;
        }
    }
}

/// The kana pass, after the accents: one weight per letter weight of the first pass,
/// [`NORMAL_KANA`] but for small kana and the long-vowel and iteration marks.
fn kana_weights(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u8> {
    letter_weights(elements, |kana, _| kana, NORMAL_KANA)
}

/// The variant pass, after the kana pass: one weight per letter weight of the first pass,
/// [`SUPERSCRIPT`] for `ª º ¹ ² ³`, [`FULL_WIDTH`] for the full-width forms, the kana type of a
/// kana, and 0 otherwise.
fn variants(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = u8> {
    letter_weights(elements, |_, variant| variant, 0)
}

/// One weight per letter weight of the first pass: `pick` of the letter's kana and variant
/// weights, and `second` for the second letter of an expansion.
fn letter_weights(
    elements: impl Iterator<Item = Element>,
    pick: impl Fn(u8, u8) -> u8,
    second: u8,
) -> impl Iterator<Item = u8> {
    elements
        .flat_map(move |element| match element {
            Element::Weights {
                primary,
                kana,
                variant,
                ..
            } => [
                Some(pick(kana, variant)),
                (primary[1] != 0).then_some(second),
            ],
            Element::Skipped(_) | Element::Mark(_) => [None, None],
        })
        .flatten()
}

/// The last pass: each hyphen or apostrophe as (position, weight), a later position first.
/// Positions count letters and skipped characters, not combining marks, so a decomposed letter
/// keeps the position of its precomposed form.
fn skipped(elements: impl Iterator<Item = Element>) -> impl Iterator<Item = (Reverse<usize>, u8)> {
    elements
        .filter(|element| !matches!(element, Element::Mark(_)))
        .enumerate()
        .filter_map(|(position, element)| match element {
            Element::Skipped(weight) => Some((Reverse(position), weight)),
            Element::Weights { .. } | Element::Mark(_) => None,
        })
}
