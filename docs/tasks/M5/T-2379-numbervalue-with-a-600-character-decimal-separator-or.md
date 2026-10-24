---
id: T-2379
title: "NUMBERVALUE with a 600-character decimal separator or case-only separator clash is #VALUE!"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
NUMBERVALUE matches Excel when a separator is longer than 255 characters or differs from the other only in case (`NUMBERVALUE(0.2,PROPER(REPT("ab",300)),"a*2")` is `#VALUE!`).

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). Excel gives `#VALUE!`; ours 0.2. The decimal separator is the 600-character "Abab...ab" (first character "A") and the group
separator "a*2" (first character "a"). Three readings fit and none is probed: a separator over 255 characters is `#VALUE!` (REPT,
CODE and UNICODE refuse such text, T-2155); the first characters clash case-insensitively (a separator equal to the other is
`#VALUE!`, T-2198); or with "A" as the decimal separator the "." of "0.2" is no longer a decimal point (ours parses
`NUMBERVALUE("0.2","A","b")` as 0.2). Observation: docs/excel-semantics/text.md, paragraph after the separator clash in "NUMBERVALUE
reads text as VALUE does, with its own separators" (added by this triage).
Needs a probe first (`excel-probe` skill, excel-graph): `NUMBERVALUE(0.2,REPT("A",255),"b")`, `(0.2,REPT("A",256),"b")`,
`(0.2,"A","a")`, `("1,5","A","a")`, `("1A5","A","a")`, `("0.2","A","b")`, `("0.2","A")`, `("1.5",",")`, `("1,5",",","x")`,
`(0.2,"b",REPT("a",256))`. Code: `fn_numbervalue` / `numbervalue_of` (functions/text/string_format.rs).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/9a7a94353845ea12 Calc!A162 | `=NUMBERVALUE(Data!D1,PROPER(REPT("ab",300)),("a*"&(Data!E4&Data!$D7)))` | #VALUE! | 0.2 |

## Acceptance criteria
- [x] A probes case with the cells above, oracle-filled (excel-graph), passes in full; text.md states the rule.
- [x] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- Separators outside the BMP (text.md "Unprobed").

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Outcome
Probed (probes/numbervalue-separator-edges A1:A20, excel-graph): a separator over 255 characters is `#VALUE!`, which the
engine now applies (`numbervalue_of`, reusing `number_text_too_long`); separators differing only in case do not clash, and a
letter decimal separator leaves "0.2" readable (both already matched). text.md states the rules. Found on the way:
`NUMBERVALUE("1.5",",")` is 15 (an omitted group separator is not "," then), filed as T-2441, 1 new known failure (A8).
Eval delta: gen-m5 fixed 1 (9a7a94353845ea12 Calc!A162), probes new case 19/20, new-coverage failures 1 (T-2441), regressions 0.
