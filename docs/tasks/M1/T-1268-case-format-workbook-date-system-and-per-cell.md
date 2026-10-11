---
id: T-1268
title: "Case format: workbook date_system and per-cell number formats; probe 1904 negatives (SPEC-0005 step 2)"
milestone: M1
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: [T-1267]
spec: docs/specs/SPEC-0005-workbook-date-system.md
---

## Goal
Probe cases can request a 1904 workbook and set a cell's number format, and the SPEC-0005
unknowns have excel-graph values.

## Context
SPEC-0005 "Excel behavior to match" (unknowns) and rollout step 2. The `rustsheet-case/v1` format
can set neither (T-1105 Context). The spec→xlsx builder in `tools/` writes the case workbook;
the engine side reads the flag through T-1267.

## Acceptance criteria
- [x] Optional case fields for `date_system` and per-cell `number_format`, documented, with builder and loader tests.
- [x] A probe case for `TEXT(-1.5,"[h] d")`, `TEXT(-1E-10,"[h]:mm")`, `TEXT(-0,"[h]:mm")` in both systems and cell display of a negative under a date format, filled by excel-graph; findings in dates.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine behavior changes (T-1105).

## Outcome
- Case format (rustsheet-case/v1) gains top-level `date_system` (1900/1904) and per-sheet `number_formats`. spec2xlsx writes them (`date_1904`, cell `numFmt`); the engine adapter applies them through `WorkbookOptions::date_system` and a new facade `Txn::set_number_format` (engine `Model::set_cell_number_format`). Documented in docs/architecture/eval-harness.md, with builder, adapter and facade tests.
- The oracle canary's date cell (A5) assumed 1900 serials and refused every 1904 workbook. It is now `DATEVALUE("1/2/2001")-DATEVALUE("1/1/2001")` = 1 (31 when day-first), which passed live in both systems.
- New probes probes/date-system-1900-negatives and probes/date-system-1904-negatives (18 questions each, excel-graph). Findings are in dates.md "The 1904 date system": negatives keep a minus in 1904 for all tokens; `[s]`-only formats print negatives in 1900 too. Cell display cannot be probed through excel-graph.
- Eval delta: +2 cases, 17 new-coverage known failures (1900: 6, 1904: 11), no regressions. Follow-ups: T-1329 (`[s]` "-0"), T-1330 (1904 epoch in date functions and TEXT tokens; SPEC-0005 step 4).
