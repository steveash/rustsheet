---
id: T-1479
title: "Engine reads NOW and TODAY from a per-model clock instead of the thread-local mock_time"
milestone: M2
lane: engine
status: in-progress
priority: P1
size: M
tier: opus
spec: docs/specs/SPEC-0008-workbook-clock.md
depends_on: [T-1478]
---

## Goal
`Model` owns its clock, as T-1478's spec defines (system time by default, or a fixed instant). NOW, TODAY and
the other in-engine readers of the time use that clock, so each workbook's results are deterministic on their
own, whatever thread they run on.

## Context
See T-1478. The readers today are `Model::now_in_timezone_arg` (functions/date_and_time.rs, NOW/TODAY),
new_empty.rs, tz/os_tz.rs and tz/browser_tz.rs (offset lookups), and the xlsx exporter's timestamp
(crates/rustsheet-xlsx/src/export/mod.rs). All of them go through `get_milliseconds_since_epoch` (model.rs, or
mock_time.rs under `cfg(test)`/`mock_time`). The harness pins the clock through
`rustsheet_xlsx::mock_time::set_mock_time_from_metadata` and `Workbook::pin_clock_from_metadata`
(crates/rustsheet-eval/src/engine.rs:80). Directive 5: setting the clock must not panic. The current
`expect("problem with system time")` in model.rs should become a typed fallback.

## Acceptance criteria
- [ ] `Model` carries the clock from the spec, and every reader listed above uses it. A `rg get_milliseconds_since_epoch` audit is recorded in the PR.
- [ ] The harness pins the clock through the new path, with the behavior the spec says for the `mock_time` feature. `make eval` shows 0 regressions and 0 fixes in every suite, including the volatile-skip and run-invariant scoring (T-0005, T-1283).
- [ ] Unit tests: two models on one thread with different fixed clocks give different NOW(), and the default clock is the system time.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The public `Workbook::clock`/`set_clock`, `Txn::set_clock` and the Python binding (T-1480). SPEC-0008's
  Rollout puts `WorkbookOptions.clock` (which the eval adapter needs), the `metadata_clock` feature rename and
  `HARNESS_CLOCK_MS` in this task.
