---
id: T-1774
title: "Move the IS*, ISEVEN/ISODD, ERROR.TYPE, ENCODEURL and FIELDVALUE lifts (lifted_arg, parity_predicate) onto the SPEC-0011 driver"
milestone: M9
lane: quality
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
IS*, ISEVEN/ISODD, ERROR.TYPE, ENCODEURL, FIELDVALUE and the offline functions lift through the SPEC-0011 driver, with no result change. `lifted_arg`, `Lifted`, `parity_predicate` and offline_errors.rs's own `map_elements` lifts are deleted.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Today's lifting code".
- IS* and ERROR.TYPE are `Lift` (information.md "IS* type tests lift over arrays and ranges"). The kernel tests the element as it is stored: `EmptyCell` is ISBLANK, and an error element is ISERROR.
- ISEVEN and ISODD are `LiftArrays` (information.md "ISEVEN/ISODD over a range").
- ENCODEURL, FIELDVALUE and the offline functions (functions/information/offline_errors.rs, functions/text/encodeurl.rs) keep their evidenced shapes.
- N and T stay on `map_elements`, because they read a range as a reference (a SPEC-0011 non-goal).

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite. probes/is-functions-array-lift and probes/info-logic-argument-modes keep passing.
- [ ] `rg -n "fn lifted_arg|fn parity_predicate|enum Lifted" crates/rustsheet-engine/src` finds nothing, and `map_elements` is called only by N and T.
- [ ] `make check` green; reviewers approve.

## Out of scope
- N, T and CELL.

## Outcome
IS*, ERROR.TYPE and ENCODEURL are `Lift` table entries and ISEVEN/ISODD `LiftArrays` entries; their kernels run through `lift_call`. The offline functions (IMAGE, STOCKHISTORY, FIELDVALUE) read their lifted argument with the new `read_lift_arg` and build their error grids with `lift_values`. `lifted_arg`, `Lifted`, `parity_predicate` and the offline `map_elements`/`error_array` lifts are deleted; `map_elements` is left to N and T. Static analysis derives the signatures and shapes of these functions from the table (`lifting_table_functions!`). Eval: 0 fixes, 0 regressions.
