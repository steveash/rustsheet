---
id: T-1525
title: "Probe the RATE, IRR, YIELD and XIRR iteration traces: Excel's start, step, stop and failure rule"
milestone: M9
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Know exactly how Excel's RATE, IRR, YIELD and XIRR iterate (starting point, update rule, stop test, iteration cap, and when #NUM! is returned), so the engine reproduces the same iterate and its last digits.

## Context
Owner request 2026-10-14: run the numeric last-digit track alongside M2 (decision 0056). On 2026-10-14 about 9,400 function-cell pairs are near misses: about 3,750 within 4 ulp (rounding and evaluation order) and about 4,600 `numeric_close` (beyond 4 ulp, within 1e-9 relative: a different algorithm, solver stop or series cut-off). 8,508 of them are M9 statistical/financial/engineering functions. Exact rates include PMT 21%, BETA.DIST 20%, GAMMA 11%, RATE 15%, IRR 13%, YIELD 4%, while within-tolerance rates are 92-99%. The `numeric_close` misses in these solvers are far beyond rounding: the engine stops at a different iterate. Documented behavior: RATE and IRR iterate up to 20 times to 0.00001%. Existing sweep: probes-numeric/m9-fin-irr-rate; fit task T-0916.

## Acceptance criteria
- [ ] Oracle probes (excel-probe skill) that separate solver models: guesses at and near the root; inputs needing 1 to 25 iterations; non-converging inputs (where #NUM! appears); a root where Newton and secant iterates differ in the last digits.
- [ ] The rule for each solver recorded in `docs/excel-semantics/financial.md` with evidence; T-0916 updated with the fitted model, or the solver fixed here if small.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bond pricing outside YIELD's solver.
