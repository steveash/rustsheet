---
id: T-1004
title: "Oracle sweep for T-0908: BESSELJ/BESSELY for x in [8,100], orders 0-10"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-0908 can fit Excel's BESSELJ/BESSELY formulation against an oracle-filled sweep instead of the few values cached in Excel-authored workbooks.

## Context
Split from T-0942 (one sweep per M9 fitting task). Build a spec case `eval/suites/probes-numeric/cases/m9-bessel.case.json` (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in `m9-fin-power.case.json` from T-0942) and fill it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill).

Grid: x in {8, 8.5, 9, 10, 12.5, 15, 20, 25, 30, 40, 50, 64, 75, 100} by orders 0..10 for BESSELJ and BESSELY, plus non-integer orders (truncation) and the largest accepted order (find the cap: probe orders 100, 1000, 1e4, 1e5).

## Acceptance criteria
- [ ] `m9-bessel.case.json` and its `.oracle.json` sidecar are committed; no pending cells.
- [ ] Quirks visible in the sweep (for example where the engine already differs, `rustsheet-eval show probes-numeric/m9-bessel.case.json`) are recorded in docs/excel-semantics/engineering.md with the case as evidence.
- [ ] T-0908 links the case and its `depends_on` no longer lists T-0942.
- [ ] Fixtures hold only formulas and Excel values; no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the formulation (T-0908 does that).

## Outcome
Added `probes-numeric/m9-bessel` (364 cells) and its excel-graph oracle sidecar, no pending cells. Findings (fractional-order truncation, BESSELJ = 0 for huge n, Excel ~1e-9 off true values for x >= 8) recorded in docs/excel-semantics/engineering.md; T-0908 links the case. The `probes-numeric` suite is ungated (`gate = false`), so no baseline changed.
