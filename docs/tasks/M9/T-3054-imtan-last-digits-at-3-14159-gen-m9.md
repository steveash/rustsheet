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

## Outcome (dropped, 2026-10-28)
Duplicate of T-2774 (blocked on T-1075 item 17). A338's argument is the real number 3.14159, and IMTAN of a real is
TAN's kernel on Excel for the web: `IMTAN(3.14159)` -2.65358979336212E-06 is already recorded in engineering.md "IM*
trigonometry of a real number is SIN, COS and TAN" (probes/t2534-im-trig-real-limit) and owned by T-2774 through
gen-m9/79dac8dde81afda9 A154. Following the web here regresses the Mac desktop cache COMPLEXs.xlsx N6, which decision
0030 makes the reference, so it is not a kernel to fit. The cell is added to T-2774's owned list.

## Probe (2026-10-28, a second run on the same task)
probes/t3054-imtan-real-tan-kernel (excel-graph) confirms the web's TAN kernel on nine more real arguments where the two
kernels print different text, and its N1 is N6's own formula on N6's inputs, `IMREAL(IMTAN(COMPLEX(5,0,"i")))`: the web
gives -3.38051500624659 against N6's Mac cache ...658, so the conflict is the platform, not the formula. Its 10 failing
cells (N1, A2, A4:A11) are known failures owned by T-2774; eval delta 0 fixed, 0 regressions.
