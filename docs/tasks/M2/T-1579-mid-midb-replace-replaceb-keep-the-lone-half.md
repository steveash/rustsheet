---
id: T-1579
title: "MID, MIDB, REPLACE, REPLACEB keep the lone half of a cut surrogate pair (SPEC-0010 step 5)"
milestone: M2
lane: functions-text
status: in-progress
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
- [ ] A11, A12, A29, B10 pass and leave eval/baseline; no eval regressions.
- [ ] text.md entry updated ("Ours" line: implemented; known-gap note removed for these cells).
- [ ] Outcome: `crates/rustsheet/benches/text.rs` before/after (release), within ±3%; the `Utf8` paths stay today's code (no UTF-16 conversion pass).
- [ ] `make check` green; reviewers approve.

## Out of scope
- `&`/truncation (T-1580), UNICODE/LEFT/RIGHT (T-1581).

## Handoff (resumed)
- Implemented on this branch: `mid_element` (functions/text/common.rs) and `replace_element` (string_format.rs) now read their text with `array_node_to_text` and use `Text::slice_utf16` / `Text::push`; `utf16_slice` is deleted from functions/text/util.rs. `cargo clippy -p rustsheet_engine --all-targets` is clean.
- Result: probes/text-surrogate-positions now 67/68 (only A38 left, which T-1581 owns); `make eval-baseline` fixes 39 cells in probes.
- Blocker: `make eval-baseline` refuses with 10 regressions, all in probes/text-lone-surrogates.case.json: A3, A10, A11, A14, A15, A27, A28, A55, A56, A57. They passed only by accident, because MID of half a pair used to return "" and now returns a lone half that UNICODE (A3), EXACT (A10, A11), `=` (A14, A15), MATCH (A27, A28) and FIND/SEARCH (A55-A57) still read through the lossy U+FFFD view. Their owners are T-1581 (UNICODE), T-1585 (compare/EXACT/MATCH) and T-1586 (FIND/SEARCH/SUBSTITUTE).
- Next: write a decision record (`make new-decision`, modelled on decisions 0034 and 0037, "allow regressions unmasked by ...") naming those 10 cells and their owner tasks, run `make eval-baseline --allow-regressions` through the documented route in docs/architecture/eval-harness.md, then update text.md ("Ours" line, known-gap note for A11, A12, A29, B10), and run the text bench (`crates/rustsheet/benches/text.rs`) before/after for the Outcome. Alternative: land T-1581/T-1585/T-1586 first, which avoids the record.
- I did not run `make check`, the bench, or the reviewers.
