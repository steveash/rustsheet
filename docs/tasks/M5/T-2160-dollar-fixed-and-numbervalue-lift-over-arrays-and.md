---
id: T-2160
title: "DOLLAR, FIXED and NUMBERVALUE lift over arrays and ranges (SPEC-0011)"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-2198]
---

## Goal
DOLLAR, FIXED and NUMBERVALUE lift over arrays and multi-cell ranges like Excel (SPEC-0011 driver).

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2198 (research), which fills `probes/t2198-m5-lift-argument-kinds`. Use that case; do not probe again. Its failing cells are this task's cells too.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). None of them is in eval/catalog/lift-kinds.json: over an array they give `#N/IMPL!` (`DOLLAR(ABS(MUNIT(4)))`), over a range
`#VALUE!`. Excel returns an array (`#SPILL!` under a filled column). Rule: docs/excel-semantics/arrays.md "LOG, DECIMAL, BASE,
ATAN2, ARABIC, DOLLAR, FIXED and NUMBERVALUE return arrays over array arguments" (added by this triage). Use `lift_text`/`lift_call`
(functions/lift/) as REPLACE does; probe each argument's kind (`excel-probe`) before choosing, and regenerate lift-kinds.json.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 5 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/6361fdf5993da954 Calc!A378 | `=DOLLAR(INDEX(Data!C4:C9,0),-2)` | "#SPILL!" | "#VALUE!" |
| gen-m5/7d1a5d223624fc70 Calc!A380 | `=DOLLAR(ABS(MUNIT(4)))` | "#SPILL!" | "#N/IMPL!" |
| gen-m5/aac6230c2fb40e1e Calc!A213 | `=NUMBERVALUE("(100)",INDEX(Data!A4:B9,0,2),"#N/A")` | "#SPILL!" | "#VALUE!" |
| gen-m5/ae9ed30812c4d62c Calc!A154 | `=NUMBERVALUE((IFS(FALSE,"TRUE",Data!$A7,1e-15)&TEXT(123456789.123,Data!A8)),REPT(INDEX(Data!B5:E10...` | "#SPILL!" | "#N/A" |
| gen-m5/d586a1e0c9896942 Calc!A364 | `=FIXED(MUNIT(3),3,INDEX(Data!B8:B9,6,1))` | "#SPILL!" | "#N/IMPL!" |

## Acceptance criteria
- [ ] These 5 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] arrays.md entry marked implemented for these three.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
- The math functions (T-2159); NUMBERVALUE parsing (T-2151, T-2152).
