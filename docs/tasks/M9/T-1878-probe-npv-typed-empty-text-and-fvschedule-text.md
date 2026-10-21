---
id: T-1878
title: "Probe NPV typed empty text and FVSCHEDULE text-vs-error order"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
NPV and FVSCHEDULE match Excel for the argument forms T-1373 left unprobed.

## Context
T-1373 (docs/excel-semantics/financial.md "NPV and FVSCHEDULE arguments"; `fn_npv`, `fn_fvschedule` in
functions/financial/mod.rs) left these decided by the code but not by oracle evidence:
- `NPV(0.1,-10000,"",4250)`: typed empty text goes through `cast_number_ref` (assumed #VALUE!).
- `FVSCHEDULE(1000,{"x",#N/A})` vs `{#N/A,"x"}`, and the same order in a range: the walk returns the first of
  text (#VALUE!) or error it meets.
- An NPV argument that evaluates to an empty cell value (e.g. `IF(TRUE,)`, `LET(x,,x)` if valid) is skipped.
Write one probe case (probes/npv-fvschedule-unprobed) with these cells, fill it from excel-graph, and change the
code only where Excel differs.

## Acceptance criteria
- [x] The probe case is filled from excel-graph and every cell passes or misses only by NPV ulp (T-1528).
- [x] financial.md records the findings.
- [x] `make check` green; reviewers approve.

## Out of scope
- NPV's last-digit misses, including fin-argument-modes H368 (NumericClose, ~2e-15 relative): T-1528.

## Outcome
Probe `probes/npv-fvschedule-unprobed` (8 cells) filled from excel-graph. All cells match the engine, so no
code change: `NPV(..,"",..)` is #VALUE!, `IF(TRUE,)`/`LET(x,,x)` count as a cash flow of 0 (not skipped), and
FVSCHEDULE returns the first of text/error left to right in arrays and ranges. A8 misses by 1 ulp (NPV, T-1528);
recorded in `eval/baseline/probes.txt`. Findings in financial.md "NPV and FVSCHEDULE arguments".
