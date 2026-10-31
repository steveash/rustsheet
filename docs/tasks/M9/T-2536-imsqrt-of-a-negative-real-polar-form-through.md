---
id: T-2536
title: "IMSQRT of a negative real: polar form through Excel's ATAN2, COS and SIN (gen-m9, 5 cells)"
milestone: M9
lane: functions-engineering
status: blocked
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
- [ ] Visible parity (gen-m9, T-2258): these 5 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for IMSQRT of -1, -2.5 and -0.0001.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Blocked because
Web-vs-desktop conflict on the same formula: the target (Excel's ATAN2 and SIN/COS of the half angle) reproduces the
four web cells but regresses two Mac desktop 16.0300 cached cells the engine passes today, ironcalc-calc/ENGINEERING/COMPLEXs.xlsx
Sheet1!AL12 (expected 2.12115047744981e-16, got 2.12201937071624e-16) and AL26 (expected 6.12323399573677e-17, got
6.1257422745431e-17). Decision 0039 keeps desktop evidence that cells pass against today, and decision 0030 rule 4 says a web
golden that misses the desktop fit only by a near-zero residue below 1e-15 absolute is not a fitting target. So nothing here is
implementable before desktop evidence. Its open question is item 16 of the M14 task T-1075 (desktop-only, decision 0039 rule 3:
it does not block a milestone). Recorded as CONFLICTING in engineering.md "IMSQRT of a negative real is polar through COS and ATAN2".
Unblocks when T-1075 item 16 is answered: if Windows desktop gives the web values, implement them (the engine change is
`excel_atan2` plus `excel_sin_cos` of the half angle in `imsqrt`) and update the two COMPLEXs.xlsx cells under decision 0030.

## Decision 0188 note (2026-10-31)
Decision 0188 R6 settles the web-vs-desktop part of the blocker: IMSQRT of a negative real follows the
Excel-for-the-web value under decision 0039 item 4, and this task's PR may regress ironcalc-calc/ENGINEERING/COMPLEXs.xlsx
Sheet1!AL12 and AL26 (lines added to eval/milestones/exit-exceptions.tsv and phase1-exceptions.tsv, owner T-1075 item
16). Status left as is by the owner's instruction for this PR; T-2117 is done, so the next steward or worker can unblock
it (docs/process/agent-workflow.md, "Unblocking").
