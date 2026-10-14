---
id: T-1563
title: "Fit Excel's SIN/COS/TAN/COT/SEC/CSC range reduction for angles near 2^27 (last digits differ)"
milestone: M2
lane: functions-math
status: todo
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
- [ ] The 6 known failures of probes/trig-huge-argument are fixed and removed from eval/baseline/ without regressing any other trig case.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
