---
id: T-0284
title: "Probe CONCAT/TEXTJOIN range-cell errors past the 32,767 limit"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for what CONCAT and TEXTJOIN return when a range cell holds an error and the text is already over 32,767 characters, so the extrapolated cases in text.md are verified or fixed.

## Context
T-0262 found that `TEXTJOIN(",",TRUE,REPT("a",32767),A1:A2)` with an error in `A2` is `#CALC!` (probes/text-result-limit C20) while an error argument wins. Unprobed: `CONCAT(REPT("a",32767),A1:A2)`; `TEXTJOIN(over-limit, A2:A2)` and `TEXTJOIN(over-limit, A2)` (error in the first cell of a range, single-cell reference); `TEXTJOIN(over-limit, A1:A2, NA())`. Our engine returns `#CALC!` for a first-cell range error and for the trailing `NA()`, and the error for a single-cell reference; `fn_concat` is unchanged and returns the error. Code: concatenate.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cells with oracle values for the four cases; the text.md Unprobed line is resolved; the engine matches (baseline shrinks or a DEVIATION entry).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed with `probes/text-result-limit-errors` (11 cells, excel-graph). The engine differed on four: a range-cell error past the limit is dropped (`#CALC!`) for CONCAT as well as TEXTJOIN, TEXTJOIN counts the delimiter before reading a reference so an error in a first range cell or a single-cell reference is `#CALC!`, and a later error argument still wins. `concatenate.rs` now matches (11/11 pass). text.md Unprobed line resolved.

## Handoff (resumed)
Tried: probed `probes/text-result-limit-errors` (11 cells, excel-graph), fixed `concatenate.rs` (range-cell and reference errors past the limit are dropped; TEXTJOIN counts the delimiter before the cell); the engine passes 11/11, `make check` green, security, architecture and quality approved. The fidelity reviewer requested changes twice; round 1 (probe delimiters/ignore_empty, narrow reference kinds) is done. Open from round 2 (docs and comments only, no behavior change):
- text.md section "CONCAT and TEXTJOIN past 32,767 characters": my scripted replacement of the `Implemented in:` and `Tasks:` lines did not apply. Rewrite `Implemented in` to name `JoinSize::exceeds_with_separator`, the `Node::ReferenceKind` arm in `fn_textjoin` and the drop-and-continue in `fn_concat`; add T-0284 to `Tasks:`.
- Outcome: say the old Unprobed items are resolved and other reference kinds (names, INDEX/IF/OFFSET/INDIRECT, `@`, 3-D) remain unprobed; file a follow-up probe task for them (`make new-task`).
- concatenate.rs: the comment above `let mut over_limit` in `fn_textjoin` still says a range-cell error past the limit is `#CALC!` (C18-C20): say it is dropped and evaluation continues (errors C2-C4). The CONCAT closure comment cites C10 as a range cell error, but C10 is a single-cell reference that reaches it as a range via `evaluate_aggregate_arg`.
Then rerun `reviewer-excel-fidelity`, set `status: done`, merge on green CI.
