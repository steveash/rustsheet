---
id: T-1874
title: "Fuzz: IMSECH(CHOOSE({1,2},{1,2,3},{4;5})) reaches a scalar-context cell with a 3x2 array (debug_assert panic)"
milestone: M2
lane: functions-engineering
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Evaluating `=IMSECH(CHOOSE({1,2},{1,2,3},{4;5}))` in a cell never panics, in debug or release
builds, and gives Excel's result.

## Context
Nightly fuzz run 39094842217 (2026-10-18, `main` 052af1ed), target `formula_eval`, artifact
`crash-ad6ef248eba724caebd2d09f3a55cefbb0ef25a8`. The input is the 34 bytes
`IMSECH(CHOOSE({1,2},{1,2,3},{4;5})` (base64 `SU1TRUNIKENIT09TRSh7MSwyfSx7MSwyLDN9LHs0OzV9KQ==`);
the target sets it as `=...` in `C3` of its small model (`fuzz/fuzz_targets/formula_eval.rs`),
and the parser closes the missing parenthesis. The steward reproduced it on cdf7d088 with a debug
facade test (`Workbook::set("C3", Input::Formula(...))`):

    panicked at crates/rustsheet-engine/src/model.rs:928:25:
    Larger-than-1x1 array reached scalar-context cell (sheet=0, row=3, column=3, 3x2);
    implicit intersection was expected to collapse it.

The same assertion as T-1303 and T-1596. Variants the steward ran on cdf7d088:
- `IMABS(CHOOSE({1,2},{1,2,3},{4;5}))` and `IMSECH(CHOOSE({1,2},1,{4;5}))` also panic, so
  every lifted IM* function is likely affected, not only IMSECH.
- `IMSECH({1,2,3})` and `IMSECH({1;2})` give a scalar text and `ABS(CHOOSE({1,2},{1,2,3},{4;5}))`
  gives 1, with no panic.
So static analysis takes the IM* call over a CHOOSE with an array index for a scalar, while the
IM* lift (T-1383, T-1384) evaluates it to a broadcast array. Find where the IM* lift's static
shape is computed and make it agree with evaluation, as the SPEC-0011 driver does for ABS.
Excel's value: use the excel-probe skill if no probe covers an IM* function over a lifted CHOOSE.

## Acceptance criteria
- [ ] The reproducer is a unit test (decision 0035) that passes in a debug build.
- [ ] Every IM* function over `CHOOSE({1,2},{1,2,3},{4;5})` agrees in static analysis and
      evaluation; the value matches excel-graph evidence, or the case is left pending and the
      task blocked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other scalar-context assertion reproducers (T-1303, T-1596, done).
