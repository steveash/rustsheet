---
id: T-2528
title: "Incomplete beta: use the unreflected side when the reflected continued fraction does not converge (T-1538 split)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The regularized incomplete beta (`incomplete_beta.rs` `beta_tails`) is accurate where the reflected
side's continued fraction does not converge within its 300-step cap but the unreflected side's does.

## Context
Found during T-1538. `F.DIST(2,5,1E10,TRUE)` = `I_x(2.5, 5E9)` at x = 1E-9 is above the mean 5E-10, so
`beta_tails` evaluates the reflected `I_y(5E9, 2.5)` with y = 1 - 1E-9, whose `incbcf` fraction needs about
42,000 steps; truncated at 300 it gave 0.9247647476587724, where Excel's 0.9247647537543169 is the true value
(probes-numeric/t1530-f-df-cut A8). The unreflected fraction converges in 12 steps.

## Acceptance criteria
- [x] `F.DIST(2,5,1E10,TRUE)`, `F.DIST.RT(2,5,1E10)` and `FDIST(2,5,1E10)` (t1530-f-df-cut A8, A12, A14) are within
      about 1E-14 relative of Excel's values (they were off by 7E-9 relative); a unit test pins it.
- [x] No regressions in any baseline (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits at a huge shape (T-1538: the large-shape prefactor and Excel's BETA.INV search).

## Outcome
- `fraction` reports whether it converged; `beta_tails` falls back to the unreflected side when the reflected
  fraction does not converge and the unreflected one does (`beta_side`, `cephes_fraction`).
- Eval: no gated baseline changed (0 fixed, 0 regressions). probes-numeric (ungated) t1530-f-df-cut A8, A12, A14
  moved from 7E-9 relative to about 1E-14 relative of Excel (NumericMismatch to NumericClose).
