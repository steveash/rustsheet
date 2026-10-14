---
id: T-1479
title: "Engine reads NOW and TODAY from a per-model clock instead of the thread-local mock_time"
milestone: M2
lane: engine
status: done
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
- [x] `Model` carries the clock from the spec, and every reader listed above uses it. A `rg get_milliseconds_since_epoch` audit is recorded in the PR.
- [x] The harness pins the clock through the new path, with the behavior the spec says for the `mock_time` feature. `make eval` shows 0 regressions and 0 fixes in every suite, including the volatile-skip and run-invariant scoring (T-0005, T-1283).
- [x] Unit tests: two models on one thread with different fixed clocks give different NOW(), and the default clock is the system time.
- [x] `make check` green; reviewers approve.

## Out of scope
- The public `Workbook::clock`/`set_clock`, `Txn::set_clock` and the Python binding (T-1480). SPEC-0008's
  Rollout puts `WorkbookOptions.clock` (which the eval adapter needs), the `metadata_clock` feature rename and
  `HARNESS_CLOCK_MS` in this task.

## Outcome
- `clock.rs` adds `Clock` (system or a fixed instant in 1900..=9999 UTC) and `ClockError`. `Model` carries
  the clock (`clock`, `set_clock`, `new_empty_with_clock`, and `from_workbook_with_clock`, needed because
  loading already evaluates the cells conditional formatting reads). NOW/TODAY, conditional formatting's
  today, year-less date text, the creation stamp and the exporter's `dcterms:modified` all read
  `Model::now_unix_millis`. The system clock clamps instead of `expect`ing.
- NOW/TODAY number early-1900 days through DATE's conversion (`TODAY()` = 1/59/61 on 1900-01-01/02-28/03-01).
- The thread-local, `mock_time.rs` and the engine/xlsx `mock_time` features are gone; the xlsx module is
  `metadata_clock`; the facade feature is `metadata_clock`; `WorkbookOptions.clock` is new (struct
  literals need `..Default::default()`). The eval adapter uses `HARNESS_CLOCK_MS`.
- `rg get_milliseconds_since_epoch` and `rg MOCK_TIME` find nothing in code.
- Eval: 0 regressions, 0 fixes. A full `make eval` report diff against `main` (c9ed6429) matches in every
  suite, including the probe-milestone rows and 5 volatile-skipped cells. The only per-cell difference is
  a RANDARRAY value in the already-failing `probes/volatile-invariant` B18.
- The `new_empty.rs` tests moved to `new_empty/tests.rs` unchanged, keeping the file under 1,000 lines.
