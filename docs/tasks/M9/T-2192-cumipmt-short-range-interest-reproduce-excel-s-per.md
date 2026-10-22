---
id: T-2192
title: "CUMIPMT short-range interest: reproduce Excel's per-period form (visibly-off cells after T-1890)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-1890]
---

## Goal
CUMIPMT is within half a unit of the 15th significant digit of Excel on every probed cell, and so are
the two CUMPRINC overflow and past-2^53 cells T-1890 left.

## Context
T-1890 made CUMPRINC a closed form at every length, so its visibly-off cells are fixed. CUMIPMT's
interest is not "close to correctly rounded": Excel's single-period `CUMIPMT(0.01,100,100000,100,100,0)`
is -15.708656559792416 where the exact value is ...592, and `CUMIPMT(0.01,1E15,100000,1E15,1E15,0)`
is -9.900990099009732 where it is ...901 (probes/fin-cumipmt-overflow-and-forms A11, A12). Excel follows
an IPMT-like balance recurrence that cancels, also near period 1E15 where `(1+r)^nper` overflows. Neither the engine's
per-period IPMT sum (up to 32 periods, kept by T-1890) nor the closed form reaches A12. A sum of `-pmt*expm1((k-1-nper)*L)`
terms is exact but further from Excel (docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over
many periods").
Visibly off on the T-1890 head:
- CUMPRINC_CUMIPMT.xlsx G2, G4, G7 and G31, and H26 (CUMPRINC -3.308722252996902E-21, ours
  ...691E-21: 2 ulps, but visibly).
- probes/fin-cumipmt-overflow-and-forms A7 and A9-A12, and A16 (`CUMPRINC(1E10,40,1,1,31,0)`,
  -9.999999990999995E-91, ours ...1000066E-91 as on main), and `CUMPRINC(1E10,40,1,1,20,0)` and `(…,15,20,0)`
  (excel-graph -9.999999980000501E-201, ours ...0004E-201 for A23 and ...0572E-201 for A24, as on main), and A20-A22
  (`CUMPRINC(1E10,32,1,1,25,0)` -9.99999999300011E-71, ours ...3000182E-71 as on main, and its siblings).
- fin-cumipmt-period-bound A18, fin-remaining-later-argument-ranges K3.
- A2 (`CUMPRINC(1E300,10,1E300,1,10,1)`: Excel -1E300, ours `#NUM!`).
- A5 and A6 (1E-15 rate past 2^53).

## Acceptance criteria
- [ ] The cells above are within half a unit of the 15th significant digit (A2 a number), with no new failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-exact fits once the cells are visibly right.

## Visible-parity stage
Decision 0083: the acceptance criteria are the visible-parity stage.
