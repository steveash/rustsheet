---
id: T-1471
title: "Report lists the probe known failures of each milestone row by case"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The "Probe milestones (decision 0051)" section of eval/runs/latest/report.md, and its report.json rows, name the
failing cases of each milestone row with their known-failure counts. Planners and the steward can then see what
blocks M2 exit without re-implementing the assignment rules.

## Context
`probe_milestones::summarize` (crates/rustsheet-eval/src/probe_milestones.rs, T-1423) reports only counts per row
(today `M0-M2`: 322 cases, 196 known failures). The 2026-10-13 planner had to rebuild the rules in a script to find
the 24 failing cases behind those 196 cells. The frozen-list table already has a `failing cases` column
(`MilestoneList::failing_cases`, milestone_lists.rs), so follow that precedent.

## Acceptance criteria
- [ ] `MilestoneRow` gains `failing_cases: Vec<(String, usize)>`: case id and the number of that row's known-failure items, sorted by count, then by id. It is serialized in report.json.
- [ ] report.md lists each row's failing cases under the table, one bullet per case, for every row with known failures.
- [ ] A unit test in probe_milestones.rs covers a catalog case whose cells fall in two rows; each row lists only its own count.
- [ ] On today's tree the `M0-M2` counts sum to the row's `known_failures` (196 before any fix).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the assignment rules or the mapping file (T-1470).

## Outcome
`MilestoneRow.failing_cases` (case id, known-failure items in that row; most first, then id) is serialized in report.json and listed per row in report.md ("Failing cases in <row>"). Unit test covers a case spread over two rows. On the run today the per-row counts sum to `known_failures` (M0-M2: 170 across 25 cases; the task's 196 and 24 were counted before later fixes landed on main). No eval cell changes.
