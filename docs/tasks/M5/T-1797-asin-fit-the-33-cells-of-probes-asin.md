---
id: T-1797
title: "ASIN: fit the 33 cells of probes/asin-argument-sweep left after T-1605 (the non-odd kernel)"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ASIN matches Excel on more of probes/asin-argument-sweep's 33 remaining known failures, or the attempt is recorded and the task blocked as time-boxed (decision 0056).

## Context
T-1605 (attempt 1 of the time-box) added the tiny-argument identity (`ASIN(±1E-13)`; 235/268). The 33 misses are in
numbers.md "ASIN of a small argument": 1-2 ulp, mostly negative arguments in -0.93..-0.82 and -0.67..-0.61, with ± pairs that
differ (`ASIN(1E-8)` is 1.0000000000000002E-8 but `ASIN(-1E-8)` -1E-8). T-1605's rejected candidates are listed there.
Not yet tried: `libm::atan` (the `libm` crate) in place of glibc's; the fit lab's x87 FPATAN model (T-1524); a shared kernel
with ACOS (T-0939); an atan kernel whose reduction is not sign-symmetric other than floor/ceil/round/trunc tables (tried).
A probe of the band -0.95..-0.80 at a 0.001 step, with the same positives, would show where the kernel switches.
Code: `fn_asin` (functions/math_and_trigonometry/mathematical.rs).

## Acceptance criteria
- [x] A fit-lab fit (`asin`) over the sweep with every candidate's counts recorded; the winner implemented if it gains cells with no regression.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Excel's ASIN is `atan(x / sqrt(y*(2-y)))` with `y = 1-x`, where the root rounds twice (`extended_sqrt`). That fits
all 570 cells of probes/asin-argument-sweep and the new probes/asin-negative-band-sweep. Rounding y drops the last bit of a
negative x, so ASIN is not odd. The tiny-argument cut T-1605 added is gone: the formula gives those cells too.
- Fit lab `asin` (tools/.../kernels/asin.py), bit-exact out of 570: the winner 570; the same with a plain sqrt 567;
  the old engine 468; `atan(x/sqrt((1-x)(1+x)))` 466; `atan(x/sqrt(1-x*x))` 408; libm asin 390; atan2 388;
  half-angle 340; `pi/2 - acos` 217. A grid of 1-x, 1+x, product, root and quotient roundings (double, 64-bit, twice)
  with glibc or a correctly rounded atan: at most 218 of the 251 non-tiny sweep cells.
- New probes (excel-graph, 2026-10-18):
  - asin-negative-input-read rules out Excel's read of stored negatives.
  - asin-atan-sign-source shows Excel's ATAN is odd and equals glibc, and that Excel's ACOS is `PI()/2 - ASIN(x)` in all 240 cells (T-0939).
  - The same probe shows ATAN2 with a positive first argument is `atan(y/x)` (follow-up T-1930).
- Eval: probes fixed 33 (all of asin-argument-sweep's known failures), 0 regressions. There are 117 new-coverage known
  failures in asin-atan-sign-source: 83 ACOS cells (T-0939) and 34 ATAN2 cells (T-1930).
