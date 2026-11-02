---
id: T-3822
title: "exit_owners names the suites a partial make eval skipped, as phase1-exit does, and --strict fails on them, instead of reporting 0 unowned cells"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`make exit-owners` and `make unowned-off` never print a clean "0 ... have no open owner" from a report that skipped
a suite: they name the skipped suites, and `--strict` exits non-zero for them, so a planner or steward cannot take a
partial run for a complete one.

## Context
- Seen on 2026-11-02 (planner run, 2290b876): on a fresh worktree `make eval` skipped calipers, corpus-bench and poi
  ("skipping calipers: not fetched ..."; report.json `run.skipped_suites`). `scripts/exit_owners.py --all-off`
  then printed `0 ... have no open owner` for every M2 function and `--unattributed` printed `0 visibly-off cells
  attributed to no function have no open owner`, with no hint that three suites were missing. After `rustsheet-eval
  fetch` of the three and a second `make eval`, `--all-off` listed 2 unowned non-deferred cells, both
  `CELL("filename",...)` in calipers pv_goldens workbooks (this planner run gave them to T-2663, which lists them).
  The first answer was wrong, and nothing said so.
- `scripts/phase1_exit.py` already reads `run.skipped_suites` and reports `run not met ... skipped suites: ...`
  (its `run` check). exit_owners.py reads the same report.json and never looks at it. The rule belongs in one place:
  reuse or move phase1_exit's reading of `skipped_suites` (one definition per concept), do not copy it.
- `--milestone` is affected for the milestones whose exit row reads a fetched suite (M11 corpus-bench, M8 the poi
  M8 subset); `--all-off` and `--unattributed` are affected for every fetched suite (Gate B counts calipers,
  corpus-bench and poi).
- T-3556 (spec: how CI and `make eval` treat the fetched suites) and T-3557 (`run --fetch-missing`) will make a
  skip rarer; this task makes the remaining skips visible to the owner tools. It does not depend on either.
- T-3438 and T-3623 (open) also change scripts/exit_owners.py, in its ownership matching; this change is in the
  report reading and the count lines, so whichever lands second rebases.
- `scripts/` is a protected path: the PR needs a decision record (scripts/check_governance.py).

## Acceptance criteria
- [ ] When report.json's `run.skipped_suites` is non-empty, every mode of scripts/exit_owners.py prints, before its
      count line, one line `warning: report skipped suites: <names> (run `rustsheet-eval fetch --suite <name>` and
      make eval again); counts below leave out their cells`, on stderr; with `--strict` it exits 1 even when the
      count is 0. With no skipped suite the output is byte-identical to today's.
- [ ] The skipped-suite list is read by one helper shared with scripts/phase1_exit.py (no second parser of
      `run.skipped_suites`).
- [ ] scripts/test_exit_owners.py gains tests: a report with a skipped suite warns in `--milestone`, `--all-off` and
      `--unattributed`, and `--strict` exits 1; a report without one is unchanged.
- [ ] A decision record (`make new-decision`) covers the scripts/ change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fetching suites from exit_owners or `make eval` (T-3557, T-3556).
- The progress dashboard's reading of a partial run (T-3556's spec decides it).
