---
id: T-4954
title: "Excel's far-tail Q at shape 5 x = 42 and shape 3/2 (T-3469 split)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3469]
campaign: visible
---

## Goal
The three cells of T-3469's "Second extension" that the direct-deviance exponent did not move, and the six of
T-3469's fresh sweep the form misses, match Excel within 15 digits, or a probe recorded in statistics.md shows why not.

## Context
- T-3469 switched `incomplete_gamma/excel_tail.rs::excel_far_upper`'s prefactor exponent from the ratio form
  a ln(x/a) + (a - x) to Loader's direct deviance a ln(a/x) + (x - a) (docs/excel-semantics/statistics.md, "Correction
  (T-3469)" under "Excel's far upper tail (T-3269)"). That fixed or narrowed every cell T-3469 named except these,
  still visibly off on the T-3469 head:
  - probes-numeric/t3269-shape6-misses Calc!A69 `POISSON(4,42,TRUE)` = Q(5, 42), 6 ulp;
  - probes-numeric/t3269-fresh-sweep-2 Calc!A173 `CHISQ.DIST.RT(69.988,3)` (17 ulp) and A233
    `CHISQ.DIST.RT(61.3502,3)` (21 ulp), shape 3/2.
- Q(6, 42) is bit-exact with the direct deviance and Q(5, 42) is not, so the residue is not the log argument alone.
  At shape 3/2 the form may not be Excel's at all (half-integer shapes, T-3461 owns [a + 1, 30]).
- Lead: a dense probe of adjacent doubles around x = 42 at shapes 4.5 to 5.5, and around x = 61.35 and 69.99 at
  shapes 1 to 2, to find whether the miss follows the exponent's rounding or a different kernel.

Cells this task owns:
- probes-numeric/t3269-shape6-misses.case.json (1): Calc!A69
- probes-numeric/t3269-fresh-sweep-2.case.json (2): Calc!A173, A233
- probes-numeric/t3469-fresh-sweep-3.case.json (6, T-3469's R4.2 sweep inside the form's range, ulps of Excel's
  value, the same on the ratio form): shape 3/2 Calc!A29 `CHISQ.DIST.RT(61.7076,3)` (18), A144
  `CHISQ.DIST.RT(66.6632,3)` (22), A174 `CHISQ.DIST.RT(65.0478,3)` (28); shape 5/2 A27 `CHISQ.DIST.RT(444.398,5)` (5),
  A175 `CHISQ.DIST.RT(66.124,5)` (5); shape 10 next to the cut A5 `POISSON(9,30.146,TRUE)` (9; 22 on the ratio form)

## Acceptance criteria
- [ ] The cells above within 15 digits of Excel, or a probe recorded in statistics.md showing why not.
- [ ] Any change passes decision 0188 R4, and no cell regresses (main-vs-HEAD per cell).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Half-integer shapes on [a + 1, 30] (T-3461); the far-tail form above shape 11 and past x = 600 (T-3640).
