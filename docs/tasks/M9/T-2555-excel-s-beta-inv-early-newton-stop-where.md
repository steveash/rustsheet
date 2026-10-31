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
campaign: visible
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

Owned gen-m9 cells (moved from T-1538, still visibly off after it): gen-m9/146b46e74f2dac3b A131,
7c20351562a2d71d A254 and A37, 85f48d5cdfb24555 A315, 589f42e67f4b4b8d A319, 8dd2312c96effb1b A169,
f34f66fb7f81e673 A209. A37, A319 (`F.INV(1E-200,...)` at a huge df) and A169 (`BETAINV(1E-100,123456789.123,0.1,...)`)
are where Excel's search fails and its value is not the root (statistics.md, UNVERIFIED); A209 has b = 1 and a root
almost halfway between two doubles, Excel taking the far one (a tie its own exp/ln rounding decides?). Also watch:
probes-numeric t1538-f-inv-huge-df A86 (0.53 units, just past 15 digits after T-1538) and
t1538-f-quantile-switchover (all 24 cells within 2E-13 relative, none bit-exact, either side of df 1E5).

Cell-list line (gap triage 2026-10-31i: the bare hash above does not name the case to `scripts/exit_owners.py`,
which needs `gen-m9/<hash>` because the hash is not unique in the report, so A209 read as unowned; the line only
restates ownership). `F.INV.RT(Data!E7,2,(Data!$C9+Data!E2))` = `F.INV.RT(0.5,2,123456789.1229)`: Excel
0.6931471803615475, ours 0.6931471872147749; with d1 = 2 the root is (d2/2)(2^(2/d2) - 1), about 0.69314718445
(mpmath), so Excel is 6E-9 relative off it and ours 4E-9.
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A209

## Visible-parity stage
Decision 0083: the cells above are visibly off; the target is that each is within half a unit of the 15th
significant digit of Excel's value, or recorded as a DEVIATION where Excel's value cannot be derived.

## Acceptance criteria
- [ ] Visible parity: the 7 owned cells above reach 15-digit parity or are DEVIATION/UNVERIFIED entries with evidence.
- [ ] The 8 small-x BETA.INV cells of t1538-f-inv-huge-df are within half a unit of the 15th digit, or the
      stop rule is recorded as UNVERIFIED with the hypotheses ruled out.
- [ ] No regressions in any baseline; `make check` green; reviewers approve.

## Out of scope
-
