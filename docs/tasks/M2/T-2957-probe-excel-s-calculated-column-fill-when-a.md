---
id: T-2957
title: "Probe Excel's calculated-column fill when a formula is typed into a table column (probes/t2786 C2:C4)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/ states, with excel-graph evidence, when typing a formula into a table column makes Excel fill it
as a calculated column, and what a later typed entry into that column does. T-2958 can then implement it without
guessing.

## Context
Found by the gap triager (2026-10-27 eval on 0f8dede8f). In probes/t2786-whitespace-structured-refs-runs, table `T`
is A1:C4 with an empty `Calc` column C. The oracle types the `entered` cells one at a time in spec order
(spec2xlsx `rustsheet.entered.<n>`), so C2, C3 and C4 were typed in that order:
- C2 `=T[[#This Row], [Col]]`, C3 `=T[ [#This Row] , [Col] ]`, C4 `=T[[#This Row],[Col]] + [@Other]`;
- Excel's values: C2 11, C3 22, C4 44, which is C4's formula in every row (Col + Other: 1+10, 2+20, 4+40).

So the first typed formula filled the empty column, and each later typed formula into a filled cell **re-filled the
whole column** rather than leaving an exception. syntax.md records only that the fill happened ("The
calculated-column fill rewrote..."). T-2800 sidesteps the fill to probe this-row syntax. Nothing records the fill
rules themselves, and the engine has no calculated columns (no `calculatedColumnFormula` anywhere in crates/).
This is one data point, and three cells of the M0-M2 exit depend on it (C2:C4 are visibly off: ours 1 and
`#ERROR!`).

Probe with the excel-probe skill on excel-graph, using `entered` cells. Use a separate case per sequence, because
the order of typing matters:
1. One formula typed into one cell of an empty table column: is every data row filled, including with a totals row?
2. Then a different formula typed into another row of that column (the t2786 sequence): is the column re-filled? Is
   the answer the same when the second formula is typed into the row that was typed first?
3. A constant typed into one row of a filled column: is that row an exception, and do the other rows keep the
   formula? Then a formula typed into another row: is the column re-filled, constant included?
4. A formula typed into a column that already holds a constant in one row: is it filled or not?
5. A formula with a plain A1 reference (`=A2*2`) rather than a structured one: is it filled, and with what
   relative shift (FORMULATEXT of each row)?
6. FORMULATEXT of the filled rows: do they print the typed form or a normalized one?
7. A formula typed into the cell just right of the table, in the header row and in a data row: does the table
   auto-expand, and is the new column filled?
Order caveat: the oracle types `entered` cells in the case's JSON order, but the engine adapter enters them
row-major (`CaseSheet::cells` is a `BTreeMap`; `row_major` in crates/rustsheet-eval/src/engine.rs). Write each
sequence so that its JSON order is also row-major order, or file a harness task to keep JSON order first.
Read back the saved table part's `calculatedColumnFormula`. The oracle downloads the workbook, so record what Excel
writes in xlsx-storage.md. Also record whether a stored workbook carrying `calculatedColumnFormula` refills on load
(an engine import question; usually it does not).

## Acceptance criteria
- [ ] Probe cases under eval/suites/probes/cases/ (tag M2, `entered` cells, excel-graph values), in the probes
      baseline.
- [ ] A docs/excel-semantics/ entry (references.md or a tables section) states the fill rules items 1-7 answer, with
      the cells as evidence, and links T-2958. syntax.md's "calculated-column fill" sentence links to it.
- [ ] T-2958's Context and acceptance criteria name the new cases' cells. An implementation task is filed for any
      finding T-2958 does not cover (auto-expand, import of `calculatedColumnFormula`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- This-row syntax without a fill (T-2800).
- Table moves on row inserts (T-2845).
