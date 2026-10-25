# 0113 — Accept EXPAND and MAKEARRAY results past the cell bound found by T-2452 as directive-5 deviations

- **Status:** accepted (2026-10-25, T-2452)
- **Context:** T-2452 probed where Excel stops building EXPAND and MAKEARRAY columns (excel-graph,
  2026-10-25). Excel builds `COLUMNS(EXPAND(1,2,524289))` (524289), `ROWS(EXPAND(1,1048576,2))`
  (1048576), `COLUMNS(MAKEARRAY(2,524289,..))` (524289) and `COLUMNS(MAKEARRAY(1,1048577,..))`
  (1048577). Each array has more than 1,048,576 cells, the engine's cell bound
  (`limits::MAX_SIZE`), which exists for CLAUDE.md directive 5 and which decision 0014 already
  accepts as a deviation (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells are
  #ERROR!"). So ours is `#ERROR!` for all four, and listing them needs a change to the protected
  eval/suites/probes/deviations file. This follows decision 0108, which did the same for
  MAKEARRAY's 1048577 rows.
- **Decision:** List probes/t2452-expand-2x524289, -expand-1048576x2, -makearray-2x524289 and
  -makearray-cols-1048577 (cell Sheet1!A1 of each) in eval/suites/probes/deviations under decision
  0014 with the pinned value `#ERROR!`, citing the existing cell-bound DEVIATION entry. No new kind
  of deviation is created; the bound is unchanged.
- **Consequences:** The cells are scored as accepted deviations instead of known failures. If the
  cell bound is ever raised past 2^21 cells, the deviations lines fail their pins and must go.
