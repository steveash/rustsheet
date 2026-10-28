---
id: T-2441
title: "NUMBERVALUE with a comma decimal separator and no group separator reads the text 1.5 as 15"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
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

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns the 1 visibly-off NUMBERVALUE cell. Fixing them lifts NUMBERVALUE to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells.
