---
id: T-1131
title: "Probe near-zero snapping for SUM over ranges, trailing zeros, arrays and =(SUM(...))"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for near-zero snapping of an outermost SUM in the forms T-0110 extrapolated, and the engine matching it.

## Context
T-0110 snaps the final addition of an outermost SUM (`fn_sum_outermost`, functions/math_and_trigonometry/sum.rs; `snap_near_zero`, arithmetic.rs) using the last two addends. The evidence covers only `SUM(X,-b)` (probes-numeric/30bc5f8fab02d0a2 `sum-fn`) and `SUM(0.1,0.2,-0.3)` (probes/precision-and-rounding A4). Unprobed (docs/excel-semantics/numbers.md "Near-zero snapping"): `=SUM(A1:A3)` over 0.1, 0.2, -0.3 (also with blanks or text between); `=SUM(0.1,0.2,-0.3,0)`; `=SUM({0.1,0.2,-0.3})`; `=(SUM(0.1,0.2,-0.3))`; and the order of addition in a 2-D range (row-major?). Use the `excel-probe` skill.

## Acceptance criteria
- [x] Probe cells with oracle values for each form; the numbers.md Unprobed line is resolved; the engine matches (baseline shrinks or a DEVIATION entry).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/near-zero-sum-forms` (17 cells, excel-graph): a SUM call snaps its final addition wherever it sits (`=(SUM(...))`, `SUM(...)+0`, `SUM(...)*1` all give 0), unlike `+`/`-`; a trailing zero addend stops the snap; arrays and ranges (blanks/text between) behave like scalars; 2-D ranges add row-major. Engine: `fn_sum` always snaps (the root special case is gone); AGGREGATE(9) and PERCENTOF use `fn_sum_unsnapped` (unprobed). numbers.md updated. Eval: probes +17 passing cells, 0 regressions.
