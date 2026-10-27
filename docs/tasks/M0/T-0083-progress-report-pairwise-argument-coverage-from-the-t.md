---
id: T-0083
title: "Progress report: pairwise argument coverage from the T-0077 census"
milestone: M0
lane: harness
status: done
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
- [x] report.json `argument_coverage[]` gains a per-function pairwise share (pairs of counted
      parameters × their expected classes seen in one call), unit-tested.
- [x] The `arguments` block on docs/progress.md shows the pairwise mean and median beside the
      single-argument ones, and the Outcome compares them with the assessment's 44% median.
- [x] `make check` green; reviewers approve.

## Out of scope
- Generator changes to raise coverage.

## Outcome
- The census keeps each call's argument classes together: `observe` returns one `CallSeen` per
  distinct call (arguments `None` where unclassified), still unserialized and deduplicated per
  case. `summarize` derives single-argument coverage from the calls as before and adds
  `argument_coverage[].pairwise`: of the pairs of two different counted parameters × an expected
  class of each, the share one Excel-evidence call passes together (`None` with fewer than two
  counted parameters; a repeated parameter pairs with the others, not itself). Unit-tested.
- The `arguments` progress block shows the pairwise mean and median after the single-argument
  ones; a report without `pairwise` is a shape error naming T-0083.
- Full eval on this branch: single-argument mean 87.0% (median 88.6%) over 499 functions;
  pairwise mean **32.2%**, median **25.1%** over 318 functions with two or more counted
  parameters, against the 2026-10-25 assessment's 44% median. The census is stricter than the
  one-off parser: eleven classes per parameter (121 pairs per two value parameters), and nested
  calls and operators count as no class, so pairs seen only through them are not counted.
- Eval delta: none (harness reporting only).
