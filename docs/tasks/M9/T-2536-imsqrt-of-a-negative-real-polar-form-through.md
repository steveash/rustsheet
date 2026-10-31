---
id: T-2536
title: "IMSQRT of a negative real: polar form through Excel's ATAN2, COS and SIN (gen-m9, 5 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: [T-2117]
campaign: visible
---

## Goal
IMSQRT of a negative real number gives Excel's tiny real part: the polar form with Excel's ATAN2, COS and SIN.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"). Excel `IMSQRT("-1")` is "6.1257422745431E-17+i", `IMSQRT(-2.5)` "9.68564897336834E-17+1.58113883008419i", `IMSQRT(-0.0001)` "6.1257422745431E-19+0.01i", `IMSQRT(-3.85841)` "1.20326949922385E-16+1.96428358441443i"; ours 6.12323399573677E-17 (libm cos(π/2)) times the root. The engine's `COS(ATAN2(-1,0)/2)` is 6.1257422745431E-17, so sqrt(r)*COS(θ/2), sqrt(r)*SIN(θ/2) with the worksheet kernels reproduces all four. T-2117 (done on its pushed branch, not merged) keeps IMSQRT's `sqrt(x²+y²)` modulus and drafts a range-rule task there (numbered T-2356 on that branch, an id main has used); this task changes only the angle half, so coordinate with that branch when it merges. Check the existing IMSQRT cells (probes eng-imsqrt-large, eng-imsqrt-large-b, COMPLEXs.xlsx) stay.
Code: `imsqrt` in complex.rs; `excel_sin_cos` and the ATAN2 kernel in functions/math_and_trigonometry.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 5 known-failure lines in eval/baseline/gen-m9.txt, 4 formulas, 5 visibly off.
Examples:
- gen-m9/516488da78a6b08c Calc!A52 `=IMSQRT((-2.5))`: Excel "9.68564897336834E-17+1.58113883008419i", ours "9.68168303635097E-17+1.58113883008419i"
- gen-m9/59a16b5ab566ee5c Calc!A6 `=IMSQRT(TEXT((-1),"0"))`: Excel "6.1257422745431E-17+i", ours "6.12323399573677E-17+i"
- gen-m9/7dc52cfcbc8b4448 Calc!A254 `=IMSQRT(IF(REGEXTEST(Data!E7,"\.",1),(Data!D1*(-2.5)),(-0.0001)))`: Excel "6.1257422745431E-19+0.01i", ours "6.12323399573677E-19+0.01i"
- gen-m9/f280df8ba95f96b9 Calc!A26 `=IMSQRT(IFERROR((2.5/Data!A1),(Data!$D10-7)))`: Excel "1.20326949922385E-16+1.96428358441443i", ours "1.20277680213541E-16+1.96428358441443i"
Cells:
- gen-m9/516488da78a6b08c.case.json (1): Calc!A52
- gen-m9/59a16b5ab566ee5c.case.json (1): Calc!A6
- gen-m9/7dc52cfcbc8b4448.case.json (1): Calc!A254
- gen-m9/f280df8ba95f96b9.case.json (1): Calc!A26
- gen-m9/fb73d8163ad87fc6.case.json (1): Calc!A398

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 5 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests for IMSQRT of -1, -2.5 and -0.0001.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Decision 0188 note (2026-10-31)
Unblocked by decision 0188 R6 (2026-10-31): the blocker was only the AL12/AL26 web-vs-desktop conflict, which R6
settles, and T-2117 is done (item 16 stays open on T-1075).

Decision 0188 R6 settles the web-vs-desktop part of the blocker: IMSQRT of a negative real follows the
Excel-for-the-web value under decision 0039 item 4, and this task's PR may regress ironcalc-calc/ENGINEERING/COMPLEXs.xlsx
Sheet1!AL12 and AL26 (lines added to eval/milestones/exit-exceptions.tsv and phase1-exceptions.tsv, owner T-1075 item
16). Status left as is by the owner's instruction for this PR; T-2117 is done, so the next steward or worker can unblock
it (docs/process/agent-workflow.md, "Unblocking").

## Outcome
`imsqrt` (crates/rustsheet-engine/src/functions/engineering/complex/mod.rs) takes its half angle from Excel's ATAN2
kernel (`excel_atan2`, with libm `atan2` where it errs, as IMPOWER does) through Excel's SIN and COS (`excel_sin_cos`);
T-2117's `sqrt(x²+y²)` modulus is unchanged. Unit test `imsqrt_of_a_negative_real_takes_excel_atan2_cos_and_sin` pins
-1, `TEXT(-1,"0")`, -2.5, -0.0001 and -3.85841. Eval: gen-m9 known failures 2975 -> 2970 (the 5 owned cells); every
other IMSQRT cell of the suites is unchanged. The regressions are exactly ironcalc-calc/ENGINEERING/COMPLEXs.xlsx
Sheet1!AL12 and AL26 (Mac 16.0300 libm digits), which decision 0188 R6 allows; they are lines of
eval/milestones/exit-exceptions.tsv and phase1-exceptions.tsv (owner T-1075 item 16). engineering.md "IMSQRT of a
negative real is polar through COS and ATAN2" moves from CONFLICTING to VERIFIED (excel-graph), web value followed under
decision 0039 item 4. The kernel's unprobed edges (a zero or subnormal real part, x < 0 with -0i) are added to T-2736.
