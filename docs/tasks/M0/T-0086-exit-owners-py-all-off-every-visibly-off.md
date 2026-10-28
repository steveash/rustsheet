---
id: T-0086
title: "exit_owners.py --all-off: every visibly-off Excel cell no open task owns, attributed, unattributed and probe, grouped by function"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
One command lists every visibly-off Excel cell of the last eval run that no open task owns, so the steward and the
gap-triager can keep the Phase 1 rule "each remaining failure has a filed task" (docs/milestones.md) true: attributed
cells grouped by function (decision 0118), unattributed cells (decision 0132), and probe-milestone cells of every
milestone, each with case, cell, formula, expected and actual.

## Context
- `scripts/exit_owners.py` today has `--milestone` (M5 and M6 only: it needs a `milestone_exit` row, which M0-M2
  and M7-M11 lack until T-0089) and `--unattributed`. Nothing lists the attributed cells of functions below the
  Phase 1 rate, which is most of the gap.
- The T-0090 triage did this by hand: for every report.json `cases[].cells[]` entry with `from_excel`, it applied
  the eval's visibly-off rule (`crates/rustsheet-eval/src/closeness.rs`: not within 4 ulp and not
  `within_fifteen_digits`, or a non-numeric verdict), attributed the cell with `sole_call` (the GLUE list in
  closeness.rs; the formula's single non-glue function), and checked `owned()` against the open tasks. It found
  about 81,000 unowned cells; after the triage's own edits (T-2981 owns the incomplete-beta sweeps, T-2425 the
  FORECAST.ETS cells) 478 attributed, 365 unattributed and 171 probe-milestone cells remain (T-0087's list). Cache the stem lookups per case: the naive loop over 100k cells x 380 tasks takes over 10 minutes.
- The visibly-off rule must not be re-implemented divergently (directive 3): either have the eval write a
  per-cell `visibly_off` flag (and the attributed function) into report.json `cases[].cells[]`, which is the
  preferred route, or reuse one Python helper with a test that pins it to closeness.rs's boundary cases.

## Acceptance criteria
- [x] `python3 scripts/exit_owners.py --all-off [--function NAME] [--strict]` (and `make unowned-off ALL=1` or a
      new make target) prints the unowned visibly-off Excel cells, grouped by attributed function (then
      unattributed, then probe milestone), one per line as `function<TAB>case!Sheet!A1<TAB>formula<TAB>expected
      <TAB>actual`, then per-function and total counts; `--strict` exits 1 when any is listed.
- [x] It runs in under a minute on a full report.json; tests in scripts/test_exit_owners.py cover attribution,
      the 4-ulp and half-unit boundaries, and deferred owners (a cell owned only by a `defer: phase-1b` task is
      listed as `deferred` and counts as unowned).
- [x] `make check` green; reviewers approve.

## Out of scope
- Filing the owners (T-0087).

## Handoff (resumed)
- Implemented and reviewed (security, architecture, fidelity, quality all APPROVE; `make check` green locally): the eval writes
  `attributed_off` into report.json (`attribution::off_excel_cells` shared with `unattributed_off`), and
  `scripts/exit_owners.py --all-off [--function NAME] [--strict]` + `make unowned-off ALL=1` list the unowned cells in ~30 s.
  PR #2123 holds it.
- What failed: CI job `hygiene` step `check_governance.py origin/main`: "protected paths changed without a decision record
  in docs/decisions/: Makefile, scripts/exit_owners.py, scripts/test_exit_owners.py". Sonnet runs do not write decisions
  for protected paths (work-loop 5b).
- Next: `make new-decision TITLE="..."` (extend decision 0132/0096 `exit_owners.py` listing with `--all-off`), fill it in,
  push, confirm CI green, re-tick the acceptance boxes, set `status: done` with an Outcome (eval delta none; 100,671
  attributed cells, 111 unowned, ~30 s), then merge under the steward policy.

## Outcome
- The eval writes `attributed_off` into report.json (`attribution::off_excel_cells` is the one filter behind it and
  `unattributed_off`; closeness.rs's tests pin the 4-ulp and half-unit boundaries), and
  `scripts/exit_owners.py --all-off [--function NAME] [--strict]` / `make unowned-off ALL=1 [FUNCTION=NAME]` list
  every unowned visibly-off Excel cell by function, then unattributed, then probe milestone; deferred-only owners
  show as `<group> (deferred)`. Decision 0142 records it.
- On the full report of the handed-off run: 100,671 attributed off cells, 111 unowned listed, in about 30 s.
- Eval delta: none (report-only change).
