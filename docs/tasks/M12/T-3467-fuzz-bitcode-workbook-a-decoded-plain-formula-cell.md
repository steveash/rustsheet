---
id: T-3467
title: "Fuzz bitcode_workbook: a decoded plain formula cell whose formula spills trips the scalar-context debug_assert"
milestone: M12
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-2895]
---

## Goal
A decoded workbook whose plain formula cell (`Cell::CellFormula`) holds a formula that returns a larger-than-1x1
array evaluates without a panic, in debug builds too. The cell takes the value Excel would show: it either spills
like a typed dynamic formula or is implicitly intersected.

## Context
Found by T-2895's `bitcode_workbook` fuzz target on its first local run: two crash inputs, both with this one cause,
in 2.5M runs.

`Model::evaluate_cell` (crates/rustsheet-engine/src/model.rs, the `None` arm for a cell with no array range) has a
`debug_assert!(false, "Larger-than-1x1 array reached scalar-context cell ...")` that holds only for formulas the
engine parsed and classified itself. A typed `=SIN(A1#)` next to a spill becomes a dynamic `ArrayFormula`. Decoded
bytes can store it as a `CellFormula` instead. Release builds give `#VALUE!` there (no panic), but the assert fires
in debug builds and under the fuzzer, and the `#VALUE!` is probably not Excel's answer either.

The minimized input (427 bytes) is a workbook where A1 holds `SEQUENCE(3,2)` (a 2x3 spill) and D1, a
`CellFormula`, holds `SIN(R[0]C[-3]#)`. Reproduce with `cargo fuzz run bitcode_workbook <file>`, the file being
this base64 decoded:

```
AAABAkExAAAGU2hlZXQxAQH/BAAEDgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAG
KwQFATQRdnV3BwQBBAACAAAAAAAAADVABAAEAAQCBAMBAgAAAAAAAADwPwQKAAQKHwQKH1UBAAAAAAAAAABAAAAAAAAACEAAAAAA
AAAQQAAAAAAAABRAAAAAAAAAGEAGAwYDAg0PU0VRVUVOQ0UoMywyKVNJTihSWzBdQ1stM10jKQQBBAAAAgAABAAEAAEEAAQBBAEE
Cg8EAQQBBAEEAQEAAAAAAQAAAAAEDAIFSW50ZXIEAgACCAEAAAAAAAAAAQQABAAEAAQAAQEBAQEBAQQABAAEAAQABAAAAAAAAAAA
AAEGbm9ybWFsBAAEAAAEc2VlZANVVEMCZW4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==
```

Options:
- At load, have `Model::from_workbook` reclassify a `CellFormula` whose parsed formula is array-shaped, as a dynamic
  anchor with r=(1,1), the way a typed formula is classified.
- Or apply implicit intersection in that arm.

Which one Excel matches for a stored non-array formula with such a result is a question for the `excel-probe` skill
(an xlsx with a `<f>` without `t="array"` whose result is an array). Either way, keep the assert for the engine's own
formulas.

## Acceptance criteria
- [ ] A test builds the minimized case (a `CellFormula` holding a spilling formula) and evaluates it in a debug
      build without a panic, with the value evidence supports.
- [ ] `cargo fuzz run bitcode_workbook` on both crash inputs no longer crashes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
