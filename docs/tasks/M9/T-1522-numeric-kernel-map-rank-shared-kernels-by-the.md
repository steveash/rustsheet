---
id: T-1522
title: "Numeric kernel map: rank shared kernels by the failing cells each one would fix"
milestone: M9
lane: research
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A ranked map of the shared numeric kernels (pow, exp/ln, erfc/normal CDF, lgamma/gamma, regularized incomplete beta and gamma, the annuity factor (1+r)^n, the Newton solvers, trig argument reduction) and, for each, the near-miss cells that depend on it, so fit work goes to the kernel that fixes the most cells first.

## Context
Owner request 2026-10-14: run the numeric last-digit track alongside M2 (decision 0056). On 2026-10-14 about 9,400 function-cell pairs are near misses: about 3,750 within 4 ulp (rounding and evaluation order) and about 4,600 `numeric_close` (beyond 4 ulp, within 1e-9 relative: a different algorithm, solver stop or series cut-off). 8,508 of them are M9 statistical/financial/engineering functions. Exact rates include PMT 21%, BETA.DIST 20%, GAMMA 11%, RATE 15%, IRR 13%, YIELD 4%, while within-tolerance rates are 92-99%. Today the fit tasks (T-0916, T-0925, T-0967 to T-0975, T-0915, T-0917) are split per function, although one kernel feeds many functions (PMT, PV, FV, PPMT, IPMT and NPER all evaluate (1+r)^n).

## Acceptance criteria
- [ ] `docs/research/numeric-kernel-map.md` lists each kernel, the engine functions that call it (file and function), and the eval cells (`numeric_ulp`, `numeric_close`, `numeric_mismatch`) of every function that uses it, from `eval/runs/latest/report.json`.
- [ ] Kernels are ranked by cells they could fix; each existing fit task is mapped to its kernel, and kernels with no task get one.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any kernel.
