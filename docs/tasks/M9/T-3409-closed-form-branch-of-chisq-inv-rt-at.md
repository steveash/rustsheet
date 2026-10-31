---
id: T-3409
title: "Closed-form branch of CHISQ.INV.RT at 1E-300 below shape 1E9: confirm the MATH-OVER-EXCEL region (decision 0188 R3)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The owner rules whether `CHISQ.INV.RT(1E-300,df)` for df from 5E6 to 1E9, where Excel returns its Wilson-Hilferty start (about 2.6E-4 above the true quantile), is in decision 0188 R3's GAMMA.INV closed-form case. The engine already returns the math there.

## Context
- probes/t3350-chisq-inv-huge-df A205, A217, A229, A241, A253, A265, A277, A289 (`CHISQ.INV.RT(1E-300,df)` at df 5E6
  to 1E9): Excel's value is its closed-form start x0 to within 3.4E-9 (the T-3070 constants), 2.6E-4 to 2.7E-4 above
  the 50-digit math, and the gamma density at x0 is below the smallest normal double, which is the rule of R3's case 3
  ("GAMMA.INV at a huge shape, closed-form branch"). But that entry's evidence and the engine's huge-shape path start
  at shape 1E9 (df 2E9). The engine returns the math at these cells (T-3350 left them so); they stay F lines of
  eval/baseline.
- Decision 0188 R3: an agent may not add or widen a case without an owner ruling.

## Acceptance criteria
- [ ] A decision record with the owner's ruling: either the case's region includes shapes from 2.5E6 (and the
      cells are R3-marked when T-3358 lands), or the engine reproduces Excel's x0 there (needs a fit for shapes
      below 1.5E10, decision 0188 R4).
- [ ] docs/excel-semantics/statistics.md's MATH-OVER-EXCEL entry is amended to match.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
