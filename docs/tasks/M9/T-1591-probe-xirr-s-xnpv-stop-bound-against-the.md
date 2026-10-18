---
id: T-1591
title: "Probe XIRR's XNPV stop bound against the span and a zero first value"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle values that pin the form of XIRR's XNPV stop bound on long spans and of the zero-first-value
behaviour, so the T-1583 model's `span_stop` is either confirmed or replaced.

## Context
T-1583 (docs/excel-semantics/financial.md "XIRR is a bracketing bisection") fits the XNPV part of the
stop test as |XNPV(m)| < 2e-8 * max|value| * max(1, years from the first date to the latest), with no
XNPV bound when the first value is 0. The span factor rests on one cell (xirr-early-dates A8, 124 years);
every fitted cell has a span of at most ten years, where Excel stops at the step test or soon after.
Two-flow probes `XIRR({-1, x}, {d0, d0 + 365*T})` have the root x^(1/T) - 1 in closed form, so the lab
can find the returned bisection step (`uv run rustsheet-fitlab xirr-oos`).

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with two-flow XIRRs over spans T = 2, 5, 10, 20, 40, 80, 120 years
  and several roots per span (e.g. 0.013, 0.1, 0.5), plus three-flow cells with a zero first value
  (first date before, equal to, and after the second), and two-flow cells with a zero first value at
  guesses 0.2 and 1 (moved from T-1155), oracle-filled.
- [ ] The probe added to `XIRR_OOS_CASES` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py); the
  T-1583 candidates' counts recorded in financial.md, and the rule there corrected if they miss.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-0916).
