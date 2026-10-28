---
id: T-3054
title: "IMTAN last digits at 3.14159 (gen-m9 756a9f7bc3a06bcc A338, unmasked by the T-3030 CELL re-fill)"
milestone: M9
lane: functions-engineering
status: dropped
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`IMTAN("3.14159")` matches Excel's last digits: -2.65358979336212E-06 (gen-m9/756a9f7bc3a06bcc Calc!A338).

## Context
- T-3030 re-filled the gen-m9 case from excel-graph, after Excel for the web began reading `CELL("contents",Data!A12)`
  on Data instead of the formula's own sheet. The cell's IMTAN argument is now "3.14159" (Data!A12) where it used to be
  "4", and ours gives -2.65358979336207E-06. Decision 0153 lets it into eval/baseline/gen-m9.txt as a known failure.
- At this argument tan(x) is near zero, so the result is all cancellation. Excel's IMTAN kernel there (sin/cos ratio,
  or sin 2x / (cos 2x + cosh 2y)) decides the digits. See docs/excel-semantics for the existing IMTAN/IMCOT entries
  (T-3032).

## Acceptance criteria
- [ ] gen-m9/756a9f7bc3a06bcc Calc!A338 passes, and no other IM* cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other complex functions.

## Outcome (2026-10-28, dropped into T-2774)
Excel for the web's IMTAN of a real number is TAN's x87 kernel (`excel_tan` gives -2.653589793362123E-06, which prints
Excel's ...212E-06), the same change as the blocked T-2774. probes/t3054-imtan-real-tan-kernel (excel-graph) confirms it
on nine discriminating real arguments, and its N1, ironcalc-calc COMPLEXs.xlsx N6's own formula, gives the web's
-3.38051500624659 against N6's Mac cache ...658.

The change is not made here. IMTAN is in numbers.md "Excel for the web and desktop Excel differ in the last digits", so
decision 0030 rule 4 (kept by decision 0039) makes the desktop N6 the fitting target, and a web known failure the engine
misses by less than 1e-9 relative (A338: 2e-14) is not a target for further fitting. Switching would also regress N6, which
needs its own decision (decision 0039 rule 4). The cell and the probe cells move to T-2774, which waits on T-1075 item 17.
Eval delta: 0 fixed, 0 regressions; the probe adds 10 known failures (new coverage, owned by T-2774): its nine discriminating IMTAN rows and N1.
