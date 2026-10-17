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
- **NORM.S.DIST.** Cody's erfc gets 991 of 1,779 cells, against 1,103 for glibc's
  `0.5*erfc(-x*sqrt(0.5))` and 881 for the engine (fit `norm-s-dist`). Cody is refuted. The glibc
  form is an interim kernel for T-0968.
- **NORM.S.INV centre.** An AS 111 start (split 0.42), with or without Newton/Halley polish on the
  Cody or glibc CDFs, peaks at 114 of 408 central cells, against 164 for statrs. Refuted
  (scratch run, not shipped).
- **NORM.S.INV round trip** (probe t1758-norm-s-inv-roundtrip): Excel's central x is a crossing of
  its own NORM.S.DIST at 234 of 340 points, the CDF is flat at 36, and 70 miss by 1 to 32 ulps of
  p, mostly below. This is the same shape as BETA.INV, so Excel's inverses are root searches on its
  own CDF, and inverse parity waits on forward parity.
- **BETA.DIST, LibreOffice.** LibreOffice's GetBetaDist gets 156 of 283 with Excel's closed forms
  (134 without), level with the Cephes combination (158); not Excel's.
- **BETA.DIST, where the gap is.** The remaining gap is the direct (unreflected) branch: at most
  18 of 110 cells for any candidate. There Excel's relative error grows with a·err(ln x), so
  Excel builds the prefactor in log space from a double ln x. Excel's own GAMMALN and 360
  combinations of fraction, prefactor and operation order do not close the rest.
- **F.DIST and T.DIST.** LibreOffice gets 15 of 33 and 100 of 150 (Cephes 15 and 105).
