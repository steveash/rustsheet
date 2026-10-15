---
id: T-1608
title: "Oracle edit replay for sheet add, rename and move, with a probe suite for reference rewriting"
milestone: M8
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The oracle can replay sheet add, rename and move in Excel, so M8's "sheet add, rename and move
with reference rewriting" has oracle evidence, and a probe suite records Excel's rewritten
formulas and values.

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- Today a case is a static workbook: the oracle uploads it, recalculates and downloads it. Sheet
  operations need an edit step inside the Graph session. The typed-input path in
  `GraphBackend.recalc` (tools/src/rustsheet_tools/oracle/backends.py, T-0280) already PATCHes
  ranges inside the session, so extend that pattern.
  - Graph supports worksheet add (`POST worksheets/add`), rename (`PATCH` the worksheet `name`)
    and move (`PATCH` `position`).
- The engine side needs the same edits applied before scoring: use the API `edit()` path
  (crates/rustsheet).
- The canary sheet must stay last or be re-checked after a move. INFO/SHEETS refusals still apply.
- 3-D span edit semantics (inserting or deleting a sheet inside a span) are partly blocked on
  desktop evidence (T-0198, M14). Cover what Excel for the web shows and leave the rest to T-0198.

## Acceptance criteria
- [ ] The case format can list sheet operations (add, rename, move) applied after load and before
  recalculation. Both the oracle backend and the eval engine apply them. Tests cover each
  operation and a refused or unknown one.
- [ ] A `probes` set (≥ 20 cases) covers references, names, 3-D spans, INDIRECT text and
  FORMULATEXT across rename/add/move, oracle-filled; failing cells have filed tasks.
- [ ] Decision record if the case schema changes in a way other suites must know about.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Row/column insert and delete (M4 structural edits).
