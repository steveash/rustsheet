---
id: T-1374
title: "Bond, T-bill and depreciation values a few ulps off Excel (probes/fin-argument-modes base rows)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The bond, T-bill, depreciation and cumulative-payment functions return Excel's bits for the probes/fin-argument-modes base rows.

## Context
probes/fin-argument-modes (excel-graph, 2026-10-11, T-1084). Base rows that miss:
- 1 to a few ulps off: CUMIPMT H72, DB H88, DDB H95, DURATION H117, INTRATE H139, MDURATION H163, ODDFPRICE H189,
  PRICE H251, YIELDDISC H352.
- Tens of ulps off: CUMPRINC H80 (about 37) and EFFECT H125 (about 32).
- Off at about 1e-10 to 1e-12 relative, so probably an iteration stopping point rather than an evaluation order: ODDFYIELD H200,
  YIELD H343. (TBILLEQ H313 and the other TBILL cells moved to T-1475 on 2026-10-13.)
The annuity family (FV, PMT, PPMT, NPV) and the solvers (IRR, RATE, XIRR) are T-0901's and T-0916's, not this task's.
One probe point per function is not enough to fit a formulation: first add an oracle sweep per function family
(excel-probe skill; probes-numeric like T-0942), then fit. Split per family if larger than M.

## Acceptance criteria
- [ ] A probes-numeric sweep for the bond and depreciation families is oracle-filled.
- [ ] The base rows above pass, or the Outcome records which formulation was ruled out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FV, PMT, PPMT, NPV (T-0901); IRR, RATE, XIRR, MIRR (T-0916).
