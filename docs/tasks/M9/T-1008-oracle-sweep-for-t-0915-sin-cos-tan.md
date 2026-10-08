---
id: T-1008
title: "Oracle sweep for T-0915: SIN/COS/TAN/SEC/CSC/COT near k*pi"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-0915 can fit Excel's SIN/COS/TAN/SEC/CSC/COT formulation against an oracle-filled sweep instead of the few values cached in Excel-authored workbooks.

## Context
Split from T-0942 (one sweep per M9 fitting task). Build a spec case `eval/suites/probes-numeric/cases/m9-trig-reduction.case.json` (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in `m9-fin-power.case.json` from T-0942) and fill it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill).

Grid: Arguments k*PI() + d for k in {1,2,3,10,100,1000,1e4,1e5,1e6} and d in {0, ±1e-12, ±1e-9, ±1e-6, ±0.25}, plus multiples of PI()/2; one row per argument, one column per function. Keep each call at valid arity (an arity-invalid call makes Excel for the web reject the whole workbook, see T-0040).

## Acceptance criteria
- [ ] `m9-trig-reduction.case.json` and its `.oracle.json` sidecar are committed; no pending cells.
- [ ] Quirks visible in the sweep (for example where the engine already differs, `rustsheet-eval show probes-numeric/m9-trig-reduction.case.json`) are recorded in docs/excel-semantics/numbers.md with the case as evidence.
- [ ] T-0915 links the case and its `depends_on` no longer lists T-0942.
- [ ] Fixtures hold only formulas and Excel values; no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the formulation (T-0915 does that).
