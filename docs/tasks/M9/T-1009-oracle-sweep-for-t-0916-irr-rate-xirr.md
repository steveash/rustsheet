---
id: T-1009
title: "Oracle sweep for T-0916: IRR/RATE/XIRR/MIRR cash-flow families and guesses"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
T-0916 can fit Excel's IRR/RATE/XIRR/MIRR formulation against an oracle-filled sweep instead of the few values cached in Excel-authored workbooks.

## Context
Split from T-0942 (one sweep per M9 fitting task). Build a spec case `eval/suites/probes-numeric/cases/m9-fin-irr-rate.case.json` (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in `m9-fin-power.case.json` from T-0942) and fill it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill).

Grid: Cash-flow families around the cached points in ironcalc-calc/FINANCIAL (IRR.xlsx, RATE.xlsx, XIRR.xlsx, MIRR.xlsx): scale and sign variations, near-zero and multiple-root flows, and guess variations (omitted, 0, 0.1, -0.5, 1, 10). Include RATE on the PMT/PV grid of probes-numeric/m9-fin-pmt so it round-trips against PMT. Record where Excel returns #NUM! (non-convergence) as well as values.

## Acceptance criteria
- [ ] `m9-fin-irr-rate.case.json` and its `.oracle.json` sidecar are committed; no pending cells.
- [ ] Quirks visible in the sweep (for example where the engine already differs, `rustsheet-eval show probes-numeric/m9-fin-irr-rate.case.json`) are recorded in docs/excel-semantics/financial.md with the case as evidence.
- [ ] T-0916 links the case and its `depends_on` no longer lists T-0942.
- [ ] Fixtures hold only formulas and Excel values; no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the formulation (T-0916 does that).
