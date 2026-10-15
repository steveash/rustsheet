---
id: T-1579
title: "MID, MIDB, REPLACE, REPLACEB keep the lone half of a cut surrogate pair (SPEC-0010 step 5)"
milestone: M2
lane: functions-text
status: done
priority: P1
size: S
tier: opus
depends_on: [T-1576]
---

## Goal
MID, MIDB, REPLACE and REPLACEB keep the lone half when a position splits a surrogate pair, as Excel does.

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md; docs/excel-semantics/text.md "MID and REPLACE positions inside a surrogate pair keep the lone half". Replace `utf16_slice` (functions/text/util.rs) by `Text::slice_utf16` in `mid_element` (common.rs) and REPLACE (string_format.rs); delete `utf16_slice`. Failing cells: probes/text-surrogate-positions A11, A12, A29, B10 (A6 also needs T-1580; A38 needs T-1581).

## Acceptance criteria
- [x] A11, A12, A29, B10 pass and leave eval/baseline; no eval regressions.
- [x] text.md entry updated ("Ours" line: implemented; known-gap note removed for these cells).
- [x] Outcome: `crates/rustsheet/benches/text.rs` before/after (release), within ±3%; the `Utf8` paths stay today's code (no UTF-16 conversion pass).
- [x] `make check` green; reviewers approve.

## Out of scope
- `&`/truncation (T-1580), UNICODE/LEFT/RIGHT (T-1581).

## Outcome
MID/MIDB (`mid_element`) and REPLACE/REPLACEB (`replace_element`) slice a `Text` by UTF-16 units
(`Text::slice_utf16`), keeping the lone half of a cut pair; `utf16_slice` is deleted. Eval: probes
known failures 5481 → 5452 (39 fixed: text-surrogate-positions A6, A11, A12, A29, B10 and 34 of
text-lone-surrogates); 10 text-lone-surrogates cells (A3, A10, A11, A14, A15, A27, A28, A55–A57)
were unmasked and recorded under decision 0061 for T-1581/T-1585/T-1586. text.md entry updated.
Bench (`cargo bench -p rustsheet --bench text`, release, median before → after): `MID(A1,2,3)`
23.66 → 23.00 ms (−2.8%); `A1&"x"` 24.57 → 24.53 ms; `LEFT(A1,2)` 30.92 → 32.43 ms (code
unchanged, run-to-run noise); the `Utf8` path is `slice_utf8`, no UTF-16 conversion pass.
