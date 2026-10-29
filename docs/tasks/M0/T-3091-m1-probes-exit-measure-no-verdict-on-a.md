---
id: T-3091
title: "m1-probes exit measure: no verdict on a partial list run; exit-owners lists a crashing listed case"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-0089]
---

## Goal
M1's probe measure (the frozen `m1-probes` list, decision 0146 item 1) gets a verdict only from a run of the
whole list, and `make exit-owners MILESTONE=M1` lists a listed case that fails with no cell, as it does for probe rows.

## Context
- T-0089 reads the list row in `crates/rustsheet-eval/src/milestone_exit.rs` (`summarize`, `need.probe_list`) with a
  verdict whenever `cases_run > 0`. A run under `--filter` or `--suite` then gives a met/unmet verdict on a subset.
  Decision 0146 says a missing input is `n/a`, never met. Probe-milestone rows behave the same way today; only the
  list row is in scope here.
- `scripts/exit_owners.py` (`exit_cells`, the `milestone_lists` branch) prints only the list row's
  `visibly_off_cells`. A listed case that crashes or errors has no cells, so it is printed nowhere. The probe-row
  branch prints such a case as `case!*` from `failing_cases`, and `ListSummary.failing_cases` has the same data.
- Both found by the excel-fidelity reviewer on T-0089 (PR for agent/harness/T-0089-milestone-exit-rows).
- `scripts/` is a protected path: amend decision 0146's Consequences in the same PR.

## Acceptance criteria
- [ ] The M1 row's `probes_met` is `None` unless `cases_run == cases`; a unit test covers a partial run.
- [ ] `exit_owners.py --milestone M1` prints a failing listed case with no cells as `case!*`;
      `scripts/test_exit_owners.py` covers it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probe-milestone rows' partial-run verdicts.
