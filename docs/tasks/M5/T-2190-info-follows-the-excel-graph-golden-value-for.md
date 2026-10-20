---
id: T-2190
title: "INFO follows the excel-graph golden: #VALUE! for every type_text"
milestone: M5
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
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

## Acceptance criteria
- [ ] probes/info-sheets-info-types Alpha!D25:D49 pass and leave eval/baseline/probes.txt, with no new known failures.
- [ ] A comment at the change cites the probe cells and T-2182.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop INFO values (T-2182).
