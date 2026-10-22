---
id: T-2197
title: "Probe how Excel resizes, converts and deletes a table and what its structured references become"
milestone: M8
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/references.md "Structured references when the table changes" is VERIFIED, or its unknowns are listed with the probe that would settle them.

## Context
SPEC-0016 keeps structured references in the parsed formula and re-derives their area when the table list changes, but Excel's own behavior is unprobed: row and column inserts and deletes inside and at the edge of a table, a table resize, "Convert to Range", and deleting a table. The engine's tables do not move on structural edits (`insert_rows` never touches `workbook.tables`). Use the `excel-probe` skill; the oracle may only be able to evaluate files, not run table edits, so say which questions it cannot answer.

Explicit questions: does a table move with a row or column inserted or deleted wholly before it, and grow with rows inserted inside its data rows (SPEC-0016's hand assumption)? What do cut and paste and row and column moves of table cells do? What is `ISREF` of `Sales[@Qty]` off the data rows? What does Excel do with a loaded formula naming an unknown table or column? What does a structured reference on another sheet become when the table's sheet is deleted (SPEC-0016 assumes `#REF!`), and after undo?

- Also probe (from T-2195): what a duplicated sheet does with its tables and the structured references to them (Excel is believed to add a renamed table to the copy; the engine writes retargeted plain ranges), and how Excel loads a formula naming a table the file does not have (the engine exports it as `#REF!`).
## Acceptance criteria
- [x] Probe cases or a documented reason why each question cannot be probed with the oracle.
- [x] references.md updated with the findings; an implementation task filed for what is probed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementation.

## Outcome
13 probe cases in eval/suites/probes/cases/table-edits, filled by excel-graph (2026-10-21). Whole-row inserts:
an insert at or above the header moves the table; one between the header and the data, inside the data,
or between the data and the totals row grows it; one just below leaves it. That contradicts SPEC-0016's
fallback rows for `r == d1` and `r == t`, and a plain range on the data column grows with those two inserts
(a quirk). `Sales[@Qty]` uses the formula's row from another sheet too and is a reference (`ISREF` TRUE)
on data rows only. A loaded formula naming an unknown table or column is `#REF!`, printed `=SUM(#REF!)`.
references.md "Structured references when the table changes" is VERIFIED; the edits the oracle cannot
replay yet are in a new UNVERIFIED entry, "Structured references under the unprobed table edits".
Follow-ups: T-2321 (engine, edge inserts and plain-range quirk), T-2322 (engine, unknown table/column on
load), T-2323 (harness, more sheet ops), T-2324 (research, probe them), T-2325 (M14, desktop-only edits).
Eval: 13 new probe cases, 3 pass; 27 new-coverage known failures, 0 regressions, owned by T-2297, T-2321, T-2322 and
T-1671 (adapter totals rows).
