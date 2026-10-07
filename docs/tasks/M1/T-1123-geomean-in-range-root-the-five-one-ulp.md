---
id: T-1123
title: "GEOMEAN in-range root: the five one-ulp misses and single-value probes"
milestone: M1
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GEOMEAN's in-range root matches Excel on probes/geomean-in-range-many A2 and B16-B19 (50 and 200 copies of 3 and 7 are exactly 3 and 7 in Excel), without losing the 67 cells and ironcalc-calc GEOMEAN it matches today.

## Context
T-1113 made the in-range root `exp(ln(P)/n)` (one value returned as is), a best fit: numbers.md "GEOMEAN in-range root" lists the models tried and their scores (glibc pow 71/72 but breaks ironcalc-calc C23 and G27; extended-precision ln/exp miss C23 and G27 too). The single-value rule rests on one cell (`GEOMEAN("47")`); probe `GEOMEAN(3)`, `GEOMEAN(0.1)`, `GEOMEAN(1E-300)` and a few one-cell ranges. Related: the overflow-path root (T-1108). Code: crates/rustsheet-engine/src/functions/statistical/geomean.rs.

## Acceptance criteria
- [ ] Single-value GEOMEAN probed (excel-graph) and recorded in numbers.md.
- [ ] A model that fixes some of the five misses with no regression, or a numbers.md note of the further models rejected.
- [ ] `make check` green; reviewers approve.

## Out of scope
The over/underflow path (T-1108).
