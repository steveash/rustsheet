# Calculation: circular references, iterative calculation, data tables

Phase 2 (M3, SPEC-0002 / T-0401) implements these; the cases below are its acceptance tests and
are known failures until then. A case sets iteration with the spec's top-level `"calculation"`
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

## A circular reference without iteration is 0, and so is every cell that reads it
Rule: with iterative calculation off, every cell of a cycle is 0, with no error value: a
self-reference `=A1+1`, `=A2`, and the 2- and 3-cell cycles `B1=C1+1, C1=B1+1` and `D1=E1+1,
E1=F1+1, F1=D1+1`. A cell outside the cycle that reads it is also 0, whatever it computes:
`=A1+10` is 0 (not 10), `=SUM(B1:C1)+5` is 0, and `=ISNUMBER(A1)` is the number 0 (not FALSE).
A reference IF does not evaluate is no cycle: `=IF(FALSE,B2,7)` in B2 and `=IF(TRUE,7,E2)` in E2 are
7. `=ROW(D2)+COLUMN(D2)` in D2 is 6 (ROW and COLUMN do not read the cell).
Evidence: probes/circular-iteration-off (excel-graph, 2026-10-16, T-0428).
Ours: every cycle cell and every reader is an error; the IF and ROW/COLUMN cells already match.
Status: VERIFIED (excel-graph)
Tasks: T-0428, T-0401

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
Ours: every cell is an error.
Status: VERIFIED (excel-graph)
Tasks: T-0428, T-0401, T-0429

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
    not at n = 3 where the change equals the delta: 4 + 2 = 6 in all, A1 2 - 2^-5 (1.96875).
- **Scoring:** the eval adapter runs an iterative case's three oracle calculations
  (`calculations` in crates/rustsheet-eval/src/engine.rs). It does not use one oracle
  calculation, because excel-graph saves only after an edit, and the edit itself calculates
  (T-0429). The engine still calculates once per recalculation.
Evidence: probes/circular-iteration-limits, probes/circular-iteration-delta,
probes/circular-iteration-delta-small (excel-graph, 2026-10-16, T-0428);
probes/circular-iteration-delta-no-reader, probes/circular-iteration-delta-boundary (excel-graph,
2026-10-16, T-0429).
Ours: every cell is an error.
Status: VERIFIED (excel-graph)
Tasks: T-0428, T-0401, T-0429

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
input>`. These are the forms spec2xlsx writes. No Excel-authored file in the repo has a data table.
Evidence: probes/data-tables (excel-graph, 2026-10-16, T-0428).
Ours: the result cells are blank (the import has no data tables), so the reads are 0 or wrong.
The table formulas themselves already match.
Status: VERIFIED (excel-graph)
Tasks: T-0428
