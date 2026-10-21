---
id: T-2304
title: "Probe YIELD's hidden start across coupon rate and term"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-computed YIELD cells that show the hidden Newton start S (T-2187, T-1592) for a grid of coupon
rates and terms, so T-1592 can fit S as a function of rate, price and term.

## Context
docs/excel-semantics/financial.md "YIELD stops short of PRICE's root". YIELD with more than one coupon
left runs Newton on PRICE from a start S that does not depend on redemption or frequency, and returns
the iterate after the first step below about 7e-6. S is visible only in cells that stop below PRICE's
root: on probes/yield-start-sweep the redemption-100.5 cells (A11, B11, C11, at frequency 2, 1 and 4)
all step back two Newton steps to the same S (`solvers.newton_back_chain`). S moves with price and with
the basis's term (T-1592: days over 360 on basis 0, over 365 on basis 1), but the probes so far fix the
coupon rate at 0.05 and the dates at 2096-03-01 to 2100-06-30, so no candidate formula for S can be
tested on rate or term.

Probe (`excel-probe` skill), one new case `probes/yield-start-rate-term`, basis 0, settlement
DATE(2096,3,1), redemption 100.5, each bond at frequency 2, 1 and 4 (three cells side by side):
- coupon rate 0, 0.02, 0.05, 0.08 and 0.12;
- maturity DATE(2098,6,30), DATE(2100,6,30), DATE(2104,6,30), DATE(2110,6,30) and DATE(2126,6,30);
- price 98 and 90.
That is 50 bonds and 150 cells. Write the grid layout into the case description.

## Acceptance criteria
- [ ] The case is filled from the excel-graph oracle and committed with its oracle file.
- [ ] A short paragraph in financial.md "YIELD stops short of PRICE's root" says which cells stop below
  the root (and so show S), with S stepped back for a few of them in the lab.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting S (T-1592).

## Outcome
Added probes/yield-start-rate-term (150 cells, excel-graph, 2026-10-21) and the findings paragraph in
financial.md: 122 cells stop below PRICE's root and show S, 28 are at the root; S stepped back moves with
term, rate and price. No engine change; eval pass counts unchanged.
