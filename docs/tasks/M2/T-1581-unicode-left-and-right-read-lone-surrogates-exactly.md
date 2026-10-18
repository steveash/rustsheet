---
id: T-1581
title: "UNICODE, LEFT and RIGHT read lone surrogates exactly (SPEC-0010 step 7)"
milestone: M2
lane: functions-text
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-1579, T-1580]
---

## Goal
UNICODE reads a lone surrogate as Excel does (lone high `#VALUE!`, lone low its code), and LEFT/RIGHT count a lone surrogate as one character without losing it.

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md; docs/excel-semantics/text.md "MID and REPLACE positions…" (A38, B6, B7) and coercion.md "Truncation inside a surrogate pair…" (E10). `fn_unicode` (functions/text/common.rs) and `left_element`/`right_element` move to `Text::chars_lossless`. Failing cells: probes/text-surrogate-positions A38, probes/concatenate-limit-spill E10; B6, B7 must keep passing.

## Acceptance criteria
- [ ] probes/text-lone-surrogates A3 and A4 pass (UNICODE of a lone high surrogate is #VALUE!, of a lone low one 56832; A3 regressed in T-1579, decision 0061).
- [ ] A38 and E10 pass and leave eval/baseline; B6, B7 still pass; no eval regressions.
- [ ] Unit test: `LEFT(MID("a😀b",3,2),1)` keeps the lone U+DE00 (not U+FFFD).
- [ ] Outcome: `crates/rustsheet/benches/text.rs` before/after (release), within ±3%; the `Utf8` paths stay today's code (no UTF-16 conversion pass).
- [ ] `make check` green; reviewers approve.

## Out of scope
- CODE of a lone surrogate (unprobed; T-1582).

## Outcome
`Text::left_chars` / `Text::right_chars` (text.rs) back LEFT and RIGHT; the `Utf8` arms are the
previous `chars().take/skip` code, the `Lone` arm walks `chars_lossless` and rebuilds through
`from_utf16`. `fn_unicode` reads `chars_lossless().next()`: a lone high surrogate is `#VALUE!`, a
lone low one its code. Eval: probes/text-lone-surrogates A3, A4, probes/text-surrogate-positions
A38 and probes/concatenate-limit-spill E10 left `eval/baseline/probes.txt` (4 fewer known
failures); B6, B7 still pass; no regressions. Bench (release, `benches/text.rs`, min of 11):
LEFT(A1,2) 27.59 ms before, 25.66 ms after; `&`, MID, CONCATENATE and `Text::push` within noise
(24.9/23.3, 22.5/22.5, 43.3/40.8, 56.8/53.7 ms before/after).
