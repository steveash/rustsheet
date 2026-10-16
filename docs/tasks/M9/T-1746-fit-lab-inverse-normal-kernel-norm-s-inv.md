---
id: T-1746
title: "Fit lab: inverse normal kernel (NORM.S.INV, NORM.INV, LOGNORM.INV) against the oracle sweeps"
milestone: M9
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A fit-lab fit for the inverse standard normal, Φ⁻¹(p), behind NORM.S.INV, NORM.INV, LOGNORM.INV and
CONFIDENCE.NORM. This is rank 11 in the numeric kernel map, with 584 near-miss cells, 97% of them
within 4 ulp. Of the three fits it is the most likely to reach bit-exact.

## Context
Owner request (2026-10-16). Decision 0056 says numeric work is kernel-first and every fit is a
fit-lab comparison of named candidates against an oracle sweep (docs/architecture/fit-lab.md,
`tools/src/rustsheet_tools/fitlab/`). The fit lab already found PV/FV/NPER, XIRR, IRR and Excel's `^`
this way (T-1524, T-1525, T-1533; implemented by T-0901 and T-0916). It has no fit yet for this kernel.
- docs/research/numeric-kernel-map.md, rank 11: the engine calls statrs `Normal::inverse_cdf`; 567 of
  584 cells are ulp misses. T-0925 names Wichura's AS 241 (PPND16) as the candidate; it now waits
  for this fit.
- Sweeps (probes-numeric, excel-graph): `m9-norm-s-inv`, `t1048-norminv`, `t1048-normsinv`,
  `m9-legacy-confidence`; NORM.INV and LOGNORM.INV cells from ironcalc-calc as a second check.
- Candidates, each in doubles and with Excel's operators:
  - AS 241 PPND16 (the double version, 7.0 and 16-digit coefficient sets) and PPND7;
  - Acklam's rational approximation, with and without one Halley refinement step;
  - Moro's algorithm;
  - Newton or Halley on the engine's `normal_cdf` (T-0968, blocked, has no fitted CDF model yet).
- Evaluation order matters for every candidate: Horner forward or reversed, and r = sqrt(-ln(min(p, 1-p))) computed as
  `ln(p)` or `ln1p(-q)`.
- Owner task that implements the winner: T-0925.

## Acceptance criteria
- [ ] `fitlab/kernels/norm_inv.py` adds the fit, shipped in `--list` and fit-lab.md.
- [ ] Every candidate's counts are recorded in the Outcome, with the engine's row.
- [ ] docs/excel-semantics/statistics.md records the fitted algorithm or the best candidate.
- [ ] Time-boxed per decision 0056; `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-0925).
