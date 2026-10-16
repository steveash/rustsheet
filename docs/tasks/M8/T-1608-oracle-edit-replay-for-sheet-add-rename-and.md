---
id: T-1608
title: "Oracle edit replay for sheet add, rename and move, with a probe suite for reference rewriting"
milestone: M8
lane: harness
status: done
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
- [x] The case format can list sheet operations (add, rename, move) applied after load and before
  recalculation. Both the oracle backend and the eval engine apply them. Tests cover each
  operation and a refused or unknown one.
- [x] A `probes` set (≥ 20 cases) covers references, names, 3-D spans, INDIRECT text and
  FORMULATEXT across rename/add/move, oracle-filled; failing cells have filed tasks.
- [x] Decision record if the case schema changes in a way other suites must know about.
- [x] `make check` green; reviewers approve.

## Out of scope
- Row/column insert and delete (M4 structural edits).

## Outcome
- **Case format:** an optional top-level `sheet_ops` list (`add`, `rename`, `move`), applied after
  load and before recalculation.
  - `SheetOp` in `crates/rustsheet-eval/src/case.rs`; an unknown op or key is refused.
  - spec2xlsx records the ops as `rustsheet.sheetop.<n>` custom properties and validates them
    (`sheet_op`, `sheet_ops`).
  - Documented in docs/architecture/eval-harness.md.
- **Oracle:**
  - The excel-graph backend replays the ops in its session after the typed entries and keeps the
    canary sheet last: an add goes before the canary, and a move onto the canary is refused.
  - Each move is read back. Graph once answered a position PATCH without moving the sheet, so a
    replay that did not happen now fails the case instead of becoming an oracle value.
  - excel-desktop and libreoffice refuse cases with sheet ops.
- **Engine:** the eval adapter applies each op as its own `Workbook::edit`. The facade gains
  `Txn::move_sheet` (with undo and `SheetChange::Moved`).
- **Tests:**
  - spec2xlsx: each op, plus refused and unknown ops.
  - Graph replay: a fake session for canary placement, a move onto the canary, and an ignored
    move.
  - Engine adapter: ops apply in order; unknown ops and an unknown sheet are refused.
  - API: `move_sheet`, including rollback and out of bounds.
- **Probes:** 24 oracle-filled cases in `eval/suites/probes/cases/sheet-ops/` (M8). They cover
  renames (quoting, case-only, chains, swaps, defined names, sheet-scoped names, INDIRECT text,
  FORMULATEXT), adds and moves into, out of and across 3-D spans, plus two INDIRECT controls.
  - 16 pass. The other 8 are baselined, with their failing cells filed:
    - T-1691: moving a span's end sheet past its other end shrinks the span.
    - T-1692: INDIRECT text naming a missing sheet is 0 and ISREF TRUE in excel-graph.
    - T-1693: a sheet-qualified sheet-scoped name gives `#NAME?`.
  - New semantics entries in docs/excel-semantics/references.md.
- **No decision record:** the schema change is one optional field, and other suites ignore it.
- **Eval delta:** probes +24 cases (106 cells checked), +21 known-failure cells, all filed;
  0 regressions.
