---
id: T-1478
title: "Spec: per-workbook clock injection for NOW and TODAY in the public API and Python binding"
milestone: M2
lane: api
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A spec settles how an embedder makes NOW and TODAY deterministic. M2's scope lists "TODAY/NOW (deterministic
clock injection)" (docs/milestones.md), and today only the test harness can do that. The spec is either a
SPEC-0001 amendment or a new SPEC under docs/specs/ (the next free number).

## Context
Today the clock is global and feature-gated:
- `rustsheet_engine::get_milliseconds_since_epoch` (crates/rustsheet-engine/src/model.rs) reads `SystemTime`, or
  a thread-local `MOCK_TIME` (crates/rustsheet-engine/src/mock_time.rs) under `cfg(test)` or the `mock_time` cargo
  feature. NOW/TODAY get there through `Model::now_in_timezone_arg` (functions/date_and_time.rs). Timezone
  code (tz/os_tz.rs, tz/browser_tz.rs), new_empty.rs and the xlsx exporter's document timestamp also read it.
- The public facade only has `Workbook::pin_clock_from_metadata`, behind `#[cfg(feature = "mock_time")]`
  (crates/rustsheet/src/workbook.rs; SPEC-0001 "Cargo feature (harness only)"). The Python binding has none.
- A thread-local clock is per-thread, not per-workbook. Two workbooks on one thread cannot have different
  clocks, and an embedder cannot pin a clock without rebuilding with a cargo feature.
Questions the spec answers: the API shape (`Workbook::set_clock(Clock::Fixed(ms) | Clock::System)` or similar,
plus the Python mirror); whether a fixed clock also fixes the export timestamp; how the timezone interacts with a
fixed instant; whether changing the clock marks volatile cells dirty (ChangeSet); and how `mock_time` and
`pin_clock_from_metadata` are retired or re-expressed on top of it, without changing harness results.

## Acceptance criteria
- [ ] Spec merged with: API (Rust and Python), semantics, interaction with volatile recalc and ChangeSet, the
      migration of the harness's `mock_time` feature, and a test plan. It names T-1479 (engine) and T-1480 (facade
      and Python) as the implementation.
- [ ] No code changes; `make check` green; reviewers (including the API reviewer) approve.

## Out of scope
- Implementation (T-1479, T-1480). RAND/RANDBETWEEN seeding (not M2 scope).
