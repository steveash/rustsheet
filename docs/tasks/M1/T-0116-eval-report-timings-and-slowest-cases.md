---
id: T-0116
title: "Eval report: per-suite wall time and slowest cases"
milestone: M1
lane: harness
status: todo
priority: P2
size: S
depends_on: []
---

## Goal
`make eval` reports wall time per suite and the 10 slowest cases in `report.md`/`report.json`, so the "eval run time before/after (no >5% regression)" criterion of SPEC-0001 / T-0103 and the T-0111 timeout fixes can be measured instead of guessed.

## Context
`CaseResult.millis` is already recorded in `crates/rustsheet-eval/src/runner.rs` but never surfaced by `crates/rustsheet-eval/src/report.rs`. Timing must not enter the gate: baselines and `check` stay timing-independent (no flakiness).

## Acceptance criteria
- [ ] `report.md` has a "Timing" section with per-suite total ms and a table of the 10 slowest `suite/case` ids; `report.json` carries the same fields.
- [ ] `rustsheet-eval check` and baseline files are unaffected by timing (unit test: two reports differing only in `millis` produce the same gate verdict).
- [ ] Unit test of the slowest-N selection (ties broken by case id for deterministic output).
- [ ] `make check` green; reviewers approve.

## Out of scope
Timing thresholds or gating (perf lane, T-0400); parallelizing the runner.
