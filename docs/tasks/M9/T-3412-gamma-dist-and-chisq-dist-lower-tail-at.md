---
id: T-3412
title: "GAMMA.DIST and CHISQ.DIST lower tail at a shape above 1: Excel's leading-factor series form (T-3060's), a staircase in x (304 of 330 probed cells, ours 46)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
GAMMA.DIST, GAMMADIST, CHISQ.DIST and CHIDIST's lower tail at a shape above 1 and x below 1 is Excel's leading-factor
series form, as T-3060 made it below shape 1, so the probe cells T-3311 left visibly off reach Excel's value within
half a unit of the 15th digit; or the Outcome lists the forms and regions tried with their counts (decision 0056).

## Context
T-3311 (docs/excel-semantics/statistics.md, T-3311 paragraphs of "Deep lower tail at a shape above 1") found that
Excel's lower-tail cdf at shapes 1.5 to 15 is `EXP(a*LN(x))/EXP(GAMMALN(a+1))` times e^-x and the series (T-3060's
form, `incomplete_gamma::excel_form_lower`). It is within an ulp at 304 of the 330 probed cells
(probes-numeric/t3311-gamma-inv-lower-tail-cdf, sheet S columns B-J and L, sheet W columns C-E), where the engine's
accurate pair is within an ulp at 46. LN(x) moves by an ulp only every few ulps of x, so Excel's cdf is a staircase,
flat across ±4 ulp of x around the probed quantiles; the pair is smooth. The engine's GAMMA.DIST takes the form only
at a shape below 1 (`excel_small_shape_lower`, where a·LN(x) is at most -16). One ordering variant,
`lead * (EXP(-x) * series)`, reaches 307 of 330, and the form's own 1-ulp residue is T-3242's.

Cells (the GAMMA.DIST and CHISQ.DIST cells of the T-3311 probes that T-3311's PR leaves visibly off or moves farther
from Excel are listed in T-3311's Outcome; they are evaluated at Excel's quantiles, which moved with T-3311).

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value;
bit-level misses left then go to a `defer: phase-1b` follow-up.

## Acceptance criteria
- [ ] The listed probe cells are within half a unit of Excel's 15th digit, or the Outcome lists the forms and regions
      tried with their bit-exact and visible counts on a hold-out split (decision 0188 R4).
- [ ] The region where the form applies is the measured one (shapes, x and a·LN(x)), with a `FITTED` tag if chosen by
      scoring; no gated cell regresses; no new known-failure line in eval/baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The quantiles (T-3311, T-3403, T-3294); shapes below 1 (T-3060, T-3242).
