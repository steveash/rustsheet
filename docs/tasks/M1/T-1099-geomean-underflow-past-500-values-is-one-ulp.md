---
id: T-1099
title: "GEOMEAN of 600 mixed-magnitude underflowing values is one ulp off"
milestone: M1
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GEOMEAN over 600 cells alternating 1E-300 and 3E-250 matches Excel bit for bit.

## Context
probes/geomean-overflow-many-values F4 (`GEOMEAN(D1:D600)`, 600 cells alternating 1E-300 and 3E-250): Excel 1.7320508075688772E-275, engine 1.7320508075688775E-275 (T-1095). The other six cells (F1 to F3, F5 to F7) pass. F4 has remainder r = 0 and q = -913, so it takes the ordinary `(product * 2^r)^(1/n) * 2^q` path in `fn_geomean` (functions/statistical/geomean.rs), not the r > 500 fallback; the miss is in the root or the scaling by 2^q for a large negative q. Fix it without breaking F1 to F3, F5 to F7 or probes/geomean-mixed-magnitude.

## Acceptance criteria
- [ ] probes/geomean-overflow-many-values F4 passes; baseline shrinks; no other cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
