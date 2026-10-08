---
id: T-1156
title: "GEOMEAN of 1500+ copies of 2^1000 is #NUM!: find the rule"
milestone: M1
lane: functions-stat
status: done
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
- [x] A bracketing probe filled from excel-graph; the rule goes in numbers.md.
- [x] The probe also checks T-1120's unprobed extrapolations: a positive exponent sum with a significand
  product above 1 that wraps (1100+ copies of 2^1000 * 1.0001) and an in-range plain product whose
  significand product wraps (1100 to 3000 copies of 1.0001); the engine follows whatever Excel shows.
- [x] The engine matches probes/geomean-exponent-sum-2 C12-C18 and the new cells, with no regression.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- probes/geomean-pow2-count (excel-graph, 2026-10-08; 107 cells): 2^e for e = 10-1023 at 500-3000
  copies, computed in the formula, is exact at every count. So the `#NUM!` of
  geomean-exponent-sum-2 C12-C18 is the input, not GEOMEAN: spec2xlsx (XlsxWriter) stores numbers
  with 16 significant digits, so the stored 2^1000 reaches Excel three ulps low, with significand
  1.99999999999999933, and the existing significand cut makes 1025+ copies `#NUM!`. The eval harness
  now feeds the engine spec numbers with the same 16 digits (`as_written` in
  crates/rustsheet-eval/src/engine.rs, documented in eval-harness.md). C12-C18 pass, and no other
  spec cell changed.
- T-1120's extrapolations probed: the 2048*k wrap applies only when the power-of-two sum is negative
  and the significand product is at least 2. Positive sums (2^1000 times 1.0001, 1.001 and 1.1) and
  2^-1000 * 1.0001 never wrap; 2^-1000 times 1.001, 1.01 and 1.1 do. An in-range product (1.0001) never
  wraps. `exponent_wrap_excess` takes both conditions; numbers.md records them.
- Eval delta: probes known failures 363 -> 359 (fixed C12-C18 and the 10 G/I cells; new coverage 4:
  three one-ulp root misses, L4, M1, M4, the known "GEOMEAN root rounding", and the case's C line);
  regressions 0.
