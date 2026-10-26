---
id: T-2822
title: "Milestone exit row for M2 so make exit-owners MILESTONE=M2 runs"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The eval report's "Milestone exit (decisions 0085, 0114)" section has an M2 row that measures M2's exit criteria from
docs/milestones.md, and `make exit-owners MILESTONE=M2` lists the failing M2 exit cells that no open M2 task owns. The
planner and the steward then check M2's exit with the tools instead of by hand.

## Context
- Today `make exit-owners MILESTONE=M2` stops with `report.json has no milestone_exit row for M2`, because
  `MILESTONES` in crates/rustsheet-eval/src/milestone_exit.rs is `["M5", "M6"]`. docs/status.md (2026-10-26 snapshot)
  records that the planner has had to check M2 by hand on every run since M2 became current.
- The second blocker is in scripts/exit_owners.py: `exit_cells` looks for a `probe_milestones` row whose `milestone` is
  the requested one, but the report labels M2's probe row `M0-M2` (decision 0051: M0, M1 and M2 probes count
  together toward M2). A run on the 2026-10-26 report with the M2 row added and `M0-M2` read as M2 found 18 exit cells,
  all in probes; 13 were owned (T-2781..T-2785, T-2769) and 5 were not, because T-2771 named them as the range
  `A52-A58`, which the ownership rule does not expand (T-2771 now names each cell).
- M2's criteria (docs/milestones.md "M2", settled by T-1472 and decision 0058; read at visible parity by decision 0114):
  - `gen-m2` has at least 10,000 scored cells and none visibly off ("100%"; not M5's 20,000 and 0.999 allowance).
  - The ironcalc-calc M2 row of "Milestone function scope" has no visibly-off cell once
    eval/milestones/exit-exceptions.tsv's M2 lines are left out (already computed: `function_scope.rows` has `M2`).
  - The `M0-M2` row of "Probe milestones" has no visibly-off cell.
  - M2 has no catalog-coverage criterion. `catalog_coverage` has an M2 row (37/37 on 2026-10-26); the M2 exit row
    shows it but does not require it, or leaves it `n/a`; say which in the decision record.
  - The structural criterion (one implementation of aggregate iteration and criteria matching, duplication down
    15%) is not a cell measure and stays out of the row.
- Code: `summarize` and `write_markdown` in crates/rustsheet-eval/src/milestone_exit.rs (its thresholds are constants
  shared with `attribution.rs` and `v15_checkpoint`; add M2's own, do not change M5's); `exit_cells` in
  scripts/exit_owners.py, tests in scripts/test_exit_owners.py. scripts/ is a protected path: the PR needs a decision
  record (`make new-decision`) that extends decision 0096 to M2 and names the `M0-M2` probe row.

## Acceptance criteria
- [ ] `make eval` writes an M2 row in report.json `milestone_exit` and in report.md's "Milestone exit" table, with
      gen-m2's threshold of 10,000 scored cells and 0 visibly off; M5 and M6 rows are unchanged (a unit test pins
      each M2 threshold at its edge: 9,999 scored cells not met, 1 visibly-off gen-m2 cell not met).
- [ ] The M2 row's probe measure reads the `M0-M2` probe row, and its ironcalc-calc measure the M2 function-scope row
      less the M2 exit exceptions (IF_ARRAY.xlsx Sheet1!C11 does not count).
- [ ] `make exit-owners MILESTONE=M2` runs and prints the unowned M2 exit cells; on the report of the PR's run it
      prints 0 (or each cell it prints gets an owner in the same PR's task files). The count is in the Outcome.
- [ ] scripts/test_exit_owners.py has a test that M2's probe cells come from the `M0-M2` row.
- [ ] Decision record extending decision 0096 to M2 added; `make check` green; reviewers approve.

## Out of scope
- Exit rows for M7 to M11 (milestone_exit.rs: "M7 to M11 need their own decision first").
- Range or span ownership (`A52-A58`, `A5:D5`): T-2731.
- Measuring the duplication or one-implementation criterion of M2.
