---
id: T-1671
title: "Txn::add_table takes a totals row; the eval adapter loads spec totals rows"
milestone: M8
lane: api
status: todo
priority: P1
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
A table added through the facade (`Txn::add_table`) can have a totals row, and the eval engine
adapter loads a spec table's `"totals_row": true` with it, so `Table[#Totals]`, `Table[#All]` and
totals-row formulas evaluate as in Excel.

## Context
- T-1607 added `totals_row` to the case format (`TableSpec`, crates/rustsheet-eval/src/case.rs) and
  spec2xlsx writes it, but `Model::add_table` (crates/rustsheet-engine/src/tables.rs) always sets
  `totals_row_count: 0`, so the adapter registers such a table without its last row
  (`table_range` in crates/rustsheet-eval/src/engine.rs, `TODO(T-1671)`). The engine already reads
  `totals_row_count` from imported xlsx tables.
- Evidence: suite gen-m8b (category `totals`, and `#Totals`/`#All` cells in the other categories).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 107 cells, 88 numeric past half a unit in the 15th digit and 19 with a wrong type, error or text, blamed on COUNT 36, ROWS 30, COUNTA 27, ISERROR 8, COUNTBLANK 4, COLUMNS 1, ISREF 1. Root cause: the adapter registers each gen-m8b table without its totals row (`totals_row_count: 0`), so `[#All]` is one row short and `[#Totals]` is `#REF!` or empty; every cell below is a `totals_row: true` table in its case spec.
- gen-m8b/44cecbb1ecd5b2da (7): Calc!K100, Calc!K110, Calc!K164, Calc!K349, Calc!K372, Calc!K375, Calc!K380
- gen-m8b/5b5c0e20e82153bb (12): Calc!K41, Calc!K69, Calc!K86, Calc!K99, Calc!K167, Calc!K179, Calc!K208, Calc!K257, Calc!K283, Calc!K287, Calc!K305, Calc!K321
- gen-m8b/9cb2e511e5e01db0 (65): Calc!K4, Calc!K10, Calc!K11, Calc!K15, Calc!K18, Calc!K19, Calc!K22, Calc!K23, Calc!K25, Calc!K34, Calc!K39, Calc!K44, Calc!K45, Calc!K48, Calc!K55, Calc!K58, Calc!K61, Calc!K81, Calc!K86, Calc!K89, Calc!K95, Calc!K100, Calc!K111, Calc!K135, Calc!K138, Calc!K163, Calc!K178, Calc!K197, Calc!K207, Calc!K209, Calc!K216, Calc!K220, Calc!K223, Calc!K241, Calc!K245, Calc!K246, Calc!K260, Calc!K270, Calc!K272, Calc!K275, Calc!K286, Calc!K290, Calc!K291, Calc!K303, Calc!K306, Calc!K312, Calc!K315, Calc!K320, Calc!K326, Calc!K330, Calc!K336, Calc!K341, Calc!K343, Calc!K348, Calc!K352, Calc!K357, Calc!K363, Calc!K364, Calc!K369, Calc!K374, Calc!K377, Calc!K379, Calc!K392, Calc!K395, Calc!K399
- gen-m8b/d86ef297e840325f (12): Calc!K49, Calc!K115, Calc!K118, Calc!K144, Calc!K155, Calc!K187, Calc!K250, Calc!K266, Calc!K331, Calc!K336, Calc!K367, Calc!K369
- gen-m8b/e804cf2f8af00680 (11): Calc!K8, Calc!K15, Calc!K23, Calc!K80, Calc!K142, Calc!K192, Calc!K203, Calc!K230, Calc!K241, Calc!K277, Calc!K396

## Acceptance criteria
- [ ] Visible parity: the 107 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 107 cells of the ungated gen-m8b suite pass in `make eval`, with no new known-failure lines in any baseline.
- [ ] `Txn::add_table` (or a sibling) can set a one-row totals row; a unit test shows `#Totals` and
  `#All` of such a table.
- [ ] The adapter passes `totals_row` through and the `TODO(T-1671)` is gone; gen-m8b totals cells
  that now pass are recorded with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Totals-row functions metadata (`totalsRowFunction`), table resizing.

## Visible-parity stage
Decision 0083: wrong values here are visible to anyone using defined names or tables (gen-m8b `[#Totals]` cells), so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.
