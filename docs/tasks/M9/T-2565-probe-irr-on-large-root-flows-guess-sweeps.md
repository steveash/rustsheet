---
id: T-2565
title: "Probe IRR on large-root flows: guess sweeps that pin Excel's early secant steps"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that locates where Excel's IRR run departs from the fitted secant in v
(financial.md "IRR is a secant in v") on flows whose root is far above the guess, so T-2131 can
fit the departure instead of guessing at it.

## Context
- T-2131 (blocked) found that on large-root flows Excel returns the same secant step as the model
  but with an error 0.1 to 0.4 % different, e.g. `IRR({-0.0001,2,42})` Excel 20019.97799564059,
  model 20019.977995638386; `IRR({-2.5,-1,123456789.123})` Excel 7026.083662734896, model
  7026.0836627358585 (gen-m9/c0681a8b150ce301 Calc!A218, gen-m9/67a4c03afd63f988 Calc!A346).
  The model's result is insensitive to rounding (a 1e6-ulp change of v1 does not move it) but
  moves continuously with the v offset, so Excel's early (far-from-root) steps differ from the
  model's. No NPV evaluation order, step form, offset in r or relative offset reproduces them
  (T-2131 "Blocked because", on branch agent/functions-financial/T-2131-irr-15-digits, draft PR
  #1658).
- Excel shows no iterates, but a guess places the run's start: with the guess set to the model's
  k-th iterate (g_k = 1/v_k - 1), each run is shorter and starts nearer the root. Comparing
  Excel's `IRR(flows, g_k)` with the model's own run from g_k, k = 0, 1, ..., shows from which
  distance to the root on the two methods agree, which separates a far-from-root rule (a step limit or
  damping) from a near-root one (the stop test or the returned iterate).
- The fallback path (`IRR({-400,1,1,1,1,1})` = -0.6737366751808405, `{42,1,1,-1}`,
  `{0.3,-2.5,1.005}`) is T-1686's; do not duplicate it here.

## Acceptance criteria
- [x] A probe case in `eval/suites/probes/` (excel-graph oracle) with, for each of the two flows
      above: `IRR(flows, g_k)` for every iterate v_k of the model's run from 0.1 (the
      guesses as 17-significant-digit literals, generated from the fit-lab model and listed in
      the case's hypotheses), and `IRR(flows, 0.1*(1+j*1E-4))` for j = -5..5.
- [x] The same for flows scaled by 1E6 and by -1 (the residual test's scale dependence).
- [x] financial.md's IRR section records the first step at which Excel's result departs from the
      model, with cells; T-2131's Context links the case.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the departure (T-2131).
- IRR's fallback path, caps and overflow (T-1686).

## Outcome
Case `eval/suites/probes/cases/t2565-irr-guess-sweeps` (generator `fitlab/irr_guess_sweep.py`) filled on
excel-graph. Excel departs from the model from the first guess on (no agreement distance); the large-root
"error" is Excel's v rounded to a 2^-53 grid: rounding the model's v to it matches 173/175 cells. Recorded in
financial.md; fitting it is T-2131.
The case has no `hypotheses` field: the guesses are the column-B literals, named by block in the description.
