---
id: T-2406
title: "NUMBERVALUE with a comma decimal separator and no group separator reads the text 1.5 as 15"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`NUMBERVALUE("1.5",",")` is 15 as in Excel (ours `#VALUE!`): with a "," decimal separator and the group separator omitted, the default group separator is not ",".

## Context
Found by T-2379 (probes/numbervalue-separator-edges A8, excel-graph, 2026-10-22). Excel gives 15, so the omitted group
separator defaulted to "." (the "." of "1.5" was removed), not to "," (that would clash with the decimal separator, `#VALUE!`,
T-2198). With an explicit group separator, `NUMBERVALUE("1,5",",","x")` is 1.5 (A9). Probe first which default Excel picks for
other decimal separators (`NUMBERVALUE("1.5",".")` vs `("1,5",";")`, `("1.000,5",",")`, `("1 5",",")`), and whether the
default depends on the decimal separator or is always the opposite of the locale's. Code: `numbervalue_of`
(crates/rustsheet-engine/src/functions/text/string_format.rs), its `separator(2, ",")` default. docs/excel-semantics/text.md
"NUMBERVALUE reads text as VALUE does".

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/numbervalue-separator-edges Sheet1!A8 | `=NUMBERVALUE("1.5",",")` | 15 | #VALUE! |

## Acceptance criteria
- [ ] The cell above passes (and leaves eval/baseline/probes.txt); the probed default rule is in text.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
