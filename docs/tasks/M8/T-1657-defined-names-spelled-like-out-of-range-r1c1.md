---
id: T-1657
title: "Defined names spelled like out-of-range R1C1 references (R0, C0, R0C1, R1048577) are #NAME? on every use"
milestone: M8
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
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

## Handoff (resumed)
Done on this branch (eval baseline shrank by 18 lines; `make check` NOT yet run):
- `is_out_of_range_r1c1_name` (expressions/utils/mod.rs) + skip in `name_operand` (parser/operands.rs): lowercase and non-`R<n>C<m>` shapes (`R0`, `C0`, `R1048577`, `r0c1`) now give `#NAME?`.
- `ISREF(INDIRECT(non-reference text))` is FALSE (information/mod.rs); INDIRECT now resolves names that are references (`indirect_defined_name`, lookup_and_reference/mod.rs) so `INDIRECT("RCR")` stays TRUE.

Still failing: uppercase `R0C1` / `R1048577C1` (probes/r1c1-name-r0c1, -r1048577c1, -use-r0c1; uses give `#ERROR!`, not `#NAME?`). The R1C1 lexer reads uppercase `R<n>C<m>` as a reference whatever the numbers, and formulas are stored in R1C1 form (T-1658), so a name spelled so cannot round-trip. Needs a design call on the lexer/storage. Next: fix that, update the semantics doc line "Ours defines it and reads 7 (T-1657)", run `make check`, review, PR.
