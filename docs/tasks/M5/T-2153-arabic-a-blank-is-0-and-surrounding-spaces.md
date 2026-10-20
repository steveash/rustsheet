---
id: T-2153
title: "ARABIC: a blank is 0 and surrounding spaces and tabs are trimmed (ARABIC(' x ') = 10)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2200]
---

## Goal
ARABIC of a blank cell is 0 and ARABIC ignores leading and trailing spaces and tabs (`ARABIC(" x ")` = 10), as Excel does.

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2200 (research), which fills `probes/t2200-blank-and-padded-m5-args`. Use that case; do not probe again. Its failing cells are this task's cells too.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_arabic` (crates/rustsheet-engine/src/functions/math_and_trigonometry/arabic_roman.rs) gives `#VALUE!` for a blank cell (a
reference, `IF(..,E5)`, or the blank VLOOKUP/HLOOKUP lands on) and for padded numerals; `ARABIC("")` is already 0. Rule:
docs/excel-semantics/numbers.md "ARABIC of a blank cell, and padded numerals" (added by this triage). Probe first (`excel-probe`):
`ARABIC(0)` (a number: 6a511bea A106 and b0d597ec A365 pass through VLOOKUP/HLOOKUP of a blank, which Excel hands on as 0 per
T-2168, so ARABIC(0) may have to be 0 too), inner spaces (`"X X"`), CHAR(10)/CHAR(13)/non-breaking space padding.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 12 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/1315170eef6c413e Calc!A37 | `=ARABIC(" x ")` | 10.0 | "#VALUE!" |
| gen-m5/1315170eef6c413e Calc!A169 | `=IFS((Data!B8>ARABIC(Data!$A$3)),-DAY(12778),INDEX(Data!B1:C5,SERIESSUM(Data!E$4,2,Data!C$5,7)),Da...` | -25.0 | "#VALUE!" |
| gen-m5/399071ff6782ab16 Calc!A256 | `=ARABIC(Data!C3)` | 0.0 | "#VALUE!" |
| gen-m5/4d099bea7898ebf1 Calc!A289 | `=ARABIC(Data!$A$5)` | 0.0 | "#VALUE!" |
| gen-m5/6a511beadb56b932 Calc!A106 | `=ARABIC(VLOOKUP(COUNTIF(Data!B3:D4,">=2"),Data!B2:E4,4))` | 0.0 | "#VALUE!" |
| gen-m5/6b831f5e2a20bc34 Calc!A388 | `=ARABIC(Data!$A10)` | 0.0 | "#VALUE!" |
| gen-m5/97737fad5004ba83 Calc!A43 | `=ARABIC(UPPER(CHAR(9)))` | 0.0 | "#VALUE!" |
| gen-m5/b0d597ec5af5fb92 Calc!A365 | `=ARABIC(HLOOKUP(VLOOKUP(3.14159,Data!B2:D4,3),Data!C4:D7,3))` | 0.0 | "#VALUE!" |
| gen-m5/d586a1e0c9896942 Calc!A346 | `=ARABIC(IF(Data!B4,Data!E5))` | 0.0 | "#VALUE!" |
| gen-m5/e0087a1fb9a36188 Calc!A25 | `=ARABIC(RIGHT(" x ",MAX(Data!D11:D12,Data!B11:B12,7)))` | 10.0 | "#VALUE!" |
| gen-m5/f0f205865a3a4328 Calc!A269 | `=ARABIC(" x ")` | 10.0 | "#VALUE!" |
| gen-m5/fc81c2f704bef138 Calc!A174 | `=SUMX2MY2(ARABIC(INDEX(Data!D6:D12,1,0)),Data!B6:D8)` | "#N/A" | "#VALUE!" |

## Acceptance criteria
- [ ] These 12 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] numbers.md entry updated with the probe results and marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ARABIC lifting over arrays (T-2159).
