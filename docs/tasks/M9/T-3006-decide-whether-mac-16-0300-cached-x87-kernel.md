---
id: T-3006
title: "Decide whether Mac 16.0300 cached x87-kernel last digits in COMPLEXs.xlsx (IMTAN N6, IMSQRT AL12/AL26) outrank excel-graph, unblocking T-2774 and T-2536 (46 visibly-off cells)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A decision record (`make new-decision`) settles, for the three Mac-cached cells of
ironcalc-calc/ENGINEERING/COMPLEXs.xlsx that block T-2774 and T-2536, whether the Mac Excel 16.0300 cached last digits
keep outranking excel-graph (decisions 0030 rule 1, 0039 rule 4) or may regress to the web value as decision 0109 allowed for
BESSEL.xlsx All!D18 and F18. Either way T-2774 and T-2536 leave `status: blocked` with a path that needs no desktop oracle,
or the record says why they must wait for T-1075 items 16 and 17.

## Context
Gap triage, 2026-10-28 (`make eval` on main 81569726): no other visibly-off cell outside functions-financial lacks an
open, undeferred owner; these 46 are owned only by blocked tasks whose sole blocker is this policy question.
- T-2774 (IMTAN of a real number takes TAN's kernel, 41 visibly-off cells): Excel for the web gives `IMTAN("5")`
  -3.38051500624659 in 35 gen-m9 cells (e.g. gen-m9/01810fc89c5ec2c0 Calc!A18, A116), `IMTAN(123456789.123)`
  56.4157574800738 and `IMTAN(3.14159)` -2.65358979336212E-06 (probes/t2534-im-trig-real-limit Sheet1!D1, D8-D10). The
  engine's `excel_tan` reproduces them. COMPLEXs.xlsx Sheet1!N6 `IMREAL(IMTAN(D6))` caches Mac 16.0300's
  -3.38051500624658 and passes today with the libm double-angle form (`tan_parts`,
  crates/rustsheet-engine/src/functions/engineering/complex/trig.rs).
- T-2536 (IMSQRT of a negative real, 5 visibly-off gen-m9 cells): the web's `IMSQRT("-1")` real part is 6.1257422745431E-17
  (Excel's COS(ATAN2(-1,0)/2)); COMPLEXs.xlsx Sheet1!AL12 caches 2.12115047744981e-16 and AL26 6.12323399573677e-17
  (libm cos(pi/2) times the root), which pass today. Decision 0030 rule 4's near-zero-residue band (below 1e-15
  absolute) applies to these two cells, unlike N6.
- Precedents: decision 0109 (Mac 16.0300 BESSEL.xlsx cells regressed to the web value: a Mac build has no x87 sine, and
  excel-graph matched the new kernel on 14 cells); decision 0086 (Mac ATAN2_POWER D14/D26 kept as known failures, engine
  keeps the web value); decision 0074 (desktop SIN(4)). engineering.md "IM* trigonometry of a real number is SIN, COS and
  TAN" and "IMSQRT of a negative real is polar through COS and ATAN2 on the web"; numbers.md "Excel for the web and desktop
  Excel differ in the last digits". T-1075 items 16 (IMSQRT) and 17 (IMTAN) are the desktop-oracle route.
- The question to answer with evidence, not preference: does the Mac 16.0300 cache reflect a Mac-only kernel (no x87, as
  0109 argued for the Bessel phase), so that it is not evidence for Windows desktop Excel's x87 kernels? Check the other
  COMPLEXs.xlsx IM* trigonometry cells the engine passes with `excel_sin_cos`/`excel_tan` against the same cache: if the
  cache agrees with the x87 kernels elsewhere, N6 is evidence and the tasks stay blocked.

## Acceptance criteria
- [ ] A decision record in docs/decisions/ (accepted by reviewers) names N6, AL12 and AL26, the options weighed (follow
      the web with `--allow-regressions` for exactly those cells, as 0109; keep the cache, as 0086), the evidence for each
      (cell ids and values as above, plus the other COMPLEXs.xlsx IM* trigonometry cells checked), and the outcome per cell.
- [ ] T-2774 and T-2536 are updated to match: `depends_on` gains this task, and each is either `status: todo` with the
      decision cited in its Context and its `## Blocked because` section removed, or stays blocked with the record cited
      as the reason. No engine code, no baseline edit in this task (the fix tasks carry the regression allowance, if any).
- [ ] engineering.md's two entries above cite the decision; T-1075 items 16 and 17 note it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing IMTAN or IMSQRT (T-2774, T-2536).
- Other ironcalc-calc Mac-cache conflicts (ATAN2_POWER stays under 0086).
