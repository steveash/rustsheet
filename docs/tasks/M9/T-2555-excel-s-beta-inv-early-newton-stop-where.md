---
id: T-2555
title: "Excel's BETA.INV early Newton stop where the quantile is tiny at a huge shape (T-1538 split)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Excel's BETA.INV (and F.INV/F.INV.RT through it) at a huge shape match Excel where the quantile is tiny
(a small first shape, a huge second one, p <= 1/2), e.g. `BETA.INV(0.025,2.5,5E9)` = 8.312123853710714E-11
and `F.INV(0.05,3,1E8)` = 0.11728210553634685.

## Context
Split from T-1538. Excel's BETA.INV is AS 109's structure (p > 1/2 is 1 - BETA.INV(1-p,b,a), bit for bit),
and where the quantile is near 1 it is the converged, correctly rounded root, which T-1538 implements
(`inverse_beta.rs` `beta_quantile_crossing`). Where the quantile is tiny (x near 1E-10 at b = 5E9) Excel's
value is near the second Newton iterate from AS 109's start (1E-12 to 4E-8 relative from ours) and up to 1E-6
relative off the true root (8 of the 24 BETA.INV cells of probes-numeric/t1538-f-inv-huge-df: A133, A135,
A137, A139, A145, A153, A155; A147 within 1 ulp). Neither a fixed acu (1E-26 to 1E-40), statrs' acu, nor
ln B from double lgamma reproduces it; the start formula, the density in the Newton step and the stop
need fitting (scratch analysis in the T-1538 PR). Owned visibly-off gen-m9 cells that stay after T-1538
are listed in its Outcome (those whose beta variable is tiny, e.g. gen-m9/146b46e74f2dac3b A131).

## Acceptance criteria
- [ ] The 8 small-x BETA.INV cells of t1538-f-inv-huge-df are within half a unit of the 15th digit, or the
      stop rule is recorded as UNVERIFIED with the hypotheses ruled out.
- [ ] No regressions in any baseline; `make check` green; reviewers approve.

## Out of scope
- `F.INV(1E-200,...)` at a huge df (Excel's search fails outright there; T-1538 notes).
