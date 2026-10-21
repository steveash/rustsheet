---
id: T-2166
title: "SWITCH does not match FALSE against a blank cell case value"
milestone: M5
lane: functions-logic
status: todo
priority: P3
size: S
tier: opus
campaign: visible
depends_on: [T-2200]
---

## Goal
SWITCH does not match a FALSE expression against a blank cell given as a case value, as Excel does.

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2200 (research), which fills `probes/t2200-blank-and-padded-m5-args`. Use that case; do not probe again. Its failing cells are this task's cells too.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `SWITCH(FALSE,..,A11,FALSE,..)` with A11 blank is `#N/A` in Excel (no match, no default); ours matches the blank and returns FALSE.
The omitted-argument case (`SWITCH(0,,"a","b")` = "a") must keep matching (logic.md "IFERROR / IFNA of a blank cell"). Probe first
(`excel-probe`): a blank case cell against FALSE, 0, "" and TRUE, and a blank expression cell against those cases. Observation:
docs/excel-semantics/logic.md "SWITCH: a blank cell case value does not match FALSE" (added by this triage). Code:
`fn_switch` (functions/logical/switch.rs), `compare_values`.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/d0419fa09d6a756a Calc!A63 | `=ISEVEN(SWITCH((Data!B9=(-0.5)),(Data!$E3*Data!A7),(Data!C1+Data!$A1),Data!A$11,IF(FALSE,Data!A6),...` | "#N/A" | "#VALUE!" |

## Acceptance criteria
- [ ] These 1 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] logic.md entry updated with the probe results.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
-
