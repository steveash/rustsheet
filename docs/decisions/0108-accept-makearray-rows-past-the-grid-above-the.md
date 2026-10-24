# 0108 — Accept MAKEARRAY rows past the grid above the cell bound as a directive-5 deviation

- **Status:** accepted (2026-10-24, T-2600)
- **Context:** Excel builds `MAKEARRAY(1048577,1,LAMBDA(r,c,1))`, so its ROWS is 1048577
  (probes/t2579-array-value-edges P!B23, excel-graph, 2026-10-24). That array has 1,048,577 cells,
  one more than the engine's cell bound (`limits::MAX_SIZE`, one whole column), which exists for
  CLAUDE.md directive 5 and which decision 0014 already accepts as a deviation
  (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells are #ERROR!"). T-2600 makes
  MAKEARRAY build arrays past the grid, so the cell is now refused by that bound (`#ERROR!`)
  rather than by a per-dimension check (`#VALUE!`), and listing it needs a change to the protected
  eval/suites/probes/deviations file.
- **Decision:** List probes/t2579-array-value-edges P!B23 in eval/suites/probes/deviations under
  decision 0014 with the pinned value `#ERROR!`, citing the existing cell-bound DEVIATION entry. No
  new kind of deviation is created; the bound is unchanged.
- **Consequences:** The cell is scored as an accepted deviation instead of a known failure. If the
  cell bound is ever raised past 1,048,577 cells, the deviations line fails its pin and must go.
