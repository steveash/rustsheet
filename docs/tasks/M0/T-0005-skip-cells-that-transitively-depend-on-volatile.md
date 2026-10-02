---
id: T-0005
title: "Skip cells that transitively depend on volatile functions; inject a fixed clock"
milestone: M0
lane: harness
status: done
priority: P2
size: S
depends_on: []
---

## Goal
Avoid false failures from cells that depend on NOW/TODAY/RAND indirectly, and make TODAY-based cases testable.

## Context
runner.rs only skips cells whose own formula is volatile. rustsheet-xlsx has a `mock_time` feature that reads a clock from a METADATA sheet.

## Acceptance criteria
- [x] Expectation filtering marks cells whose precedents (static reference analysis is sufficient) include a volatile cell as `skipped_volatile`.
- [x] When a workbook has IronCalc's METADATA mock-time convention, the engine adapter evaluates with that clock and volatile date cells are scored.
- [x] Tests cover both; eval baseline unchanged or improved.
- [x] `make check` green; reviewers approve.

## Out of scope
General dependency graph (M3).

## Outcome
- New `rustsheet-eval/src/volatile.rs` (single home for volatile detection): static-reference taint
  propagation (cell refs, ranges, quoted/qualified sheets; names, INDIRECT/OFFSET, whole row/column
  and 3-D refs are not followed; shared-formula members use the master's unshifted text).
- The eval crate enables the engine `mock_time` feature: the clock is thread-local and fixed per
  case. A `METADATA` row `NOW | <serial>` pins it to Excel's saved value, and then NOW/TODAY cells
  (and dependents) are scored; RAND* stays skipped. Without it NOW/TODAY remain skipped.
  Caveat: cargo feature unification means a workspace-wide build of the eval crate and the Python
  binding together would also mock the clock there; release wheels build `-p` alone.
- Scoring them exposed that `NOW()` truncated to whole seconds while Excel's cached NOW keeps
  sub-second precision; fixed in `tz/os_tz.rs` (the wasm `browser_tz.rs` path is unchanged).
- Eval: ironcalc-calc 69479 -> 69481 passed (2 newly scored cells, both pass); baseline unchanged.
