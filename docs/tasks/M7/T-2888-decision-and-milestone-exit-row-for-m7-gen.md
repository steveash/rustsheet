---
id: T-2888
title: "Decision and milestone exit row for M7 (gen-m7, spill extents, M7 probes) so make exit-owners MILESTONE=M7 runs"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2822]
---

## Goal
The eval report's "Milestone exit (decisions 0085, 0114)" table has an M7 row that measures M7's exit criteria from
docs/milestones.md at visible parity, and `make exit-owners MILESTONE=M7` lists the failing M7 exit cells that no open
M7 task owns. A decision record fixes how each M7 criterion is measured, as decision 0085 did for M5. M7 comes next
in the phase order after M6, so the planner and steward can then check it with the tools and not by hand.

## Context
- M7's exit text (docs/milestones.md "M7"): "generated dynamic-array suite (≥ 10k cells, spill shapes included) at
  ≥ 99.9%; spill extents identical to Excel; its `probes` cases (decision 0051) at 100% bit-exact". It has no catalog
  coverage criterion and no ironcalc-calc criterion. `MILESTONES` in crates/rustsheet-eval/src/milestone_exit.rs
  covers M5 and M6 (T-2822 adds M2 and per-milestone thresholds), and its doc comment says "M7 to M11 need their own
  decision first". This task is that decision for M7.
- Planner run of 2026-10-27 (`make eval` on 6087413bc, calipers, corpus-bench and poi not fetched, none of which
  these rows read):
  - Suite gen-m7: 55 cases, 36,641 scored cells, 0 known failures (eval/baseline/gen-m7.txt has no F line). Its
    spill shapes come from T-2224's reserved-footprint spilling probes (T-2225, done).
  - "Probe milestones (decision 0051)" M7 row: 242 cases, 34,704 scored, 8 deviations, 55 known failures, 55 visibly
    off.
  - Spill extents: the spill-extent check of decision 0063 (crates/rustsheet-eval/src/spill_extent.rs, T-1660) fails
    each cell the engine fills that Excel left empty, in every spec case with an Excel sidecar, but keeps those cells
    out of `checked`. So "spill extents identical" is not visible in a suite's scored-cell rate: the row must count
    extent failures of gen-m7 and the M7 probe cases separately, and require 0 of them. Say in the decision whether a
    spill smaller than Excel's (a recorded member the engine leaves empty) is already an ordinary failing scored cell
    (it should be: the oracle records every spill member).
- A local approximation of `exit-owners` over the M7 probe row (scripts/exit_owners.py `owned` on the
  `probe_milestones` M7 failing cases) found 13 cells with no M7 owner on 2026-10-27. All 13 were owned in substance by
  T-2519 and T-2460, which named them only as spans (`A5:D5`, `D49-D65`); this planner run added each cell by name to
  those tasks, so the PR's run should print 0 unless new failures appear.
- Code: `summarize` and `write_markdown` in milestone_exit.rs; `gen-m<k>` and the probe row are already matched by
  milestone name in scripts/exit_owners.py `exit_cells`, so M7 needs no change there unless the extent cells are not in
  `failing_cells`; if exit_owners.py changes, it is a protected path and the decision record covers it.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) fixes M7's measures: gen-m7 alone, at least 10,000 scored cells and at
      least 0.999 of them not visibly off (allowance `floor(0.001 * scored)`); spill-extent failures of gen-m7 and the
      M7 probe cases counted separately and required to be 0; the M7 probe row with 0 visibly-off cells; coverage and
      ironcalc-calc shown as `n/a` (not required). docs/milestones.md's M7 exit text cites it.
- [ ] `make eval` writes an M7 row in report.json `milestone_exit` and report.md's table; M5, M6 (and T-2822's M2)
      rows are unchanged. Unit tests pin each M7 threshold at its edge (9,999 scored cells not met; allowance + 1
      visibly-off cells not met; one extent failure not met).
- [ ] `make exit-owners MILESTONE=M7` runs; every cell it prints on the PR's run gets an owner in the same PR (case stem
      and A1 name added to an open M7 task, or a new task filed). The count is in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Exit rows for M8 to M11 (their decision tasks are filed separately).
- Span and shared-stem ownership in exit_owners.py (T-2731, T-2880).
- Fixing any M7 cell.

## Outcome
- Decision 0140 fixes M7's measures. `milestone_exit` gains an M7 row:
  - gen-m7 needs 10,000 scored cells with allowance `floor(0.001 * scored)`;
  - spill-extent failures of gen-m7 and the M7 probe row are counted apart and must be 0
    (`extent_failures`, `extent_met`);
  - the M7 probe row needs 0 visibly-off cells;
  - coverage and ironcalc-calc counts are shown with a null verdict and are not required.
- Plumbing:
  - Spill-extent cells carry `spill_extent: true` in report.json (runner.rs `CellResult`, set by `spill_extent.rs`).
  - Probe-milestone rows count them in `spill_extent_failures`.
  - report.md's exit table gains a "spill extents" column (`n/a` for M2, M5 and M6).
  - `scripts/exit_owners.py` skips the function-scope row of an exit row that shows it without requiring it.
  - The progress page prints "ironcalc-calc … (not required)" and M7's spill extents.
  - docs/milestones.md's M7 exit text cites 0140.
- Tests:
  - `m7_needs_ten_thousand_gen_cells_at_the_rate_and_no_extent_failure` and `one_m7_extent_failure_is_not_met`
    (milestone_exit.rs) cover 9,999 cells, allowance + 1 and one extent failure in gen-m7 or the probes.
  - `spill_extent_cells_are_counted_in_their_row` (probe_milestones.rs).
  - `test_a_calc_row_shown_but_not_required_lists_no_cell` (exit_owners).
  - `test_an_m7_row_shows_calc_not_required_and_its_spill_extents` (progress goals).
- PR's run (`make eval` on 3eec4e19 plus this branch): the M7 row reads
  `| M7 | 32/32 n/a | 36641, 1.00000, 0/36 [0] met | 0 [0] n/a | 39 [39] not met | 0 met | not met |`.
  The M7 probes have 39 visibly-off cells, so the exit is not met. The run's only 2 spill-extent cells are in M8 and M9
  probes (`table-delete-row-all-data-below` C7, `t2576-error-type-cell-paired` F34).
- M2, M5 and M6 rows are field-for-field the same as main's, apart from the two new null fields. M2's probe count
  differs only by #2098, which merged in between.
- `make exit-owners MILESTONE=M7`: 0 unowned (all 39 cells already have M7 owners), so no task was added. Eval delta: none
  (no verdict changes).
