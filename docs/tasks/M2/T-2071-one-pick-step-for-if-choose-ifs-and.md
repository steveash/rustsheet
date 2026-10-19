---
id: T-2071
title: "One pick step for IF, CHOOSE, IFS and SWITCH under TAKE, DROP, TRIMRANGE and reference arguments"
milestone: M2
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TAKE, DROP and TRIMRANGE over an IFS or SWITCH whose chosen value is a reference-returning XLOOKUP
read its cells as Excel does (if Excel does), and the two walks that follow a picked arm in reference
context end up as one definition.

## Context
Found in T-2014. Two walks follow the arm an IF, CHOOSE, IFS or SWITCH picks:
`Model::picked_reference_arg_reading` (functions/util.rs; IF, CHOOSE, IFS, SWITCH, LET; T-2009,
T-2014) and `Model::evaluate_picked_source` (lookup_and_reference/drop_take.rs; IF, CHOOSE, LET
only; T-1998), and `Model::pick_branch` (external_reference.rs) does IF/CHOOSE for external
references. Reviews of T-1998, T-2009 and T-2014 asked for one pick step (`if_pick`,
`choose_pick`, `ifs_pick`, `switch_pick` by function kind). Whether
`SUBTOTAL(9,TAKE(IFS(TRUE,XLOOKUP(1,J1:J2,H1:I2)),1,1))` is 3 (as the IF form is,
probes/take-drop-trimrange-over-xlookup) has no evidence yet: probe it first (excel-probe skill),
docs/excel-semantics/references.md "TAKE, DROP and TRIMRANGE over XLOOKUP pick its cells".

## Acceptance criteria
- [ ] A probe of TAKE/DROP/TRIMRANGE over IFS and SWITCH of XLOOKUP is filled and passes.
- [ ] One helper maps a function kind to its pick step; both walks use it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
