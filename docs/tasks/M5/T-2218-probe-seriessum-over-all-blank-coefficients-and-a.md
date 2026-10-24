---
id: T-2218
title: "Probe SERIESSUM over all-blank coefficients and a negative power that underflows"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence (excel-graph) for two SERIESSUM edges T-2163 left unprobed, and the engine matching it.

## Context
- T-2163 drops blank coefficient cells before numbering the powers (gen-m5 evidence). So an all-blank coefficient
  range now has no terms: `SERIESSUM(0,0,1,A1)` with A1 blank is 0, where the old code read a 0 term at power 0 and gave
  `#NUM!` (numbers.md "SERIESSUM with x = 0"). Excel is unprobed for this.
- A term whose `^` fails maps to `#NUM!` in `fn_seriessum`, including a negative power that underflows, where the `^`
  operator gives `#DIV/0!` (`1E-10^-33.5`, numbers.md). Unprobed for SERIESSUM.
- Probe cells to write (a new probes case, blank cells left out of the spec): `SERIESSUM(0,0,1,A1)` and `SERIESSUM(2,0,1,A1)`
  with A1 blank; `SERIESSUM(0,0,1,A1:A2)` with A1 blank, A2 = 4; `SERIESSUM(0,1,-1,A1:A3)` with A1 = 4, A2 blank, A3 = 5;
  `SERIESSUM(1E-10,-33.5,1,1)`; `SERIESSUM(1E-10,-33.5,1,{1,"a"})`.
- Code: crates/rustsheet-engine/src/functions/math_and_trigonometry/seriessum.rs `seriessum_of`.

## Acceptance criteria
- [ ] The probe case is oracle-filled (excel-graph) and passes, with the engine changed only where the evidence demands.
- [ ] numbers.md records the results under "SERIESSUM: blank coefficient cells are dropped" (replacing the "Unprobed" line).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-integer power last digits.

## Outcome
Added probes/seriessum-blank-and-underflow (9 cells, excel-graph filled). Results: a two-cell all-blank range has no terms (0 for x = 0, n = 0 and for x = 2), while a blank one-cell reference gives #NUM! for SERIESSUM(0,0,1,F1) and 0 for x = 2; the mixed blank cases are #NUM! under either reading of the blank; a negative power whose reciprocal underflows is #NUM! (the ^ operator gives #DIV/0!). The engine already matched all 9 cells, so no engine change; the baseline only gains the new case line (9 checked cells, no failures). Recorded in numbers.md.
