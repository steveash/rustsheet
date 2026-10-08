---
id: T-1006
title: "Oracle sweep for T-0925: NORM.S.INV at NORM.S.DIST(k), tails to 1e-300"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
T-0925 can fit Excel's NORM.S.INV formulation against an oracle-filled sweep instead of the few values cached in Excel-authored workbooks.

## Context
Split from T-0942 (one sweep per M9 fitting task). Build a spec case `eval/suites/probes-numeric/cases/m9-norm-s-inv.case.json` (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in `m9-fin-power.case.json` from T-0942) and fill it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill).

Grid: p = NORM.S.DIST(k,TRUE) for k in -38..8 step 0.25, plus literal tails p = 1e-300, 1e-200, 1e-100, 1e-50, 1e-20, 1e-10, 0.02425 ± 1e-6 (AS241/Acklam region edges), 0.5 ± 1e-9, and 1 - 1e-k; include NORMSINV and NORM.INV(p,0,1) on the same p.

## Acceptance criteria
- [ ] `m9-norm-s-inv.case.json` and its `.oracle.json` sidecar are committed; no pending cells.
- [ ] Quirks visible in the sweep (for example where the engine already differs, `rustsheet-eval show probes-numeric/m9-norm-s-inv.case.json`) are recorded in docs/excel-semantics/numbers.md with the case as evidence.
- [ ] T-0925 links the case and its `depends_on` no longer lists T-0942.
- [ ] Fixtures hold only formulas and Excel values; no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the formulation (T-0925 does that).

## Outcome
Added `probes-numeric/m9-norm-s-inv` (648 cells: 639 sweep cells plus 9 flush/subnormal probes) with its excel-graph oracle sidecar, which has no pending cells. 454 of 648 cells differ from the engine:
- 438 ulp, 4 close, 2 mismatch, 3 bool and 7 type cells;
- the six flushed-tail `#NUM!` cells are among the type mismatches.

The quirks are in numbers.md, and the flush cells point at T-0968/T-1040 (NORM.S.DIST) and T-1027 (subtraction). T-0925 already depends on T-1006, not on T-0942. Eval gates are unchanged, because probes-numeric is ungated.

## Handoff (resumed)
Resumed by an opus run (2026-10-08). The round 2 fidelity items are addressed:
- A648 is relabelled: 1E-309 reads as 0.
- The ulp measure is defined, with k = +-1 explained.
- The flush cells are routed to T-0968/T-1040 and T-1027.
- A644/A645 are not flush evidence.
- The Phi cutoff is near k = -37.52 (unprobed).
- The counts are updated to 648 cells.
