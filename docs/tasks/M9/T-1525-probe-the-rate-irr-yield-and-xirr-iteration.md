---
id: T-1525
title: "Probe the RATE, IRR, YIELD and XIRR iteration traces: Excel's start, step, stop and failure rule"
milestone: M9
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1009]
---

## Goal
Know exactly how Excel's RATE, IRR, YIELD and XIRR iterate (starting point, update rule, stop test, iteration cap, and when #NUM! is returned), so the engine reproduces the same iterate and its last digits.

## Context
Owner request 2026-10-14 (decision 0056); the near-miss figures and their source are in T-1522. The `numeric_close` misses in these solvers are far beyond rounding: the engine stops at a different iterate. Documented behavior: RATE and IRR iterate up to 20 times to 0.00001%. T-1009 already landed a 691-cell IRR/RATE/XIRR sweep (probes-numeric/m9-fin-irr-rate), and financial.md "IRR, RATE and XIRR stop early" records the default guess, early stop and #NUM! findings. This task reuses those cells and only probes what they lack: which update rule, derivative and stop test produce Excel's last digits, plus YIELD, which has no sweep. T-1050: RATE's guess argument cannot be reached today. The engine fix stays in T-0916, which now depends on this task.

## Acceptance criteria
- [ ] Oracle probes (excel-probe skill), on top of the T-1009 cells, that separate solver models by their last digits: Newton versus secant versus other update rules, the derivative form, the exact stop test and iteration count; YIELD across price, coupon count and basis.
- [ ] The rule for each solver recorded in `docs/excel-semantics/financial.md` with evidence; T-0916 (and a new YIELD task if needed) gets the fitted model in its Context; no engine change here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bond pricing outside YIELD's solver.
