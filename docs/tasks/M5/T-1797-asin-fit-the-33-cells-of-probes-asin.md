---
id: T-1797
title: "ASIN: fit the 33 cells of probes/asin-argument-sweep left after T-1605 (the non-odd kernel)"
milestone: M5
lane: functions-math
status: todo
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
- [ ] A fit-lab fit (`asin`) over the sweep with every candidate's counts recorded; the winner implemented if it gains cells with no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
