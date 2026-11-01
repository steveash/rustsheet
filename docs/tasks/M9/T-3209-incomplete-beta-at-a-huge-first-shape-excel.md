---
id: T-3209
title: "Incomplete beta at a huge first shape: Excel's bgrat is up to 1.5E-14 off the true value (T-2528 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2510]
campaign: visible
defer: phase-1b
---

## Goal
The 22 cells of probes-numeric/t2528-f-dist-huge-df2 that are more than half a unit of the 15th digit off Excel
are within it: Excel's bratio L140 bup + bgrat arithmetic is modelled and matched (decision 0114 `visible`:
reproducing Excel's own algorithm where Excel is the inaccurate one).

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell this task owns is a BETA.DIST or
  F.DIST density cell or a dense probes-numeric sweep cell, within Gate A's envelope (most agree with Excel to 12 or
  more digits, and Excel and the engine are both close to the math), so R1(c) counts it as Phase 1b evidence. A
  point that Gate A's tables show outside the envelope gets its own task without the key.
- T-2528 replaced Cephes' non-converging fraction at a first shape of 1E6 or more and b/a at most 1E-6 with an
  accurate DiDonato-Morris expansion (`large_shape_beta.rs`), within 2E-15 of a 120-digit mpmath reference.
- Excel is within about 1E-15 of the true value for tails above 1E-4 but up to 1.5E-14 off further out
  (`F.DIST.RT(50,5,1E10)` 5.4969308523163854E-52, true 5.496930852316323E-52; `F.DIST.RT(2,5,1E9)`
  0.07523524713822176, true 0.07523524713822164). docs/excel-semantics/statistics.md "Incomplete beta at a huge first
  shape".
- The error grows with z = -nu ln x, which suggests z formed in Excel's own LN/log1p in bgrat (bratio L140 bup + bgrat
  at b0 = 1/2): T-2510's models (`bratio_excel.py` `bgrat`) are the place to test it.

- References: `docs/research/t2528/reference.py` (120-digit mpmath at the double arguments).
- `large_shape_beta.rs` is an accuracy stand-in: when T-2510's bratio bgrat lands and the old kernel goes, it is
  folded into or replaced by `bratio`'s bgrat, so the two series do not stay as parallel definitions.

## Acceptance criteria
- [ ] The 22 cells are within half a unit of the 15th digit of Excel (a model of Excel's bgrat at these points).
- [ ] No cell of t2528-f-dist-huge-df2, t1530-f-df-cut or t1538-* moves away from Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells below the mean (Cephes' incbcf side), which are T-2510's.
