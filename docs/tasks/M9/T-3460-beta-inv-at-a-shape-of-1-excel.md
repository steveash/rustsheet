---
id: T-3460
title: "BETA.INV at a shape of 1: Excel's last bit where the closed form is 1-3 ulp off (T-3457 residue, gen-m9 589f42e67f4b4b8d A221)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3457]
defer: phase-1b
---

## Goal
Find Excel's own BETA.INV formulation at a shape of exactly 1 (p >= 1E-15) to the last bit, so the cells T-3457's
closed form leaves 1 to 12 ulp from Excel become bit-exact without moving any cell off 15 digits.

## Context
- T-3457 replaced the crossing search at a shape of 1 with the libm closed form (p; -expm1(log1p(-p)/beta);
  exp(ln(p)/alpha)). Every shape-1 cell is within 15 digits, but Excel is not that closed form to the bit
  (docs/excel-semantics/statistics.md "BETA.INV at a shape of 1 is the closed form to 15 digits for p of 1E-15 or more
  (T-3457)").
- Cells the closed form moved from bit-exact to 1 ulp (T-3457's run, decision 0083's standing allowance, condition 4):
  - gated: gen-m9/589f42e67f4b4b8d Calc!A221 `BETAINV(0.001,1,Data!A2)` with Data!A2 = 42: Excel
    2.3821152785727803E-5, ours 2.3821152785727806E-5.
  - probes-numeric/m9-beta-inv Calc!A12, A13, A40, A41, A55, A57, A83, A117 and their
    probes-numeric/t1745-beta-inv-roundtrip rows J11, J12, J39, J40, J53, J55, J80, J113; probes-numeric/
    t1543-inverse-cdf-tiny-p Calc!A90 `BETA.INV(1-1E-10,1000,1)`.
- Cells still off after T-3457 (1 to 12 ulp): m9-beta-inv A43, A44, A59, A66, A67, A68, A69, A78, A113 and t1745 J41,
  J42, J57, J64, J65, J66, J67, J75, J109; ironcalc-calc BETADIST_BETAINV.xlsx H31;
  probes/t2435-dist-extremes A72.
- Hints: A12 `BETA.INV(0.95,0.5,1)` is 0.9024999999999999 in Excel, not 0.95^2, so Excel does not square; the
  crossing of our cdf matched these 18 cells, so a crossing of a cdf that is not flat (an accurate shape-1 cdf, or
  the closed form polished by one Newton step on Excel's own BETA.DIST) is a candidate.

## Acceptance criteria
- [ ] A candidate formulation chosen with the R4 guards of reviewer-excel-fidelity check 7 (decision 0188): hold-out
      split fixed before scoring, fresh oracle sweep after the freeze, Gate A envelope result, `FITTED` tag and
      range if fitted, reproducibility floor, no keep-matched branch.
- [ ] gen-m9/589f42e67f4b4b8d A221 is bit-exact again and no shape-1 cell leaves 15 digits; known failures only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- p below 1E-15 (T-2808, T-1871); shapes other than exactly 1 (T-3388, T-3398).
