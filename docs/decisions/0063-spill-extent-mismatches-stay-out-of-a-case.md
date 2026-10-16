# 0063 — Spill-extent mismatches stay out of a case's checked count

- **Status:** accepted (2026-10-16, T-1660)
- **Context:** The harness scored only the cells in a spec case's oracle sidecar. So when the engine spilled into a cell
  Excel left empty, the case still passed. probes/ceiling-floor-mround-array-lift was found this way in the T-1589 review,
  and probes/radians-sqrtpi-munit B80 is another: Excel gives one `#VALUE!` and the engine spills a second value. T-1660
  adds a spill-extent check to every spec case with an Excel sidecar. The check fails each cell the engine fills that is
  neither an input nor recorded by the oracle; the oracle records every cell Excel's formulas fill, spill members
  included. Cells near a formula the oracle left pending are excepted. The baseline (`baseline.rs`) counts a new failure
  as a regression only while the case's checked count is unchanged. If extent cells added to that count, an engine
  change that spilled further would be recorded as "new coverage" and pass the gate.
- **Decision:** Spill-extent cells are listed in the case's results and fail it, but they are not counted in `checked`.
  A new extent failure in an existing case is therefore a regression. The extent mismatches that already exist when the
  check lands (5 cells in probes on 2026-10-16) are recorded once with `rustsheet-eval update-baseline
  --allow-regressions`. They are not engine regressions: the check is new, and they are listed in T-1660's Outcome.
- **Consequences:** A PR that makes the engine spill past Excel's extent fails `make eval-check`. A fix removes the
  extent F lines like any other fix. Later scoring checks that add failing items to existing cases follow the same
  pattern: stay out of `checked`, and record the first baseline under a decision record.
