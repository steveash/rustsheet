---
id: T-3268
title: "XIRR stop with flows dated before the first: what sets the A4 family apart (attempt 2 after T-2045)"
milestone: M9
lane: functions-financial
status: done
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
- [x] The 14 cells above leave eval/baseline (probes.txt) with no new line, or the task records each candidate
      with its lab counts (decision 0056).
- [x] Any new candidate is a named candidate in kernels/xirr.py.
- [x] `make check` green; reviewers approve.

## Out of scope
- Numeric text in XIRR's ranges (T-3267).

## Outcome
Attempt 2 fixed it, with no new probe. Splitting the 267 numeric cells of `xirr` and `xirr-oos` that have a nonzero first value by the root's sign separates them cleanly:
- On all 184 positive roots, Excel stops at the first step-test pass. That covers E7, F3, the 4-flow back-dated cells, A8's 124 years and T-1591's long spans.
- On all 83 negative roots (the A4 family, A21-A28 and the fit set's downward chain), the unscaled bound |XNPV(m)| < 2e-8·max|value| fits (feasible (1.93e-8, 2.05e-8]).

`compute_xirr` now applies the XNPV bound only when m < 0, and drops T-1583's span factor.
- Lab candidate "bisect, first-value sign, |f| bound on a negative root only, Excel ^ (T-3268)": 227/227 on `xirr` and 80/81 on `xirr-oos`. The miss is xirr-early-dates A7's input check. Before this change, T-2328's rule scored 69/81.
- Eval delta: probes 2071 → 2057 known failures (the 14 cells: whole-column-empty-walkers E7 and F3, xirr-back-dated-stop A1-A3, A6-A12, A17 and A18). 0 regressions.
- Semantics: financial.md "XIRR is a bracketing bisection", T-3268 bullet.
- Open, not separated by any cell: whether the switch is the midpoint's sign or the bracket's side, and whether a negative root spanning several years scales the bound.
