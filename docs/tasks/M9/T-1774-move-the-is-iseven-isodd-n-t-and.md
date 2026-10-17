---
id: T-1774
title: "Move the IS*, ISEVEN/ISODD, ERROR.TYPE, ENCODEURL and FIELDVALUE lifts (lifted_arg, parity_predicate) onto the SPEC-0011 driver"
milestone: M9
lane: quality
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
IS*, ISEVEN/ISODD, ERROR.TYPE, ENCODEURL and FIELDVALUE lift through the SPEC-0011 driver, with no result change. `lifted_arg`, `Lifted` and `parity_predicate` are deleted.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Migration of the existing helpers".
- IS* and ERROR.TYPE are `Lift` (information.md "IS* type tests lift over arrays and ranges"). The kernel tests the element as it is stored: `EmptyCell` is ISBLANK, an error element is ISERROR.
- ISEVEN and ISODD are `LiftArrays` (information.md "ISEVEN/ISODD over a range"). This drops their hand-written range check.
- ENCODEURL, FIELDVALUE and the offline functions (functions/information/offline_errors.rs, functions/text/encodeurl.rs) keep their evidenced shapes.
- N and T stay on `map_elements`, because they read a range as a reference (SPEC-0011 non-goals).

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite. probes/is-functions-array-lift and probes/info-logic-argument-modes keep passing.
- [ ] `rg -n "fn lifted_arg|fn parity_predicate|enum Lifted" crates/rustsheet-engine/src` finds nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- N, T and CELL.
