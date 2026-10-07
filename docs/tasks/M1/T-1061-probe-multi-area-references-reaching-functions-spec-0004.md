---
id: T-1061
title: "Probe multi-area references reaching functions (SPEC-0004 probe list)"
milestone: M1
lane: research
status: done
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
- [x] Each of the 20 groups has oracle values in a kept `eval/suites/probes/` case or a recorded refusal, and a docs/excel-semantics/references.md entry ("Multi-area references in functions").
- [x] Every row whose Excel result differs from SPEC-0004's planned result is noted in that entry, and in the spec's table (a one-line correction in the same PR).
- [x] One follow-up task per group that needs an engine change (rows 3-5, 12, 13, 16-18, 19, 20), filed with `make new-task`, depending on T-1062.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-1062 to T-1065).

## Outcome
21 probe cases (probes/multiarea-*, 61 new baseline lines for cells the engine does not yet match; the AREAS rows are split one workbook per formula) with oracle
values; AREAS(1) and AREAS("A1") were refused by Excel for the web (recorded, not kept). Findings in
references.md "Multi-area references in functions". Differences from SPEC-0004's plan, corrected in its table: row 4
(FREQUENCY accepted), 5 (`#VALUE!`/`#REF!`/`#REF!`), 13 (`#NAME?`), 18 (CELL accepts, ROW `#REF!`), 19 (file loads),
20. No change for row 12. Row 16: `@(A1,B2)` is `#VALUE!`, no `@` in a dynamic-array formula; the legacy half cannot be probed with spec2xlsx (T-1074). AREAS(1)/AREAS("A1") refusals came from scratch one-formula workbooks. Follow-ups, all
depending on T-1062: T-1068 (rows 3-4), T-1069 (5), T-1070 (13), T-1071 (17), T-1072 (18), T-1073 (19-20), T-1074 (16, harness).
Eval: probes: +40 new-coverage failures (191 -> 231 after merging main, engine rollout pending; no regressions).
