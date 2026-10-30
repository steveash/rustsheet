---
id: T-3268
title: "XIRR stop with flows dated before the first: what sets the A4 family apart (attempt 2 after T-2045)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
XIRR's stop fits flows dated before the first: whole-column-empty-walkers Sheet1!E7 and F3 and
probes/xirr-back-dated-stop A1-A3, A6-A12, A17 and A18 return Excel's bits. No XIRR cell that passes today may
regress, xirr-xnpv-whole-column-nogaps A4 among them. Otherwise attempt 2 records a new hypothesis and its lab
counts (decision 0056).

## Context
- Follow-up of T-2045 (attempt 1, decision 0056). Semantics: financial.md "XIRR is a bracketing bisection, not
  Newton", T-2045 bullet.
- Finding so far: 25 of the 28 literal probe cells stop at the first iterate past the step test. The A4 family
  (nogaps A4, probe A4, A5, A23) takes one step more. A4 and A6 differ only by one 1100 flow at the latest date, and
  no |XNPV| bound normaliser fits both (13 ruled out).
- Lab: `uv run rustsheet-fitlab xirr-oos` (81 cells). The T-2045 candidates (the span from the earliest date)
  score 76/81 and miss only the A4 family and xirr-early-dates A7 (the input check). The engine's T-2328 rule
  scores 69/81. kernels/xirr.py.
- Hypothesis to test: what sets the A4 family apart (three flows, the back-dated flow last, a negative root). Probe
  one change at a time: move the back-dated flow earlier in the list, add a fourth flow at the first date, and add
  positive-root twins. Then fit the |f| test on the cells where it bites.
- Engine: `compute_xirr` in crates/rustsheet-engine/src/functions/financial/financial_util.rs.

## Acceptance criteria
- [ ] The 14 cells above leave eval/baseline (probes.txt) with no new line, or the task records each candidate
      with its lab counts (decision 0056).
- [ ] Any new candidate is a named candidate in kernels/xirr.py.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Numeric text in XIRR's ranges (T-3267).
