---
id: T-0089
title: "Milestone exit rows for M1, M2 and M7 to M11 in the eval report, per the new exit-measure decision"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0088]
---

## Goal
The eval report has a `milestone_exit` row for every Phase 1 milestone (M1, M2, M5 to M11), measured as decision
0085 and T-0088's decision define, so `make eval` says which Phase 1 milestone exits are met and
`scripts/exit_owners.py --milestone M<k>` lists the failing exit cells of any of them.

## Context
- `crates/rustsheet-eval/src/milestone_exit.rs` covers M5 and M6 only (T-2177). T-0088 writes the decision for the
  rest; this task implements it, reusing the existing row inputs (catalog coverage, generated suites, function
  scope, probe milestones, milestone lists) without scoring anything new.
- The progress page (tools/src/rustsheet_tools/progress_report/) shows the rows in its Milestones table; check it
  renders the new rows.

## Acceptance criteria
- [ ] `milestone_exit::MILESTONES` (or its replacement) covers M1, M2 and M5 to M11 with T-0088's measures; unit
      tests cover a met and an unmet row for each new kind of measure.
- [ ] `make eval` report.md and report.json show the rows; `python3 scripts/exit_owners.py --milestone M9` runs.
- [ ] `make progress-report` renders them; `make check` green; reviewers approve.

## Out of scope
- Making the milestones pass (their tasks).
