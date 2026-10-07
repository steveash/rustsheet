---
id: T-1095
title: "Probe GEOMEAN overflow path with more than 500 values"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for GEOMEAN when the plain product overflows and there are more than 500 values.

## Context
From the T-1076 reviews. In `fn_geomean` (functions/statistical/geomean.rs) the remainder `r` of the
power-of-two exponent goes under the root only up to 500; past that (possible only with over 500 values)
the root is taken as `product^(1/n) * 2^(r/n)`, whose last digit is unprobed. Use the `excel-probe`
skill: e.g. GEOMEAN over a range of 600 cells holding 1E+300 and a few mixed magnitudes such as 600
cells alternating 1E+300 and 3E+250 (generate them with a formula column so the case stays small).

## Acceptance criteria
- [ ] A probe case with an oracle sidecar for at least two >500-value overflow GEOMEANs.
- [ ] numbers.md records the result; an engine mismatch is filed as a functions-stat task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
