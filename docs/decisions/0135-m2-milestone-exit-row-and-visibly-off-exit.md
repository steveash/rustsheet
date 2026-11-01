# 0135 — M2 milestone exit row and visibly-off exit owners

- **Status:** accepted (2026-10-27, T-2822).
- **Amends:** decision 0096: it extends to M2, and the exit-owner report lists visibly-off cells only.
- **Context:**
  - `make exit-owners MILESTONE=M2` stopped with "report.json has no milestone_exit row for M2".
    `milestone_exit.rs` covered only M5 and M6, so the planner and the steward checked M2's exit
    by hand on every run since M2 became current (docs/status.md).
  - `scripts/exit_owners.py` looked for a probe row labelled `M2`. The report labels it `M0-M2`,
    because M0, M1 and M2 probes count together toward M2 (decision 0051 item 3).
  - Decision 0114 reads every exit at visible parity, and the M5 and M6 rows count visibly-off
    cells. `exit_owners.py` still listed every bit-level failure. For example, T-2771's
    `t2756-roundup-smallest-normal-threshold` A52–A58 are within 15 digits (`numeric_close`), so
    they are not M2 exit cells.
  - M2's criteria (docs/milestones.md "M2"; decision 0058; read at visible parity by decision
    0114): `gen-m2` has at least 10,000 scored cells with none visibly off; the ironcalc-calc M2
    function-scope row has no visibly-off cell once the M2 exit exceptions are left out; and the
    `M0-M2` probe row has no visibly-off cell. M2 has no catalog-coverage criterion. The
    structural criterion (one aggregate walker, one criteria matcher, duplication down 15%) is not
    a cell measure.
- **Decision:**
  1. `milestone_exit` gains an M2 row:
     - `gen-m2` needs `M2_MIN_SCORED` = 10,000 scored cells with an allowance of 0 visibly off.
       M5's 20,000 cells and 0.999 rate are unchanged.
     - The ironcalc-calc measure is the M2 function-scope row less its `excepted` cells.
     - The probe measure reads the `M0-M2` row; each exit row names the row it read in
       `probe_row`.
     - Catalog coverage is shown (`covered`, `functions`) but is not required: `coverage_met` is
       null (`n/a`) and `exit` ignores it.
     - The structural criterion stays out of the row and is checked by its own audit (T-2910).
  2. The report carries the visibly-off cells the exit counts: `milestone_exit[].gen_visibly_off_cells`
     and `probe_milestones[].visibly_off_cells`, both as `case!Sheet!A1`. The function-scope rows
     already carry `visibly_off`.
  3. For every milestone, `scripts/exit_owners.py` lists only the visibly-off exit cells from
     those lists, plus each probe case that fails with no cell (`case!*`). Within-15-digit
     failures are Phase 1b's and need no M-milestone owner. It reads closeness from the report and
     never re-derives it (CLAUDE.md, tools never re-implement value parsing).
  4. The ownership rule of decision 0096 is unchanged. Ranges are still not expanded (T-2731).
- **Consequences:**
  - The steward and the planner run `make exit-owners MILESTONE=M2` like M5 and M6.
  - For M5 and M6, `exit-owners` may list fewer cells than before: bit-only failures drop out.
    Those cells remain known failures in eval/baseline and in the bracketed Phase 1b counts.
  - Exit rows for M7 to M11 still need their own decision.
