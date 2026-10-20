---
id: T-1592
title: "Fit YIELD's Newton start: a frequency-independent first iterate"
milestone: M9
lane: research
status: blocked
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-2187]
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
- [x] A backward-iterate report in the lab (or a test) that reproduces the starting values above.
- [ ] Start formulas and stop rules tried, each with its bit-exact and within-4-ulp counts as lab fits
  (decision 0056); a model that raises the bit-exact count, or the task set `blocked` with the candidates
  tried. Findings in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-1532).

## Blocked because
No start formula fits, so the model cannot be raised from 3/90 with the evidence at hand. The lab now has
`newton_back_chain` (solvers.py). A test pins the shared one-step-back iterate of S!A1 and S!F1:
0.05523446268, equal to 1.1e-11 across frequencies 2 and 1. Candidates tried, with counts in financial.md
"YIELD stops short of PRICE's root":
- Newton with the residual stop from the approximate yield: 1/90.
- Six closed-form families fitted for a span T: none consistent.

The iterate sits about 1.4e-6 below the redemption-100 root and moves by only about 2e-9 at redemption
100.5 (S!E10). That points to an earlier solver phase, not a closed form. Unblocking needs per-iterate
evidence: Excel cells whose YIELD is forced to stop after k iterations do not exist. Two routes:
- A sweep that moves only redemption (100 to 101 in 0.05 steps) and only the frequency at fixed dates,
  to map how the shared iterate depends on each.
- Or the same back-chain analysis on PRICE's inverse with other day-count bases (probe cells with
  basis 1 to 4 and more than one coupon).

Needs an oracle sweep: T-2187 (redemption 100 to 101 in 0.05 steps at frequencies 1, 2 and 4, bases 1 to 4,
and a price sweep at fixed dates, each cell with its back-chain). Reopen when it lands.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 139 in total.
- YIELD: 139 cells (13-14 digits right 48, 10-12 66, 6-9 25, fewer or 0 vs nonzero 0); most in `probes/yield-solver-sweep.case.json` 60; `probes/solver-iteration-traces.case.json` 22. e.g. `YIELD(A3,B3,C3,H3,E3,F3,G3)` Excel 0.06499999999999766, ours 0.06499999999999986

Excel stops below the root by up to ~1e-9 relative (financial.md "YIELD stops short of PRICE's root"); 15 digits means reproducing that stop.
