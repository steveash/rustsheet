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
- [x] `choose_array` is the kernel of a `lift_call`; eval/catalog/lift-kinds.json regenerated (`rustsheet-eval lift-kinds`) if CHOOSE gets a row. Re-scoped (see Outcome): `choose_array` is the kernel of `lift_grids`, not a `lift_call` table entry; CHOOSE gets no row, so lift-kinds.json is unchanged.
- [x] No eval cell changes (0 regressions, 0 fixes) and test_fn_choose passes unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any change to what CHOOSE returns.

## Outcome
Handed off by a sonnet run (design question) and resumed by opus, so `tier: opus` stays. CHOOSE
over an index array now runs on the SPEC-0011 driver: `choose_array` reads the arms as before
(`evaluate_if_arg`) and hands the index grid and the arms to `lift_grids` with a per-element pick
kernel. The hand-rolled extent, size check and broadcast loop are gone, and `if_arg_at` and
`if_arg_dims` are private to IF again. CHOOSE gets no lift-kinds row, so
eval/catalog/lift-kinds.json is unchanged: a scalar index must leave the arms unevaluated and
return the picked one as a reference, and the result's extent includes every arm. A `Whole` row
under `lift_call` would read the arms eagerly and as scalars. The index read stays
`get_number_or_array` (a 3-D index is `#ERROR!` there, where a `Lift` read gives `#REF!`). The
SPEC-0011 table row records this as built. The driver adds its visit cap and text budget to this
path (directive 5 limits that were not applied here before). Eval delta: 0 fixes, 0 regressions
(every suite's known failures unchanged); test_fn_choose unchanged and passing.
