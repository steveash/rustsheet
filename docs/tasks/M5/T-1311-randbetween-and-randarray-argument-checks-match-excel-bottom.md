---
id: T-1311
title: "RANDBETWEEN and RANDARRAY argument checks match Excel (bottom>top on raw values, booleans, omitted args, fractional bounds with whole_number)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
RANDBETWEEN and RANDARRAY give Excel's errors and results for the argument edge cases of
probes/volatile-invariant (7 cells fail today).

## Context
T-1283 made run-invariant volatile cells scorable and probed them on excel-graph. The rule is in
docs/excel-semantics/numbers.md "RANDBETWEEN and RANDARRAY argument checks". Failing cells
(`make eval-show CASE=probes/volatile-invariant.case.json`):
- A2 `RANDBETWEEN(2,1)` → `#NUM!` (engine 2); A13 `RANDBETWEEN(5,4.5)` → `#NUM!` (engine 5): compare raw bottom > top.
- A6 `RANDBETWEEN(5.5,5.7)` → 6: the draw starts at CEILING(bottom). The engine draws from
  `[FLOOR(bottom), CEILING(top)]`, so it gives 5 or 6 at random; A6 is therefore listed in the
  case's `invariant` but left out of `check` so the gate cannot flake. Add `Calc!A6` to `check`
  with the fix. Whether `RANDBETWEEN(5.5,6.2)` can give 7 in Excel is unknown (two fills gave 6);
  do not probe it as invariant.
- A8 `RANDBETWEEN(TRUE,TRUE)` → `#VALUE!` (engine 1): booleans are rejected.
- A12 `RANDBETWEEN(,)` → `#N/A` (engine 0).
- B9 `RANDARRAY(1,1,1.5,1.5,TRUE)`, B10 `RANDARRAY(1,1,1.5,2.5,TRUE)` → `#VALUE!` (engine 1, 2).
Code: crates/rustsheet-engine/src/functions/math_and_trigonometry/random.rs.

## Acceptance criteria
- [ ] The 7 cells above pass (A6 added to the case's `check`) and the other cells of probes/volatile-invariant still pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The random distribution itself and the RNG.
