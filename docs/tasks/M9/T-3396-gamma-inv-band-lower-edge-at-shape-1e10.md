---
id: T-3396
title: "GAMMA.INV band lower edge at shape 1E10: GAMMA.INV(1E-80,1E10,1) is #NUM! below the cdf-ratio edge"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
GAMMA.INV, CHISQ.INV and the other gamma quantiles at a huge shape are `#NUM!` exactly where Excel is, at the band's lower edge when the start lies above the mean (shapes from about 7E9).

## Context
- T-3350 replaced the band's lower edge `offset - z < 39` with a cdf ratio: Excel finds the true quantile only while
  ln P(a, x0) - ln p < 205.3 (incomplete_gamma/huge_shape.rs `BAND_LOWER_EDGE`). It separates 548 of 549 huge-shape
  quantile cells; the miss is probes/t3350-gamma-inv-band-lower-edge A122 `GAMMA.INV(1E-80,1E10,1)`, Excel `#NUM!`
  at 184.2, while A121 `(1E-70,1E10,1)` is a value at 161.2. At 7E9 and 1E10 the start's lower tail is about 1
  (the start is above the mean), so the cap there may count other steps (docs/excel-semantics/statistics.md
  "GAMMA.INV and CHISQ.INV at a huge shape").
- Probe p between 1E-70 and 1E-90 at shapes 7E9 to 1.2E10 (and CHISQ.INV at df 2a) to pin the edge there, with a
  hold-out split (decision 0188 R4).
- More of the same at 1.1E10, from T-3530's fit audit (probes-numeric/fit-audit-*, excel-graph; scored on main 4e2f5f3df after T-3350;
  docs/research/fit-audit/README.md): fit-audit-t3070-band-upper A47 `GAMMA.INV(4.11E-29,11000000000,1)`, A48
  `(3.38E-27,..)` and A49 `(2.89E-26,..)` are `#NUM!` in Excel and values in the engine. T-3409 (a model of
  Excel's search) may explain them together with A122; check it first.

## Acceptance criteria
- [ ] A filled probe pins the lower edge at shapes 7E9 to 1.2E10; the rule separates every huge-shape quantile cell
      in eval/, A122 included, or the remaining misses are listed with the reason.
- [ ] No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A model of Excel's whole search (T-3409). If T-3409's model explains A122, close this task as its duplicate.
- The band's upper edge and the closed-form branch.
