---
id: T-2330
title: "Re-probe YIELD's step-size stop rule against yield-start-rate-term"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A YIELD stop rule (the step size below which Excel returns the iterate) that fits every cell of probes/yield-start-rate-term and the earlier YIELD probes.

## Context
docs/excel-semantics/financial.md "YIELD stops short of PRICE's root" (T-2304 paragraph): B2 stops after a last Newton step of 7.54e-6 (2.4e-4 relative) and A26 after 6.87e-6 (1.43e-4 relative), contradicting the cut of T-2187 (6.66e-6 to 7.11e-6 absolute, or 1.17e-4 to 1.25e-4 relative). The lab test pins 6.9e-6, which is now suspect. Fidelity reviewer also suggests pinning the forward check from S=0.0552344627 in tools/tests/test_fitlab_solvers.py.

## Acceptance criteria
- [x] A stop rule (or a documented reason none fits) is recorded in financial.md with the cells that bound it.
- [x] The lab test pin is updated to match.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fitting S (T-1592); engine changes.

## Outcome
No new probe was needed: T-1592's short stop (a Newton step below the cut is not taken) was checked on all 741
YIELD probe cells with more than one coupon left. 721 are within 4 ulp at some cut, and all of them at any cut in
(9.6715e-11, 1.0063e-10]. The bounds are yield-start-sweep C18 (step not taken) and yield-start-one-input A5 (step
taken). yield-start-rate-term B2 and A26 return before a 9.13e-11 and a 6.52e-11 step. Their 7.54e-6 and 6.87e-6
"last steps" are the step before. A relative-step stop scores 677/741 and a residual stop 681/741; the bracketing pair
rules both out. Recorded in financial.md "YIELD stops short of PRICE's root". The lab's 6.9e-6 pin is now the short
stop at 1e-10, and a new test pins the two bounding cells. No eval delta (docs and lab only).
