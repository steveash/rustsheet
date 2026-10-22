---
id: T-2351
title: "Fuzz crash: a one-sheet 3-D range in a scalar argument reaches the cell as an array (model.rs debug_assert)"
milestone: M5
lane: engine
status: in-progress
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
- [ ] A unit test with the reproducer formula (and the LEN/ABS variants) evaluates without panic in
  a debug build (decision 0035: the reproducer becomes a unit test).
- [ ] The cell's value follows Excel evidence for 3-D references in scalar arguments, or a probe is
  filed and the value is the plain-range behavior pending it.
- [ ] `cd fuzz && cargo fuzz run --target "$(rustc -vV | sed -n 's/^host: //p')" formula_eval <reproducer>` passes.
- [ ] `make check` green; reviewers approve.
