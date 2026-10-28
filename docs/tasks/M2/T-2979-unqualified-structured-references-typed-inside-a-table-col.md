---
id: T-2979
title: "Unqualified structured references typed inside a table ([@Col], [Col]) resolve to that table, as Excel does"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2958]
campaign: semantics
---

## Goal
A structured reference written without its table name in a cell of a table (`=[@Col]*2`, `=SUM([Col])`,
`=[[#This Row],[Col]]`) resolves to that table, as Excel does, and FORMULATEXT prints it as typed (`=[@Col]*2`).
Today the parser refuses an operand that starts with `[` (`Unexpected token: '['`, parser/mod.rs `parse_operand`), so
the cell is a parse error and evaluates to `#ERROR!`.

## Context
Found by T-2958 (the calculated-column fill). The fill works, but every T-2957 probe that types `[@Col]` stays
`#ERROR!` in the filled cells. Evidence that Excel accepts the unqualified form inside the table:
docs/excel-semantics/references.md "Structured references: this row, totals and ranges" (in-table
19fdc25163755713: "Inside the table (a formula column) `[@Col]` reads the same row") and "Calculated columns" rule 6
(FORMULATEXT prints `=[@Col]*2` in every filled row). The file form is qualified: Excel saves a typed `=[@Col]*2` as
`T[[#This Row],[Col]]*2` (xlsx-storage.md, "A calculated column is saved as `calculatedColumnFormula` plus one
formula per row").

Design points (record a decision if not obvious):
- The lexer reads `[` as an external-workbook prefix first (`[1]Sheet!A1`, lexer/mod.rs), so `[@Col]` reaches the
  parser as `LeftBracket`; the parser knows the formula's cell (`self.context`) and the tables (`self.tables`), so it
  can rewind and read the reference with the containing table's name (`Lexer::consume_structured_reference`). The
  totals row counts as inside the table (Excel's `SUBTOTAL(109,[Col])`).
- Storage: shared formulas are deduplicated per sheet by their R1C1 text (`Model::shared_formula_index`) and
  re-parsed at R1C1 with no table context (`Model::parse_formulas`), so the stored text must carry the table name
  (two tables on one sheet with the same column name must not share `[@Col]*2`) and still print unqualified after a
  re-parse. `StructuredReference::text` feeds every printer and rewrite (stringify/structured.rs), so an
  `unqualified` flag on `StructuredReference` has to survive the R1C1 round trip.
- Export writes the qualified file form (`StructuredReference::file_text`).

Known failures this task should clear (eval/baseline/probes.txt, after T-2958):
probes/t2786-whitespace-structured-refs-runs S!C2, S!C3, S!C4; t2957-fill-one-totals S!C2:C4;
t2957-refill-second S!C2:C4, S!F2, S!F4; t2957-constant-exception S!C2, S!C4; t2957-constant-then-refill S!C2,
S!C4; t2957-existing-constant S!C3 (check the list against the baseline when starting).

## Acceptance criteria
- [ ] The known failures above leave eval/baseline/probes.txt with no new ones.
- [ ] Engine tests: `[@Col]`, `[Col]` and `[[#This Row],[Col]]` typed in a table cell evaluate as the qualified form;
      FORMULATEXT prints the unqualified text; the same text in two tables on one sheet reads each its own table;
      the text survives a sheet rename (re-parse); export writes the qualified form; outside any table the operand
      is still refused.
- [ ] docs/excel-semantics/references.md updated with "Ours:"; `make check` green; reviewers approve.

## Out of scope
- Whether Excel prints a qualified `T[@Col]` typed inside its own table without the name (unprobed).
- Totals-row formulas (`totalsRowFormula`) on import.
