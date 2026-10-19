---
id: T-2009
title: "SUBTOTAL and AREAS over IF or CHOOSE of a reference XLOOKUP read its cells"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SUBTOTAL and AREAS over an IF or CHOOSE (and LET body, if Excel agrees) whose picked arm is an
XLOOKUP that returns cells of a range read those cells, as Excel does.

## Context
Found in T-1998. Excel gives 9 for `SUBTOTAL(9,IF(TRUE,XLOOKUP(1,J1:J2,H1:I2)))` and
`SUBTOTAL(9,CHOOSE(1,XLOOKUP(1,J1:J2,H1:I2)))` (probes/take-drop-trimrange-over-xlookup-b A34,
A38; H1:I2 = 3,6;4,8, J1:J2 = 1,2); the engine spills two `#VALUE!` because IF/CHOOSE in
aggregate mode (`IfBranchMode::Aggregate`, `choose_impl(.., true)`) read the arm with
`evaluate_reference_arg`, which evaluates XLOOKUP for its values, and static analysis
(`is_read_reference`, `lifts_to_values` in static_analysis/aggregate.rs) sizes it as an array.
T-1998 added `Model::evaluate_picked_source` (drop_take.rs) for the same walk below TAKE, DROP and
TRIMRANGE; reuse it or its rule, and move it next to `evaluate_reference_form` /
`reference_arg_reading` so the walk has one home (it may also absorb `Model::pick_branch`,
external_reference.rs, which walks IF/CHOOSE for external references; T-1998 review). Check whether SUM/COUNT (which share `reference_arg_reading`)
also read the cells (text/boolean cells tell), and probe the LET-body form `SUBTOTAL(9,LET(v,1,XLOOKUP(..)))`.

## Acceptance criteria
- [ ] Probe of SUBTOTAL/AREAS over IF, CHOOSE and LET of a reference XLOOKUP (and SUM over a
  text/boolean picked cell) recorded in `docs/excel-semantics/references.md`.
- [ ] probes/take-drop-trimrange-over-xlookup-b passes in full; debug tests show no
  scalar-context assertion.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
