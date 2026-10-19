---
id: T-2014
title: "SUBTOTAL, AREAS and ISREF over IFS, SWITCH, IF or CHOOSE of a reference XLOOKUP read its cells"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SUBTOTAL and AREAS over a scalar IFS or SWITCH whose chosen value is a reference-returning XLOOKUP
read its cells, and ISREF of an IF or CHOOSE picking such an XLOOKUP is TRUE, as Excel does.

## Context
Found in T-2009. probes/subtotal-if-choose-over-xlookup (excel-graph, 2026-10-19; H1:I2 =
3,6;4,8, J1:J2 = 1,2): `SUBTOTAL(9,IFS(TRUE,XLOOKUP(1,J1:J2,H1:I2)))` and
`SUBTOTAL(9,SWITCH(1,1,XLOOKUP(..)))` are 9 (A78, A82; the engine spills two `#VALUE!`), and
`ISREF(IF(TRUE,XLOOKUP(..)))` and `ISREF(CHOOSE(1,XLOOKUP(..)))` are TRUE (A90, A94; the engine
gives FALSE). T-2009 made `picked_reference_arg_reading` (functions/util.rs) follow the arm a
scalar IF or CHOOSE picks and a LET body; IFS and SWITCH have no pick step (`ifs_impl`,
`switch_impl` evaluate the chosen value in place), so they need one (an `ifs_pick`/`switch_pick`
returning `Picked`, like `if_pick`). ISREF reads its argument in functions/information/mod.rs.
`Model::evaluate_picked_source` (drop_take.rs, T-1998) does the same walk below TAKE/DROP/
TRIMRANGE: the two walks should end up as one definition (T-1998, T-2009 reviews), and
`Model::pick_branch` (external_reference.rs, IF/CHOOSE for external references) may fold into it.

## Acceptance criteria
- [ ] probes/subtotal-if-choose-over-xlookup passes in full; debug tests show no scalar-context
  assertion.
- [ ] Any new probe evidence recorded in `docs/excel-semantics/references.md`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
