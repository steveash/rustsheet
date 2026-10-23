---
id: T-2550
title: "Oracle sweep of BESSELK for x from 500 to 713 at orders 0 to 5 (for the BESSELK near-underflow kernel)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's BESSELK values near the bottom of the double range are known well enough to fit the kernel: x from 500 to 713
at orders 0 to 5, oracle-filled.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep: `BESSELK(700,3)` is 4.69979514014056E-306 in Excel and
4.699871435825504E-306 in ours (gen-m9/8cbfd5e99d2e82da A166), and BESSELK(713,n) and (714,n) are 0 in Excel (T-2421).
One cell cannot fit the kernel, so T-2549 waits for this sweep (docs/excel-semantics/engineering.md "Engineering functions
in the gen-m9 engineering sweep (T-2258)"). Write a probes-numeric case (excel-probe skill) with BESSELK(x,n) for x = 500,
600, 650, 680, 690, 695, 700, 702, 705, 707, 708, 709, 709.5, 709.78, 710, 710.5, 711, 711.5, 712, 712.5, 713 and n = 0 to
5, plus BESSELI(x,0) at the same x (its `#NUM!` side; BESSELI(713,0) is `#NUM!`), and fill it through excel-graph.
Coordinate with T-2421, which probes 713 and 714 and the flush point: one case may serve both.

## Acceptance criteria
- [ ] The probes-numeric case is committed, oracle-filled (excel-graph, no pending cells) and listed in engineering.md with
      what it shows (where Excel's value departs from ours, where it flushes to 0).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the kernel (T-2549).
