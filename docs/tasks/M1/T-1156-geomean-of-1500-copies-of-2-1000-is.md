---
id: T-1156
title: "GEOMEAN of 1500+ copies of 2^1000 is #NUM!: find the rule"
milestone: M1
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GEOMEAN of many copies of a large power of two gives Excel's `#NUM!` (2^1000 x 1500 and more) while staying finite where Excel is, and the rule is in numbers.md.

## Context
Found in T-1120. probes/geomean-exponent-sum-2 C12-C18: 1500 to 3000 copies of 2^1000 are `#NUM!` in Excel,
1000 copies are finite (two doubles below 2^1000, C11), and 2 x 1101 / 2 x 2000 are exactly 2
(geomean-overflow-count F1, geomean-overflow-cut C4, -cut-2 C18); 1770 copies of 1E+300 (larger exponent
sum, significand 1.49) are finite. So it is neither the exponent sum alone nor the T-1120 2048 wrap
(numbers.md, "GEOMEAN is #NUM! when the product of significands overflows"). Probe powers of two 2^e for
several e (10, 100, 500, 900, 1000, 1023) across counts, and non-powers with a significand just above 1
(2^1000 * 1.0001), to bracket the cut. Engine: `fn_geomean`
(crates/rustsheet-engine/src/functions/statistical/geomean.rs).

## Acceptance criteria
- [ ] A bracketing probe filled from excel-graph; the rule goes in numbers.md.
- [ ] The probe also checks T-1120's unprobed extrapolations: a positive exponent sum with a significand
  product above 1 that wraps (1100+ copies of 2^1000 * 1.0001) and an in-range plain product whose
  significand product wraps (1100 to 3000 copies of 1.0001); the engine follows whatever Excel shows.
- [ ] The engine matches probes/geomean-exponent-sum-2 C12-C18 and the new cells, with no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
