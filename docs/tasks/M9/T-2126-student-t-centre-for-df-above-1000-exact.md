---
id: T-2126
title: "Student-t centre for df above 1000: exact tail without the rounded-y prefactor"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---


## Goal
T.DIST/T.DIST.RT/T.DIST.2T away from the deep tail (t^2 < n) are within an ulp of the exact value
for df above 1000 (incl. Welch's fractional df in T.TEST), as they are below it since T-0969.

## Context
T-0969 (`functions/statistical/student_t.rs`) sums the integer-df series in double-double up to
df 1000; above it the centre falls back to `beta_reg_upper` (Cephes' fraction), whose prefactor
uses ln y with y = n/(n+t^2) rounded, so it loses about |n ln y| eps: `T.DIST(1,1E6,TRUE)` is
76,677 ulp off (Excel 0.8413446250832111, within 1 ulp of exact; T-0969 attempt 2), and at df 1E10
`T.DIST(2,1E10,TRUE)` is a NumericMismatch (probes-numeric m9-t-edges). Remedy to try: the
prefactor from log1p(-z) or from -log1p(t^2/n) in double-double, or a normal-approximation
expansion (Cornish-Fisher / Hill's asymptotic) for very large n; score against probes-numeric
`m9-t-dist-cdf` (df 1000, 1E6), `m9-t-edges` and `m9-t-nonint-df` with the fit lab first.

## Acceptance criteria
- [ ] probes-numeric m9-t-dist-cdf: every df 1E6 cell within 4 ulp (now 76,677 worst).
- [ ] m9-t-edges: no T.DIST/T.DIST.RT/T.DIST.2T NumericMismatch at df 1E10.
- [ ] No new baseline lines; `make check` green; reviewers approve.

## Out of scope
- Excel's tail noise at moderate df (T-0969's remaining F7/F8/C7/C8).

## Progress (2026-10-20, T-0969 attempt 4)
`centre_tail` in `student_t.rs` implements this task's remedy for integer df: the fractions on z, t^2/n and n/t^2.
- m9-t-edges has no NumericMismatch left at df 1E10.
- The df-1E6 cells of m9-t-dist-cdf are within a few ulp of the exact value. The ones still more than 4 ulp from Excel
  (`T.DIST(-10,1E6,TRUE)`, A150) are Excel's own noise.

Open: Welch's fractional df keeps the plain fraction, because the new path moved t1048-ttest A3 and t1049-ttest A21 the
wrong way. This task now covers fractional df only.

## Cell found by T-2217 (2026-10-28)
gen-m9/01523d247594952e Calc!A218 `=RRI(T.DIST.RT(TIME(5,2,16),1E+8),8370,40876)` is Excel 43.89282054161517, ours
43.892820541615116 (visibly off). RRI fits every probed point since T-2217; the miss is the nper: ours is
`T.DIST.RT(0.2099...,1E8)` = 0.4168699705898027, and Excel's RRI value needs an nper about 2 ulp lower (RRI moves about 3
result ulp per nper ulp here). So the cell passes once T.DIST.RT at df 1E8 in the centre is within an ulp of Excel.
