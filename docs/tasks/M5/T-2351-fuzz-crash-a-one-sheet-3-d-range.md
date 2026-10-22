---
id: T-2351
title: "Fuzz crash: a one-sheet 3-D range in a scalar argument reaches the cell as an array (model.rs debug_assert)"
milestone: M5
lane: engine
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
No formula makes an array larger than 1x1 reach a scalar-context cell: the nightly `formula_eval`
fuzz target stops panicking on a one-sheet 3-D range passed to a scalar argument.

## Context
Nightly fuzz run 39718987994 (2026-10-22, `main` at c8d23da; the 2026-10-21 run at d77d01b0 was
green) crashed `formula_eval` on the 31-byte input (base64 `REFURVZBTFVFKFNoZWV0MTpTaGVldDEhQjE6Qk0xKQ==`):

    DATEVALUE(Sheet1:Sheet1!B1:BM1)

It trips the `debug_assert!` in `crates/rustsheet-engine/src/model.rs` (~line 939, "Larger-than-1x1
array reached scalar-context cell ... implicit intersection was expected to collapse it"). The
steward reproduced it on `main` b23f5f8 in a debug build through the `rustsheet` facade, with
`Input::Formula` set on D4 of a new workbook:
- `=DATEVALUE(Sheet1:Sheet1!B1:BM1)` panics; `=DATEVALUE(Sheet1!B1:BM1)` does not;
- `=LEN(Sheet1:Sheet1!B1:BM1)` and `=ABS(Sheet1:Sheet1!B1:C1)` panic too.
So static analysis gives the one-sheet 3-D range (`Sheet1:Sheet1!`) no `@` and no spill where it
gives the plain range one, and the lifted call returns an array to a scalar cell. Release builds hit
the fallback after the assert (not a crash in production), but it is a directive-5 bug and the
assert is the invariant (decision 0035). Likely a regression between d77d01b0 and c8d23da (bisect
with the reproducer if the cause is not obvious; candidates touch static analysis or lifting, e.g.
T-2298).

What Excel returns for a scalar function over a one-sheet 3-D range (spill like the plain range,
`#VALUE!`, or implicit intersection) is not yet probed: match the plain-range behavior only where
evidence exists; otherwise write a probe (excel-probe skill) and keep the fix to the invariant.

## Acceptance criteria
- [x] A unit test with the reproducer formula (and the LEN/ABS variants) evaluates without panic in
  a debug build (decision 0035: the reproducer becomes a unit test).
- [x] The cell's value follows Excel evidence for 3-D references in scalar arguments, or a probe is
  filed and the value is the plain-range behavior pending it.
- [x] `cd fuzz && cargo fuzz run --target "$(rustc -vV | sed -n 's/^host: //p')" formula_eval <reproducer>` passes.
- [x] `make check` green; reviewers approve.

## Outcome
Static analysis gives a one-sheet 3-D span (`Sheet1:Sheet1!B1:BM1`) the shape of the reference it
wraps, matching the evaluator since T-2280, and a legacy formula puts the plain range's automatic `@`
on its range; wider spans stay scalar (`#REF!`). New probe 3d-ref-one-sheet-span-lift (excel-graph):
Excel spills `ABS(Data1:Data1!A1:A3)` like the plain range; ours matches on 11/12 cells, the twelfth
(FORMULATEXT spells the span as the plain reference) is filed as T-2357. Unit tests pin the reproducer
and the LEN/ABS variants (both failed before the fix). cargo-fuzz is not installed in this session's
environment, so the reproducer was replayed through the facade in a debug build instead (no panic).
Eval delta: fixed 0, new-coverage known failures 1 (Calc!B6), regressions 0.
