---
id: T-2763
title: "Convert-to-range keeps an off-row #This Row reference as structured text (FORMULATEXT =Sales[@Qty], value #VALUE!)"
milestone: M8
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2643]
---

## Goal
After a table is converted to a range, a cell whose `#This Row` reference was off the table's data rows prints the
structured text Excel keeps (`=Sales[@Qty]`) in FORMULATEXT and still evaluates `#VALUE!`.

## Context
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
- [ ] The three cells pass and their known-failure lines leave eval/baseline/probes.txt; no new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
