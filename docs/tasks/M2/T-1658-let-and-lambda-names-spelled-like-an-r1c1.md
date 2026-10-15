---
id: T-1658
title: "LET and LAMBDA names spelled like an R1C1 cell (R1C1, R5C7, R0C1) survive the R1C1 storage round-trip"
milestone: M2
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
`LET(R1C1,5,R1C1*2)`, `LET(R5C7,...)` and `LET(R0C1,...)` (and LAMBDA parameters spelled the same) are 10 as in Excel, and stay so after `Model::to_bytes`/`from_bytes` and an xlsx round trip.

## Context
Found by T-1652 (docs/excel-semantics/references.md "R1C1-shaped sheet and defined names"). Excel accepts every probed R1C1-shaped LET name (probes/r1c1-let-name-r1c1, -r5c7, -r0c1 fail in ours). Formulas are stored in R1C1 form and re-lexed in R1C1 mode on load, where an uppercase `R<n>C<m>` is read as a cell reference: allowing the name in `is_valid_a1_identifier` gave `=LET($A$1,5,$A$1*2)` (#VALUE!) after a reload, and `R0C1` became `#REF!`. Importing an Excel-written `_xlfn.LET(_xlpm.R1C1,...)` is wrong on main too. So `is_r1c1_cell_text` (expressions/utils/mod.rs) keeps refusing that spelling, with TODO(T-1658) there and in test_fn_let.rs. The fix needs the R1C1 stringify/lexer to keep a LET/LAMBDA name apart from a reference (for instance by lexing a LET name position as an identifier, or escaping it in the stored form).

## Acceptance criteria
- [ ] probes/r1c1-let-name-r1c1, -r5c7, -r0c1 pass; a reload test (`to_bytes`/`from_bytes`) and an xlsx import of `_xlfn.LET(_xlpm.R1C1,...)` keep the value 10 and the formula text; `is_r1c1_cell_text` and its TODO are removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
