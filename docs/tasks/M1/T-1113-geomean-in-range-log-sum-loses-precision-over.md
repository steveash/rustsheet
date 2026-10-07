---
id: T-1113
title: "GEOMEAN in-range log-sum loses precision over many values (1101 x 1.5)"
milestone: M1
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
GEOMEAN of many in-range values matches Excel: `GEOMEAN` over 1101 copies of 1.5 is exactly 1.5.

## Context
Found in T-1108 (probes/geomean-overflow-count H1, excel-graph, 2026-10-07): Excel gives 1.5, while the
engine's in-range path `exp(sum(ln v) / n)` gives 1.499999999999995, which is 5E-15 low
(crates/rustsheet-engine/src/functions/statistical/geomean.rs). The log-sum was kept in range because it
matched ironcalc-calc GEOMEAN C23 where an n-th root of the product did not, so the fix must keep C23
passing. Candidates: Excel's own `^` on the product (see numbers.md, "GEOMEAN root rounding"), a
compensated log-sum, or a product root for this case. Probe a few more in-range sets with many values first.

## Acceptance criteria
- [ ] A probe with several many-value in-range sets, filled from excel-graph.
- [ ] geomean-overflow-count H1 passes, and ironcalc-calc GEOMEAN does not regress.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The over/underflow path (T-1112, and numbers.md "GEOMEAN root rounding").
