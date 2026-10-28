# 0140 — M7 exit measures: gen-m7, spill extents and M7 probes

- **Status:** accepted (2026-10-28, T-2888).
- **Amends:** decision 0096 (the exit-owner report extends to M7) and decision 0135 (a milestone
  without an ironcalc-calc criterion).
- **Context:**
  - M7's exit text (docs/milestones.md "M7"): "generated dynamic-array suite (≥ 10k cells, spill
    shapes included) at ≥ 99.9%; spill extents identical to Excel; its `probes` cases (decision
    0051) at 100% bit-exact". Decision 0114 reads the exit at visible parity. M7 has no
    catalog-coverage criterion and no ironcalc-calc criterion.
  - `milestone_exit.rs` covered M2, M5 and M6, so `make exit-owners MILESTONE=M7` stopped with
    "report.json has no milestone_exit row for M7".
  - Spill extents (decision 0063, T-1660): in a spec case with an Excel sidecar, each cell the
    engine fills that Excel left empty fails, but stays out of `checked`. A suite's scored-cell
    rate therefore cannot show that "spill extents are identical". A spill smaller than Excel's is
    different: the oracle records every spill member, so a member the engine leaves empty is
    already an ordinary failing scored cell.
- **Decision:**
  1. `milestone_exit` gains an M7 row with these measures:
     - **gen-m7 alone.** It needs at least 10,000 scored cells, and at least 0.999 of them not
       visibly off: the allowance is `floor(0.001 * scored)`, M5's rate.
     - **Spill extents.** Extent failures are counted apart from the scored cells: the cells
       `spill_extent.rs` adds, marked `"kind": "spill_extent"` in report.json. M7 counts those of
       gen-m7 and those of the `M7` probe row, and requires 0 (`extent_failures`, `extent_met`). Rows
       without the criterion leave both null.
     - **The `M7` probe row.** It must have 0 visibly-off cells.
     - **Not required.** Catalog coverage and the ironcalc-calc M7 function-scope row are shown
       with a null verdict (`n/a`), as M2 shows coverage, and `exit` ignores them.
  2. A measure that is shown but not required has its counts and a null verdict. For
     ironcalc-calc that means `calc_failing` set and `calc_met` null. `scripts/exit_owners.py`
     lists ironcalc-calc cells only for a row whose `calc_met` is not null, so M7's exit owners
     are its gen-m7 cells and its `M7` probe cells. The extent cells are among those, since they
     fail and are visibly off.
  3. The ownership rule of decision 0096 is unchanged.
- **Consequences:**
  - The steward and the planner run `make exit-owners MILESTONE=M7` like M2, M5 and M6.
  - report.md's milestone-exit table gains a spill-extent column. It reads `n/a` for M2, M5 and
    M6.
  - Exit rows for M8 to M11 still need their own decision.
