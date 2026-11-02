---
id: T-3823
title: "exit_owners --all-off --function NAME narrows to that group before matching owners (37 s to a few seconds per call)"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3822]
---

## Goal
`make unowned-off ALL=1 FUNCTION=<NAME>` answers in a few seconds on a full report, so a planner can check the
owners of every function of a milestone in one loop, with output identical to today's.

## Context
- Measured on 2026-11-02 (planner run, 2290b876, full `make eval` with calipers, corpus-bench and poi fetched):
  `python3 scripts/exit_owners.py --all-off --function CELL` takes 36.8 s wall; `--milestone M2` takes 6.4 s.
  A loop over M2's 37 functions, one call each, takes about 23 min.
- Cause (cProfile of the same call, 46 s under the profiler): `main_all_off` calls `unowned_off(report, ...)`,
  which matches every visibly-off cell of every function against the open tasks, and only then keeps the cells of
  `--function` (`cells = [(o, d) for o, d in listed if function in (None, o.cell.criterion)]`). Of the 46 s,
  39.9 s are in `unowned_off`: 34.4 s in `any(...)` over `names_case` / `names_beside_case`, 21.5 s in
  `re.Pattern.search`, and 7.2 s compiling about 46,600 regular expressions (`names`, `names_beside_case` build
  patterns per call).
- Fix direction: narrow `all_off_cells` to the group before `report_case_names` and the owner matching, and compile
  each per-case or per-name pattern once (`functools.lru_cache` on the builder, as exit_owners.py already imports
  `functools`). Keep one ownership rule: `owned` / `names_case` stay the only matchers (decisions 0136, 0138, 0139,
  0159).
- `scripts/` is a protected path: the PR needs a decision record (scripts/check_governance.py).

## Acceptance criteria
- [ ] On a full report (calipers, corpus-bench and poi fetched), `scripts/exit_owners.py --all-off --function CELL`
      runs in under 8 s wall on the same machine where today's code takes over 30 s; the PR states both timings.
- [ ] For `--function` set to CELL, BETA.DIST, `unattributed` and one `probe M<k>` group, and with no
      `--function`, the output is byte-identical before and after (the PR shows the diff of the saved outputs is
      empty).
- [ ] scripts/test_exit_owners.py gains a test that `--function` returns the same cells as the unfiltered run
      restricted to that group, on a report fixture with at least two groups.
- [ ] A decision record (`make new-decision`) covers the scripts/ change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing which task owns a cell (the matching rules themselves).
- The skipped-suite warning (T-3822, which this task follows so the two edits of exit_owners.py do not conflict).
- `scripts/phase1_exit.py`'s own run time, unless it calls the same narrowed path for free.
