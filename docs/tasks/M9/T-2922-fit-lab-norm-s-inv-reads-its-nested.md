---
id: T-2922
title: "fit lab: norm-s-inv reads its nested NORM.S.DIST through the sweep's calls cases"
milestone: M9
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2527]
---

## Goal
The fit lab has one way to read a nested call at Excel's own bits: the `norm-s-inv` fit uses the sweep's
`calls` cases (`Fit.calls`, `sweep.call_values`, T-2527) instead of its own lookup.

## Context
T-2527 added `Fit.calls`: an argument whose text is the whole formula of a cell of the `calls` cases reads
Excel's value of that cell (the `rate` fit reads nested `PMT(..)` from probes-numeric/m9-fin-pmt).
kernels/norm_inv.py predates it: its pattern captures the k of a nested `NORM.S.DIST(k,TRUE)` and a
candidate looks p up in its own table of Excel's NORM.S.DIST values (`excel_cdf`, `p_of` and the comments at
lines 66-100). Migrating it removes the second mechanism; the `norm-s-inv` scores must not change.

## Acceptance criteria
- [ ] `norm-s-inv` declares its CDF sweeps as `calls` and its pattern captures the whole argument; the
      kernel-local lookup is gone.
- [ ] `uv run rustsheet-fitlab norm-s-inv` reports the same cell count and per-candidate exact counts as
      before (quote both in the Outcome).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New NORM.S.INV candidates.
