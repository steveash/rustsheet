---
id: T-1639
title: "Defined names with relative references resolve relative to the cell that uses them (gen-m8a names-relative)"
milestone: M8
lane: engine
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
A defined name whose reference is relative (`=Jan!A1`, `=Jan!$A1`, `=Jan!A$1`) reads the cell at the same offset from the cell that uses it, as Excel does.

## Context
gen-m8a (T-1606) case 051f538767ab0cfb (names-relative, 25/300 pass) and the `f_rel_sum` probes of names-formula d8859220cf857941 and mixed f25484e70039830f. xlsx stores a relative name relative to A1; Excel resolves it against the using cell: `rel_same` (`=Jan!A1`) from Calc!K1 is Jan!K1, `rel_down` (`=Jan!A2`) from Calc!K6 is Jan!K7, `rel_col_abs` (`=Jan!$A1`) from row i is Jan!$Ai. The engine reads them as absolute today (`=rel_same+1` from K1 is -3, Jan!A1+1, not 2). docs/excel-semantics/references.md "Relative references in defined names".

## Acceptance criteria
- [x] The relative-name probes of gen-m8a pass (names-relative, and the `rel_*` / `f_rel_sum` probes of names-formula and mixed), with no regression elsewhere.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: wrong values here are visible to anyone using defined names or tables (gen-m8a names-relative (about 223 wrong numbers from `rel_*` names alone)), so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Outcome
A reference-bodied defined name now keeps which rows and columns are absolute
(`ParsedReference::parse_anchored_reference_formula`), and `NameReference::at` reads its relative
parts at the same offset from the cell that uses it (also through `INDIRECT("name")`; `SHEET` and
`ISREF` unchanged). Before, a name stored as xlsx stores it (`Jan!A1`, no `=`) was read as
absolute. gen-m8a 2976 → 3372 cells (+396): names-relative 051f538767ab0cfb 25 → 300/300,
names-formula d8859220cf857941 232 → 280/280, mixed f25484e70039830f 207 → 280/280; no case
lost cells. Unprobed edges (wrap past the last row, INDIRECT of a relative name) noted in
references.md "Relative references in defined names".
