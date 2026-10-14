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
Owner request 2026-10-14 (decision 0056); the near-miss figures and their source are in T-1522. The 80-bit division model fixed `/` and the reciprocal in `^`. `^` is known not to be correctly rounded (numbers.md, T-0221); GEOMEAN's root models failed (T-1284), but x87 compositions were only tried for GEOMEAN.

## Acceptance criteria
- [ ] Using the fit lab, each of pow, exp, ln, erfc and gamma is scored on its sweep with: correctly rounded, the engine's current code, and an x87 model (operations rounded to a 64-bit significand, final rounding to double). Plus any classic algorithm suggested by the evidence (for example Cephes, Cody's erfc).
- [ ] Findings recorded in `docs/excel-semantics/numbers.md` / `statistics.md` with the sweep cells; engine changes stay with the kernels' own tasks: erfc and the normal CDF in T-0968, gamma in T-0973, `^` in T-0221's successors. This task only models and scores, and adds the winning model to those tasks' Context.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Solvers (T-1525).
