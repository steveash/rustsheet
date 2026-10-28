# 0153 — Allow one IMTAN regression unmasked by the T-3030 CELL re-fill

- **Status:** accepted (2026-10-28, T-3030)
- **Context:** Excel for the web (excel-graph) now reads `CELL("contents", ref)` and `CELL("type", ref)` on the sheet
  that `ref` names. Fills up to 2026-10-26 read the same address on the formula's own sheet (T-1230, T-1433). Fills
  on 2026-10-28 read the named sheet in every case: the re-filled probes/info-cell-contents-range (twice, identical) and
  info-cell-legacy-range, the new probes/t3030-cell-first-sheet (sheet order Data, Calc, Other), and
  t3030-cell-contents-nested-sheet (docs/excel-semantics/information.md). T-3030 makes CELL read the named sheet and
  re-fills from excel-graph the 47 cases whose old sidecars encoded the old reading. After that re-fill, one cell
  that passed before fails: gen-m9/756a9f7bc3a06bcc Calc!A338, `=IMTAN((CELL("contents",Data!$A$12)&OFFSET(..)))`.
  Its argument used to be Calc!A12 ("4") and is now Data!A12 ("3.14159"), where our IMTAN differs from Excel's in the
  last digits (-2.65358979336207E-06 against -2.65358979336212E-06). The engine's IMTAN did not change. The new
  evidence tests a different input.
- **Decision:** in the T-3030 PR, run `rustsheet-eval update-baseline --allow-regressions` once, for exactly this cell.
  The cell goes to T-3054.
- **Consequences:** eval/baseline/gen-m9.txt gains this one F line. Reviewers confirm that the baseline diff adds no
  other regression line. T-3054 must make the cell pass.
