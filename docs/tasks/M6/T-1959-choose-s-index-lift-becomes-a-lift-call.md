---
id: T-1959
title: "CHOOSE's index lift becomes a lift_call kernel on the SPEC-0011 driver"
milestone: M6
lane: quality
status: todo
priority: P2
size: S
tier: opus
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

## Handoff
Tried: read `lift_call` (lift/call.rs), the kind table and `choose_array`/`choose_pick`
(lookup_and_reference/mod.rs). No code written. Why it is not a mechanical move:
- `Whole` arms are evaluated eagerly in the read phase, but CHOOSE with a scalar index must stay
  lazy (unpicked arms unevaluated: cycles, cost) and keeps reference/union arms (`choose_impl`
  `as_reference`, `external_reference_picked_value`). A `WholeByCaller` repeat kind avoids the eager read.
- `choose_array`'s result extent is the max over the index AND every arm (an unpicked arm still
  widens it); `lift_call` sizes the output from the lifted (index) argument only, so the broadcast
  over arms does not fit a per-element kernel without a driver extension.
Next: decide (opus) whether the driver gets an "extent from Whole arms" option or the task is
re-scoped to only the index read (`read_lift_arg` already gives Scalar/Grid for the index, replacing
`get_number_or_array`); possibly a short spec note in SPEC-0011.
