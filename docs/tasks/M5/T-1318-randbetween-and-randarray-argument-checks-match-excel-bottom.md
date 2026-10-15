---
id: T-1318
title: "RANDBETWEEN and RANDARRAY argument checks match Excel (bottom>top on raw values, booleans, omitted args, fractional bounds with whole_number)"
milestone: M5
lane: functions-math
status: done
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
- A6 `RANDBETWEEN(5.5,5.7)` → 6: Excel draws from CEILING(bottom) to FLOOR(top) and gives
  CEILING(bottom) when FLOOR(top) is below it (1000 draws each in F1:F4, F6:F9; F8:F9 show `RANDBETWEEN(5.2,7.9)` spans 6 to 7; (5.5,5.7) and
  (5.5,6.2) are always 6, (-5.5,-5.2) always -5). The engine draws from
  `[FLOOR(bottom), CEILING(top)]`, so it gives 5 or 6 at random; A6 is therefore listed in the
  case's `invariant` but left out of `check` so the gate cannot flake. Add `Calc!A6` to `check`
  with the fix. F1:F9 also need RANDBETWEEN to lift over arrays (T-1316).
- A8 `RANDBETWEEN(TRUE,TRUE)` → `#VALUE!` (engine 1): booleans are rejected.
- A12 `RANDBETWEEN(,)` → `#N/A` (engine 0).
- B9 `RANDARRAY(1,1,1.5,1.5,TRUE)`, B10 `RANDARRAY(1,1,1.5,2.5,TRUE)`, B18
  `RANDARRAY(1,1,1,2.5,TRUE)` → `#VALUE!` (engine returns a number).
Code: crates/rustsheet-engine/src/functions/math_and_trigonometry/random.rs.

## Acceptance criteria
- [x] The 8 cells above pass (A6 added to the case's `check`) and the other cells of probes/volatile-invariant still pass; baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- The random distribution itself and the RNG.

## Outcome
RANDBETWEEN compares the raw bounds, draws from CEILING(bottom)..FLOOR(top) (CEILING(bottom) when empty), rejects booleans and gives
#N/A for `RANDBETWEEN(,)`; RANDARRAY gives #VALUE! for a fractional bound with whole_number TRUE. Calc!A6 is now in the case's `check`.
Eval delta: probes 7 fixed (A2, A8, A12, A13, B9, B10, B18), A6 newly covered and passing, 0 regressions. Array lift F1:F9 stays with T-1316.
