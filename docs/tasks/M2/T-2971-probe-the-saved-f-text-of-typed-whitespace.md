---
id: T-2971
title: "Probe the saved <f> text of typed whitespace beside @, LET names, a sign and a leading run"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the `<f>` text Excel saves for typed whitespace in shapes T-2948 export writes by derivation, recorded in docs/excel-semantics/xlsx-storage.md "Typed whitespace is saved…".

## Context
T-2948 writes the written tree into `<f>`. probes/t2786-whitespace-structured-refs-runs (H14:H17) verified `1 + 2`,
`SUM( 1 )` and line feeds only. Unverified shapes our export now writes (crates/rustsheet-xlsx/src/export/test/test_export.rs
`test_typed_whitespace_is_saved_in_the_formula_text`): `=SUM( @A1:A3, 1)` → `SUM( _xlfn.SINGLE(A1:A3), 1)`,
`=SUM( @ A1:A3 )` (run after `@`), `=LET( x, 1, x + 1)` → `_xlfn.LET( _xlpm.x, 1, _xlpm.x + 1)`, `=- 5 * 2`, and a typed
leading run `= 1+2` (we write `<f> 1+2</f>`; reading stored text drops it, T-2784). Use the `excel-probe` skill and read
the saved sheet XML as t2786 did.

## Acceptance criteria
- [ ] A probe case with each shape above and FORMULATEXT beside it; the saved `<f>` text of each recorded in xlsx-storage.md.
- [ ] Any shape where our export differs gets a follow-up engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the export (follow-up tasks only).
