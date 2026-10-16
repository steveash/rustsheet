---
id: T-1310
title: "Probe 3-D spans after deleting or moving an end sheet (desktop oracle)"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for what a 3-D span (`Data1:Data3!A1`) becomes when an end sheet is deleted, when a middle sheet is
deleted, and when an end sheet is moved outside or inside the span, with the engine matching it.

## Context
T-1181 could not probe this: the excel-graph oracle only recalculates an uploaded workbook and has no sheet-delete or
sheet-move operation. The engine's current behavior (an end-sheet delete makes the reference `#REF!`) is recorded as
UNVERIFIED in docs/excel-semantics/references.md, "3-D references when sheets move or are renamed".

## Acceptance criteria
- [ ] Desktop-oracle (or Excel-authored workbook) evidence for the delete and move forms, recorded in references.md.
- [ ] The engine matches, or a follow-up task is filed for each mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
Phase 4 (decision 0039): not a "Needs owner" item, and it blocks no milestone or phase exit. The evidence needs
desktop Excel (a COM `Worksheets(..).Delete` / `.Move` before recalculation) or an Excel-authored workbook edited by
hand. Unblock when `cd tools && uv run rustsheet-oracle status` shows `excel-desktop OK`, or when such a workbook is committed.
