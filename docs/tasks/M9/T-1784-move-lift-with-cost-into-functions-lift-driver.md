---
id: T-1784
title: "Move lift_with_cost into functions/lift/driver.rs with no result change (SPEC-0011 step 1a)"
milestone: M9
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-1767]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
`lift_with_cost`, `CriteriaArg`, `count_walks` and `is_lifted` live in `crates/rustsheet-engine/src/functions/lift/driver.rs` as SPEC-0011's `lift_values` and `LiftedArg`, with no result change. This is the pure-move first half of the driver work; T-1770 adds the table.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "The driver" and "Today's lifting code".
- Callers today: the *IF(S) functions (`lift_criteria` in statistical/if_ifs.rs, which stays as the front end), AGGREGATE (aggregate.rs), IFS and SWITCH (logical/mod.rs, switch.rs, `is_lifted`), `lookup_each` and XLOOKUP.
- Changes in this task:
  - The `previous`-key memo becomes `LiftOptions::reuse_equal_runs`, comparing grid slots at the previous position through `bcast_get` instead of building a key `Vec` per element.
  - The cost argument becomes `LiftCost` (`Walk` and `Cells` for the existing callers).
  - The kernel becomes `&mut dyn FnMut`.
- No new behavior: the same shapes, bounds and results.

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite.
- [ ] `rg -n "fn lift_with_cost|enum CriteriaArg|fn count_walks|fn is_lifted" crates/rustsheet-engine/src` finds nothing outside functions/lift/.
- [ ] A criterion bench in crates/rustsheet/benches/ covers a lifted whole-column COUNTIFS (`=SUM(COUNTIFS(A:A,A1:A100000))` over a filled column, the `reuse_equal_runs` path) and a SUMIFS with an array criterion. Both run on main and on the branch in release, are within 5%, and the numbers are in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The argument-kind table, `lift_call` and any new consumer (T-1770).
