---
id: T-2891
title: "Decision record: how M10's exit criteria are measured (gen-m10 displayed text, M10 probes) and which cells exit-owners lists for them"
milestone: M10
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A decision record (`make new-decision`) fixes how M10's exit criteria are read from `make eval`'s report at visible
parity (decision 0114), so an M10 exit row and `make exit-owners MILESTONE=M10` can be built from it. It adds no gate and
changes no baseline.

## Context
- M10's exit text (docs/milestones.md "M10"): "formatted-text oracle suite (≥ 10k cells) at 100%; its `probes` cases
  (decision 0051) at 100% bit-exact". `MILESTONES` in crates/rustsheet-eval/src/milestone_exit.rs stops at M6.
- Planner run of 2026-10-27 (`make eval` on 6087413bc): the report's "Displayed text" table gives gen-m10 10,400 text
  cells, 10,323 passing (99.26%), and probes 261 text cells at 100%. gen-m10's value cells: 1,690, 1 failing.
  Displayed-text items are baselined as `TC`/`TF` lines (decision 0070, T-1739). "Probe milestones (decision 0051)"
  M10 row: 86 cases, 5,494 scored, 199 known failures, 199 visibly off.
- `scripts/exit_owners.py` `failing_cells` reads each case's failing value cells; check whether a displayed-text
  mismatch is among them. If it is not, exit-owners cannot list M10's main criterion, and the record must say how it
  will (a protected-path change for the follow-up task).
- A local approximation over the M10 probe row on 2026-10-27 found 13 cells with no M10 owner by exit-owners' matching
  rule; this planner run named each of them in its open functions-text owner (T-1875, T-2729, T-2759, T-2798).
- Questions the record answers:
  1. Which cells are the "formatted-text oracle suite": gen-m10's displayed-text items alone, or with gen-m10's value
     cells and the probes' text items; "≥ 10k" counts which of them.
  2. "100%" at visible parity for text: a text item has no 15-digit reading, so 0 failing text items (say so).
  3. Whether the M10 probe row counts the TEXT-function probes the catalog assigns to M5 (decision 0051 decides; name
     the rule).
  4. How exit-owners lists displayed-text failures (case stem and cell, as for value cells).

## Acceptance criteria
- [ ] Decision record added answering questions 1–4 with the 2026-10-27 (or newer) report numbers; docs/milestones.md's
      M10 exit text cites it (edit only the M10 section).
- [ ] A follow-up harness task is filed with `make new-task` for the M10 exit row and exit-owners support, with concrete
      thresholds.
- [ ] `python3 scripts/tasks.py validate` passes; `make check` green; reviewers approve.

## Out of scope
- Implementing the exit row (follow-up task).
- Fixing any M10 cell.
