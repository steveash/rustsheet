---
id: T-1639
title: "Defined names with relative references resolve relative to the cell that uses them (gen-m8a names-relative)"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A defined name whose reference is relative (`=Jan!A1`, `=Jan!$A1`, `=Jan!A$1`) reads the cell at the same offset from the cell that uses it, as Excel does.

## Context
gen-m8a (T-1606) case 051f538767ab0cfb (names-relative, 25/300 pass) and the `f_rel_sum` probes of names-formula d8859220cf857941 and mixed f25484e70039830f. xlsx stores a relative name relative to A1; Excel resolves it against the using cell: `rel_same` (`=Jan!A1`) from Calc!K1 is Jan!K1, `rel_down` (`=Jan!A2`) from Calc!K6 is Jan!K7, `rel_col_abs` (`=Jan!$A1`) from row i is Jan!$Ai. The engine reads them as absolute today (`=rel_same+1` from K1 is -3, Jan!A1+1, not 2). docs/excel-semantics/references.md "Relative references in defined names".

## Acceptance criteria
- [ ] The relative-name probes of gen-m8a pass (names-relative, and the `rel_*` / `f_rel_sum` probes of names-formula and mixed), with no regression elsewhere.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
