---
id: T-2190
title: "INFO follows the excel-graph golden: #VALUE! for every type_text"
milestone: M5
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: [T-2115]
---

## Goal
INFO gives what Excel for the web, the oracle of record (decision 0039 rule 2), gives: `#VALUE!` for every type_text.

## Context
- From T-2115. probes/info-sheets-info-types Alpha!D25:D49 (excel-graph, 2026-10-20): every `INFO(...)` is `#VALUE!`,
  valid types ("recalc", "system", "release", "directory", "osversion", "origin"), legacy ("memavail", "memused",
  "totmem"), `""` and `"x"`, in any case; `TYPE(INFO("release"))` 16, `ERROR.TYPE(INFO("x"))` 3, `ISTEXT(..)` FALSE.
  docs/excel-semantics/information.md "INFO is #VALUE! in Excel for the web".
- The engine returns "Automatic", a version and an OS name (`fn_info`, crates/rustsheet-engine/src/functions/information/).
- Same treatment as ENCODEURL and FILTERXML (docs/excel-semantics/web.md; decision 0039 rule 2): match the web, cite
  the cells, and leave the desktop values to T-2182 (M14). Keep the argument-count error and how an error argument
  propagates as they are unless evidence says otherwise (unprobed: `INFO(#N/A)`, `INFO(1)`, arrays).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 14 cells, 1 numeric past half a unit in the 15th digit and 13 with a wrong type, error or text, blamed on INFO 14, ISTEXT 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/info-sheets-info-types Alpha!D25 | `=INFO("recalc")` | #VALUE! | "Automatic" |
| probes/info-sheets-info-types Alpha!D26 | `=INFO("RECALC")` | #VALUE! | "Automatic" |
| probes/info-sheets-info-types Alpha!D27 | `=INFO("system")` | #VALUE! | "linux" |
| probes/info-sheets-info-types Alpha!D28 | `=INFO("SYSTEM")` | #VALUE! | "linux" |
| probes/info-sheets-info-types Alpha!D29 | `=INFO("Recalc")` | #VALUE! | "Automatic" |
| probes/info-sheets-info-types Alpha!D30 | `=INFO("System")` | #VALUE! | "linux" |
| probes/info-sheets-info-types Alpha!D36 | `=TYPE(INFO("release"))` | 16 | 2 |
| probes/info-sheets-info-types Alpha!D38 | `=LEN(INFO("osversion"))>0` | #VALUE! | #N/IMPL! |
| probes/info-sheets-info-types Alpha!D39 | `=LEN(INFO("release"))>0` | #VALUE! | TRUE |
| probes/info-sheets-info-types Alpha!D40 | `=LEN(INFO("origin"))>0` | #VALUE! | #N/IMPL! |
| probes/info-sheets-info-types Alpha!D41 | `=ISTEXT(INFO("recalc"))` | FALSE | TRUE |
| probes/info-sheets-info-types Alpha!D44 | `=LEN(INFO("OSVERSION"))>0` | #VALUE! | #N/IMPL! |
| probes/info-sheets-info-types Alpha!D45 | `=LEN(INFO("RELEASE"))>0` | #VALUE! | TRUE |
| probes/info-sheets-info-types Alpha!D46 | `=LEN(INFO("ORIGIN"))>0` | #VALUE! | #N/IMPL! |

## Acceptance criteria
- [ ] Visible parity: the 14 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 14 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/info-sheets-info-types Alpha!D25:D49 pass and leave eval/baseline/probes.txt, with no new known failures.
- [ ] A comment at the change cites the probe cells and T-2182.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop INFO values (T-2182).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (INFO returns text where Excel for the web is `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
