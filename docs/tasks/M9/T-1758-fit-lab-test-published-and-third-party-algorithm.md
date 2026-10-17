---
id: T-1758
title: "Fit lab: test published and third-party algorithm claims (web research) against the normal and beta oracle sweeps"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Find what the web knows about the algorithms behind Excel's normal and incomplete-beta kernels
and their inverses, and settle every concrete claim in the fit lab so the residual few-ulp
mismatches have a documented cause or a narrowed search.

## Context
Owner request (2026-10-17): several distributions differ from Excel only slightly. Check what
people have found online (forums, blogs, papers, Microsoft docs) about these specific functions
and test their insights in the fit lab. This follows T-1744, T-1745 and T-1746 (fits
`beta-dist`/`f-dist`/`t-dist`, `beta-inv` and the inverses, `norm-s-inv`) and feeds T-0968,
T-0970 and T-0974.

## Acceptance criteria
- [x] docs/research/excel-algorithm-web-research-2026-10-17.md records what is public (Microsoft,
  papers, third-party code), each hypothesis tested and its result.
- [x] The testable claims are fit-lab candidates or probes:
  - Cody's erfc in a new `norm-s-dist` fit;
  - LibreOffice GetBetaDist in `beta-dist`, `f-dist` and `t-dist`;
  - the "iterative search on own CDF" claim in probe t1758-norm-s-inv-roundtrip.
- [x] docs/excel-semantics/statistics.md records the findings.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-0925, T-0968, T-0970, T-0974).

## Outcome
Done 2026-10-17. Nothing public names Excel's algorithms or reverse-engineers them at the last
digit. Microsoft documents only that the inverses are iterative searches on the matching DIST
function, which run "to the limit of Excel's precision" since Excel 2003.

Results:
- **NORM.S.DIST.** Cody's erfc gets 991 of 1,773 cells (1,562 within 4 ulp), against 1,103
  (1,524) for glibc's `0.5*erfc(-x*sqrt(0.5))` and 879 (1,473) for the engine (fit `norm-s-dist`). Cody is refuted. The glibc
  form is an interim kernel for T-0968.
- **NORM.S.INV centre.** All of these are `norm-s-inv` candidates now; scores are central cells
  out of 408. AS 111's central rational (split 0.42) alone gets 12. With 1 or 3 Newton steps on
  Cody or glibc it gets 70 to 95. The best polish overall is PPND16 plus one Newton step on Cody,
  at 114. statrs gets 164. Refuted.
- **NORM.S.INV round trip** (probe t1758-norm-s-inv-roundtrip): Excel's central x is a crossing of
  its own NORM.S.DIST at 234 of 340 points, the CDF is flat at 36, and 70 miss by 1 to 32 ulps of
  p, mostly below. This is the same shape as BETA.INV, so Excel's inverses are root searches on its
  own CDF, and inverse parity waits on forward parity.
- **BETA.DIST, LibreOffice.** LibreOffice's GetBetaDist gets 156 of 283 with Excel's closed forms
  (134 without), level with the Cephes combination (158); not Excel's.
- **BETA.DIST, where the gap is.** The remaining gap is the direct (unreflected) branch: at most
  15 of 107 points for any candidate, against 92% (LibreOffice) and 79% (Cephes) in the reflected
  branch.
  - Excel's relative error there tracks the rounding of fl(a·fl(ln x)): 1.18E-14 predicted at
    x = 0.1, a = 100, against 1.04–1.35E-14 observed. That suggests x^a is a separate
    EXP(a·LN(x)) factor (hypothesis).
  - Excel's own GAMMA values (best 62 of 136) and 360 combinations of fraction, prefactor and
    operation order (best 16 of 107) do not close the rest.
  - Script: docs/research/t1758_beta_direct_branch.py.
- **F.DIST and T.DIST.** LibreOffice gets 15 of 33 and 100 of 150 (Cephes 15 and 105).
