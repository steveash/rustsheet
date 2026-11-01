---
id: T-3409
title: "Model Excel's gamma inverse search at a huge shape: its start, step rule, step limit and give-up value"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV at a large and huge shape return Excel's value because the engine
reproduces how Excel's inverse search actually works (its start, step rule, step limit and what it returns when it gives
up), not because of more fitted cuts. In particular `CHISQ.INV.RT(1E-300,df)` for df 5E6 to 1E9 is Excel's value.

## Context
- **Owner ruling (2026-10-31, on T-3350's review): R3 is NOT widened.** Asked whether decision 0188 R3's closed-form
  case should cover these cells, the owner said: "I want you to match Excel here but not overfit. Can you just explore
  further and crack how Excel works in this region?" So the target at the cells below is Excel's value (directive 1),
  and no MATH-OVER-EXCEL case is added or widened.
- The cells: probes/t3350-chisq-inv-huge-df A205, A217, A229, A241, A253, A265, A277, A289 (`CHISQ.INV.RT(1E-300,df)`,
  df 5E6 to 1E9). Excel's value is its Wilson-Hilferty start x0 (the T-3070 constants, within 3.4E-9), 2.6E-4 to
  2.7E-4 above the 50-digit math, where the gamma density at x0 underflows. At df 2E6 (A193) Excel is the true
  quantile. The engine returns the true quantile, so these stay F lines of eval/baseline.
- **Hypothesis to test (UNVERIFIED).** Excel runs one search: it starts from x0, takes Newton-like steps, stops after a
  step limit, and on giving up returns x0 (or `#NUM!`). Three facts so far look like that one search:
  - T-3350's cut `BAND_LOWER_EDGE` (huge_shape.rs: ln P(a, x0) - ln p < 205.3) reads as Newton from the inner side of
    a convex tail giving up after about 200 steps, each dividing the tail by about e.
  - The x0 cells above look like the same give-up, where the density at x0 underflows and a step cannot move.
  - The closed-form branch from shape 1E9 (T-3070) and the band's upper edge (`BAND_UPPER_SLOPE`, `BAND_UPPER_EDGE`)
    may be the same thing.
  If one model explains all of them, it replaces the fitted cuts `BAND_UPPER_SLOPE`, `BAND_UPPER_EDGE` and
  `BAND_LOWER_EDGE`, and T-3396's miss (`GAMMA.INV(1E-80,1E10,1)`, `#NUM!` where the cut says a value) is the test of
  it.
- Evidence to start from: probes/t3350-chisq-inv-huge-df, probes/t3350-gamma-inv-band-lower-edge,
  probes-numeric/t3350-r4-chisq-inv-sweep, probes/t3070-gamma-inv-band, probes/t2982-gamma-inv-huge-shape;
  docs/excel-semantics/statistics.md "GAMMA.INV and CHISQ.INV at a huge shape"; docs/research/t3350/sweep.py (the
  mpmath reference).

## Acceptance criteria
- [ ] A written model of Excel's search (start, step rule, step limit, the value on giving up), with probes designed
      to tell it apart from the alternatives (for example p just either side of a predicted step count, and shapes
      from 1E6 to 1E10), filled by the oracle.
- [ ] Reviewer check 7 (decision 0188 R4): the model's free parameters are frozen on a training set and stated before a
      held-out set is scored; then a fresh probes-numeric sweep, drawn after the freeze, is scored against Excel and
      the 50-digit math. The Outcome puts train, held-out and fresh-sweep results side by side.
- [ ] Cells A205-A289 above are Excel's value (within 15 significant digits), and no cell of the probes above that
      matches Excel now regresses.
- [ ] Where the model replaces a fitted constant, its FITTED tag goes and quality/fitted.json's count falls; any constant
      the model still needs carries a FITTED tag with its hold-out result.
- [ ] docs/excel-semantics/statistics.md describes the model, and its status (VERIFIED only where the probes show it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Widening decision 0188 R3, or any new MATH-OVER-EXCEL case (the owner ruled against it).
- The digits of the closed form at shape 1E9 and up (T-3272) and its move to the math under R3 (T-3364), unless the
  model shows they are the same mechanism; then say so in those tasks.
- Small-shape quantiles (below shape 1E5).
