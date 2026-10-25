---
id: T-2541
title: "IM* inumber reading: VALUE-readable text, a tab or U+3000 is #NUM!, over 255 characters #VALUE!, a number at full precision (gen-m9, 8 cells)"
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
The IM* functions read an inumber as Excel does: VALUE-readable text (currency, time) as a real number, a leading tab or ideographic space as unreadable, text over 255 characters as `#VALUE!`, and a number argument at full precision.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" "How an inumber is read"). Excel: `IMPOWER(DOLLAR(4),1)` ("$4.00") is "4", `IMSEC(DOLLAR(3.14159))` ("$3.14") -1.00000126827407, `IMCONJUGATE(TEXT(0.1,"h:mm:ss"))` ("2:24:00") "0.1", `IMSIN(TEXT(0,"[h]:mm"))` ("0:00") "0" (ours `#NUM!`); `IMCOT(CHAR(9)&"123456789.123")` and `IMCOT(UNICHAR(12288)&"2.5")` are `#NUM!` (ours reads them; a leading space is read, `IMSECH(" 3+4i")`); `IMSECH("TRUE"&REPT("AB",300))` (604 characters) is `#VALUE!` (ours `#NUM!`); `IMPOWER(<1.6564497455696935E-10>,3)` is 4.54500948137352E-30, the full double cubed (ours ...349, which a 15-digit reading of the base gives). Probe first (excel-probe skill), since each rule rests on one or two cells: "$1,234.50", "50%", "1E3", "2024-01-01", "$1+2i", "\t3+4i", "3+4i " (trailing space), 255 and 256 characters, a number argument 1/3 through IMREAL(..)-1/3 and IMPOWER, in IMABS, IMREAL, IMSUM and IMSUB.
Code: `parse_complex_number` and the argument reader in complex.rs (`lift_complex`).

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 8 known-failure lines in eval/baseline/gen-m9.txt, 8 formulas, 8 visibly off.
Examples:
- gen-m9/12e4aa13a20cc1f8 Calc!A224 `=IMCOT((TRIM(CHAR(9))&TOROW(Data!E7:E8)))`: Excel #NUM!, ours "0.0177255441485821"
- gen-m9/12e4aa13a20cc1f8 Calc!A391 `=IMCONJUGATE(TEXT(0.1,"h:mm:ss"))`: Excel "0.1", ours #NUM!
- gen-m9/3cf48801617a72ad Calc!A91 `=IMSIN(TEXT(ROUND(1e-15,5),"[h]:mm"))`: Excel "0", ours #NUM!
- gen-m9/65ff9a692665eba9 Calc!A28 `=IMPOWER(DOLLAR(COUNT(Data!C3:C12)),1)`: Excel "4", ours #NUM!
- gen-m9/756a9f7bc3a06bcc Calc!A345 `=IMPOWER(IFNA(NA(),NORM.DIST(Data!E11,Data!A6,Data!$E2,"true")),3)`: Excel "4.54500948137352E-30", ours "4.54500948137349E-30"
- gen-m9/8fa8b572ebc1f001 Calc!A377 `=IMCOT((SUBSTITUTE(UNICHAR(12288),Data!$C7,Data!C6,3)&Data!$A$9))`: Excel #NUM!, ours "-1.33864812830415"
Cells:
- gen-m9/12e4aa13a20cc1f8.case.json (2): Calc!A224, A391
- gen-m9/3cf48801617a72ad.case.json (1): Calc!A91
- gen-m9/65ff9a692665eba9.case.json (1): Calc!A28
- gen-m9/756a9f7bc3a06bcc.case.json (1): Calc!A345
- gen-m9/8fa8b572ebc1f001.case.json (1): Calc!A377
- gen-m9/b5aef3645cec1b95.case.json (1): Calc!A101
- gen-m9/c2d778463fe008ed.case.json (1): Calc!A232

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 8 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The probe case is oracle-filled and every cell passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.
