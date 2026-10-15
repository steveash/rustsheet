#![allow(clippy::unwrap_used)]

//! `Text` against a `Vec<u16>` model of Excel text (SPEC-0010).

use bitcode::{Decode, Encode};

use crate::expressions::token::Error;
use crate::test::util::Rng;
use crate::text::{Char, Segment, Text};

const HIGH: u16 = 0xD83D;
const LOW: u16 = 0xDE00;

/// "a😀b" as UTF-16: `a`, the pair D83D DE00, `b`.
fn a_emoji_b() -> Text {
    Text::from("a😀b")
}

/// Random units, mostly well-formed: ASCII, a 2-byte, a 3-byte and a 4-byte (pair) character,
/// U+FFFD, and now and then a lone high or low surrogate.
fn random_units(rng: &mut Rng) -> Vec<u16> {
    let len = rng.below(10);
    let mut units = Vec::new();
    for _ in 0..len {
        match rng.below(10) {
            0 => units.push(HIGH),
            1 => units.push(LOW),
            2 => units.push(0xFFFD),
            3 => units.push(0xE9),
            4 => units.push(0x20AC),
            5 | 6 => units.extend([HIGH, LOW]),
            _ => units.push(u16::from(b'a') + rng.below(3) as u16),
        }
    }
    units
}

/// Every property a `Text` must keep for the units `model`.
fn assert_matches_model(text: &Text, model: &[u16]) {
    assert_eq!(text.units().collect::<Vec<u16>>(), model);
    assert_eq!(text.len_utf16(), model.len());
    assert_eq!(&**text, String::from_utf16_lossy(model));
    // Normalization: `Utf8` iff well-formed.
    assert_eq!(
        text.has_lone_surrogate(),
        String::from_utf16(model).is_err()
    );
    assert_eq!(matches!(text, Text::Utf8(_)), !text.has_lone_surrogate());
    let chars: Vec<Char> = text.chars_lossless().collect();
    let expected: Vec<Char> = char::decode_utf16(model.iter().copied())
        .map(|d| match d {
            Ok(ch) => Char::Scalar(ch),
            Err(e) => Char::Lone(e.unpaired_surrogate()),
        })
        .collect();
    assert_eq!(chars, expected);
    let mut from_segments = Vec::new();
    let mut previous_str = false;
    for segment in text.segments() {
        match segment {
            Segment::Str(s) => {
                assert!(!s.is_empty());
                assert!(!previous_str, "adjacent Str runs are merged");
                from_segments.extend(s.encode_utf16());
                previous_str = true;
            }
            Segment::Lone(unit) => {
                from_segments.push(unit);
                previous_str = false;
            }
        }
    }
    assert_eq!(from_segments, model);
    assert_eq!(
        &bitcode::decode::<Text>(&bitcode::encode(text)).unwrap(),
        text
    );
    // Equal units, equal values: the same units through the other constructor.
    assert_eq!(&Text::from_utf16(model.to_vec()), text);
}

#[test]
fn model_round_trip() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..5_000 {
        let units = random_units(&mut rng);
        assert_matches_model(&Text::from_utf16(units.clone()), &units);
    }
}

#[test]
fn model_slice_utf16() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..5_000 {
        let units = random_units(&mut rng);
        let text = Text::from_utf16(units.clone());
        let start = rng.below(12) as usize;
        let end = rng.below(12) as usize;
        let e = end.min(units.len());
        let s = start.min(e);
        assert_matches_model(&text.slice_utf16(start, end), &units[s..e]);
    }
}

#[test]
fn model_concat_and_push() {
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    for _ in 0..5_000 {
        let a = random_units(&mut rng);
        let b = random_units(&mut rng);
        let joined: Vec<u16> = a.iter().chain(&b).copied().collect();
        let (ta, tb) = (Text::from_utf16(a), Text::from_utf16(b));
        assert_matches_model(&Text::concat(&ta, &tb), &joined);
        let mut pushed = ta.clone();
        pushed.push(&tb);
        assert_matches_model(&pushed, &joined);
    }
}

#[test]
fn model_push_many() {
    let mut rng = Rng(0xA076_1D64_78BD_642F);
    for _ in 0..500 {
        let mut text = Text::default();
        let mut model = Vec::new();
        for _ in 0..rng.below(8) {
            let part = random_units(&mut rng);
            text.push(&Text::from_utf16(part.clone()));
            model.extend(part);
        }
        assert_matches_model(&text, &model);
    }
}

#[test]
fn model_truncate_utf16() {
    let mut rng = Rng(0xE703_7ED1_A0B4_28DB);
    for _ in 0..5_000 {
        let units = random_units(&mut rng);
        let max = rng.below(12) as usize;
        let mut text = Text::from_utf16(units.clone());
        text.truncate_utf16(max);
        assert_matches_model(&text, &units[..max.min(units.len())]);
    }
}

#[test]
fn mid_keeps_the_lone_half() {
    // probes/text-surrogate-positions: MID("a😀b",3,2) is lone DE00 then "b" (A11 = 2),
    // MID("a😀b",2,1) is lone D83D (A12 = 1), MID("😀",2,5) is lone DE00 (B10 = 1).
    assert_eq!(
        a_emoji_b().slice_utf16(2, 4).units().collect::<Vec<_>>(),
        [LOW, u16::from(b'b')]
    );
    assert_eq!(a_emoji_b().slice_utf16(1, 2).len_utf16(), 1);
    assert_eq!(Text::from("😀").slice_utf16(1, 6).len_utf16(), 1);
    // A cut that keeps the pair whole stays well-formed.
    assert_eq!(a_emoji_b().slice_utf16(1, 3), Text::from("😀"));
    // The lossy view shows U+FFFD for the lone unit.
    assert_eq!(&*a_emoji_b().slice_utf16(2, 4), "\u{FFFD}b");
}

#[test]
fn lone_high_then_lone_low_is_the_pair() {
    // A hand expectation from UTF-16 until T-1582 probes
    // EXACT(MID("😀",1,1)&MID("😀",2,1),"😀").
    let emoji = Text::from("😀");
    let joined = Text::concat(&emoji.slice_utf16(0, 1), &emoji.slice_utf16(1, 2));
    assert_eq!(joined, emoji);
    assert!(!joined.has_lone_surrogate());
    // With other lone units around, the joined text stays `Lone` but its lossy view pairs.
    let left = Text::from_utf16(vec![LOW, u16::from(b'x'), HIGH]);
    let right = Text::from_utf16(vec![LOW, HIGH]);
    let joined = Text::concat(&left, &right);
    assert_eq!(&*joined, "\u{FFFD}x😀\u{FFFD}");
    assert!(joined.has_lone_surrogate());
}

#[test]
fn truncate_keeps_the_high_half() {
    // probes/concatenate-limit-spill E9: REPT("a",32766)&"😀" is 32,767 units, the last a lone
    // high surrogate (E10: UNICODE of it is #VALUE!).
    let mut text = Text::from("a".repeat(32766) + "😀");
    text.truncate_utf16(32767);
    assert_eq!(text.len_utf16(), 32767);
    assert_eq!(text.chars_lossless().last(), Some(Char::Lone(HIGH)));
    // E8: REPT("😀",16383)&"😀" keeps 16,383 pairs and the 16,384th's high half.
    let mut text = Text::from("😀".repeat(16384));
    text.truncate_utf16(32767);
    assert_eq!(text.len_utf16(), 32767);
    assert_eq!(text.units().last(), Some(HIGH));
    // Short text is untouched.
    let mut text = Text::from("abc");
    text.truncate_utf16(32767);
    assert_eq!(text, Text::from("abc"));
}

#[test]
fn push_after_a_lone_unit_is_linear() {
    // A lone first argument, then many more (CONCATENATE's argument loop): each push appends in
    // place, so the total work is linear; a quadratic push would copy ~10^10 units here.
    let mut text = Text::from("😀").slice_utf16(1, 2);
    let part = Text::from("x".repeat(10));
    for _ in 0..100_000 {
        text.push(&part);
    }
    assert!(text.has_lone_surrogate());
    assert_eq!(text.len_utf16(), 1 + 1_000_000);
    assert_eq!(text.chars_lossless().next(), Some(Char::Lone(LOW)));
}

#[test]
fn segments_split_at_lone_units() {
    let text = Text::concat(
        &a_emoji_b().slice_utf16(0, 2),
        &Text::from_utf16(vec![u16::from(b'c'), LOW, LOW]),
    );
    assert_eq!(
        text.segments().collect::<Vec<_>>(),
        [
            Segment::Str("a"),
            Segment::Lone(HIGH),
            Segment::Str("c"),
            Segment::Lone(LOW),
            Segment::Lone(LOW),
        ]
    );
    assert_eq!(
        Text::from("a😀b").segments().collect::<Vec<_>>(),
        [Segment::Str("a😀b")]
    );
    assert_eq!(Text::default().segments().count(), 0);
}

#[test]
fn lone_unit_is_not_a_replacement_character() {
    let lone = Text::from_utf16(vec![LOW]);
    let replacement = Text::from("\u{FFFD}");
    assert_eq!(&*lone, &*replacement);
    assert_ne!(lone, replacement);
    assert_eq!(lone.clone().into_lossy_string(), "\u{FFFD}");
}

#[test]
fn sizes() {
    // SPEC-0010 "Representation": `Text` costs no more than `String`, and an `ArrayNode`-shaped
    // enum carrying it stays 24 bytes.
    #[allow(dead_code)]
    enum ArrayNodeShape {
        Boolean(bool),
        Number(f64),
        String(Text),
        Error(Error),
        Empty,
    }
    assert_eq!(size_of::<Text>(), size_of::<String>());
    assert_eq!(size_of::<ArrayNodeShape>(), 24);
}

/// The serialized shape of a `Text`, written independently of the engine's private one.
#[derive(Encode, Decode)]
enum CraftedText {
    Utf8(String),
    Lone(Vec<u16>),
}

#[test]
fn decode_normalizes_crafted_bytes() {
    // A `Lone` holding well-formed units decodes to the `Utf8` form.
    let bytes = bitcode::encode(&CraftedText::Lone("a😀".encode_utf16().collect()));
    assert_eq!(bitcode::decode::<Text>(&bytes).unwrap(), Text::from("a😀"));
    // A `Lone` holding a lone unit decodes with a lossy view that agrees with it.
    let bytes = bitcode::encode(&CraftedText::Lone(vec![u16::from(b'a'), LOW]));
    let text = bitcode::decode::<Text>(&bytes).unwrap();
    assert!(text.has_lone_surrogate());
    assert_eq!(&*text, "a\u{FFFD}");
    let bytes = bitcode::encode(&CraftedText::Utf8("x".to_string()));
    assert_eq!(bitcode::decode::<Text>(&bytes).unwrap(), Text::from("x"));
    // The encoder writes exactly that shape.
    assert_eq!(bitcode::encode(&Text::from("x")), bytes);
    let lone = vec![u16::from(b'a'), LOW];
    assert_eq!(
        bitcode::encode(&Text::from_utf16(lone.clone())),
        bitcode::encode(&CraftedText::Lone(lone))
    );
}

#[test]
fn decode_rejects_bad_bytes_without_panicking() {
    let bytes = bitcode::encode(&vec![
        Text::from_utf16(vec![u16::from(b'a'), LOW, HIGH]),
        Text::from("hello"),
    ]);
    for cut in 0..bytes.len() {
        assert!(bitcode::decode::<Vec<Text>>(&bytes[..cut]).is_err());
    }
    let mut rng = Rng(0x1234_5678_9ABC_DEF1);
    for _ in 0..2_000 {
        let garbage: Vec<u8> = (0..rng.below(24)).map(|_| rng.next() as u8).collect();
        // Any outcome but a panic; a decoded value is normalized.
        if let Ok(texts) = bitcode::decode::<Vec<Text>>(&garbage) {
            for text in texts {
                let units: Vec<u16> = text.units().collect();
                assert_matches_model(&text, &units);
            }
        }
    }
}

/// Evaluates `formula` in A1 and returns its text.
fn eval_text(formula: &str) -> String {
    let mut model = crate::test::util::new_empty_model();
    model._set("A1", formula);
    model.evaluate();
    model._get_text("A1")
}

#[test]
fn rept_and_text_keep_lone_units() {
    // Hand expectations (UTF-16 model; the oracle cells A62-A80 read lengths only), with H = MID("😀",1,1), L = MID("😀",2,1): REPT
    // repeats the units, TEXT puts them where the format has `@`; a lone unit is never U+FFFD.
    let l = "MID(\"😀\",2,1)";
    let h = "MID(\"😀\",1,1)";
    assert_eq!(eval_text(&format!("=UNICODE(REPT({l},2))")), "56832");
    assert_eq!(eval_text(&format!("=EXACT(REPT({l},2),{l}&{l})")), "TRUE");
    // Repeating "low, high" re-pairs at each join, as `&` does.
    assert_eq!(
        eval_text(&format!("=EXACT(REPT({l}&{h},2),{l}&{h}&{l}&{h})")),
        "TRUE"
    );
    assert_eq!(eval_text(&format!("=LEN(REPT({h},3))")), "3");
    assert_eq!(eval_text(&format!("=UNICODE(T({l}))")), "56832");
    assert_eq!(eval_text(&format!("=UNICODE(TEXT({l},\"@\"))")), "56832");
    assert_eq!(
        eval_text(&format!("=EXACT(TEXT({h},\"@@\"),{h}&{h})")),
        "TRUE"
    );
    assert_eq!(
        eval_text(&format!("=UNICODE(MID(TEXT({l},\"x@y\"),2,1))")),
        "56832"
    );
    // A format holding U+E000 cannot confuse the lone unit with a literal.
    assert_eq!(
        eval_text(&format!(
            "=UNICODE(RIGHT(TEXT({l},\"@\"\"\u{e000}\"\"\"),1))"
        )),
        "57344"
    );
    assert_eq!(
        eval_text(&format!(
            "=UNICODE(LEFT(TEXT({l},\"@\"\"\u{e000}\"\"\"),1))"
        )),
        "56832"
    );
    // The 255-character limit sees the real text, not the one-character stand-in.
    assert_eq!(
        eval_text(&format!("=TEXT({h}&REPT(\"a\",127),\"@@\")")),
        "#VALUE!"
    );
    assert_eq!(
        eval_text(&format!("=LEN(TEXT({h}&REPT(\"a\",126),\"@@\"))")),
        "254"
    );
}
