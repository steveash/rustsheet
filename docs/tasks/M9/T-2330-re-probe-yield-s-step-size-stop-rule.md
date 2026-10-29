---
id: T-2330
title: "Re-probe YIELD's step-size stop rule against yield-start-rate-term"
milestone: M9
lane: research
status: in-progress
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
- [ ] A stop rule (or a documented reason none fits) is recorded in financial.md with the cells that bound it.
- [ ] The lab test pin is updated to match.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting S (T-1592); engine changes.
