---
id: T-2011
title: "Redo of a paste or fill keeps a legacy N/T marker"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1852]
---

## Goal
After a paste or fill of a legacy cell is undone and redone, the target cell still holds the
legacy N/T marker (SPEC-0012), so `=N(IF(1,{2;3}))+$A$9` pasted from a legacy cell is still
`#VALUE!` after redo.

## Context
T-1852 made paste and fill re-enter a marked cell's display text through the legacy entry
(`Model::reenter_user_input`). Their undo diffs are `Diff::SetCellValue { new_value: String }`,
and redo (user_model/undo_redo.rs, the `Diff::SetCellValue` arm) writes `new_value` back with
`set_user_input`, which parses it as a dynamic formula, so the marker is lost on redo. Row and
column inserts, deletes and moves redo by calling the action again, so they are not affected.
Fix sketch: carry whether the value was written as legacy in the diff (a `#[serde(default)]`
bool, so older undo queues still deserialize) and redo through `reenter_user_input`.

## Acceptance criteria
- [ ] Unit tests in test/user_model/test_legacy_marker_edits.rs: paste and fill of a legacy cell,
  undo, redo: the target is `#VALUE!` and `cell_has_legacy_marker` holds; a dynamic twin still
  spills after redo.
- [ ] references.md "Legacy N and T over a reference-returning call" drops the redo caveat.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing Excel (redo of a paste is not an oracle-expressible edit).
