---
id: T-1778
title: "Kernel map: ERF, ERFC, NORM, CHISQ, POISSON share Excel's incomplete gamma"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Fewer, better-aimed kernel tasks. Oracle probes show which Excel distribution functions are bit-identical compositions of a shared kernel, and what that kernel's error structure is. The engine can then route them through one implementation and fit one kernel instead of a dozen.

## Context
Owner request (2026-10-17): finish the open-source sweep and run more probes to crack the remaining kernel mysteries: NORM.S.DIST/ERFC (T-0968, T-1187), the NORM.S.INV centre (T-1746) and BETA.DIST (T-1744).
- Sweep covered: Cephes (xsf), Boost.Math, Apache Commons Math, LibreOffice, ONLYOFFICE, EPPlus, ClosedXML, HyperFormula, formula.js, pycel, formulas, MS Chart Controls, Moshier's cmath. None claims Excel parity, and none reproduces it.
- Previous negatives: docs/excel-semantics/statistics.md (T-1187, T-1744, T-1746), docs/research/excel-algorithm-web-research-2026-10-17.md (T-1759).

## Acceptance criteria
- [x] Probes for the identities and the kernel's error structure, filled by excel-graph: probes-numeric `kmyst-*` (8 cases).
- [x] Findings and negatives are in docs/excel-semantics/statistics.md, "One incomplete-gamma kernel …". A script reprints them: docs/research/t1778_incomplete_gamma_kernel_map.py.
- [x] Follow-up implementation tasks are filed for routing the identities through one kernel.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the routing (follow-up task).
- Disassembling Excel.

## Outcome
Done 2026-10-17. Excel's distribution functions collapse onto far fewer kernels than the task list assumed.
- NORM.S.DIST, NORM.DIST, LOGNORM.DIST and GAUSS are ERFC.PRECISE compositions, with the scaling a multiply by sqrt(0.5).
- ERF and ERFC equal GAMMA.DIST/CHISQ.DIST at shape 0.5 / one degree of freedom.
- CHISQ, GAMMA.DIST scale and the POISSON CDF are GAMMA.DIST/CHISQ.DIST.RT compositions. Every identity holds bit for bit at 60 to 1,545 of the same number of points.
- Kernel structure:
  - small-x erf constant 1.1283791670955128, with at least two roundings;
  - generic-shape prefix exp(fl(a*fl(ln z))) with Excel's GAMMALN, and density = prefix/z;
  - shapes 0.5 and 1 off that path;
  - naive exp(-fl(x*x)) in the ERFC tail, plus a smooth ±2.5-ulp drift.
- The exact operation sequence is still open; the candidates tried are listed as negatives in statistics.md.
- Follow-up: T-1780 routes the engine through the identities. T-0968, T-1187, T-0971 and T-0972 now target one kernel.
