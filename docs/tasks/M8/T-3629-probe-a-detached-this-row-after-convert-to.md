---
id: T-3629
title: "Probe a detached #This Row after convert to range: a re-created same-name table, and its xlsx save and reload"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2763]
---

## Goal
Excel evidence for two choices T-2763 made without it, for a `#This Row` reference left off the data rows by a
convert to range (FORMULATEXT `=Sales[@Qty]`, value `#VALUE!`; `StructuredReference::detached`):
(a) whether a table later created with the same name (`Sales`, over the same area or another) takes the reference
back (value and FORMULATEXT of the off-row cells afterwards); the engine says it does not;
(b) what Excel writes to the cell's `<f>` when it saves such a workbook, and what the cell reads after a reload; the
engine's xlsx export writes `#VALUE!`.

## Context
- T-2763 code: `StructuredReference::detached` (parser/structured_reference.rs), the export arm of
  `reference_text` (parser/stringify/structured.rs), references.md convert-to-range entry.
- (a) needs a sheet op that adds a table after the convert: the oracle's sheet ops (tools/src/rustsheet_tools/
  sheetop.py) have `convert_table` and `delete_table` but no `add_table`; Graph has `tables/add`. Adding the op
  is part of this task.
- (b) needs a save of a workbook edited through Graph and the downloaded file's `<f>`; if excel-graph cannot give
  it, it is a desktop question (M14, decision 0039).

## Acceptance criteria
- [ ] Probe cases with oracle values for (a), and for (b) where the oracle can answer (else an M14 note).
- [ ] The engine matches them, or follow-up tasks are filed; the `TODO(T-3629)` markers are resolved;
  findings recorded in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other structural edits of a detached reference.
