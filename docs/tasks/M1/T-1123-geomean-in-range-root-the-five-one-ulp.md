---
id: T-1123
title: "GEOMEAN in-range root: the five one-ulp misses and single-value probes"
milestone: M1
lane: functions-stat
status: done
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
- [x] Single-value GEOMEAN probed (excel-graph) and recorded in numbers.md.
- [x] A model that fixes some of the five misses with no regression, or a numbers.md note of the further models rejected.
- [x] `make check` green; reviewers approve.

## Out of scope
The over/underflow path (T-1108).

## Outcome
- Probed (excel-graph, 2026-10-08): probes/geomean-single-and-copies (one value in 15 forms; n
  copies of 3, 7, 0.1, 1.1, 1.5 with Excel's own PRODUCT, EXP(LN(P)/n), P^(1/n), LN(P)) and
  probes/geomean-root-refinement (48 sets separating root models). A single value is returned
  unchanged in every form. Excel's LN and EXP equal glibc's bit for bit, and an exact root is exact
  (GEOMEAN(1,9) = 3), so the old misses are not a libm difference and not an all-equal shortcut.
- The in-range root now snaps `exp(ln(P)/n)` to its 15-digit value when that value, multiplied n
  times, gives P back. That fixes geomean-in-range-many B16-B19 (4 of the 5 misses) and the
  exact-root cells of the new probes, with no regression in any suite. A2 is still one ulp off. The Newton, correctly rounded,
  all-equal and scaled models were rejected; numbers.md "GEOMEAN in-range root" records them.
- Eval delta: probes known failures 166 -> 363 (fixed 4; new coverage 201: 175 cells from PRODUCT
  of an array argument returning #N/IMPL!, filed as T-1165, and 26 one-ulp GEOMEAN misses, filed as
  T-0288); regressions 0.
