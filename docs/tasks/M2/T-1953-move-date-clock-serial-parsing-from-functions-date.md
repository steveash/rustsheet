---
id: T-1953
title: "Move date+clock serial parsing from functions::date_and_time::time_text down to formatter (remove formatter->functions import)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The date+clock serial parser (`parse_date_clock_serial_in`, `parse_date_clock_parts`, clock parsing helpers) lives below both `formatter` and `functions`, so the module dependency runs one way, functions to formatter.

## Context
T-1907 added `formatter::parse_formatted::parse_typed_number`, which calls `functions::date_and_time::time_text::parse_date_clock_serial_in`, while `time_text` already imports `formatter::date_text::{parse_date, CurrentYear}`: a two-way dependency (architecture review of PR #1113). Move the parser next to `formatter/date_text.rs` (or a sibling `formatter/date_clock.rs`) and have `time_text.rs` and `parse_formatted.rs` both call it. Pure move: no behavior change.

## Acceptance criteria
- [ ] No non-test `use crate::functions` in `crates/rustsheet-engine/src/formatter/`.
- [ ] Eval baseline unchanged (no cell changes).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
