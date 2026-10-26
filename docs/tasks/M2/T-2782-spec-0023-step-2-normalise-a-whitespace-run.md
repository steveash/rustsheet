---
id: T-2782
title: "SPEC-0023 step 2: normalise a whitespace run to its line feeds and last spaces, wrapping the space count at 256"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2781]
---

## Goal
A typed whitespace run prints as Excel stores it: only its line feeds and the spaces after the last
one, the space count wrapping at 256 (300 spaces print as 44); the line-feed count as a probe shows.

## Context
- From T-2781: a run holding a line feed prints nothing yet (`Run::of`); probes/t2769-formula-text-whitespace
  C17 and t2776-formula-text-whitespace-shapes C13 (a line feed after `=`) are this task's too.
SPEC-0023 step 2 ("A run is a line-feed count plus a space count"). Evidence:
probes/t2776-formula-text-whitespace-shapes C19:C22, F21 (excel-graph, 2026-10-26). Exactly 256 spaces is
unprobed: probe it first (excel-probe skill) before relying on the byte reading.

## Acceptance criteria
- [x] t2776-formula-text-whitespace-shapes C19:C22, F21 pass, plus any t2769 line-feed cell T-2781 handed over.
- [x] Probes, before any code: 255, 256 and 257 typed spaces, and 255, 256, 257 and 300 typed line feeds (the
      line-feed wrap is unprobed; the BIFF layout alone is not evidence). The parser matches them; no panic on any count.
- [x] `make check` green; reviewers approve.

## Out of scope
- Stored-text runs (T-2784).

## Outcome
Handed off by a sonnet run before any code (tier set to opus); resumed and finished by an opus run.
- Probe first: probes/t2782-whitespace-run-wrap (excel-graph, 2026-10-26). Both counts of a run are one byte and
  wrap at 256 on their own: 255/256/257 spaces print 255/0/1, 255/256/257/300 line feeds print 255/0/1/44, 256 line
  feeds then 3 spaces print the 3 spaces, and 3 spaces then 256 line feeds print nothing (spaces before the last line
  feed are dropped even when the line-feed count wraps to none). Every value is 3. Recorded in
  docs/excel-semantics/syntax.md and SPEC-0023.
- `Run` (parser/spacing.rs) now holds a line-feed count and the spaces after the last line feed, each wrapping at 256;
  printing writes the line feeds, then the spaces. Unit tests cover the probed shapes and both wraps.
- Eval delta: probes fixed 6 (t2769 C17; t2776-shapes C13, C19, C20, C22 and E6, a stored formula with a line feed),
  1 new case (36 cells, all pass), 0 regressions. t2776 C21/F21 already passed (space wrap, T-2781).
