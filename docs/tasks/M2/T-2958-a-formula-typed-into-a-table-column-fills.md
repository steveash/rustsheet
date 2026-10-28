---
id: T-2958
title: "A formula typed into a table column fills the calculated column, as Excel does (probes/t2786 C2:C4)"
milestone: M2
lane: engine
status: done
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

T-2957's evidence (docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table column";
the saved form in xlsx-storage.md, "A calculated column is saved as `calculatedColumnFormula` plus one formula per
row"): a typed formula fills every data row (not the totals row) when every other data cell of the column is blank;
a different formula typed into a filled column re-fills it; a typed constant is an exception, and a column with an
exception is not re-filled; no fill when another data cell already holds a value; an A1 formula fills shifted. So the
fill decision needs only the column's cells, but the export's `calculatedColumnFormula` is state (it stays the first
formula after exceptions): keep it on the table. Known failures of the T-2957 cases this task owns (all in
eval/baseline/probes.txt): t2957-fill-one-totals S!C2, S!C3, S!C4, S!F2, S!F4; t2957-refill-second S!C2:C4, S!F2,
S!F4; t2957-constant-exception S!C2, S!C4, S!F4; t2957-constant-then-refill S!C2, S!C4; t2957-existing-constant S!C3;
t2957-a1-reference S!C3, S!C4, S!F3, S!F4. The expand-right cases are T-2968; a stored column formula on load is
T-2969.

## Visible-parity stage
Target: S!C2:C4 of probes/t2786-whitespace-structured-refs-runs and every cell of T-2957's fill cases match Excel's
type and value. The stage is `semantics`: a reference and entry rule, not a numeric one.

## Acceptance criteria
- [ ] ~~These 3 known-failure lines disappear from eval/baseline/probes.txt with no new ones:
      `probes/t2786-whitespace-structured-refs-runs.case.json` S!C2, S!C3, S!C4. So do the known failures of the T-2957
      cases listed in Context (fill-one-totals, refill-second, constant-exception, constant-then-refill,
      existing-constant, a1-reference).~~ Partly met: a1-reference and the FORMULATEXT cells cleared; every cell that
      types the unqualified `[@Col]` moved to T-2979 (see Outcome).
- [x] Engine unit tests: a typed formula fills an empty table column; a stored formula does not; a typed formula
      over a filled column re-fills it (or not) as T-2957's evidence says; undo restores the column.
- [x] docs/excel-semantics/ entry from T-2957 updated with "Ours:"; `make check` green; reviewers approve.

## Out of scope
- Table auto-expansion when typing beside a table (T-2968), and re-filling on xlsx load (T-2969).
- Whitespace inside structured references (T-2801).

## Outcome
- A formula typed into a table's data rows (`Model::set_typed_user_input`; the facade's `Input::Parse` and
  `Input::Formula`, which both type) fills the column's other data rows when they are all blank or all hold one
  shared formula (crates/rustsheet-engine/src/calculated_columns.rs); a constant typed into a filled column is an
  exception and blocks later re-fills; the totals row is never filled; a stored formula never fills. The fill
  rewrites the typed cell's shared formula into each target, so an A1 formula fills shifted. Settled design: no
  column-formula state on `Table` yet, since rules 1-5 depend only on the column's cells; `calculatedColumnFormula`
  on import/export stays with T-2969. The fill goes through `prepare_cell_for_user_input` for every target, and the
  facade logs every target before the write, so the fill is one edit and rolls back with it.
- The eval adapter now types every `entered` cell after the tables (the oracle types them after load), so a
  constant typed into a filled column is an exception, as in Excel.
- Eval: probes known failures 2656 -> 2647 (t2957-a1-reference S!C3, C4, F3, F4; constant-exception S!F4;
  fill-one-totals S!F2, F4; refill-second S!F2, F4), no regressions.
- Not met here: the cells that type the unqualified `[@Col]` (t2786 S!C2:C4 and the C cells of fill-one-totals,
  refill-second, constant-exception, constant-then-refill, existing-constant) stay `#ERROR!`: an unqualified
  structured reference typed inside a table does not parse at all (the parser refuses an operand starting with
  `[`). That is a parser and storage change of its own (the R1C1 storage text must keep the table), filed as T-2979.
- Two unprobed engine choices (formulas mixed with blanks block the fill; identical stored formulas without
  `calculatedColumnFormula` re-fill) go to the probe task T-2983.
