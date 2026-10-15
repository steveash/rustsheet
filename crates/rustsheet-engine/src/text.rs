//! Excel text: a sequence of UTF-16 code units that may hold a lone (unpaired) surrogate
//! (docs/specs/SPEC-0010-text-with-lone-surrogates.md).
//!
//! Excel keeps the lone half of a surrogate pair that a cut splits: `MID`/`REPLACE` positions
//! inside a pair (docs/excel-semantics/text.md "MID and REPLACE positions inside a surrogate pair
//! keep the lone half"; probes/text-surrogate-positions A6, A11, A12, A29, A38, B10) and the
//! 32,767-unit truncation of `&`/`CONCATENATE` (docs/excel-semantics/coercion.md "Truncation
//! inside a surrogate pair keeps a lone high surrogate"; probes/concatenate-limit-spill E8-E11).
//! A Rust `String` cannot hold such a unit, so [`Text`] stores well-formed text as UTF-8 and only
//! ill-formed text in a boxed exact form.

use std::char::DecodeUtf16;
use std::fmt;
use std::iter::Copied;
use std::ops::Deref;
use std::slice;
use std::str::{Chars, EncodeUtf16};

use bitcode::__private::{Buffer, Decoder, Encoder, Result as CoderResult, View};
use bitcode::{Decode, Encode};

/// Excel text: a sequence of UTF-16 code units. Well-formed text (every surrogate paired) is
/// stored as UTF-8; text holding a lone surrogate also keeps its exact units.
///
/// Invariant, kept by every constructor (the only way to build a `Text`): `Lone` if and only if
/// the units are ill-formed UTF-16. So each value has one form and `==` is value equality: a lone
/// surrogate never equals a real U+FFFD.
///
/// `Deref<Target = str>` is the **lossy view**: each lone unit reads as U+FFFD. Lengths and
/// positions counted in UTF-16 units are the same in both views, since U+FFFD and a lone
/// surrogate are each one unit.
///
/// [`Text::slice_utf16`], [`Text::truncate_utf16`] and [`Text::concat`] supersede the lossy
/// `utf16_slice`, `truncate_text` and `concat_text`, which drop a cut pair whole; the tasks that
/// wire `Text` in (T-1579, T-1580) delete them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Text {
    /// Well-formed text.
    Utf8(String),
    /// Text holding at least one lone surrogate.
    Lone(Box<LoneText>),
}

/// The exact units of a [`Text`] holding a lone surrogate, with its lossy view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoneText {
    /// The exact text; holds at least one lone surrogate. A `Vec` (not a boxed slice) so that
    /// [`Text::push`] appends in amortized linear time.
    units: Vec<u16>,
    /// `String::from_utf16_lossy(&units)`: each lone unit as U+FFFD.
    lossy: String,
    /// How many lone units `units` holds (at least one); lets [`Text::push`] tell, without a
    /// scan, when a join that pairs a lone high with a lone low surrogate leaves the text
    /// well-formed.
    lone_count: usize,
}

/// One character of a [`Text`], read by [`Text::chars_lossless`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Char {
    /// A Unicode scalar value (one unit, or a surrogate pair).
    Scalar(char),
    /// A lone surrogate unit.
    Lone(u16),
}

/// A run of a [`Text`], read by [`Text::segments`]: the boundary writers (xlsx, harness) write
/// `Str` runs as text and each `Lone` unit as an escape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Segment<'a> {
    /// Well-formed text.
    Str(&'a str),
    /// A lone surrogate unit.
    Lone(u16),
}

/// The `_xHHHH_` escape (upper-case hex) Excel writes for a lone surrogate unit; the xlsx writer
/// and the eval harness render lone units with it (SPEC-0010).
pub fn lone_escape(unit: u16) -> String {
    format!("_x{unit:04X}_")
}

const HIGH_SURROGATES: std::ops::RangeInclusive<u16> = 0xD800..=0xDBFF;
const LOW_SURROGATES: std::ops::RangeInclusive<u16> = 0xDC00..=0xDFFF;

/// The UTF-8 length of U+FFFD, which stands for each lone unit in the lossy view.
const REPLACEMENT_LEN: usize = 3;

impl Text {
    /// The text holding exactly `units`, `Utf8` when they are well-formed.
    pub fn from_utf16(units: Vec<u16>) -> Text {
        let mut lossy = String::with_capacity(units.len());
        let mut lone_count = 0;
        for decoded in char::decode_utf16(units.iter().copied()) {
            match decoded {
                Ok(ch) => lossy.push(ch),
                Err(_) => {
                    lossy.push(char::REPLACEMENT_CHARACTER);
                    lone_count += 1;
                }
            }
        }
        if lone_count == 0 {
            Text::Utf8(lossy)
        } else {
            Text::Lone(Box::new(LoneText {
                units,
                lossy,
                lone_count,
            }))
        }
    }

    /// The length in UTF-16 code units, the unit Excel's `LEN` counts.
    pub fn len_utf16(&self) -> usize {
        match self {
            // Bulk ASCII check first: one unit per byte.
            Text::Utf8(s) if s.is_ascii() => s.len(),
            Text::Utf8(s) => s.chars().map(char::len_utf16).sum(),
            Text::Lone(lone) => lone.units.len(),
        }
    }

    /// Whether the text holds a lone surrogate (it is the `Lone` form).
    pub fn has_lone_surrogate(&self) -> bool {
        matches!(self, Text::Lone(_))
    }

    /// The exact UTF-16 code units.
    pub fn units(&self) -> Units<'_> {
        match self {
            Text::Utf8(s) => Units::Utf8(s.encode_utf16()),
            Text::Lone(lone) => Units::Lone(lone.units.iter().copied()),
        }
    }

    /// The units `[start, end)` (clamped to the text), keeping the lone half of a surrogate pair
    /// that either bound cuts, as `MID` and `REPLACE` do (probes/text-surrogate-positions A11,
    /// A12, B10).
    #[must_use]
    pub fn slice_utf16(&self, start: usize, end: usize) -> Text {
        match self {
            Text::Utf8(s) => slice_utf8(s, start, end),
            Text::Lone(lone) => {
                let len = lone.units.len();
                let end = end.min(len);
                let start = start.min(end);
                Text::from_utf16(lone.units[start..end].to_vec())
            }
        }
    }

    /// The characters, a lone surrogate counting as one character of its own (`LEFT`, `RIGHT`,
    /// `UNICODE`; probes/text-surrogate-positions A38, B6, B7).
    pub fn chars_lossless(&self) -> CharsLossless<'_> {
        match self {
            Text::Utf8(s) => CharsLossless::Utf8(s.chars()),
            Text::Lone(lone) => CharsLossless::Lone(char::decode_utf16(lone.units.iter().copied())),
        }
    }

    /// `a` followed by `b`, as unit sequences: a lone high surrogate at the end of `a` and a lone
    /// low one at the start of `b` become the pair (UTF-16's rule; a hand expectation until
    /// T-1582 probes `EXACT(MID("😀",1,1)&MID("😀",2,1),"😀")`). Not truncated: callers apply
    /// [`Text::truncate_utf16`].
    #[must_use]
    pub fn concat(a: &Text, b: &Text) -> Text {
        let mut joined = a.clone();
        joined.push(b);
        joined
    }

    /// Appends `other` in place, in amortized time linear in `other` (so a loop of pushes, as
    /// `CONCATENATE`'s arguments, stays linear even after a lone unit arrives). Pairs a lone high
    /// surrogate at the end with a lone low one at the start of `other`, as [`Text::concat`].
    pub fn push(&mut self, other: &Text) {
        match (&mut *self, other) {
            (Text::Utf8(s), Text::Utf8(o)) => s.push_str(o),
            (Text::Lone(lone), Text::Utf8(o)) => {
                lone.units.extend(o.encode_utf16());
                lone.lossy.push_str(o);
            }
            (Text::Utf8(s), Text::Lone(o)) => {
                // Well-formed text never ends in a lone high surrogate, so nothing pairs.
                let mut units: Vec<u16> = Vec::with_capacity(s.len() + o.units.len());
                units.extend(s.encode_utf16());
                units.extend_from_slice(&o.units);
                let mut lossy = std::mem::take(s);
                lossy.push_str(&o.lossy);
                *self = Text::Lone(Box::new(LoneText {
                    units,
                    lossy,
                    lone_count: o.lone_count,
                }));
            }
            (Text::Lone(lone), Text::Lone(o)) => {
                let pairs = match (lone.units.last(), o.units.first()) {
                    (Some(high), Some(low))
                        if HIGH_SURROGATES.contains(high) && LOW_SURROGATES.contains(low) =>
                    {
                        Some((*high, *low))
                    }
                    _ => None,
                };
                lone.units.extend_from_slice(&o.units);
                match pairs {
                    None => {
                        lone.lossy.push_str(&o.lossy);
                        lone.lone_count += o.lone_count;
                    }
                    Some((high, low)) => {
                        // Both U+FFFD stand-ins become the one paired character.
                        lone.lossy.truncate(lone.lossy.len() - REPLACEMENT_LEN);
                        let paired = char::decode_utf16([high, low])
                            .next()
                            .and_then(Result::ok)
                            .unwrap_or(char::REPLACEMENT_CHARACTER);
                        lone.lossy.push(paired);
                        lone.lossy
                            .push_str(o.lossy.get(REPLACEMENT_LEN..).unwrap_or_default());
                        lone.lone_count = lone.lone_count + o.lone_count - 2;
                        if lone.lone_count == 0 {
                            *self = Text::Utf8(std::mem::take(&mut lone.lossy));
                        }
                    }
                }
            }
        }
    }

    /// Truncates the text to at most `max` UTF-16 units. A surrogate pair cut by the limit keeps
    /// its high half, as Excel's 32,767-unit truncation does (probes/concatenate-limit-spill E8,
    /// E9, E10, E11).
    pub fn truncate_utf16(&mut self, max: usize) {
        match self {
            Text::Utf8(s) => {
                // UTF-8 never takes fewer bytes than UTF-16 takes units, so short text skips the
                // scan.
                if s.len() <= max {
                    return;
                }
                let mut units = 0;
                for (byte, ch) in s.char_indices() {
                    let next = units + ch.len_utf16();
                    if next > max {
                        if units < max {
                            // `ch` is a pair cut after its high half.
                            let mut exact: Vec<u16> = s[..byte].encode_utf16().collect();
                            let mut pair = [0; 2];
                            exact.push(ch.encode_utf16(&mut pair)[0]);
                            *self = Text::from_utf16(exact);
                        } else {
                            s.truncate(byte);
                        }
                        return;
                    }
                    units = next;
                }
            }
            Text::Lone(lone) => {
                if lone.units.len() > max {
                    let mut units = std::mem::take(&mut lone.units);
                    units.truncate(max);
                    *self = Text::from_utf16(units);
                }
            }
        }
    }

    /// The text as runs of well-formed text and lone units, in order.
    pub fn segments(&self) -> Segments<'_> {
        match self {
            Text::Utf8(s) => Segments {
                lossy: s,
                units: &[],
                unit: 0,
                byte: 0,
            },
            Text::Lone(lone) => Segments {
                lossy: &lone.lossy,
                units: &lone.units,
                unit: 0,
                byte: 0,
            },
        }
    }

    /// The lossy view as an owned `String` (each lone unit as U+FFFD), without a copy for
    /// well-formed text.
    pub fn into_lossy_string(self) -> String {
        match self {
            Text::Utf8(s) => s,
            Text::Lone(lone) => lone.lossy,
        }
    }
}

/// [`Text::slice_utf16`] of well-formed text: a plain substring unless a bound cuts a pair.
fn slice_utf8(s: &str, start: usize, end: usize) -> Text {
    if start >= end {
        return Text::default();
    }
    let mut unit = 0;
    let mut first_byte = s.len();
    let mut last_byte = s.len();
    // The low half of a pair cut by `start`, the high half of a pair cut by `end`.
    let mut leading = None;
    let mut trailing = None;
    for (byte, ch) in s.char_indices() {
        if unit >= end {
            last_byte = byte;
            break;
        }
        let next = unit + ch.len_utf16();
        if next > start && first_byte == s.len() {
            first_byte = byte;
            if unit < start {
                let mut pair = [0; 2];
                leading = Some(ch.encode_utf16(&mut pair)[1]);
                first_byte = byte + ch.len_utf8();
            }
        }
        if next > end {
            // Only a pair can overshoot a bound by one unit.
            if unit >= start {
                let mut pair = [0; 2];
                trailing = Some(ch.encode_utf16(&mut pair)[0]);
            }
            last_byte = byte;
            break;
        }
        unit = next;
    }
    let first_byte = first_byte.min(last_byte);
    let middle = &s[first_byte..last_byte];
    if leading.is_none() && trailing.is_none() {
        return Text::Utf8(middle.to_string());
    }
    let mut units = Vec::with_capacity(middle.len() + 2);
    units.extend(leading);
    units.extend(middle.encode_utf16());
    units.extend(trailing);
    Text::from_utf16(units)
}

impl Default for Text {
    fn default() -> Self {
        Text::Utf8(String::new())
    }
}

impl From<String> for Text {
    /// A move, no scan: valid UTF-8 is always well-formed UTF-16.
    fn from(s: String) -> Self {
        Text::Utf8(s)
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Text::Utf8(s.to_string())
    }
}

impl Deref for Text {
    type Target = str;

    /// The lossy view: each lone unit reads as U+FFFD.
    fn deref(&self) -> &str {
        match self {
            Text::Utf8(s) => s,
            Text::Lone(lone) => &lone.lossy,
        }
    }
}

impl fmt::Display for Text {
    /// Writes the lossy view.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

/// The iterator of [`Text::units`].
#[derive(Clone, Debug)]
pub enum Units<'a> {
    #[doc(hidden)]
    Utf8(EncodeUtf16<'a>),
    #[doc(hidden)]
    Lone(Copied<slice::Iter<'a, u16>>),
}

impl Iterator for Units<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self {
            Units::Utf8(it) => it.next(),
            Units::Lone(it) => it.next(),
        }
    }
}

/// The iterator of [`Text::chars_lossless`].
#[derive(Clone, Debug)]
pub enum CharsLossless<'a> {
    #[doc(hidden)]
    Utf8(Chars<'a>),
    #[doc(hidden)]
    Lone(DecodeUtf16<Copied<slice::Iter<'a, u16>>>),
}

impl Iterator for CharsLossless<'_> {
    type Item = Char;

    fn next(&mut self) -> Option<Char> {
        match self {
            CharsLossless::Utf8(it) => it.next().map(Char::Scalar),
            CharsLossless::Lone(it) => it.next().map(|decoded| match decoded {
                Ok(ch) => Char::Scalar(ch),
                Err(e) => Char::Lone(e.unpaired_surrogate()),
            }),
        }
    }
}

/// The iterator of [`Text::segments`]: walks the exact units and the lossy view in step, so each
/// `Str` run borrows the lossy string.
#[derive(Clone, Debug)]
pub struct Segments<'a> {
    lossy: &'a str,
    /// Empty for well-formed text, which is one `Str` run.
    units: &'a [u16],
    unit: usize,
    byte: usize,
}

impl<'a> Iterator for Segments<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Segment<'a>> {
        if self.byte >= self.lossy.len() {
            return None;
        }
        if self.units.is_empty() {
            self.byte = self.lossy.len();
            return Some(Segment::Str(self.lossy));
        }
        let start = self.byte;
        for decoded in char::decode_utf16(self.units[self.unit..].iter().copied()) {
            match decoded {
                Ok(ch) => {
                    self.unit += ch.len_utf16();
                    self.byte += ch.len_utf8();
                }
                Err(e) => {
                    if self.byte > start {
                        break;
                    }
                    self.unit += 1;
                    self.byte += REPLACEMENT_LEN;
                    return Some(Segment::Lone(e.unpaired_surrogate()));
                }
            }
        }
        self.lossy.get(start..self.byte).map(Segment::Str)
    }
}

/// What a [`Text`] is serialized as: `Lone` as its units only, so decoding untrusted bytes
/// (`Model::from_bytes`) rebuilds it through [`Text::from_utf16`] and can never produce a
/// non-normalized `Text` or a lossy view that disagrees with its units.
#[derive(Decode)]
enum TextRepr {
    Utf8(String),
    Lone(Vec<u16>),
}

/// [`TextRepr`] for encoding, borrowing well-formed text so the common case copies nothing
/// first; same variants and field encodings (`&str` as `String`), so the same bytes. bitcode has
/// no `Encode` for `&[u16]`, so the rare `Lone` units are copied.
#[derive(Encode)]
enum TextReprRef<'a> {
    Utf8(&'a str),
    Lone(Vec<u16>),
}

// bitcode 0.6 has no `#[bitcode(with = ...)]`; the coder traits its derive uses are
// `bitcode::__private` (the version is pinned by Cargo.lock). The encoder converts each `Text`
// to a `TextReprRef`, the decoder converts back through `Text::from_utf16`.

impl Encode for Text {
    type Encoder = TextEncoder;
}

impl<'a> Decode<'a> for Text {
    type Decoder = TextDecoder<'a>;
}

/// The bitcode encoder of [`Text`] (implementation detail of its `Encode`).
#[doc(hidden)]
#[derive(Default)]
pub struct TextEncoder(<TextReprRef<'static> as Encode>::Encoder);

impl Buffer for TextEncoder {
    fn collect_into(&mut self, out: &mut Vec<u8>) {
        self.0.collect_into(out);
    }

    fn reserve(&mut self, additional: std::num::NonZeroUsize) {
        self.0.reserve(additional);
    }
}

impl Encoder<Text> for TextEncoder {
    fn encode(&mut self, t: &Text) {
        let repr = match t {
            Text::Utf8(s) => TextReprRef::Utf8(s),
            Text::Lone(lone) => TextReprRef::Lone(lone.units.clone()),
        };
        self.0.encode(&repr);
    }
}

/// The bitcode decoder of [`Text`] (implementation detail of its `Decode`).
#[doc(hidden)]
#[derive(Default)]
pub struct TextDecoder<'a>(<TextRepr as Decode<'a>>::Decoder);

impl<'a> View<'a> for TextDecoder<'a> {
    fn populate(&mut self, input: &mut &'a [u8], length: usize) -> CoderResult<()> {
        self.0.populate(input, length)
    }
}

impl<'a> Decoder<'a, Text> for TextDecoder<'a> {
    fn decode(&mut self) -> Text {
        match self.0.decode() {
            TextRepr::Utf8(s) => Text::Utf8(s),
            TextRepr::Lone(units) => Text::from_utf16(units),
        }
    }
}
