---
id: T-1524
title: "x87 hypothesis: does 80-bit evaluation of pow, exp, ln, erfc and gamma reproduce Excel's last digits"
milestone: M9
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1523]
---

## Goal
Settle whether Excel evaluates its elementary kernels in x87 80-bit extended precision (a 64-bit significand, then one rounding to double), as its division provably does (numbers.md "Division rounds twice"), and if so, model it for pow, exp, ln, erfc and gamma.

## Context
Owner request 2026-10-14: run the numeric last-digit track alongside M2 (decision 0056). On 2026-10-14 about 9,400 function-cell pairs are near misses: about 3,750 within 4 ulp (rounding and evaluation order) and about 4,600 `numeric_close` (beyond 4 ulp, within 1e-9 relative: a different algorithm, solver stop or series cut-off). 8,508 of them are M9 statistical/financial/engineering functions. Exact rates include PMT 21%, BETA.DIST 20%, GAMMA 11%, RATE 15%, IRR 13%, YIELD 4%, while within-tolerance rates are 92-99%. The 80-bit division model fixed `/` and the reciprocal in `^`. `^` is known not to be correctly rounded (numbers.md, T-0221); GEOMEAN's root models failed (T-1284), but x87 compositions were only tried for GEOMEAN.

## Acceptance criteria
- [ ] Using the fit lab, each of pow, exp, ln, erfc and gamma is scored on its sweep with: correctly rounded, the engine's current code, and an x87 model (operations rounded to a 64-bit significand, final rounding to double). Plus any classic algorithm suggested by the evidence (for example Cephes, Cody's erfc).
- [ ] Findings recorded in `docs/excel-semantics/numbers.md` / `statistics.md` with the sweep cells; a winning model gets an engine task (or is implemented here if small).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Solvers (T-1525).
