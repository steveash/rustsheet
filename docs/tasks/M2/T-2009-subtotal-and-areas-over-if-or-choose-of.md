---
id: T-2009
title: "SUBTOTAL and AREAS over IF or CHOOSE of a reference XLOOKUP read its cells"
milestone: M2
lane: functions-lookup
status: done
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
- [x] Probe of SUBTOTAL/AREAS over IF, CHOOSE and LET of a reference XLOOKUP (and SUM over a
  text/boolean picked cell) recorded in `docs/excel-semantics/references.md`.
- [x] probes/take-drop-trimrange-over-xlookup-b passes in full; debug tests show no
  scalar-context assertion.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/subtotal-if-choose-over-xlookup (excel-graph): SUBTOTAL and AREAS read the cells of a
reference-returning XLOOKUP (or TAKE) that a scalar IF or CHOOSE picks or a LET body is, nested or
not; a lifting lookup value or literal return array still gives values. SUM, COUNT and COUNTA over
text/boolean picked cells cannot tell a reference from values there. `picked_reference_arg_reading`
(functions/util.rs) now follows IF/CHOOSE arms and LET bodies with the picked rule. Eval delta:
probes fixed 4 (take-drop-trimrange-over-xlookup-b A34, A38, B34, B38), +1 case of 24 cells,
18 pass; its 6 known failures (IFS, SWITCH, ISREF) are T-2014. Static analysis unchanged
(conservative). The walk below TAKE (T-1998) and this one are still two definitions: T-2014.
