---
id: T-2403
title: "GETPIVOTDATA with an odd count of five or more arguments is #ERROR!, like other refused argument counts"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`GETPIVOTDATA` with an odd argument count of 5 or more (a second or later field without its item) gives `#ERROR!`
(`CalcResult::new_args_number_error`), as RustSheet does for every other argument count Excel refuses to load.

## Context
Found in T-2396: Excel for the web refuses the whole workbook holding `GETPIVOTDATA("Units",Pivot!$H$3,"Product","C","Rep")`
or a 7-argument odd call, while the 3-argument `("Units",Pivot!$H$3,"Product")` loads and is `#REF!` (probes/getpivotdata-real-pivot
H15). docs/excel-semantics/syntax.md "GETPIVOTDATA with an odd count of five or more arguments refuses the workbook" and "Minimum
argument counts" (refused counts give `#ERROR!`). Ours: `#REF!` (`fn_getpivotdata`,
crates/rustsheet-engine/src/functions/lookup_and_reference/getpivotdata.rs, before and after a pivot is found).
No Excel value exists, so the check is a unit test, not an eval case.

## Acceptance criteria
- [ ] A unit test: 5 and 7 odd arguments are `#ERROR!` with and without a pivot table; 3 arguments stay `#REF!`.
- [ ] syntax.md's entry says RustSheet gives `#ERROR!`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Refusing the workbook on import (no other refused argument count does).
