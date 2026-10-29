# Calculation: circular references, iterative calculation, data tables

Iterative calculation (T-0447) and data tables (T-0441) are implemented without the Phase 2
dependency graph (M3, SPEC-0002 / T-0401); a circular reference without iteration keeps the
cells' values from before the recalculation, 0 for a new formula (T-0445, T-0461, below). A case sets iteration with the spec's top-level `"calculation"`
and data tables with a sheet's `"data_tables"`
(tools/src/rustsheet_tools/spec2xlsx.py, T-0428).

## How excel-graph calculates a probe workbook
Rule: the excel-graph oracle calculates each workbook three times: on load (`fullCalcOnLoad`),
at the explicit `calculate` `FullRebuild`, and at the scratch-sheet edit that makes Excel save
(tools/src/rustsheet_tools/oracle/backends.py). Iterative calculation resumes from the previous
values each time, so an oracle value of a cycle that has not converged shows 3 × iterateCount
iterations. A single desktop recalculation would show iterateCount. To score these cells, the eval
adapter runs the same three calculations for a case with `calculation.iterate` (T-0429). The
engine itself still calculates once per recalculation.
Evidence: probes/circular-iteration-on (`=A1+1` is 300 with iterateCount 100) and
probes/circular-iteration-limits (`=A1+1` is 15 with iterateCount 5); excel-graph, 2026-10-16,
T-0428.
Status: VERIFIED (excel-graph)
Tasks: T-0428, T-0401, T-0429

## The order Excel calculates a freshly loaded workbook in
Rule: a full recalculation of a freshly loaded workbook runs the formula cells in the reverse of
their load order: the last sheet first, and within a sheet the bottom row first, each row right to
left. A cell that reads a formula cell not yet calculated is abandoned, moved to just after that
precedent, and re-run from its start at its new place; the cells in between run first. A sheet moved after loading keeps its load position, and `calcChain.xml` does not change
the order on a full-calc load. Formulas entered or edited in a live session leave this order and
are not reproducible from the file. The order is visible only through WEEKNUM's stale return_type
state (dates.md, "WEEKNUM's stale return_type state") and AMORDEGRC's left-over value below period 1
(financial.md, "AMORDEGRC below period 1", which follows the same order: probes/amordegrc-fraction-state-n,
-o, -p, -q), the calc-order-dependent values known so far; iterative calculation sweeps a cycle in forward row-major order instead (below).
Evidence: probes/weeknum-stale-grid, -order-row, -order-column, -order-column-40, -order-2x3,
-order-3x2, -sheet-order, -sheet-moved, -dependency, -precedent-between and -precedent-restart (excel-graph, 2026-10-19, T-1484);
calcChain.xml and session edits: docs/research/t1484-weeknum-calc-order-2026-10-19.md.
Ours: `Model::run_pass` evaluates spill anchors first, then every cell in forward storage order
(sheet, row, column ascending). SPEC-0017 reproduces Excel's order where WEEKNUM's or AMORDEGRC's
state needs it (`OrderState`, decision 0131).
Status: VERIFIED (excel-graph)
Tasks: T-1484, T-2068, T-2069, T-2070, T-2825

## A circular reference without iteration keeps the cells' values from before the recalculation
Rule: with iterative calculation off, every cell of a cycle, and every cell whose evaluation reads one, keeps the value
it had before the recalculation, with no error value. In a loaded workbook that is the cached value; a formula never
calculated (every cell of a spec probe, whose builder caches 0) is 0 in ours; Excel with no cached `<v>` is
unprobed (T-0464). In fixtures/circular-keeps-cached-values the cached
values are A1 `=B1+1` 5, B1 `=A1` 4, C1 `=A1*2` 999, D1 `=D1+1` 7, E1 `=D1+100` 0, F1 `=1+1` 55, G1 `=F1*3` 0, H1 `=C1+1` 0,
I1 `=A1+F1` 0 and J1 `=IF(F1>100,A1,F1*10)` 0. After the recalculation, F1 is 2 and G1 6 (recalculated). A1 through E1,
H1 and I1 keep 5, 4, 999, 7, 0, 0 and 0; I1 keeps its value though it also reads the changed F1. J1, which never evaluates its
A1 branch, recalculates to 20. Calipers pv23_022, a Mac Excel model with two 3-statement loops, iteration off and `calcMode="manual"`, keeps
its cached loop values, one-cell dynamic arrays (`cm="1"`, D88 `=D37`) included. The rest of this entry was probed on
workbooks whose cached values are all 0, so each "is 0" below is "keeps 0". With no cached values, every cell of a cycle is 0: a
self-reference `=A1+1`, `=A2`, and the 2- and 3-cell cycles `B1=C1+1, C1=B1+1` and `D1=E1+1,
E1=F1+1, F1=D1+1`. A cell outside the cycle that reads it is also 0, whatever it computes:
`=A1+10` is 0 (not 10), `=SUM(B1:C1)+5` is 0, and `=ISNUMBER(A1)` is the number 0 (not FALSE). So is a
reader of a reader, at any depth (`=B1+1` reading `=A1+10`), and so are `=IFERROR(A1,5)`, `=ISERROR(A1)`,
`=TYPE(A1)`, `=ISBLANK(A1)` and a formula that would give text (`=IF(A1=0,"zero","other")`).
A reference IF does not evaluate is no cycle: `=IF(FALSE,B2,7)` in B2 and `=IF(TRUE,7,E2)` in E2 are
7. `=ROW(D2)+COLUMN(D2)` in D2 is 6 (ROW and COLUMN do not read the cell).
A formula that would spill does not spill when it reads a loop: `=SEQUENCE(3)+A1` (A1 `=A1+1`) is the single
number 0 and B2 is blank (`ISBLANK(B2)` TRUE, `ISREF(B2)` TRUE, `=B3` 0); `=ROWS(B1#)` over it is 0 (not 1) and `=SUM(B1:B3)` is 0 (both read B1, which is circular, so they are circular and 0).
Readers that take a reference without its value are not circular: `ISREF(A1)` TRUE, `CELL("row",A1)` 1,
`ROWS(A1)` 1, `COLUMNS(A1:B1)` 2, `ISFORMULA(A1)` TRUE, `OFFSET(A1,1,0)` (target holds 7) 7,
`INDEX(A1:A2,2)` 7, `ROWS(OFFSET(A1,0,0,2,1))` 2; `INDIRECT("A1")` reads the value, so it is 0.
Evidence: probes/circular-iteration-off (excel-graph, 2026-10-16, T-0428); probes/t0445-circular-readers
(excel-graph, 2026-10-22, T-0445); probes/t0451-circular-spill-and-ref-readers (excel-graph, 2026-10-22, T-0451).
Evidence (prior values): fixtures/circular-keeps-cached-values (excel-graph, 2026-10-25, T-0461); calipers
pv23_022 (Excel-cached, and the same values from an excel-graph recalculation of the file, 2026-10-25).
Ours: matches since T-0445, and the T-0451 probe passes in full (17/17). A read that closes a loop, or reads a cell
already known to be circular, marks every cell on the evaluation stack circular. A circular cell keeps its stored
value, which is 0 for a formula never evaluated, and does not spill (`Model::mark_stack_circular` and
`Model::circular_value`, crates/rustsheet-engine/src/circular.rs; T-0461). A spilling anchor on a loop is 0, and so are its
readers. An array anchor whose spill contradicts its own inputs (which the driver marks circular) keeps a cached value
when it is stored as a one-cell array, and is 0 otherwise. No probe covers an earlier spill kept on a loop, a loop
member with no cached value, or a cached text, boolean or error on a loop (T-0464). The fixture is the reproducible
evidence: pv23_022 is not vendored, and its file is in manual calculation mode.
Status: VERIFIED (excel-graph)
Tasks: T-0428, T-0401, T-0445, T-0451, T-0461, T-0464

## Iterative calculation sweeps the cycle in sheet order, in place
Rule: with iteration on, each iteration recalculates the cycle's cells once in row-major sheet
order, and each cell reads the values already updated in the same sweep (Gauss-Seidel, not
Jacobi). With iterateCount 100 and the three oracle calculations (300 sweeps):
- `B1=C1+1, C1=B1+1` is B1 599, C1 600;
- `C2=D2+1, D2=E2+1, E2=C2+1` is C2 449, D2 450, E2 450 (only the C2, D2, E2 order gives this);
- `=A1+1` is 300 and `=A1+10` reading it 310;
- `=F1*2+1` is 2^300 - 1 (2.037035976334486E+90);
- `=D1/2+1` converges to exactly 2, `=MIN(E1+1,5)` is 5, `=IF(A2<50,A2+1,A2)` is 50;
- `=B2` stays 0.
Evidence: probes/circular-iteration-on (iterate, iterateCount 100, iterateDelta 0.001;
excel-graph, 2026-10-16, T-0428).
Ours: matches (probes/circular-iteration-on 12/12). With iteration on, no evaluation marks a cycle
circular (0, as without iteration); a read that closes one returns the cell's current value (0 for a formula never
evaluated). The cells on the stack at that moment join the recalculated set, and so does every
cell that later reads a member. After the pass the members get back their values from before the
recalculation. They are then swept in row-major order (sheet, row, column), each reading the values
already updated. The xlsx import reads `iterate`, `iterateCount` and `iterateDelta`, and the export
writes them back. The Calipers models saved with iteration on now match Excel (pv23_021, pv26_004,
pv23_019, pv23_027 and pv26_012 bit for bit).
In a workbook with data tables, the substituted evaluations read a cycle's current values and do
not iterate it. Four data-table results of calipers pv26_014 differ from Excel by up to 1e-3
relative (T-0454). The reverse holds too: in the evaluation after the data tables, a cycle's members
get the values the first sweeps settled on and are not swept again. So a cycle that reads a data
table's results keeps the values it computed from the results before this recalculation (T-0454).
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/iteration.rs
Tasks: T-0428, T-0401, T-0429, T-0447, T-0454

## iterateDelta stops the iterations for the whole workbook, not per cycle
Rule: the delta stop ends the iterations of the whole workbook, not of each cycle on its own.
- **Some cell still changes by more than the delta.** With iterateCount 5 and iterateDelta 0.5,
  `=A1+1` changes by 1 and `=F1+0.6` by 0.6 on every iteration. So every cycle runs its 5
  iterations in each of the three calculations (15 in all), including cycles whose own change is
  already below the delta:
  - `=D1+0.1` is 1.5 (1.5000000000000002);
  - `=E1+0.5` is 7.5;
  - `=F1+0.6` is 9 (8.999999999999998);
  - `=B1/2+1` is 2 - 2^-14 (1.99993896484375);
  - `=C1/2+8` is 16 - 2^-11 (15.99951171875).
- **Every cycle converges.** With iterateCount 100, the cycles `=A1/2+1` and `=B1/2+8` stop early:
  - with iterateDelta 0.5, after 8 iterations in all over the three calculations: A1 2 - 2^-7
    (1.9921875), B1 16 - 2^-4 (15.9375);
  - with iterateDelta 0.001, after 17: A1 2 - 2^-16, B1 16 - 2^-13;
  - without the stop, the three calculations would run 300 iterations.
- **The stopping test** (T-0429): a calculation stops at the first iteration whose largest change
  over every recalculated cell is strictly below the delta. Cells outside the cycles that read them
  count too. Each later calculation then runs one more iteration (each one's first change is
  already below the delta). Three probes check it:
  - With C1 = A1+B1 outside the cycles (change 9 * 2^(1-n) at iteration n), 0.5 stops at n = 6
    (0.28; n = 5 gives 0.5625), so 6 + 2 = 8 iterations in all. 0.001 stops at n = 15 (0.00055),
    so 15 + 2 = 17.
  - Without C1 (probes/circular-iteration-delta-no-reader), 0.001 stops at n = 14 (B1's change
    8 * 2^(1-n) is 0.00098; n = 13 gives 0.0020): 14 + 2 = 16 in all, A1 2 - 2^-15 and
    B1 16 - 2^-12. The reader cell is what made the earlier count 17.
  - Strictly below: the lone cycle `=A1/2+1` with iterateDelta 0.25
    (probes/circular-iteration-delta-boundary) changes by 1, 1/2, 1/4, 1/8. It stops at n = 4,
    not at n = 3 where the change equals the delta: 4 + 2 = 6 in all, A1 2 - 2^-5 (1.96875). This reading
    rests on the one-more-iteration-per-later-calculation model, which the 8 and 17 counts back
    independently (`<=` would give 5, A1 1.9375).
- **Scoring:** the eval adapter runs an iterative case's three oracle calculations
  (`calculations` in crates/rustsheet-eval/src/engine.rs). It does not use one oracle
  calculation, because excel-graph saves only after an edit, and the edit itself calculates
  (T-0429). The engine still calculates once per recalculation.
Evidence: probes/circular-iteration-limits, probes/circular-iteration-delta,
probes/circular-iteration-delta-small (excel-graph, 2026-10-16, T-0428);
probes/circular-iteration-delta-no-reader, probes/circular-iteration-delta-boundary (excel-graph,
2026-10-16, T-0429).
Ours: matches (the five probes above). A sweep's change is the largest absolute difference over all
members, readers included. A member whose value changes type, text, boolean or error never counts
as converged. Excel's rule for those is unprobed (T-0459).
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/iteration.rs (`Model::sweep`)
Tasks: T-0428, T-0401, T-0429, T-0447

## Iterative sweeps evaluate at most 2^25 members per recalculation
Rule: Excel's limit on the work of iterative calculation is unprobed; it runs up to iterateCount
(at most 32,767) sweeps over every member.
Evidence: none for Excel; this is a resource limit, not an Excel observation.
Ours: the sweeps of one recalculation evaluate at most 33,554,432 members together
(`MAX_SWEEP_EVALUATIONS`). They stop where they are when the budget is spent, so a large cycle with
a large iterateCount cannot hold the evaluation without bound (CLAUDE.md directive 5). Their runs
also draw on the 2^32 ceiling of "One recalculation does at most 2^32 work units", and running out
of it stops them as their count cap does: the member being evaluated keeps its value from before
that run.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/iteration.rs (`MAX_SWEEP_EVALUATIONS`, `sweep`)
Tasks: T-0447, T-3009

## One recalculation does at most 2^32 work units, and a pass past 2^30 shares them out
Rule: Excel has no known limit on the work of a recalculation; a long one runs until done or
until interrupted, and a workbook that long cannot be probed (Excel for the web's session times
out).
Evidence: none for Excel; a resource limit (CLAUDE.md directive 5, SPEC-0028).
Ours: the formula runs of one recalculation may spend 2^32 work units in all (SPEC-0024's units:
every pass, restarted pass and run cut short by the evaluation depth limit, every iterative sweep
run and every data-table run). A pass whose formulas
commit more than 2^30 units, or a round whose restarted passes and cut-short runs spend more than
2^30, is run again with every formula limited to 2^30 / (4 × the number of formula cells) units;
a formula past that share is `#VALUE!` ("Recalculation needs too much work to calculate this
formula"), which `IFERROR` cannot catch, though a reader of the cell can. Past 2^32 every formula
cell is `#VALUE!` ("Recalculation needs too much work"), dynamic anchors spill nothing, and
neither the sweeps nor the data tables run. Which cells give up does not depend on the order the
cells are evaluated in, except through restarted passes and runs cut short by the evaluation depth
limit (SPEC-0028 section 3). Calibration (T-3008, every suite including calipers, poi and
corpus-bench): the heaviest eval pass commits 180,967,707 units and the heaviest eval
recalculation spends the same (probes/t2759-lunar-packed-7101), under a quarter of 2^30 and of
2^32; no eval workbook reaches share mode. Worked example of SPEC-0028 section 5 (release build,
heavy cells of 249,980,402 units each): A1:A4 compute in 24 s; A1:A5 give up in 31 s; A1:A1000
give up in 33 s; A1:A1000 with 99,000 light cells in 27 s, every light cell computed (before this
bound, about two hours). The sweeps and the data tables do not count against the 2^30 pass
allowance (their order is canonical, SPEC-0028 section 4) and, in share mode, run under the share.
When the ceiling runs out during a sweep, the sweeps stop where they are; when it runs out during a
data table, that result cell and every later one are `#NUM!`, and the second round of passes after
the tables is exhausted. A table the memo skips (T-0453) is charged the units its last
recomputation spent, and is skipped only in normal mode and while the ceiling has that many left,
so its results are those of a recomputation either way.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/recalc_budget.rs (`MAX_PASS_WORK`, `MAX_RECALC_WORK`), crates/rustsheet-engine/src/area_budget.rs (`with_area_budget`), crates/rustsheet-engine/src/evaluation.rs (`evaluate_passes`), crates/rustsheet-engine/src/iteration.rs (`sweep`), crates/rustsheet-engine/src/data_tables.rs (`spend_all_past_ceiling`, `evaluate_data_tables`)
Tasks: T-2807, T-3008, T-3009

## Data tables (TABLE()) are recalculated, and FORMULATEXT shows `{=TABLE(row,column)}`
Rule: excel-graph recalculates a what-if data table stored as `<f t="dataTable" ref=… dt2D=…
dtr=… r1=… r2=…>` in the top-left result cell. Each result cell is the table's formula, with the
input cell replaced by that row's or column's value. Cells that read the results see them:
- with A1 1, B1 2 and the formula `=A1*10+B1`, a column table over A1 = 1, 2, 3 is 12, 22, 32
  (`SUM` 66, `INDEX(…,3)` 32);
- the row table is the same (`SUM` 66);
- a two-variable table with row input B1 = 100, 200, 300 and column input A1 = 1, 2 sums to 1290
  (C12 is 220);
- a table whose formula `=B3*2` reads the formula B3 follows the chain: 104 and 124 for A1 = 5
  and 6.
The input cell keeps its value outside the table (`=B3` is 12). Every result cell counts as a
formula: `ISFORMULA` of a non-anchor result cell is TRUE, and `COUNT` of the results is 3.
`FORMULATEXT` of a result cell is the array-formula text `{=TABLE(,A1)}`: TABLE's first argument
is the row input and its second the column input, so the column table leaves the first empty.
Storage: excel-graph reads a one-variable column table written as `dt2D="0" dtr="0" r1=<input>`,
a row table as `dtr="1"`, and a two-variable table as `dt2D="1" dtr="1" r1=<row input> r2=<column
input>`. These are the forms spec2xlsx writes. No vendored Excel-authored file has a data table.
The fetched calipers suite has 21 workbooks saved by desktop Excel that do: 19 pv_goldens financial
models, roundtrip/i18n_hebrew and roundtrip/lbo_model (T-0446).
Evidence: probes/data-tables (excel-graph, 2026-10-16, T-0428).
Ours: matches (probes/data-tables 20/20). Each table is the sheet's `DataTable` metadata (its
results, row input and column input), and its result cells hold plain values. After an evaluation
pass, the tables are recomputed in sheet order. For each result cell the input cells are replaced
by the header values, the table's formula is evaluated on demand, and the inputs are put back. The
workbook is then evaluated again, so readers see the results. A table is not recomputed when
nothing its last recomputation read has changed: while it is recomputed every sheet records the
cells and areas read, and their contents (constants, formula texts, styles, spills), the sheets'
extents, the names, tables, settings and hidden rows and columns are kept with its results. A
formula that reaches RAND, RANDBETWEEN, RANDARRAY, NOW, TODAY, CELL, INFO or GETPIVOTDATA, or
iterative calculation, recomputes every time; when no table is recomputed the second evaluation
is skipped (`data_table_memo.rs`, T-0453). ISFORMULA and FORMULATEXT of a result
cell are TRUE and `{=TABLE(row,column)}`. The xlsx import reads `dt2D`, `dtr`, `r1`, `r2`, `del1`
and `del2`, and the export writes them back on the first result cell.
Unprobed, our choices: a deleted input (`del1`/`del2`), or a table with no input, makes every
result `#REF!`. A header cell or formula that spills or gives a range is read as its top-left
value (`scalar_at`). Neither is in probes/data-tables (T-0459).
Status: VERIFIED (excel-graph) for the rules above; the unprobed choices are not
Implemented in: crates/rustsheet-engine/src/data_tables.rs, crates/rustsheet-engine/src/data_table_memo.rs, crates/rustsheet-xlsx/src/import/sheet_data.rs (`data_table`), crates/rustsheet-xlsx/src/export/data_tables.rs
Tasks: T-0428, T-0441, T-0446, T-0453

## Data tables re-evaluate the workbook at most 2^25 formulas per recalculation
Rule: Excel's limit on the work of data tables is unprobed; it is expected to compute every
result cell of every table, however large the model.
Evidence: none for Excel; this is a resource limit, not an Excel observation.
Ours: every result cell re-evaluates the formulas its table's formula depends on. All the tables
of one recalculation may evaluate at most 33,554,432 formulas together
(`MAX_DATA_TABLE_EVALUATIONS`). The result cells computed after the budget is spent are `#NUM!`,
so a large table over a large model cannot hold the evaluation without bound (CLAUDE.md
directive 5). A table that is not recomputed (T-0453) is charged what its last recomputation
spent, so the tables after it find the budget as a recomputation would leave it. It is skipped
only while the budget left covers that cost, and one whose recomputation ran out is always
recomputed, so the results are those of a recomputation either way. Each run also draws on the
2^32 ceiling of "One recalculation does at most 2^32 work units", and running out of it stops the
tables as their count cap does: the result cell being computed and every later one are `#NUM!`.
A table that is not recomputed is charged the ceiling units its last recomputation spent too.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/data_tables.rs (`MAX_DATA_TABLE_EVALUATIONS`, `spend_all_past_ceiling`)
Tasks: T-0441, T-3009

## A data table holds at most 2^20 result cells, and a sheet at most 1,024 tables
Rule: Excel accepts a data table as large as the sheet; its limits are unprobed.
Evidence: none for Excel; this is a resource limit, not an Excel observation.
Ours: a table may have at most 1,048,576 result cells (`MAX_DATA_TABLE_CELLS`) and a sheet at
most 1,024 tables (`MAX_DATA_TABLES_PER_SHEET`). A table past either bound, overlapping another,
or with an input among its results is refused, by `Model::add_data_table` and the edit API and
by the xlsx import, which then fails (`DataTable::check`). The tables of one recalculation fill at
most 4,194,304 result cells together (`MAX_DATA_TABLE_RESULTS`). A table past that budget is left
as it is. A crafted `ref="B2:XFD1048576"` can therefore not force billions of result cells
(CLAUDE.md directive 5).
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/data_table_types.rs (`DataTable::check`), crates/rustsheet-engine/src/data_tables.rs (`MAX_DATA_TABLE_RESULTS`), crates/rustsheet-xlsx/src/import/sheet_data.rs
Tasks: T-0441
