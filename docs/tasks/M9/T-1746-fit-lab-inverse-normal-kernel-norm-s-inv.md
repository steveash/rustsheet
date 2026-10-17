---
id: T-1746
title: "Fit lab: inverse normal kernel (NORM.S.INV, NORM.INV, LOGNORM.INV) against the oracle sweeps"
milestone: M9
lane: research
status: done
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
- [x] `fitlab/kernels/norm_inv.py` adds the fit, shipped in `--list` and fit-lab.md.
- [x] Every candidate's counts are recorded in the Outcome, with the engine's row.
- [x] docs/excel-semantics/statistics.md records the fitted algorithm or the best candidate.
- [x] Time-boxed per decision 0056; `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-0925).

## Outcome
Done 2026-10-16. The fit is `uv run rustsheet-fitlab norm-s-inv` (`fitlab/kernels/norm_inv.py`)
over 1,024 cells. Nested `NORM.S.DIST(k,TRUE)` inputs are looked up in Excel's own CDF sweeps. A
new probe, probes-numeric/t1746-norm-s-inv-central, covers 341 central p, with NORM.S.DIST of each
result and of PPND16(p).

Exact counts, from `uv run rustsheet-fitlab norm-s-inv` with the engine row (the release `rustsheet-eval`):

| candidate | exact |
|---|---|
| PPND16 tails, statrs centre | 775 |
| PPND16 horner, upper log(1-p) | 693 |
| PPND16 horner, upper log1p(-p) | 690 |
| PPND16 + Newton on 0.5*erfc(-x*sqrt(0.5)) | 469 |
| PPND16 + Newton or Halley on libm erfc | 358 |
| engine today (statrs) | 341 |
| correctly rounded | 326 |
| PPND16 power sum | 268 / 265 |
| Acklam + Halley | 200 |
| statrs centre only | 164 of 408 central |
| Acklam | 12 |
| Moro | 12 |
| PPND7, every variant | 12 |

At ≤4 ulp every candidate scores 975 to 987, and all 1,024 cells are within 1e-9.

Findings (docs/excel-semantics/statistics.md, "NORM.S.INV is AS 241 PPND16 in the tails"):
- **Tails are PPND16.** The tails (|p-0.5| > 0.425) are AS 241 PPND16 in doubles: Horner, with
  log(1-p) above. They are bit-exact on 611 of 616 cells; two upper-tail p next to the split
  miss by 1 to 2 ulp.
- **The centre is unsolved.** It was time-boxed after two failed hypothesis families, per
  decision 0056:
  - the PPND16 central rational in every evaluation order, with and without FMA;
  - refinement on Excel's own CDF: one step, or a Newton solve from four starts under several
    stopping rules. Excel's NORM.S.DIST(PPND16(p)) is already exactly p at 232 of the 341 points,
    yet Excel returns a different x at 177 of them.
  Excel is up to 31 ulp from the correctly rounded value there, symmetric in p ↔ 1-p.
- **What T-0925 should implement.** The hybrid: PPND16 tails, statrs centre (775 vs 341 exact).
  The centre reopens only on a new hypothesis. The task is done rather than blocked because the
  fit raised the bit-exact count (341 to 775), which is what decision 0056 measures.
- **Lead for T-0968.** NORM.S.DIST equals `0.5*erfc(-x*sqrt(0.5))` on 260 of 341 probe points,
  against 199 for `/sqrt(2)`.
- LOGNORM.INV and CONFIDENCE.NORM, listed in the goal, are not scored: their mean, sd and exp or
  scaling steps add arithmetic of their own; they follow once T-0925 lands the kernel.
