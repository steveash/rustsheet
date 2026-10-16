---
id: T-1657
title: "Defined names spelled like out-of-range R1C1 references (R0, C0, R0C1, R1048577) are #NAME? on every use"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A workbook-level defined name spelled like an out-of-range R1C1 reference (`R0`, `C0`, `R0C1`, `R1048577`, `R1048577C1`, any case) is defined, but every use gives `#NAME?` as in Excel, and `ISREF(INDIRECT("R0"))` is FALSE.

## Context
Found by T-1652 (docs/excel-semantics/references.md "R1C1-shaped sheet and defined names"). Excel for the web loads such a name from a file (an in-range one like `R1C1` makes it reject the workbook), but `=R0*1`, `=SUM(R0)` and `INDIRECT("R0")` do not resolve it. FORMULATEXT keeps `=R0*1`. Ours defines the name and reads its value (7). Failing cases: probes/r1c1-name-r0, -c0, -r0c1, -r0c1-lower, -r1048577, -r1048577c1, r1c1-name-use-r0, -c0, -r0c1, -r1048577. Do not refuse the name: the eval harness fails a case whose name cannot be defined. Make its uses `#NAME?` (name resolution in the evaluator, or how the lexer reads an identifier that would be an out-of-range R1C1 reference, with no sheet range check). Check that `LET(R0,5,R0*2)` stays 10 (probes/r1c1-let-name-r0). The bound-free shape is `R<digits>`, `C<digits>`, `R<digits>C<digits>` with a number out of range; `RCR`, `CR` and `RCD` stay valid names.

## Acceptance criteria
- [ ] Every cell of the r1c1-name-* and r1c1-name-use-* cases listed in Context passes; the r1c1-let-name-* cases stay green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
- Done: `is_unusable_r1c1_name` (expressions/utils/mod.rs) and `Parser::get_defined_name` returning
  None for it, so `=R0*1`, `=SUM(C0)`, `=R1048577*1` and `ISREF(R0)` match Excel (r1c1-name-r0, -c0,
  -r0c1-lower, -r1048577 pass; r1c1-let-name-r0 stays green). No unit test yet for the predicate
  (add cases to expressions/utils/test.rs next to `r1c1_shaped_names`); `make check` not run.
- Still failing: (1) uppercase `R0C1` / `R1048577C1` lex as an R1C1 cell token (`is_r1c1_cell_text`),
  so the parse is `#ERROR!` (r1c1-name-r0c1, -r1048577c1, r1c1-name-use-r0c1 A1/A4, and the
  pre-existing r1c1-let-name-r0c1 A1); look at `is_valid_r1c1_identifier` in expressions/lexer/mod.rs.
  (2) `ISREF(INDIRECT("R0"))` is TRUE while Excel gives FALSE: `fn_indirect` does not resolve names
  at all, so check how ISREF treats an INDIRECT whose text does not parse (probably ISREF of the
  INDIRECT node, not of its result).
- Pause reason: session time budget (opus worker, 2026-10-16). No attempt failed.
