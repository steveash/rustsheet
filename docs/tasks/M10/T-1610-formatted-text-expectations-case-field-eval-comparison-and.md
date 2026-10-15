---
id: T-1610
title: "Formatted-text expectations: case field, eval comparison and oracle read of Excel's range text"
milestone: M10
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A case can carry Excel's displayed text for a cell (`.Text`), and the eval compares it with
`rustsheet::Workbook::formatted`. That is the prerequisite for M10's formatted-text oracle suite.

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- M10's exit criterion is "formatted-text oracle suite (≥ 10k cells) at 100%". The xlsx file
  stores only the raw value, not the displayed text, so the oracle cannot get `.Text` from the
  saved file.
- Graph's range resource returns a `text` matrix. After `calculate`, read `text` for the cells
  the case marks, in the same session as the recalculation (backends.py).
- The engine already exposes `Workbook::formatted` (crates/rustsheet/src/workbook.rs).
- Column width changes `.Text` (`####` and the General-format shortening). Pin the column width in
  the cases, or record width as part of the expectation. Probe which one Excel needs.

## Acceptance criteria
- [ ] The case/sidecar format has an optional per-cell `text` expectation. The eval scores it as
  its own verdict column, so value scoring is unchanged, and the report shows a text pass rate
  per suite.
- [ ] The excel-graph backend fills `text` for marked cells. A unit test uses a fake backend.
- [ ] One small probe case (≥ 20 cells: General, 0.00, dates, %, text, `####` width) is
  oracle-filled and its results recorded.
- [ ] Decision record for the schema addition. `make check` green; reviewers approve.

## Out of scope
- Number-format engine fixes (file them). The large generated suite (T-1611).
