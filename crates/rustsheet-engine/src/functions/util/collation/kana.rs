//! Kana weights for the collation (probes/t3275-collation-width-kana; docs/excel-semantics/coercion.md
//! "Full-width and half-width forms, kana, voicing and the long-vowel mark").

use super::{
    other, Element, HALF_WIDTH_KANA, HIRAGANA, KATAKANA, NORMAL_KANA, NO_ACCENT, OTHER_BASE,
    SMALL_KANA, VOICED,
};

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
pub(super) fn kana(c: char) -> Option<Element> {
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
pub(super) fn vowel(primary: u32) -> Option<u32> {
    const VOWELS: [u32; 5] = [0x3042, 0x3044, 0x3046, 0x3048, 0x304a];
    let code = primary.checked_sub(OTHER_BASE)? / 2;
    if !primary.is_multiple_of(2) || !(0x3042..=0x3093).contains(&code) {
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
