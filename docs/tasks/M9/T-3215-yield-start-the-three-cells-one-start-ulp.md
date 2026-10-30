---
id: T-3215
title: "YIELD start: the three cells one start-ulp off after T-3152 attempt 1 (rate-term B30, term-basis C47, one-step A37)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
yield-start-rate-term S!B30, yield-start-term-basis S!C47 and yield-one-step-start S!A37 are within 15 digits of
Excel, with no YIELD or ODDFYIELD cell added to the known failures. Otherwise attempt 2 records a new hypothesis and
its lab counts (decision 0056).

## Context
- Follow-up of T-3152 (attempt 1, decision 0056). Each of the three cells needs a start one ulp from the engine's
  `yield_start` (crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs): B30 and C47 one
  lower, A37 one higher (that one crosses a PRICE step, so it shows as 32 ulps). The fit lab reproduces all three
  (`uv run rustsheet-fitlab yield-caret`; kernels/yield_caret.py `unrounded_term_yield_start`).
- Ruled out (financial.md "YIELD stops short of PRICE's root", T-2974 attempts 1-4 and T-3152 attempt 1): a grid of
  13,824 arrangements, with each of the start's nine operations exact or rounded and the term in each product as the
  double, `whole + fl(days/year)` exact or `whole + days/year` exact. The best scores 945 of 1,100 cells, and that is
  the engine. Also ruled out: x87 intermediates, FMA placements, a perturbed or day-count term, finite-difference
  slopes, and a constant slope error.
- Misses are not per term: within the 8 + 309/360 group (yield-one-step-start), 20 cells take the engine's start and
  24 need one ulp more.
- `fn_yield_start_term_whole_years_then_leftover_days` pins A1 and A2 within one ulp. Restore their exact bits with
  the fix.

## Acceptance criteria
- [ ] The three cells are within 15 digits and `make eval-baseline` adds no known-failure line, or the task records
      each candidate tried with its lab counts (decision 0056).
- [ ] Any new candidate is a named `yield-caret` candidate in kernels/yield_caret.py.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ODDFYIELD's own slope (T-2007) and the engine's PRICE last bits (T-3144).
