---
id: T-1575
title: "Text type: engine text that can hold lone surrogates (SPEC-0010 step 1)"
milestone: M2
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1483]
---

## Goal
The engine has a `Text` value type that holds any UTF-16 code-unit sequence, lone surrogates included, at no cost for well-formed text.

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md "Representation". New public module `crates/rustsheet-engine/src/text.rs`: `enum Text { Utf8(String), Lone(Box<LoneText>) }` with the invariant `Lone` iff ill-formed UTF-16, `Deref<Target = str>` as the lossy view (lone unit = U+FFFD), `from_utf16`, `len_utf16`, `has_lone_surrogate`, `units`, `slice_utf16` (keeps the lone half of a cut pair), `chars_lossless`, `concat`, `truncate_utf16` (keeps a cut pair's high half), `segments`, `into_lossy_string`; bitcode `Encode`/`Decode`. Not wired into any value type yet (T-1576 does that).

## Acceptance criteria
- [x] `text.rs` with the API in the spec, doc comments citing probes/text-surrogate-positions and probes/concatenate-limit-spill.
- [x] Unit and property tests against a `Vec<u16>` model: round trip, `slice_utf16`, `concat` (lone high + lone low becomes the pair: a hand expectation from UTF-16, pending T-1582's probe of `EXACT(MID("😀",1,1)&MID("😀",2,1),"😀")`), `truncate_utf16`, `chars_lossless`, normalization invariant (`Utf8` iff well-formed).
- [x] Tests assert `size_of::<Text>() == size_of::<String>()` and that an `ArrayNode`-shaped enum carrying `Text` is 24 bytes.
- [x] Hand-written bitcode `Encode`/`Decode`: `Lone` serializes its units only and decodes through `from_utf16`; a test that crafted bytes (a `Lone` with well-formed units, a truncated buffer) decode to a normalized `Text` or an `Err`, never a panic.
- [x] `push(&mut self, &Text)` appends in amortized linear time (stays `Utf8` until a lone unit arrives; on the `Lone` path grows a `Vec<u16>` and the lossy `String`, frozen once); a test or bench case with a lone early argument followed by many more.
- [x] `crates/rustsheet/benches/text.rs` (recalc of `=A1&"x"`, `=MID(A1,2,3)`, `=LEFT(A1,2)` over a large text column, and a many-argument CONCATENATE near 32,767 units); its main numbers recorded in the Outcome as the baseline for T-1576 and T-1580.
- [x] `Text` is public and documented; the Python binding surface is unchanged.
- [x] No `unwrap`/`expect`/`panic`; no new dependency.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `CalcResult`, `ArrayNode` or any function (T-1576 onward).

## Outcome
`crates/rustsheet-engine/src/text.rs`: public `Text` (`Utf8(String)` | `Lone(Box<LoneText>)`,
re-exported as `rustsheet_engine::Text`) with the SPEC-0010 API: `from_utf16`, `From<String>`/
`From<&str>`, `Deref<Target = str>` (lossy view), `len_utf16`, `has_lone_surrogate`, `units`,
`slice_utf16`, `chars_lossless`, `concat`, `push`, `truncate_utf16`, `segments`,
`into_lossy_string`. `LoneText` keeps its units in a `Vec<u16>` plus a lone-unit count, so `push`
appends in place (amortized linear) and knows without a scan when pairing a lone high with a lone
low surrogate makes the text well-formed again. Encode/Decode are hand-written over bitcode's
`__private` coder traits (bitcode has no `with` attribute): `Lone` is encoded as its units only
and decoded through `from_utf16`. Not wired into any value type (T-1576).

Tests (`src/test/test_text.rs`): seeded property tests against a `Vec<u16>` model (round trip,
`slice_utf16`, `concat`/`push`, many pushes, `truncate_utf16`, `chars_lossless`, `segments`,
bitcode round trip, normalization), the probe cells' shapes (A11, A12, B10, E8-E10), lone high +
lone low → the pair, 100,000 pushes after a lone unit, crafted/truncated/random bytes → normalized
`Text` or `Err`, `size_of::<Text>() == size_of::<String>()` and a 24-byte `ArrayNode`-shaped enum.

Bench baseline (`cargo bench -p rustsheet --bench text`, release, median of 11, three runs on the
shared CI-class container; spread is ±15% run to run, so T-1576/T-1580 should compare several
runs): `A1&"x"` over 100,000 rows 24-26 ms; `MID(A1,2,3)` 23-29 ms; `LEFT(A1,2)` 34-52 ms;
1,000 × `CONCATENATE` of 255 × 130-unit cells (truncated to 32,767) 73-78 ms; `Text::push` of
1,000 such lists starting with a lone unit 55-56 ms. Eval delta: none (not wired).
