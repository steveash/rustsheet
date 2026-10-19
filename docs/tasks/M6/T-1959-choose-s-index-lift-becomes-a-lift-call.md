---
id: T-1959
title: "CHOOSE's index lift becomes a lift_call kernel on the SPEC-0011 driver"
milestone: M6
lane: quality
status: done
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
- [x] ~~`choose_array` is the kernel of a `lift_call`~~ Dropped (see Outcome): CHOOSE stays off the driver; SPEC-0011's table row records why. No lift-kinds row, so lift-kinds.json is unchanged.
- [x] No eval cell changes (0 regressions, 0 fixes) and test_fn_choose passes unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any change to what CHOOSE returns.

## Outcome
Handed off by a sonnet run (design question) and resumed by opus, so `tier: opus` stays. No code
change: CHOOSE stays off the SPEC-0011 driver, and the SPEC-0011 table row now says why. A table
entry does not fit, because a scalar index must leave the arms unevaluated and return the picked
one as a reference, and the result's extent includes every arm. A `Whole` row would read them
eagerly and as scalars. The first attempt moved `choose_array` onto `lift_grids` with a pick
kernel. It gave the same results, but the performance reviewer found that the driver fills
every arm's slot at every element: O(arms) clones per element instead of one, which untrusted
input can reach with many text arms. That attempt was reverted. Moving CHOOSE needs a driver
kernel that receives the output position, and that is worth building only when a second caller
needs it. Eval delta: 0 fixes, 0 regressions.
