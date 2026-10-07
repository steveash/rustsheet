---
id: T-1061
title: "Probe multi-area references reaching functions (SPEC-0004 probe list)"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
spec: docs/specs/SPEC-0004-multi-area-references.md
depends_on: []
---

## Goal
Oracle evidence for every row of the probe table in SPEC-0004 ("Excel behavior to match"), so the multi-area rollout implements recorded behaviour, not guesses.

## Context
SPEC-0004 settles how the engine represents a multi-area reference, but which functions accept one, and with what result, is unknown. The table lists 20 groups (aggregates, LARGE/RANK/FREQUENCY, SUMIF/COUNTIF, COUNTBLANK/ROWS/COLUMNS, AREAS, INDEX area_num, LET, IF/CHOOSE, an intersection leaving two areas, cross-sheet and 3-D operands, `_xlnm.Print_Area`, ISERROR/IFERROR, `@`, SUBTOTAL, CELL/OFFSET/ROW) with the result the rollout plans. Use the `excel-probe` skill. Put each group in its own workbook: Excel for the web refuses a whole workbook it cannot parse (T-0138, T-0187). Existing union evidence: docs/excel-semantics/references.md "Union: parsing and evaluation"; cases probes/union-*.

## Acceptance criteria
- [ ] Each of the 20 groups has oracle values in a kept `eval/suites/probes/` case or a recorded refusal, and a docs/excel-semantics/references.md entry ("Multi-area references in functions").
- [ ] Every row whose Excel result differs from SPEC-0004's planned result is noted in that entry, and in the spec's table (a one-line correction in the same PR).
- [ ] One follow-up task per group that needs an engine change (rows 3-5, 12, 13, 16-18, 19, 20), filed with `make new-task`, depending on T-1062.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-1062 to T-1065).
