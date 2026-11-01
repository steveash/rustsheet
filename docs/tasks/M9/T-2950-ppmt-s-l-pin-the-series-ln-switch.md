---
id: T-2950
title: "PPMT's L: pin the series/LN switch on both signs and commit the candidates to the fit lab"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Pin where Excel's PPMT log `L = ln(1+rate)` switches from the x87 atanh series to LN, on positive and on negative
rates, and leave the series and LN candidates in the fit lab so the fit can be re-run.

## Context
- Split from T-2588. `growth_log` (crates/rustsheet-engine/src/functions/financial/annuity.rs) takes the series
  for |rate| < 0.375 and LN above (`GROWTH_LOG_SERIES_TO`). That round point is not pinned between 0.37 and 0.38,
  and nobody has checked that the switch is on |rate| for negative rates (financial.md "PPMT discounts with an
  atanh-series log", Open bullet).
- Evidence on hand: probes-numeric/ppmt-log-rate-sweep (positive rates) and probes-numeric/ppmt-negative-rate-sweep
  (-0.9 to -0.3, with a fine grid at -0.38 to -0.37; T-2588, 36 of 79 cells an ulp off). Neither has been scored by
  L candidate. T-2242's fit counts came from one-off scripts, not from tools/src/rustsheet_tools/fitlab/kernels/annuity.py.
- First step: add both L candidates to the fit lab and score them per rate on both sweeps. If the switch still
  cannot be pinned, probe a finer grid near ±0.375 with long terms, so that one ulp of L moves the value.

## Acceptance criteria
- [ ] fitlab/kernels/annuity.py scores the series and LN candidates for L on both sweeps.
- [ ] financial.md states the switch on each sign (or the narrowest bracket the evidence gives), and
      `GROWTH_LOG_SERIES_TO` follows it with no new known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PMT/IPMT/PPMT bits beyond L (T-2243).
