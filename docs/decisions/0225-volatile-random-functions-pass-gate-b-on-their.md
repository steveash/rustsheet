# 0225 — Volatile random functions pass Gate B on their semantic probes

- **Status:** accepted (2026-11-01, T-3631). Owner ruling (2026-11-01, in chat with the repository owner):
  RAND, RANDARRAY and RANDBETWEEN are covered by their own semantic probes, not by the 99.9% rate.
  Amends decision 0188 R1(b) (Gate B) and decision 0118's rate for these three functions only.
- **Context:**
  - Gate B (decision 0188 R1(b)) needs every in-scope function at ≥ 99.9% of its attributed cells
    within 15 digits. The eval run of 2026-11-01 (b6c931bd2) gives RAND, RANDARRAY and RANDBETWEEN 0
    attributed cells and 0 calling cells: every cell that depends on a volatile result is skipped
    (T-0005), so the rate can never be measured and Gate B can never reach 506/506.
  - The harness already scores invariant cells, whose value does not depend on the draw
    (`volatile::tests::invariant_cells_are_scored_and_stop_the_taint`): for example
    `=RANDBETWEEN(5,5)`, `=ROWS(RANDARRAY(3,2))` or `=AND(RAND()>=0,RAND()<1)`. No oracle-filled case
    holds such cells for these three functions.
- **Decision:** For RAND, RANDARRAY and RANDBETWEEN, Gate B and the decision 0118 rate are read on
  their invariant cells: an oracle-filled (excel-graph) semantic probe that pins what does not depend
  on the draw (range, spill shape, integer-ness, degenerate ranges, argument coercion and argument
  errors). Each function meets the rate when it has at least 20 such scored cells and all of them
  match Excel. Values that depend on the draw are never compared with Excel. The 20-cell minimum is
  the existing evidence floor (`EVIDENCE_FLOOR`): the three functions are exempt from decision
  0094's cell bars, so they are covered at 20 Excel cells, and a function with fewer stays below
  Gate B. Every scored cell of the `probes` suite that calls one of the three counts toward it (by
  decision 0118's attribution), not only the T-3631 cases: the harness already skips every
  draw-dependent cell (T-0005), so a scored probes cell of theirs is an invariant one.
- **Consequences:**
  - T-3631 added the probe cases and made the report count invariant cells for these three functions
    (Gate B and the M5/M7 rows read them like any other cell).
  - No other function is affected. A future volatile function in scope (for example a new random
    function) needs its own decision record to use this rule.
  - Distribution properties of the draws (uniformity, independence) are out of scope for Phase 1;
    they are not Excel parity and stay unmeasured.
