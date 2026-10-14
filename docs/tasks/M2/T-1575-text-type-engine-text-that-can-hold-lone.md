---
id: T-1575
title: "Text type: engine text that can hold lone surrogates (SPEC-0010 step 1)"
milestone: M2
lane: engine
status: todo
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
- [ ] `text.rs` with the API in the spec, doc comments citing probes/text-surrogate-positions and probes/concatenate-limit-spill.
- [ ] Unit and property tests against a `Vec<u16>` model: round trip, `slice_utf16`, `concat` (lone high + lone low becomes the pair: a hand expectation from UTF-16, pending T-1582's probe of `EXACT(MID("😀",1,1)&MID("😀",2,1),"😀")`), `truncate_utf16`, `chars_lossless`, normalization invariant (`Utf8` iff well-formed).
- [ ] Tests assert `size_of::<Text>() == size_of::<String>()` and that an `ArrayNode`-shaped enum carrying `Text` is 24 bytes.
- [ ] Hand-written bitcode `Encode`/`Decode`: `Lone` serializes its units only and decodes through `from_utf16`; a test that crafted bytes (a `Lone` with well-formed units, a truncated buffer) decode to a normalized `Text` or an `Err`, never a panic.
- [ ] `push(&mut self, &Text)` appends in amortized linear time (stays `Utf8` until a lone unit arrives).
- [ ] `crates/rustsheet/benches/text.rs` (recalc of `=A1&"x"`, `=MID(A1,2,3)`, `=LEFT(A1,2)` over a large text column, and a many-argument CONCATENATE near 32,767 units); its main numbers recorded in the Outcome as the baseline for T-1576 and T-1580.
- [ ] `Text` is public and documented; the Python binding surface is unchanged.
- [ ] No `unwrap`/`expect`/`panic`; no new dependency.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `CalcResult`, `ArrayNode` or any function (T-1576 onward).
