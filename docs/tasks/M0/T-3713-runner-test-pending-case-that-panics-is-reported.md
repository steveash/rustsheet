---
id: T-3713
title: "runner test pending_case_that_panics_is_reported_as_panic reads Timeout under load: a panic's backtrace outlasts the 5 s case limit"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`runner::tests::pending_case_that_panics_is_reported_as_panic` (crates/rustsheet-eval/src/runner/tests.rs) passes on
a loaded machine. A panicking case is reported as `Panic`, never as `Timeout`.

## Context
- On 2026-11-01, during `make check` on a machine running a second `make check` and the sidecar generator, the test
  failed with `left: Timeout, right: Panic` (tests.rs:75). The scripted engine's panic ("boom", tests.rs:31) did
  happen. With `RUST_BACKTRACE=1`, which this environment sets, capturing and symbolizing the backtrace took longer
  than the case's `Duration::from_secs(5)` (tests.rs:43), so `run_isolated` (runner.rs:435) reported a timeout.
- Alone it passes in about 0.2 s (3 of 3 runs).
- A Timeout verdict for a real panic would also misreport a panicking eval case under load, not only in this test.
- Possible fixes:
  - classify a panic caught after the deadline as `Panic` when the worker reports it;
  - or set a panic hook that skips backtrace capture in the isolated runner;
  - or give the test a generous limit;
  - whichever matches the runner's contract.

## Acceptance criteria
- [ ] The test passes with `RUST_BACKTRACE=1` under CPU load (for example while `cargo build` runs), or the runner
      reports a caught panic as `Panic` whatever the time.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other runner timeouts.
