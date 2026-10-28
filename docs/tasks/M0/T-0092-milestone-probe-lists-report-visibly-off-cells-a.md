---
id: T-0092
title: "Milestone probe lists report visibly-off cells: a count and the case!Sheet!A1 list on each milestone_lists row"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: [T-0088]
---

## Goal
Each frozen milestone probe list in the eval report (today only `m1-probes`) says how many of its scored cells
are visibly off (decision 0114) and which ones, so M1's exit row can read the list at visible parity
(decision 0146 item 1(c)) instead of by known failures.

## Context
- Decision 0146 (T-0088, PR #2144) item 1: "The report adds a visibly-off count and the visibly-off cells to each
  `milestone_lists` row so (c) is read at visible parity." This task is that report input only; the M1 exit row
  that reads it is T-0089.
- `crates/rustsheet-eval/src/milestone_lists.rs`: `ListSummary` has `known_failures` and `failing_cases` but no
  visible-parity count. `probe_milestones::MilestoneRow` already has `visibly_off` and `visibly_off_cells`
  (`case!Sheet!A1`), computed with `closeness::Visible::is_off`; reuse that predicate and that cell-name format
  (directive 3), do not write a second one.
- report.md renders the lists in "Milestone probe lists" (`report/` module); today `m1-probes` has 89 cases,
  1,858 cells, 0 known failures, so the new column reads 0.

## Acceptance criteria
- [ ] `ListSummary` gains `visibly_off: usize` and `visibly_off_cells: Vec<String>` (`#[serde(default)]`), the
      cells named as `probe_milestones::MilestoneRow::visibly_off_cells` names them and counted with
      `Visible::is_off`; deviation cells are never counted.
- [ ] Unit tests in `milestone_lists.rs`: a listed case with one visibly-off cell and one bit-level-only failure
      (within 15 digits) gives `visibly_off == 1`, `known_failures == 2` and exactly that cell in the list; a
      case not in the list contributes nothing.
- [ ] report.md's "Milestone probe lists" table has a `visibly off` column; `make eval` shows 0 for `m1-probes`
      on `main`.
- [ ] No change to baselines, gates or `scripts/`; `make check` green; reviewers approve.

## Out of scope
- The M1 exit row and exit-owners support (T-0089).
- Changing which cases a list holds (frozen, decision 0043).
