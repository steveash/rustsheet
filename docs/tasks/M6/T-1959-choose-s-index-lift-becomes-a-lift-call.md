---
id: T-1959
title: "CHOOSE's index lift becomes a lift_call kernel on the SPEC-0011 driver"
milestone: M6
lane: quality
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CHOOSE's index_num lift runs on the SPEC-0011 driver (`index_num` `Lift`, the value arms `Whole`), with no change in any result.

## Context
Split from T-1810 (its acceptance item "CHOOSE's `choose_array` is the kernel of a `lift_call`"). Today `choose_array` (functions/lookup_and_reference/mod.rs) broadcasts the index array itself. Move it onto `lift_call` (functions/lift/call.rs) and the lift-kind table. CHOOSE's reference arms (it can return a range) must keep their behaviour: test_fn_choose.rs and every eval case that uses CHOOSE stay green.

## Acceptance criteria
- [ ] `choose_array` is the kernel of a `lift_call`; eval/catalog/lift-kinds.json regenerated (`rustsheet-eval lift-kinds`) if CHOOSE gets a row.
- [ ] No eval cell changes (0 regressions, 0 fixes) and test_fn_choose passes unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what CHOOSE returns.
