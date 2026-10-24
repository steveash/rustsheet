---
id: T-2540
title: "IMCSCH of 0 and of a denominator that underflows is #NUM!, not NaN or inf text (gen-m9, 11 cells)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
IMCSCH of 0 and of a number whose sinh has a squared modulus that underflows is `#NUM!`, as Excel gives, not "NaNNaNi" or "inf-infi" text.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"). `IMCSCH("0")`, IMCSCH of a 0 cell, of an empty cell (`IMCSCH(Data!Z1)`), of `IFS(..)` and `TOROW(..)` giving 0 are `#NUM!` (ours "NaNNaNi"), and `IMCSCH("1E-200+1E-200i")` is `#NUM!` (ours "inf-infi"). This settles two UNVERIFIED notes of engineering.md "IM* results that overflow (T-1827)" for IMCSCH (plain IMCSCH("0"), a denominator that underflows to 0). T-2334 did the same for IMCSC of 0: reuse its check.
Code: `imcsch` in complex.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 11 known-failure lines in eval/baseline/gen-m9.txt, 8 formulas, 11 visibly off.
Examples:
- gen-m9/1068c44afcd56d53 Calc!A220 `=IMCSCH(Data!C6)`: Excel #NUM!, ours "NaNNaNi"
- gen-m9/1068c44afcd56d53 Calc!A269 `=IMCSCH(TOROW(Data!A5:D9,(Data!E$10-0.2),IFERROR(Data!E1,Data!C7)))`: Excel #NUM!, ours "NaNNaNi"
- gen-m9/2b0324e2de2e7b74 Calc!A99 `=IMCSCH(Data!C11)`: Excel #NUM!, ours "NaNNaNi"
- gen-m9/2b0324e2de2e7b74 Calc!A334 `=IMCSCH(IFS(Data!$B4,Data!E$2))`: Excel #NUM!, ours "NaNNaNi"
- gen-m9/2f0d384b8ed0d392 Calc!A315 `=IMCSCH("0")`: Excel #NUM!, ours "NaNNaNi"
- gen-m9/59a16b5ab566ee5c Calc!A274 `=IMCSCH("1E-200+1E-200i")`: Excel #NUM!, ours "inf-infi"
Cells:
- gen-m9/1068c44afcd56d53.case.json (2): Calc!A220, A269
- gen-m9/2b0324e2de2e7b74.case.json (2): Calc!A99, A334
- gen-m9/2f0d384b8ed0d392.case.json (1): Calc!A315
- gen-m9/59a16b5ab566ee5c.case.json (1): Calc!A274
- gen-m9/74779cff2810ccf4.case.json (1): Calc!A334
- gen-m9/9fa170a9f17624fd.case.json (1): Calc!A71
- gen-m9/a333e1febf9e93cd.case.json (1): Calc!A251
- gen-m9/f280df8ba95f96b9.case.json (1): Calc!A164
- gen-m9/f8bc18a0e45a0e1a.case.json (1): Calc!A325

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 11 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] engineering.md's two UNVERIFIED notes are updated for IMCSCH (IMCSC("1E-200") and IMCOT("1E-310") stay unprobed unless a probe is added).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.
