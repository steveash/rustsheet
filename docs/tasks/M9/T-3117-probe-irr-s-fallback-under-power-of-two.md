---
id: T-3117
title: "Probe IRR's fallback under power-of-two scaling and negation: separate its absolute residual test from its arithmetic"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel values for IRR's fallback path on flows scaled by exact powers of two and negated, so that T-2928 can tell
an absolute residual test (which sees the scale) from the iteration's own arithmetic (which a power-of-two scale
leaves bit-for-bit unchanged) and from a step whose direction depends on the sign of the NPV.

## Context
T-2928 is blocked after two fit attempts (decision 0056); this probe is the new evidence its "Blocked because"
section asks for. The T-2927 sweep (probes-numeric/t2927-irr-fallback-sweep) scales by 1E6 and negates. A decimal
scale changes the flows' bits, so it cannot separate a stop test |P| < 1E-7 in the flows' units from rounding.
Scaling by 2^k changes the bits of no intermediate value: Newton or a secant in v gives the same bits at every k
unless a test compares an absolute quantity (|P| or |NPV|) with a constant. Negation also leaves every Newton
iterate bit-for-bit unchanged, so a difference between a row and its negation proves a step that depends on the
sign of P, such as a secant's signed first offset (financial.md "IRR's fallback when the secant fails", Fit, attempt 2).
Write the case with the generator tools/src/rustsheet_tools/fitlab/irr_fallback_sweep.py (extend `families()` or add a
second case next to it), then fill it on the oracle and write the roots TSV:
- `{k,1,1,-1}` for k = 4, 5, 6, 7, 42, 70 and `{-N,1,1,1,1,1}` for N = 284, 400, 496: each multiplied by 2^j for
  j = -30, -20, -10, -5, 0, 5, 10, 20, 30, 40, and each of those negated;
- `{0.3,-2.5,1.005}` times 2^j for the same j, and negated;
- column B: the same with guess 0.5 on the j = 0 and j = 20 rows.
Fewer than 300 cells. The flows are written as exact decimal literals of the scaled doubles, so the case's
values are exactly c·2^j.

## Acceptance criteria
- [ ] The case is in eval/suites/probes-numeric with Excel's values (excel-graph) and a roots TSV with each value's
      signed offset in ulps from the exact root.
- [ ] financial.md "IRR's fallback when the secant fails" records, per family, the j at which Excel's bits change
      (or that they never do), and whether each negated row equals its positive row.
- [ ] T-2928's "Blocked because" section is updated to name this evidence, so the steward can reopen it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing the engine (T-2928).
