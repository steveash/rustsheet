---
id: T-0080
title: "Eval report run metadata (commit, date, skipped suites) and v15_checkpoint Phase 1 rate on the attributed count (decision 0118)"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-0068]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
Two changes to report.json, in their own diff:
- it records the commit, date and skipped suites of the run, so the dashboard can tell a complete
  run from a stale one;
- `v15_checkpoint` and its Phase 1 line measure decision 0114's exit with decision 0118's
  attributed count. The stricter count stays, labelled as information.

## Context
- Decision 0118 (owner, 2026-10-25) and SPEC-0022, "Glue-aware attribution lives in the Rust eval
  report" and "Failure handling".
- The attributed and `excel` closeness fields come from T-0068.
- crates/rustsheet-eval/src/v15_checkpoint.rs: `V15Function::rate`, `meets_rate`, `at_rate`,
  `phase1_met` and `phase1_line`.
- main.rs prints "skipping …: not fetched" for an unfetched suite. Record that same reason.
- Determinism: `run.date` and `run.commit` vary per run. On 2026-10-25 no test or golden file
  compares a whole report.json. tools/tests/test_parity_report.py and scripts/test_exit_owners.py
  build synthetic reports, and no file under crates/rustsheet-eval/tests exists. Keep it that way.

## Acceptance criteria
- [ ] report.json has `run: {commit, date, skipped_suites: [{name, reason}]}`:
  - `commit` is `git rev-parse --short=8 HEAD`, or null when git fails; a git failure is never
    an error;
  - `date` is the UTC run date (YYYY-MM-DD);
  - `skipped_suites` lists each suite under eval/suites that did not run, with its reason.
- [ ] `V15Function` measures its rate over T-0068's `attributed` cells, or over `excel` below
      `EVIDENCE_FLOOR` attributed cells. `meets_rate`, `at_rate`, `phase1_met` and the Phase 1
      line follow it. `covered` still counts every Excel cell that calls the function (decision
      0094).
- [ ] The stricter share (all Excel cells that call the function, with decision 0088's `visible`
      blame) stays as information fields, for example `strict_rate`, `meets_strict_rate` and a
      strict at-rate total. report.md prints them on a line labelled "information".
- [ ] Unit tests cover:
  - a function that meets the rate on attributed cells but not on the strict count;
  - the fewer-than-20 fallback;
  - `skipped_suites` for an unfetched suite;
  - a null `commit`.
- [ ] No test or golden file compares a whole report.json, so `run` cannot make a test flaky.
      Re-check this and say so in the Outcome.
- [ ] docs/milestones.md's Phase 1 exit paragraph cites decision 0118 for how a function's cells
      are counted. The steward skill's wording is T-0076's.
- [ ] The Outcome lists every function whose `meets_rate` is newly true only because of the
      attribution or the fallback, by comparing the strict and attributed counts on a fresh
      `make eval` of `main` (poi, corpus-bench and calipers fetched). It gives the at-rate totals
      under both counts.
- [ ] `make check` green, with no baseline or lock change; reviewers approve.

## Out of scope
- Changing decision 0094's checkpoint (`met`, `covered`, `visibly_off`) or any gate.
- Rendering (T-0081, T-0072).
