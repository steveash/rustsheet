---
id: T-1711
title: "Displayed text of fractions, round-half #,##0 and negative dates matches Excel"
milestone: M10
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
`Workbook::formatted` (and TEXT where it shares the formatter) gives Excel's displayed text for the two cells probes/displayed-text-formats fails besides the fraction (S!A17, T-2030).

## Context
T-1610's first displayed-text probe (probes/displayed-text-formats, excel-graph `Range.text`) fails three cells: `# ?/?` on 12.5 is "12 1/2" in Excel, " 1/2" in ours (S!A17); `TEXT(1234.5,"#,##0")` is "1,235", ours "1,234" (S!A21, a value failure too: half rounds away from zero); a negative serial under `yyyy-mm-dd` displays as 255 `#` characters, ours `#VALUE!` (S!B4). excel-graph's text does not depend on the column width (B1:B3), so width is not needed. Code: crates/rustsheet-engine/src/formatter/format.rs. Rule: docs/excel-semantics/number-formats.md "Displayed text (`Range.text`) of a formatted cell".

Gap triage 2026-10-19: S!A17 (`# ?/?` on 12.5) moved to T-2030, which implements fraction formats (the engine
has none). Suspected root cause of the round-half cells: `format.rs`'s `ParsePart::Number` branch rounds through
`number_format::to_precision`, which formats with Rust's `{:e}`/`{:.N}` and so rounds an exact binary tie to even;
Excel rounds it away from zero. More value known failures with the same cause, eval/baseline/gen-m10.txt
(excel-graph, T-1611): 810fee377a42575b T!A2 `TEXT(36.5,"0")` "37" (ours "36"); 039accabcfe1e6c7 T!A12
`TEXT(2.5,"""(""0"")""")` "(3)"; 0e3d8eba9432a18d T!A18 `TEXT(-2.5,"0;-0;""zero"";""text: ""@")` "-3";
4398098060a7fa9d T!A5 `TEXT(0.125,"#.#0")` ".13"; 659cd5776d423ce3 T!A5 `TEXT(2.5,"#,##0 ;(#,##0)")` "3 ";
b4f66526f822ff2c T!A24 `TEXT(36.5,"0\%")` "37%". `to_precision` has other callers: check each before changing it.

## Acceptance criteria
- [ ] probes/displayed-text-formats S!A21 leaves eval/baseline/probes.txt, and its text section shows A21 and B4 passing (`make eval`).
- [ ] The six gen-m10 TEXT() lines above leave eval/baseline/gen-m10.txt; no new F or TF lines.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column width: excel-graph's displayed text does not depend on it (decision 0068).
