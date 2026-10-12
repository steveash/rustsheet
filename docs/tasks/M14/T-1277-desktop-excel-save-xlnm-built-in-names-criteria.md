---
id: T-1277
title: "Desktop Excel save: _xlnm. built-in names (Criteria, Database, Extract, both spellings in one scope, ROWS(_FilterDatabase))"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Blocked because
Phase 4 (decision 0039): not a "Needs owner" item, and it blocks no milestone or phase exit. excel-graph cannot save a workbook, and no Excel-saved workbook in the repo holds an `_xlnm.` name (T-1250).
The evidence needs desktop Excel. Unblock when `cd tools && uv run rustsheet-oracle status` shows
`excel-desktop OK` (2026-10-12: `n/a`).

## Goal
Evidence from a desktop-Excel save for how Excel stores and reads its `_xlnm.` built-in names, so `BUILT_IN_NAMES` in crates/rustsheet-xlsx/src/util.rs can cover the full ECMA-376 list or stay narrow.

## Context
Left open by T-1250: excel-graph cannot save a workbook and no Excel-saved workbook in the repo holds an `_xlnm.` name. Needs desktop Excel (M14, Phase 4, decision 0039). Questions: does a user-defined `Criteria`, `Database` or `Extract` save with the prefix? Does `=ROWS(_FilterDatabase)` resolve in a workbook with an AutoFilter? What does Excel write back for a file holding both `_xlnm.Print_Area` and a plain `Print_Area` in one scope? See docs/excel-semantics/xlsx-storage.md "Built-in defined names".

## Acceptance criteria
- [ ] The three questions are answered in docs/excel-semantics/xlsx-storage.md with the evidence cited.
- [ ] `BUILT_IN_NAMES` matches the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
