---
id: T-1197
title: "MULTINOMIAL(1E+15) hangs the engine: bound the loop (gen-m5 timeout)"
milestone: M5
lane: functions-math
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
MULTINOMIAL of a huge argument (e.g. 1E+15) terminates within the resource limits and matches Excel's result.

## Context
T-1192's gen-m5 case 21b9ae9affd6b38c times out (60s): `MULTINOMIAL(1E+15)` hangs the engine; in the case it is Calc!A25 `=(LOG(MULTINOMIAL(Data!A5:A5),(Data!A4/Data!B12))*...)`, whose oracle `#NUM!` comes from the base-0 LOG, so probe MULTINOMIAL(1E+15) on its own, not from this cell, a loop proportional to the argument. Directive 5 requires bounded work on untrusted input. Probe what Excel returns for large arguments (likely `#NUM!`), then bound the loop; any deviation gets a DEVIATION entry. Code: crates/rustsheet-engine/src/functions/math_and_trigonometry/multinomial.rs. Related: T-1190 (inexact values).

## Acceptance criteria
- [x] `MULTINOMIAL(1E+15)` returns Excel's result in well under a second (probe evidence cited).
- [x] gen-m5/21b9ae9affd6b38c runs to completion.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (probes/multinomial-large-arguments, 20 cells, excel-graph 2026-10-09): Excel gives `#NUM!` whenever the
arguments add up to more than 170, `MULTINOMIAL(1E+15)` included. `fn_multinomial` now checks a saturating sum against 170
before computing, so it does at most 170 loop steps and cannot overflow the sum. A unit test covers `1E+300` and
`2^64` in debug builds. gen-m5/21b9ae9affd6b38c now runs (39/40 cells). The duplicate T-1217, filed by T-1088 from a
second gen-m5 timeout, is removed. Eval delta: 1 new probe case with 20 cells; 14 pass (every `#NUM!` and 0/1 cell), and 6 are
new known failures. Those 6 are Excel's inexact values, T-1190.
