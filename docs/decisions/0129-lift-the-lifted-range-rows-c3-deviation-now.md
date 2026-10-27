# 0129 — Lift the lifted-range-rows C3 deviation now that a spill anchor's over-bound lift is #SPILL!

- **Status:** accepted (2026-10-26, T-0483)
- **Context:** probes/lifted-range-rows C3 (`=DATE(11:1899,12,1)` in a cell) is `#SPILL!` in Excel
  (excel-graph, 2026-10-06). It was listed in `eval/suites/probes/deviations` under decision 0014, pinned to the
  array bound's `#ERROR!`. T-0483 decides that cell from the shapes of the lifted ranges before anything is
  built (`Model::unspillable_lift`, docs/excel-semantics/arrays.md "Array results above 1,048,576 cells are
  #ERROR!"), so the engine now gives Excel's value and the harness reports the line as stale.
- **Decision:** remove the C3 line from `eval/suites/probes/deviations`, as decision 0036 prescribes for a lifted
  deviation. This only removes a line: no cell is newly hidden from bit-exact. C1, C2 and E1 (value contexts)
  stay listed, pinned to `#ERROR!`.
- **Consequences:** C3 is scored bit-exact again and a regression there fails the eval gate. No gate is weakened.
