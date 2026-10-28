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
defer: phase-1b
---

## Goal
`GEOMEAN(5,7)` returns Excel's exact value 5.916079783099616.

## Context
`=GEOMEAN(5,7)` is 5.916079783099616 in Excel (probes/subtotal-reference-operands-d H28; excel-graph) and the engine gives 5.916079783099615. Look at `functions/statistical/geomean.rs` for how the product/log is accumulated and find the order of operations that reproduces Excel's bits across the existing GEOMEAN cases.

More cells of the same class (T-2676 sweep, gen-m9 round 2, gap triage 2026-10-26, excel-graph), all one ulp and within
15 digits. In each, our r = exp(ln P/n) is already faithful (within one ulp of the exact root of the double P) but on
the other side of it from Excel; P and n read with PRODUCT and COUNT of the same arguments, distances in ulps from the
exact root (Excel / ours):
- gen-m9/290180acf017709c Calc!A326 P=20 n=3 (+0.44 / -0.56)
- gen-m9/932b33b1ef8b7608 Calc!A30 and A75 P=0.1 n=2 (-0.01 / +0.99), A55 P=1E+24 n=3
  (-0.86 / +0.14), A185 `=GEOMEAN({1,2,3,4,5,6,7,8,9,10})` (-0.24 / +0.76), A320 P=588 n=3 (+0.15 / -0.85)
- gen-m9/f81bd3b6dc373e5c Calc!A9 P=152.681 n=6 (-0.49 / +0.51), A14 P=26.389 n=4 (-0.39 / +0.61), A59 P=0.05 n=2
  (-0.43 / +0.57), A249 P=1701 n=6 (-0.94 / +0.06), A294 P=2.222E+09 n=5 (-0.40 / +0.60), A319 P=168 n=3 (+0.19 / -0.81)
A55 and A249 are cells where Excel is the farther one (0.86 and 0.94 ulp), more evidence that Excel's root is a library
power with its own error, not the correctly rounded root. The multi-ulp cells of that sweep, where r is not faithful,
are T-2794's.
Cells:
- gen-m9/290180acf017709c.case.json (1): Calc!A326
- gen-m9/932b33b1ef8b7608.case.json (5): Calc!A30, A55, A75, A185, A320
- gen-m9/f81bd3b6dc373e5c.case.json (6): Calc!A9, A14, A59, A249, A294, A319

## Acceptance criteria
- [ ] `GEOMEAN(5,7)` matches Excel bit-exactly, with a probe case added (or the -d case extended) as evidence and no existing GEOMEAN case regressing.
- [ ] The 12 T-2676 known-failure lines listed in Context leave eval/baseline/gen-m9.txt with no new ones, or the
      Outcome lists the ones still open with the model tried.
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

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
