---
id: T-1566
title: "Probe the aggregate differences SPEC-0009 keeps unevidenced: PRODUCT over picked references, direct blank/LAMBDA values, A-variant error order"
milestone: M2
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle values settle three differences between aggregates that SPEC-0009 keeps only because nothing evidences them. Each is either recorded as an Excel quirk or marked for unification in the walker's policy.

## Context
SPEC-0009 ("Excel behavior to match", the rows marked "current behavior; unprobed") keeps these as policy knobs. Use the `excel-probe` skill, with one probe case tagged `M2`:
- PRODUCT reads with `evaluate_reference_arg` (product.rs), not `evaluate_aggregate_arg`. A cell that XLOOKUP, TAKE, DROP or TRIMRANGE picks is therefore a value, not a reference. Probe `PRODUCT(XLOOKUP(1,A1:A2,B1:B2),2)` with the picked cell holding "x" (SUM gives 2: probes/subtotal-reference-operands-d), and the same with TAKE and DROP.
- A direct value that is neither number, boolean, text nor an empty argument: SUM/MIN/MAX/PRODUCT cast it (`cast_aggregate_arg_to_number`: a blank value is 0, a LAMBDA is an error), while AVERAGE/MEDIAN/COUNT skip it. Probe `SUM(LAMBDA(x,x))`, `AVERAGE(LAMBDA(x,x),2)`, `MIN(LAMBDA(x,x),2)` and `COUNT(LAMBDA(x,x),2)`. Also probe a direct blank value, if one can be formed without IFERROR (e.g. `MIN(LET(b,Z99,b),5)` with Z99 blank; check whether LET hands on a reference first).
- AVERAGEA, MAXA and MINA return their first error at once (`for_each_value_a`), while AVERAGE/MIN/MAX rank a referenced error below a direct one (syntax.md "Aggregate error precedence"). Probe `MAXA(A2,#DIV/0!)`, `MAXA(#DIV/0!,A2)`, `AVERAGEA(A2,1/0)` and `MINA(A2,A3)` with A2 = `#N/A` and A3 = `#VALUE!`, reading ERROR.TYPE as probes/aggregate-error-precedence does.
Array-error order and the `for_each_value` family's referenced errors are T-1439's, not this task's.

## Acceptance criteria
- [ ] A probe case with oracle values for every formula above. The findings are in docs/excel-semantics/coercion.md or syntax.md, and the matching rows of SPEC-0009's table cite them.
- [ ] Where Excel differs from the engine, file a task that changes the policy constant (after T-1474/T-1476 have landed) or the current code. Do not fix it here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-1439's error-order probes.
- Changing engine code.
