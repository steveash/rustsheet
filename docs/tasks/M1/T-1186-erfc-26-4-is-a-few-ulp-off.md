---
id: T-1186
title: "ERFC(26.4) is a few ulp off Excel in the erfc continued-fraction tail"
milestone: M1
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ERFC and ERFC.PRECISE match Excel bit-exactly in the far tail (x of about 26).

## Context
Kernel map (T-1522, decision 0056): rank 2 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
probes/subnormal-function-boundaries cell A5: `ERFC(26.4)` is 4.4017768588035533E-305 in Excel (excel-graph, T-1180), the engine gives 4.401776858803543E-305 (about 2E-15 relative). `erfc_cf` in crates/rustsheet-engine/src/functions/statistical/special.rs is the tail path. Numeric fitting against Excel's cached values, hence tier opus.

## Acceptance criteria
- [ ] Cell A5 of probes/subnormal-function-boundaries passes, with no regression in ERFC cells elsewhere (`make eval-baseline` records it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the subnormal flush (T-1180).
