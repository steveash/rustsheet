---
id: T-1579
title: "MID, MIDB, REPLACE, REPLACEB keep the lone half of a cut surrogate pair (SPEC-0010 step 5)"
milestone: M2
lane: functions-text
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: [T-1576]
---

## Goal
MID, MIDB, REPLACE and REPLACEB keep the lone half when a position splits a surrogate pair, as Excel does.

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md; docs/excel-semantics/text.md "MID and REPLACE positions inside a surrogate pair keep the lone half". Replace `utf16_slice` (functions/text/util.rs) by `Text::slice_utf16` in `mid_element` (common.rs) and REPLACE (string_format.rs); delete `utf16_slice`. Failing cells: probes/text-surrogate-positions A11, A12, A29, B10 (A6 also needs T-1580; A38 needs T-1581).

## Acceptance criteria
- [ ] A11, A12, A29, B10 pass and leave eval/baseline; no eval regressions.
- [ ] text.md entry updated ("Ours" line: implemented; known-gap note removed for these cells).
- [ ] Outcome: `crates/rustsheet/benches/text.rs` before/after (release), within ±3%; the `Utf8` paths stay today's code (no UTF-16 conversion pass).
- [ ] `make check` green; reviewers approve.

## Out of scope
- `&`/truncation (T-1580), UNICODE/LEFT/RIGHT (T-1581).
