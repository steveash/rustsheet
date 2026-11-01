---
id: T-2763
title: "Convert-to-range keeps an off-row #This Row reference as structured text (FORMULATEXT =Sales[@Qty], value #VALUE!)"
milestone: M8
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-2643]
campaign: semantics
---

## Goal
After a table is converted to a range, a cell whose `#This Row` reference was off the table's data rows prints the
structured text Excel keeps (`=Sales[@Qty]`) in FORMULATEXT and still evaluates `#VALUE!`.

## Context
- Decision 0188 (owner's 2026-10-31 rulings, R1(c)): this task owns probe-row or text cells of its milestone's exit,
  which Phase 1 still counts in full, so it is in the semantics stage at P1 (was P3).
T-2643 lowers each `#This Row` cell on convert (`Model::lower_this_row_references`, tables.rs), so off-row cells
store `#VALUE!` and their values match, but FORMULATEXT prints `=#VALUE!`. Failing: probes/table-edits/
table-convert-names-this-row S!F2, S!F10, T!A2 (references.md, convert-to-range entry). The engine has no form for
a structured reference to a table that no longer exists: unparsed, it is a `ParseErrorKind`, and on load an
unresolved one becomes `#REF!` (`structured_reference_operand`, parser/operands.rs). It needs a node, or stored text,
that prints the structured reference, evaluates `#VALUE!`, survives a reload and has an xlsx export decision.
Also probe a cross-sheet `#This Row` on one of the table's data rows (T!A5 `=Sales[[#This Row],[Qty]]` and its
FORMULATEXT): T-2643 keeps such cells on the shared, row-relative lowering without evidence.
Small design note first (docs/specs/ or a decision record) if it touches the parser's node set.

## Acceptance criteria
- [x] The three cells pass and their known-failure lines leave eval/baseline/probes.txt; no new known failures.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A `#This Row` reference off its table's data rows is no longer lowered to `#VALUE!` by a convert to range: it
becomes a *detached* structured reference (`StructuredReference::detached`), the existing
`StructuredReferenceKind` node with `lowered` = `#VALUE!` (no new node). It prints `=Sales[@Qty]` (FORMULATEXT,
cell content), evaluates `#VALUE!`, resolves against no table (a later table of the same name does not take it
back; unprobed), and is stored with an engine-only `[!]` prefix (`[!]Sales[[#This Row],[Qty]]`) that only the
engine's own text reads (`Lexer::stored_detached_marker`; typed and loaded text refuse it), so it survives
reparses, row and column inserts (a cell moved along its row is re-entered through the new
`Rewrite::Reenter` printer, which keeps the prefix) and the other re-entry printers. xlsx export writes it as
`#VALUE!` (a file never holds a structured reference without its table part). The lexer's `padded_bare_columns`
flag is renamed `engine_text`, its real meaning.
The cross-sheet probe the task asked for (probes/table-edits/table-convert-cross-sheet-this-row, excel-graph
2026-11-01): a `#This Row` on a data row from another sheet is lowered to `=S!$C5` (row relative), which the
engine already did; recorded in references.md.
Eval: probes/table-edits/table-convert-names-this-row S!F2, S!F10, T!A2 now pass (3 known failures removed);
the new 6-cell probe passes; no new failures.
