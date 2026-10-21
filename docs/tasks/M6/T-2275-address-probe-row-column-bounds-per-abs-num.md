---
id: T-2275
title: "ADDRESS: probe row/column bounds per abs_num and a1 (relative R1C1 takes 0 and negative offsets, past the grid is #VALUE!)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
ADDRESS gives Excel's result for rows and columns past the grid (`#VALUE!` in every mode) and for relative R1C1 offsets of 0 or below (`ADDRESS(10,-2.5,4,FALSE)` = `R[10]C[-3]`), backed by a probe of the edges.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Cells: docs/excel-semantics/lookup.md "ADDRESS: argument errors first,
sheet_text "TRUE"/"FALSE" quoted, bounds", third bullet (added by this triage). `address_of`
(crates/rustsheet-engine/src/functions/lookup_and_reference/address_areas.rs) casts row and column with `as i32` (saturating at
2147483647) and refuses anything below 1 in every mode, so huge values print and relative R1C1 offsets of 0 or below are `#VALUE!`.
gen-m6/be0e3a1b0fc3f7a0 A167, `ADDRESS(9,3,4,(Data!A5=MULTINOMIAL(<blank>)),T("1E3"))`, is `#VALUE!` in Excel and unexplained: probe
`MULTINOMIAL(<blank cell>)` and `ADDRESS(9,3,4,FALSE,"1E3")`.
Probe (excel-probe skill, excel-graph, about 40 cells): for abs_num 1-4 and a1 TRUE/FALSE, row in {0, -1, 1048576, 1048577, 2.9, -2.5}
and column in {0, -1, -2.5, 16384, 16385, 0.5}; then record and implement.
Why opus: the rules need a probe.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 10 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/04fdc9a67474aab9 Calc!A98 | `=ADDRESS(17,123456789.123,2,IF(Data!$D8,(0.5-Data!C$9),IF(FALSE,(-0.0001))))` | "#VALUE!" | "R17C[123456789]" |
| gen-m6/36d5ce763ea63631 Calc!A154 | `=ADDRESS(SWITCH((100-2.5),AND(Data!C3,Data!C6:C8),Data!$D6,Data!E$6),7)` | "#VALUE!" | "$G$2147483647" |
| gen-m6/3a26794015dfbe9d Calc!A275 | `=ADDRESS(10,VALUE(IFERROR((-2.5),Data!$D$11)),4,ISNUMBER(#N/A))` | "R[10]C[-3]" | "#VALUE!" |
| gen-m6/44194b2fefa30de2 Calc!A307 | `=DATEVALUE((ADDRESS(9,Data!B$12,2,FALSE,"Data")&OFFSET(Data!A4:D10,0,-1)))` | "#REF!" | "#VALUE!" |
| gen-m6/9615a6de0f3ac417 Calc!A374 | `=ADDRESS(SUBTOTAL(COMBINA(10,0),Data!B11:C12,Data!A5:D6,Data!D7:E11),14,4,Data!B$6,"Data")` | "#VALUE!" | "Data!N2147483647" |
| gen-m6/a1e6448d0590f7c3 Calc!A373 | `=ADDRESS(9,((Data!D5+Data!D8)/ODD(Data!E9)),4,FALSE,"#N/A")` | "#VALUE!" | "#N/A'!R[9]C[2147483647]" |
| gen-m6/acdfeaf6e6eadd4e Calc!A379 | `=ADDRESS(2,ROUNDUP(DATEVALUE("1/15/2024"),5),2,(("?b"&Data!D11)<CSC(Data!B9)),"Data")` | "#VALUE!" | "Data!R2C[45306]" |
| gen-m6/b54448af3fde07ca Calc!A220 | `=ADDRESS((QUOTIENT(2^53+1,123456789.123)*0.3),5)` | "#VALUE!" | "$E$21887494" |
| gen-m6/be0e3a1b0fc3f7a0 Calc!A167 | `=ADDRESS(9,3,4,(Data!A5=MULTINOMIAL(Data!$D12)),T("1E3"))` | "#VALUE!" | "1E3'!R[9]C[3]" |
| gen-m6/d0c30225243fde60 Calc!A63 | `=ADDRESS(SUM(Data!E1:E11),3,1,ISNA(SUMPRODUCT(Data!$A2)),"Data")` | "#VALUE!" | "Data!R123456831C3" |

## Acceptance criteria
- [ ] These 10 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] A new probes case (excel-graph) holds the grid; every cell passes or is a filed task named in the Outcome.
- [ ] lookup.md's third bullet states the rule with cells and no UNVERIFIED left.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Argument error order and sheet quoting (T-2274).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
