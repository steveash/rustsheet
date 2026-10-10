---
id: T-1181
title: "Probe 3-D span with both ends missing, CHOOSE-wrapped span in COUNTBLANK, deleted span end"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for 3-D span forms T-1170 left unprobed, with the engine matching it.

## Context
T-1170 maps a span whose prefix has an unresolved end to `#NAME?` (`Model::wrong_sheet_error`) and follows only IF in `is_span_or_if_of_span`. Unprobed: `Nope1:Nope2!A1`, a span that lost a sheet when a sheet was deleted, `COUNTBLANK(CHOOSE(1,Data1:Data3!A1:B3))`.

## Acceptance criteria
- [x] Probe the three forms with the excel-probe skill; the engine matches or the mismatch is fixed; references.md 3-D span entry updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- New probe `3d-ref-span-missing-ends` (excel-graph, 12 cells): `Nope1:Nope2!A1` is `#NAME?` like a span with one missing
  end, and CHOOSE-wrapped spans in COUNTBLANK act like IF-wrapped ones. The engine already matched every cell except
  `COUNTBLANK(CHOOSE(1,Data1:Nope!A1:B3))`, where it gave 0 for Excel's `#NAME?`.
- New probes `countblank-choose-errors`, `countblank-error-cell-refs` and `countblank-error-forms-b` (excel-graph, 25 cells):
  COUNTBLANK of an error argument that is not a reference gives that error (`#NAME?`, `#N/A`, `#NULL!`, `#REF!`); the engine
  counted 0. A one-cell reference to an error cell counts 0, through CHOOSE, INDEX, OFFSET, IF, XLOOKUP and TAKE too. Fixed in
  `fn_countblank`, which now reads the argument once through `evaluate_picked_reference_arg` (review round 1: a second
  evaluation made nested COUNTBLANK exponential; unit test `nested_countblank_of_an_error_reads_each_argument_once`).
- Not probed: a span that lost an end sheet. excel-graph cannot delete a sheet, so this is filed as T-1310 (M14, blocked
  on the desktop oracle, decision 0039). A one-missing-sheet reference was left out of the probe, because the web oracle
  resolves it to an empty cell (lookup.md).
- Eval delta: probes +4 cases, 37 cells, all pass. Known failures 914 → 914. Regressions 0.
