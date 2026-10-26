---
id: T-0083
title: "Progress report: pairwise argument coverage from the T-0077 census"
milestone: M0
lane: harness
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-0077]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
The progress page shows pairwise argument coverage per in-scope function: of the pairs of
parameters and their expected classes, the share some Excel-evidence call passes together.

## Context
- T-0077 left pairwise coverage out of scope. The 2026-10-25 assessment measured a 44% pairwise
  median with a one-off parser.
- Build on the T-0077 census: `crates/rustsheet-eval/src/argument_coverage.rs` (`observe`,
  `param_of`, `expected_classes`) and the engine hook `rustsheet_engine::call_arguments`. Pairs
  need the classes of one call kept together, so `ArgumentSeen` (one class per position) must
  grow a per-call record; keep `CaseResult::arguments` unserialized and deduplicated per case.
- The page block is tools/src/rustsheet_tools/progress_report/arguments.py.

## Acceptance criteria
- [ ] report.json `argument_coverage[]` gains a per-function pairwise share (pairs of counted
      parameters × their expected classes seen in one call), unit-tested.
- [ ] The `arguments` block on docs/progress.md shows the pairwise mean and median beside the
      single-argument ones, and the Outcome compares them with the assessment's 44% median.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Generator changes to raise coverage.
