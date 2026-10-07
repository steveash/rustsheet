---
id: T-1099
title: "GEOMEAN underflow past 500 values is one ulp off"
milestone: M1
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GEOMEAN over more than 500 values whose plain product underflows matches Excel bit for bit.

## Context
probes/geomean-overflow-many-values F4 (`GEOMEAN(D1:D600)`, 600 cells alternating 1E-300 and 3E-250): Excel 1.7320508075688772E-275, engine 1.7320508075688775E-275 (T-1095). The other four cells (over/under 500 values, overflow) pass. The engine's fallback root for r > 500 in `fn_geomean` (functions/statistical/geomean.rs) is `product^(1/n) * 2^(r/n)`; try other formulations until F4 matches without breaking F1, F2, F3, F5 or probes/geomean-mixed-magnitude.

## Acceptance criteria
- [ ] probes/geomean-overflow-many-values F4 passes; baseline shrinks; no other cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
