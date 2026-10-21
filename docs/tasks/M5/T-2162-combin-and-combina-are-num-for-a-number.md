---
id: T-2162
title: "COMBIN and COMBINA are #NUM! for a number past a bound (COMBINA(FACT(13),10))"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-2199]
---

## Goal
COMBIN and COMBINA are `#NUM!` for a `number` past Excel's bound, at the exact cut.

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2199 (research), which fills `probes/t2199-text-length-and-combin-cuts`. Use that case; do not probe again. Its failing cells are this task's cells too.
- Result (T-2199): COMBIN is `#NUM!` for number >= 2^31-1; COMBINA for n+k-1 >= 2^31-1. See docs/excel-semantics/numbers.md.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `COMBINA(FACT(13),10)` (6227020800) is `#NUM!` in Excel and 2.4E+91 here; `COMBINA(25,123456789.123)` and
`COMBIN(123456789.123,10)` are numbers, so the cut lies in (1.23E+8, 6.2E+9]; 2^31 is the guess. Observation:
docs/excel-semantics/numbers.md "COMBINA is COMBIN(n+k-1, k)", paragraph "Bound" (added by this triage). Probe first
(`excel-probe`): COMBIN and COMBINA at number = 2^31-1, 2^31, 2^32 with k = 0, 1, 2, and COMBINA where n < 2^31 <= n+k-1.
Code: `fn_combin`/`fn_combina` (mathematical.rs). 6b831f5e A5 is `COMBIN(1E+15,1E-15)` inside MMULT.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 5 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/34f69d858a39f497 Calc!A231 | `=COMBINA(ROUNDUP(2^53+1,IF(FALSE,Data!C$1)),1)` | "#NUM!" | 9007199254740990.0 |
| gen-m5/50d8a085335c05e6 Calc!A173 | `=COMBINA(FACT(13),10)` | "#NUM!" | 2.4156840630757255e+91 |
| gen-m5/6b831f5e2a20bc34 Calc!A5 | `=MMULT((COMBIN(1000000000000000.0,1e-15)-LN(9999999999999999)),Data!A7:D9)` | "#NUM!" | "#VALUE!" |
| gen-m5/e0087a1fb9a36188 Calc!A379 | `=COMBINA((1000000000000000.0+Data!$B6),2)` | "#NUM!" | 4.999999999999975e+29 |
| gen-m5/fc81c2f704bef138 Calc!A265 | `=COMBINA(SUM(Data!E4:E8,TAN(710)),SUM(Data!D5:D12,Data!D3:D10))` | "#NUM!" | 1000000000000107.0 |

## Acceptance criteria
- [ ] These 5 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] numbers.md entry updated with the probed cut.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
- COMBIN rounding (T-1598, done).
