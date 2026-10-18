---
id: T-1284
title: "GEOMEAN(5,7) is 1 ulp off Excel (5.916079783099616, engine ...615)"
milestone: M1
lane: functions-stat
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`GEOMEAN(5,7)` returns Excel's exact value 5.916079783099616.

## Context
`=GEOMEAN(5,7)` is 5.916079783099616 in Excel (probes/subtotal-reference-operands-d H28; excel-graph) and the engine gives 5.916079783099615. Look at `functions/statistical/geomean.rs` for how the product/log is accumulated and find the order of operations that reproduces Excel's bits across the existing GEOMEAN cases.

## Acceptance criteria
- [ ] `GEOMEAN(5,7)` matches Excel bit-exactly, with a probe case added (or the -d case extended) as evidence and no existing GEOMEAN case regressing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because: time-boxed (decision 0056)
H28 is one more cell of the in-range root class in docs/excel-semantics/numbers.md, "GEOMEAN in-range
root". Excel gives the correctly rounded sqrt(35), and the engine's r = exp(ln 35 / 2) is 0.90 ulp
below the exact root. Picking the correctly rounded root for n = 2 breaks ironcalc-calc GEOMEAN C23
(GEOMEAN(1,3)), where Excel keeps r. The distance of r from the exact root does not separate the two
outcomes either: over the 70 in-range literal GEOMEAN probe cells, r is kept at 0.548 and 0.809 ulp and
replaced at 0.546 ulp (geomean-root-refinement A22). Candidates tried: every model listed in that entry
(T-1113, T-1123, T-0288, scored on 159 or 167 cells), and a distance threshold (this task, draft PR #553, closed
by T-0059). That is the second attempt with no rise in the bit-exact count, which is decision 0056's bar.
Reopen when a new root hypothesis is recorded here, for example a reconstruction of Excel's own pow in
the style of T-1524 or T-1778. A new dependency landing does not reopen it.
