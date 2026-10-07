---
id: T-1120
title: "GEOMEAN of 2400 x 1E-300 is 1.8066704015823645E-300: model Excel's exponent sum"
milestone: M1
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
GEOMEAN of 2400 copies of 1E-300 gives Excel's 1.8066704015823645E-300, and the rule behind it is in numbers.md.

## Context
Found in T-1112 (probes/geomean-overflow-cut-3 C7, excel-graph): GEOMEAN of 2400 copies of 1E-300 is
1.8066704015823645E-300, which is 1E-300 * 2^(2048/2400). So Excel's power-of-two sum is 2048 too high
there, while 1100 copies of 1E-300 give exactly 1E-300 (probes/geomean-overflow-cut row 24). The
significand product, 1.339^2400 = 2^1011, is still finite. Probe counts between 1100 and 2400 and other
small magnitudes (5E-205, 1E-200, 1E-100) to find where the 2048 appears. A wrap or clamp of an exponent
counter (an underflow rescale step of 2^2048?) is a guess to test. Engine: `fn_geomean`
(crates/rustsheet-engine/src/functions/statistical/geomean.rs); numbers.md "GEOMEAN is #NUM! when the
product of significands overflows".

## Acceptance criteria
- [ ] A probe that brackets where the extra power of two appears, filled from excel-graph; the rule goes in numbers.md.
- [ ] The engine matches probes/geomean-overflow-cut-3 C7 and the new cells, with no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
