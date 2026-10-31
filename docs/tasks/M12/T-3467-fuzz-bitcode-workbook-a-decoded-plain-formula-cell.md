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
in debug builds and under the fuzzer.

What Excel does here is open, and narrower than "is `#VALUE!` right":
- A formula stored without the dynamic-array flag (no `cm`) is a legacy formula. In it, a range in a value argument
  takes the implicit `@`: the intersected value on the range's rows, `#VALUE!` off them. See
  docs/excel-semantics/references.md "Legacy formulas intersect the ranges in value arguments" and arrays.md (legacy
  `=PMT(C1:C60,10,100)` shows `=PMT(@C1:C60,10,100)`).
- For this cell, D1 against a 3-row, 2-column A1:B3, that rule predicts `#VALUE!`, since column D is outside A:B. That
  is the release-build answer.
- What is not covered is a spill reference `A1#` in a formula stored without `cm`. Spill references postdate the legacy
  format, and no entry in docs/excel-semantics covers this case.

The minimized input (427 bytes) is a workbook where A1 holds `SEQUENCE(3,2)` (3 rows, 2 columns) and D1, a
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

Probe first with the `excel-probe` skill.
- **The file:** an xlsx with a one-column spill, A1 `=SEQUENCE(3)`. Its formula is in a plain `<f>` (no
  `t="array"`, no `cm`) holding `SIN(A1#)`, placed outside the spill in C2 (on its rows) and C5 (off them). Next to
  it goes the plain-range form `SIN(A1:A3)` in the same rows. A cell inside the spill would block it, and a range of
  several rows and columns intersects to `#VALUE!` everywhere, so a single column is what separates the outcomes:
  - intersection: SIN(2) in C2 and `#VALUE!` in C5;
  - a spill: a 3x1 array from each cell;
  - plain `#VALUE!`: `#VALUE!` in both.
- **What to capture:** each cell's value and its FORMULATEXT (does Excel add an `@`?).
- **The minimized 3x2 input** stays the reproduction.

Then, by what the probe shows:
- **Excel intersects.** Treat a decoded `CellFormula` as a legacy formula and reuse the engine's existing
  legacy-formula `@` path, the one the xlsx importer uses for formulas stored without the dynamic-array flag
  (references.md, legacy-import workbooks). Do not add a new intersection in `evaluate_cell` (directive 3).
- **Excel spills.** Reclassify such a cell at load as a dynamic anchor with r=(1,1), the way a typed formula is
  classified. Limit this to what the probe shows for `#` references, since plain ranges in legacy formulas
  intersect.
- **Excel gives `#VALUE!` (no intersection, no spill).** Keep the release-build answer and only replace the
  `debug_assert!` path with a handled `#VALUE!` for decoded cells.

Either way, keep the assert for the engine's own formulas.

## Acceptance criteria
- [ ] A test builds the minimized case (a `CellFormula` holding a spilling formula) and evaluates it in a debug
      build without a panic, with the value evidence supports.
- [ ] `cargo fuzz run bitcode_workbook` on both crash inputs no longer crashes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- How formulas typed into the engine are classified (static analysis); only decoded cells are in scope.
