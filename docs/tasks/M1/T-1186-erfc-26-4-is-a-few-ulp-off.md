---
id: T-1186
title: "ERFC(26.4) is a few ulp off Excel in the erfc continued-fraction tail"
milestone: M1
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1916]
---

## Goal
ERFC and ERFC.PRECISE match Excel bit-exactly in the far tail (x of about 26).

## Context
Kernel map (T-1522, decision 0056): rank 2 in `docs/research/numeric-kernel-map.md`.
probes/subnormal-function-boundaries cell A5: `ERFC(26.4)` is 4.4017768588035533E-305 in Excel (excel-graph, T-1180), the engine gives 4.401776858803543E-305 (about 2E-15 relative). `erfc_cf` in crates/rustsheet-engine/src/functions/statistical/special.rs is the tail path.
The tail is now known exactly: T-1778 (docs/research/t1778-erf-kernel-research-plan.md, research updates 4 and 7) gives ERFC(x) = fl(E*R) for 4 < x <= 26.55, with E = Excel's EXP(-fl(x*x)) and R a stored-Horner (8,9) rational. A continued fraction like `erfc_cf` cannot match it. T-1916 implements that model, and A5 should pass with it, so this task waits on T-1916 and closes with it.
Draft PR #436 (closed by T-0059) dropped the compensation for the rounding of x*x in `erfc_cf` on its own. That fixed no gated cell (A5 went to 1 ulp) and lost 3 probes-numeric/m9-norm-s-inv cells, so it was not landed.

## Acceptance criteria
- [x] Cell A5 of probes/subnormal-function-boundaries passes, with no regression in ERFC cells elsewhere (`make eval-baseline` records it).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the subnormal flush (T-1180).

## Outcome
Closed by T-1916: ERFC now runs Excel's own kernels (crates/rustsheet-engine/src/functions/statistical/erf.rs). `ERFC(26.4)`
is 4.4017768588035533E-305 and probes/subnormal-function-boundaries passes 30 of 30; no ERFC cell regressed.
