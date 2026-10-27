---
id: T-2958
title: "A formula typed into a table column fills the calculated column, as Excel does (probes/t2786 C2:C4)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2955, T-2957]
campaign: semantics
---

## Goal
A formula typed into a table column (facade `Input::Parse`, the engine's `set_typed_user_input`) fills the column as
a calculated column, and later typed entries into it behave as T-2957's probes show Excel's do. A stored formula
(xlsx import, `Input::StoredFormula`) never fills.

## Context
Found by the gap triager (2026-10-27 eval on 0f8dede8f). probes/t2786-whitespace-structured-refs-runs types C2, C3
and C4 of table `T` (A1:C4, empty column `Calc`). Excel shows C4's formula `=T[[#This Row],[Col]] + [@Other]` in every
row:

| cell | typed | Excel | ours today | ours after T-2955 (triager's local check) |
|---|---|---|---|---|
| S!C2 | `=T[[#This Row], [Col]]` | 11 | `#ERROR!` | 1 |
| S!C3 | `=T[ [#This Row] , [Col] ]` | 22 | `#ERROR!` | `#ERROR!` |
| S!C4 | `=T[[#This Row],[Col]] + [@Other]` | 44 | `#ERROR!` | `#ERROR!` |

These are the last three visibly-off M0-M2 cells of that case once T-2955, T-2956, T-2801 and T-2944 land. Before
starting, find out why C4 is still `#ERROR!` after T-2955. Its text has no inner whitespace, so look at `[@Other]`
typed inside the table, or the `+` run. If it is a separate parse bug, fix it here when small, or file it.

Design point to settle and record (decision record if it is not obvious from the evidence): the facade's
`Input::Formula` also goes through `set_typed_user_input` (crates/rustsheet/src/edit.rs). Excel's API and UI both type,
and this harness models typed entry as `Input::Parse`. Decide whether `Input::Formula` fills too, and say so in the
facade's doc comment. The fill writes formulas into other cells, so it must respect the existing spill, merge and
array-formula guards (`prepare_cell_for_user_input`) and stay one undoable edit. Tables live in
`rustsheet_engine::types::Table` (no calculated-column state today). T-2957 decides whether the table needs to remember
a column formula, for re-fill and for `calculatedColumnFormula` on export.

## Visible-parity stage
Target: S!C2:C4 of probes/t2786-whitespace-structured-refs-runs and every cell of T-2957's fill cases match Excel's
type and value. The stage is `semantics`: a reference and entry rule, not a numeric one.

## Acceptance criteria
- [ ] These 3 known-failure lines disappear from eval/baseline/probes.txt with no new ones:
      `probes/t2786-whitespace-structured-refs-runs.case.json` S!C2, S!C3, S!C4. So do the known failures of the T-2957
      cases that T-2957 lists for this task.
- [ ] Engine unit tests: a typed formula fills an empty table column; a stored formula does not; a typed formula
      over a filled column re-fills it (or not) as T-2957's evidence says; undo restores the column.
- [ ] docs/excel-semantics/ entry from T-2957 updated with "Ours:"; `make check` green; reviewers approve.

## Out of scope
- Table auto-expansion when typing beside a table, and re-filling on xlsx load (file follow-ups from T-2957's
  findings).
- Whitespace inside structured references (T-2801).
