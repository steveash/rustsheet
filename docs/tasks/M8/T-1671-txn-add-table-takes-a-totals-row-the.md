---
id: T-1671
title: "Txn::add_table takes a totals row; the eval adapter loads spec totals rows"
milestone: M8
lane: api
status: todo
priority: P1
size: S
tier: opus
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

More visibly-off cells (gap triage 2026-10-25, decision 0083): 170 more gen-m8b cells, none listed above, have the same root
cause: 169 read `[#Totals]` of a `totals_row: true` table (`SUM(Sales[[#Totals],[Price]:[a.b]])`, MIN, MAX, AVERAGE,
bare `Sales[[#Totals],[Val]]`, `INDEX(Wide[#Totals],1,1)`) and get `#REF!`, where Excel gives the totals cell's value or
its error (0.5, 2, 8.75, `#N/A`, `#DIV/0!`); one is `SUBTOTAL(103,Wide[#All])` (Excel 34, ours 33, one row short). Blamed on
SUM 39, MIN 39, AVERAGE 37, MAX 36, the bare reference 17, SUBTOTAL 1, INDEX 1. Examples:
- gen-m8b/44cecbb1ecd5b2da Calc!K28 `=SUM(Sales[[#Totals],[Price]:[a.b]])`: Excel 0.5, ours #REF!
- gen-m8b/9cb2e511e5e01db0 Calc!K60 `=MIN(Wide[[#Totals],[Total$]:[Total$]])`: Excel 0.5, ours #REF!
- gen-m8b/9cb2e511e5e01db0 Calc!K33 `=AVERAGE(Sales[[#Totals],[Unit Price]:[a.b]])`: Excel #N/A, ours #REF!
- gen-m8b/9cb2e511e5e01db0 Calc!K27 `=SUBTOTAL(103,Wide[#All])`: Excel 34, ours 33
- gen-m8b/44cecbb1ecd5b2da.case.json (4): Calc!K28, K173, K215, K327
- gen-m8b/9cb2e511e5e01db0.case.json (155): Calc!K2, K5, K7, K9, K24, K27, K30, K31, K33, K35, K38, K40, K47, K51, K53, K54, K60, K67, K73, K91, K94, K96, K97, K98, K102, K103, K105, K110, K120, K121, K123, K126, K127, K128, K130, K133, K139, K140, K146, K148, K149, K154, K155, K164, K165, K169, K171, K172, K179, K180, K182, K185, K190, K193, K198, K200, K202, K203, K204, K205, K206, K210, K211, K213, K218, K219, K221, K227, K228, K229, K230, K236, K237, K238, K248, K249, K250, K251, K252, K255, K256, K257, K259, K261, K262, K263, K264, K266, K268, K269, K274, K277, K280, K285, K287, K288, K292, K293, K294, K296, K299, K300, K301, K304, K307, K313, K317, K319, K322, K324, K325, K327, K328, K329, K334, K335, K337, K338, K340, K344, K346, K347, K349, K351, K353, K354, K355, K356, K358, K359, K360, K361, K365, K366, K367, K370, K371, K372, K373, K375, K376, K378, K380, K383, K385, K386, K387, K388, K389, K393, K394, K396, K397, K398, K400
- gen-m8b/e804cf2f8af00680.case.json (11): Calc!K68, K74, K110, K160, K217, K221, K247, K269, K276, K286, K344

## Acceptance criteria
- [ ] Visible parity: the 107 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 107 cells of the ungated gen-m8b suite pass in `make eval`, with no new known-failure lines in any baseline.
- [ ] Visible parity: the 170 further gen-m8b cells listed under Context (2026-10-25) pass in `make eval` too, or the ones that
      still differ once the totals row loads are filed as a follow-up with their cells.
- [ ] `Txn::add_table` (or a sibling) can set a one-row totals row; a unit test shows `#Totals` and
  `#All` of such a table.
- [ ] The adapter passes `totals_row` through and the `TODO(T-1671)` is gone; gen-m8b totals cells
  that now pass are recorded with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Totals-row functions metadata (`totalsRowFunction`), table resizing.

## Visible-parity stage
Decision 0083: wrong values here are visible to anyone using defined names or tables (gen-m8b `[#Totals]` cells), so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Handoff
Done and pushed on branch `agent/api/T-1671-add-table-totals` (code committed): `Model::add_table_with_totals`
(crates/rustsheet-engine/src/tables.rs; `add_table` delegates), `Txn::add_table_with_totals`
(crates/rustsheet/src/edit.rs), the eval adapter passes `totals_row` through (`table_range` helper and the
`TODO(T-1671)` removed), plus a facade test and a rewritten adapter test (both pass).

Blocked: `make eval-baseline` refuses 4 regressions in suite `probes`, all structural edits on a table that now
really has a totals row (previously the adapter left the totals row out of the table, so they passed by accident):
- probes/table-edits/table-delete-row-totals F!A3 (`ROWS(Sales[#All])` Excel 5, ours `#ERROR!`) and F!A15
  (`FORMULATEXT(=Sales[[#Totals],[Qty]])` Excel keeps it, ours `=#REF!`)
- probes/table-edits/table-delete-row-last-data-totals F!A15
- probes/table-edits/table-insert-row-above-totals F!A12 (`=S!$C$9` instead of `=Sales[[#Totals],[Qty]]`); its
  F!A2/A7 (`ROWS(Sales)` 5, range growth) also differ.
`make eval-baseline` also reports 11 probe cells fixed. `make check` fails only at the eval gate for this reason.

Next: make the engine's structural edits (insert/delete row, tables.rs / the move-formula path) treat
`totals_row_count` right (grow/shrink the table range, keep `[#Totals]` references valid) using the oracle
values in those probe cases, then `make eval-baseline` (must show REGRESSIONS 0), then check the gen-m8b cells in
the acceptance criteria, then reviewers. Needs engine judgement beyond a sonnet S task.
