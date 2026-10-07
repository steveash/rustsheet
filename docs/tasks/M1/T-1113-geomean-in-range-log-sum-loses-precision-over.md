---
id: T-1113
title: "GEOMEAN in-range log-sum loses precision over many values (1101 x 1.5)"
milestone: M1
lane: functions-stat
status: done
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
- [x] A probe with several many-value in-range sets, filled from excel-graph.
- [x] geomean-overflow-count H1 passes, and ironcalc-calc GEOMEAN does not regress.
- [x] `make check` green; reviewers approve.

## Out of scope
- The over/underflow path (T-1112, and numbers.md "GEOMEAN root rounding").

## Outcome
In range GEOMEAN is now `exp(ln(P) / n)` of the plain product, one value returned as is (was a
log-sum). New probe probes/geomean-in-range-many (72 sets, excel-graph): the model matches 67;
the five one-ulp misses beat every other tried model over all evidence (glibc pow matches 71 of 72
but regresses ironcalc-calc C23; see numbers.md "GEOMEAN in-range root"). The over/underflow path
keeps `powf` (`exp/ln` there fixes 10 probed cells but breaks 4; left to T-1108).
Eval delta: fixed 7 (ironcalc-calc GEOMEAN B27, G27; probes geomean-overflow-count H1,
geomean-overflow-cut-3 C1, C5, C11, E1), new-coverage failures 5 (the new probe), 0 regressions.
