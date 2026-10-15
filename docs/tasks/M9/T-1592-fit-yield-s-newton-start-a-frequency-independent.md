---
id: T-1592
title: "Fit YIELD's Newton start: a frequency-independent first iterate"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A fit-lab model that reproduces YIELD (basis 0, more than one coupon left) bit for bit on more than
the 3/90 cells any candidate reaches today, starting from the first-iterate evidence T-1583 found.

## Context
docs/excel-semantics/financial.md "YIELD stops short of PRICE's root"; `uv run rustsheet-fitlab yield`.
T-1583: Excel's values are either at PRICE's root or below it with |PRICE(y) - pr| up to 5e-8, as a
Newton iteration on the convex PRICE that stops on a residual test without its last step would leave
them. Stepping analytic Newton backwards from Excel's below-root values (scratch analysis, reproduce it in
the lab) gives a shared earlier iterate that does not depend on frequency or redemption:
0.0552344627 for yield-solver-sweep S!A1 (frequency 2, 9 coupons) and S!F1 (frequency 1, 5 coupons),
the same within 2e-9 for S!E10 (redemption 100.5), and about 0.133352 for all three frequencies of
solver-iteration-traces Y!C2:E2 (price 50, ten years). The approximate-yield formula
(C + (100 - P)/n) / ((100 + P)/2), annual-coupon IRRs over whole or fractional years, the coupon
rate, 0, 0.1 and the current yield do not give these values.

## Acceptance criteria
- [ ] A backward-iterate report in the lab (or a test) that reproduces the starting values above.
- [ ] Start formulas and stop rules tried, each with its bit-exact and within-4-ulp counts as lab fits
  (decision 0056); a model that raises the bit-exact count, or the task set `blocked` with the candidates
  tried. Findings in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-1532).
