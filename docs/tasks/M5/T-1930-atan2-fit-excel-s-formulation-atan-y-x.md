---
id: T-1930
title: "ATAN2: fit Excel's formulation (atan(y/x), not libm atan2) over all four quadrants"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
ATAN2 matches Excel bit for bit on probes/asin-atan-sign-source column G and on a new probe covering every quadrant.

## Context
probes/asin-atan-sign-source G1:G120 is `ATAN2(SQRT((1-x)*(1+x)),x)` for x in -0.9999..-1E-9 (a positive first
argument). Excel equals glibc `atan(y/x)` of the quotient in all 120 cells, while libm `atan2(y, x)` (the engine's
`fn_atan2`, mathematical.rs) matches 86, so 34 are known failures (T-1797; numbers.md "ASIN's formulation").
The other quadrants are unprobed. A negative x probably adds or subtracts π, and x = 0 gives ±π/2. Probe them with a
grid over the signs of x and y (including y = 0 and -0, x = 0, and large and tiny ratios), then fit the formulation.

## Acceptance criteria
- [x] A probe covers all four quadrants and both axes, with oracle values.
- [x] The 34 G-column lines of probes/asin-atan-sign-source leave eval/baseline/probes.txt, and nothing regresses.
- [x] The formulation is recorded in docs/excel-semantics/numbers.md.
- [x] `make check` green; reviewers approve.

## Out of scope
IMARGUMENT, which may share the kernel (note it if it does).

## Outcome
`fn_atan2` now computes `ATAN(y/x)` ± π with Excel's subnormal flush, x = 0 → ±π/2, #DIV/0! for both-zero and #NUM!
for an overflowing quotient. Three new probes (atan2-quadrants, atan2-error-edges, atan2-kept-subnormal) with
excel-graph values all pass; the 34 G-column lines of probes/asin-atan-sign-source left the baseline. Recorded in
numbers.md ("ATAN2 is ATAN(y/x) plus or minus π"). IMARGUMENT sharing the kernel is unprobed.
