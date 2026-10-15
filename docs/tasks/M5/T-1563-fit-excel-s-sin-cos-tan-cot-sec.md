---
id: T-1563
title: "Fit Excel's SIN/COS/TAN/COT/SEC/CSC range reduction for angles near 2^27 (last digits differ)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SIN, COS, TAN, COT, SEC and CSC match Excel to the last bit for angles below 2^27, including near the cut.

## Context
probes/trig-huge-argument A1, A15, A29, A43, A57, A71 (`f(2^27-1)`): Excel differs from libm in the 10th-12th digit, so its argument reduction is inexact for large angles. Fit it with the fit lab (docs/research/numeric-kernel-map.md); see docs/excel-semantics/numbers.md "Trig of a huge angle". Code: `fn_sin` etc. in functions/math_and_trigonometry/mathematical.rs.

## Acceptance criteria
- [x] The 6 known failures of probes/trig-huge-argument are fixed and removed from eval/baseline/ without regressing any other trig case.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Excel reduces trig arguments by multiples of π₆₄/2 (x87 `FLDPI` π rounded to 64 bits), then
rounds the true function of the remainder once: SIN(x) = round(sin(x + n·δ/2)), n = round(2x/π),
δ = π − π₆₄. SEC/CSC/COT are 1 over the rounded COS/SIN/TAN. New
`functions/math_and_trigonometry/trig.rs` computes the remainder exactly as a double-double and
the series in double-double, so the final rounding is the only one. An exact-rational fit over all
672 numeric cells of probes-numeric/m9-trig-reduction and probes/trig-huge-argument matches every
one (the first WIP model, shift round(x/π)·δ, missed 137 at odd multiples of π/2). Quirk written up
in docs/excel-semantics/numbers.md "Trig of a huge angle". Eval delta: gate ironcalc-calc −45 and probes −6 known failures, 0 regressions; non-gate probes-numeric 8,756 → 9,175 passing cells (m9-trig-reduction 247 → 666 of 666), no other case changed.
