---
id: T-3062
title: "Probe the formula text of a sheet-qualified #REF! after a sheet rename or delete, unqualified and typed"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-2855]
---

## Goal
The formula text of a sheet-qualified `#REF!` matches Excel in the four unprobed shapes T-2855 left: after its sheet is renamed, after its sheet is deleted, for a wholly deleted range with no sheet prefix, and for a typed one with a sheet name in another case or for a sheet that does not exist.

## Context
T-2855 keeps the sheet of `S!#REF!` in the written tree (`Node::SheetRefErrorKind`; references.md "A
sheet-qualified #REF! is #REF!"). Only `=SUM(S!#REF!)` after a row or column delete is oracle-backed
(probes/table-edits plain-range-delete-row-first F!A6, table-delete-column-qty F!A7). Unprobed:
- renaming S to T: the engine keeps `S!#REF!` (sheet_edits.rs skips the node); Excel likely prints `T!#REF!`;
- deleting sheet S: the engine keeps `S!#REF!`;
- a wholly deleted unqualified range (`=SUM(C4:C4)`, delete row 4): the engine prints `=SUM(#REF!)`
  (stringify.rs, `wrong_reference::ref_error_text`), before T-2855 `#REF!:#REF!`;
- typed `=sheet1!#REF!` and `=NoSuchSheet!#REF!`: printed as typed; probes/sheet-qualified-ref-error-forms
  records values only.
Use the `excel-probe` skill (FORMULATEXT cells), then match the engine to the evidence.

## Acceptance criteria
- [ ] A probe case records FORMULATEXT for the four shapes (oracle values filled).
- [ ] The engine matches every probed cell; references.md records the rule with the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Values: a sheet-qualified `#REF!` already evaluates to `#REF!` (VERIFIED).
