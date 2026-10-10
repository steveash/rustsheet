---
id: T-1305
title: "Probe MATCH descending search over mixed types and a non-numeric match_type with a 2-D or out-of-range lookup (3 gen-m2 known failures)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that explains three gen-m2 MATCH cells the documented search rule does not, with
an engine task filed for each rule the evidence settles.

## Context
docs/excel-semantics/lookup.md "Approximate-match lookups: type-skipping binary search" gives the
pseudo-code the engine follows (`approximate_search`, crates/rustsheet-engine/src/functions/
lookup_and_reference/). Three excel-graph cells disagree with it:
1. **Descending search skips a cell of another type.** `gen-m2/f7e6d351c8538ef5` Calc!A334
   `=(MATCH(Data!C9,Data!A11:C11,-1)>((Data!A5-Data!B2)*MIN((-1))))`, with C9 = 0.5,
   A11:C11 = 0.1, TRUE, 3.14159 and A5 = "abc". Excel gives `#N/A`, so MATCH itself is `#N/A`.
   The engine's search skips TRUE forward to 3.14159 and returns 3, so the comparison falls through
   to the right side's `#VALUE!`. Skipping backward (towards 0.1) for a descending search would give
   `#N/A`. That fits this cell, but one cell does not prove it.
2. **Non-numeric match_type read from a cell.** `gen-m2/04cfd82f54dbc2b2` Calc!A81
   `=IF(INDEX(Data!D5:E11,MATCH(" x ",Data!A8:B8,Data!A11),3),LEFT((Data!$A6&" x ")))` with
   A11 = "a", where Excel gives `#REF!` and the engine `#VALUE!`. Also `gen-m2/f9d4df4efdfbcbd5`
   Calc!A14
   `=AVERAGE((MATCH(Data!$E9,Data!C4:D10,Data!$E6)+SUM(Data!D3:D12,Data!A2,Data!E1:E6)),(SUM(Data!A$5,(-0.5))/COUNTA(Data!C11)))`
   with E6 = " x " and a 2-D lookup range C4:D10, where Excel gives `#REF!` and the engine `#VALUE!`.
   The probe evidence says a literal text match_type is `#VALUE!`
   (probes/lookup-approximate-search N44 `MATCH(1,F1:F5,"x")`). So either a 2-D lookup array is
   checked before match_type (`#REF!`?), or INDEX's out-of-range column 3 wins over an error row
   argument (A81).

Probe with the excel-probe skill. Suggested cells:
- `MATCH(0.5,{0.1,TRUE,3.14159},-1)`, `MATCH(0.5,{3,TRUE,0.1},-1)`, `MATCH(2,{5,"x",3,1},-1)`,
  `MATCH(2,{5,TRUE,TRUE,1},-1)`, and the same arrays as ranges.
- `MATCH(1,A1:B3,"x")` (2-D), `MATCH(1,A1:B3,0)`, `MATCH(1,A1:B3,1)`, `MATCH(1,A1:A3,"x")`, and a
  match_type from a text cell for both shapes.
- `INDEX(A1:B3,NA(),3)`, `INDEX(A1:B3,MATCH(1,A1:A2,"x"),3)` and `INDEX(A1:B3,1,3)` (error row
  vs out-of-range column).

## Acceptance criteria
- [ ] A probe case with the cells above, oracle-filled (excel-graph, excel-probe skill), with its
      results recorded in docs/excel-semantics/lookup.md (descending skip direction; argument check
      order).
- [ ] For each mismatch the probe shows, an engine task is filed (functions-lookup) that names the
      probe cells and these gen-m2 cells. Or the engine is fixed here, if the fix is one rule in
      `approximate_search` or `fn_match`/`fn_index`, and A334, A81 and A14 leave
      eval/baseline/gen-m2.txt with no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fractional match_type (T-1304).
- Text collation order (T-0220).
