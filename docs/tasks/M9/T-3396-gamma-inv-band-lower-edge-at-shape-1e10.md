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

## Acceptance criteria
- [ ] A filled probe pins the lower edge at shapes 7E9 to 1.2E10; the rule separates every huge-shape quantile cell
      in eval/, A122 included, or the remaining misses are listed with the reason.
- [ ] No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
