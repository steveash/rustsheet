---
id: T-0258
title: "AND/OR/XOR: a non-empty text literal argument is #VALUE! (poi J136); probe computed text"
milestone: M2
lane: functions-logic
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
AND, OR and XOR treat a text argument the way Excel does. Excel-cached evidence says a non-empty
text literal that is not "TRUE"/"FALSE" makes the call `#VALUE!` even next to a logical value, while
the engine ignores it; the boundary with empty text and with computed text must be probed first.

## Context
Found by the gap triager on 2026-10-06. Evidence already in eval (semantics:
docs/excel-semantics/coercion.md "AND/OR/XOR text arguments", Status partly UNVERIFIED):
- poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!J136 `AND("1",1)`:
  expected `#VALUE!`, actual TRUE (the only failing cell). I136 `AND(E14,E17,"1")` is `#VALUE!` too.
- ironcalc-calc LOGICAL/AND_OR_XOR.xlsx (Excel-cached) sheet "AND XOR OR": `AND("", TRUE)` = TRUE
  (E52), `AND("", FALSE)` = FALSE (E53), `AND("1")` and `AND("0")` = `#VALUE!` (E49, E50), the same
  for OR (C column) and XOR (D column). So an empty literal is skipped but "1" is not.
- gen-m2/893302d6c5b99e2e Calc!A108 (excel-graph, dynamic-array formula)
  `=IF(AND(Data!B9:B9,IFERROR("1",Data!E$3),Data!D$10),...,LEN(("ü"&Data!E10)))` = 4 (the FALSE
  branch), so text "1" returned by IFERROR is skipped, not `#VALUE!`.

Open questions for the probe (excel-probe skill, one case, AND/OR/XOR each): `AND("1",TRUE)`,
`AND("abc",TRUE)`, `AND(" ",TRUE)`, `AND("",TRUE)`, `AND(""&"1",TRUE)`, `AND(IFERROR("1",0),TRUE)`,
`AND(T("1"),TRUE)`, `AND({"1"},TRUE)`. This tells whether the rule is "literal vs computed", "empty vs
non-empty", or a desktop-2013 vs Microsoft 365 difference (decision 0030: Microsoft 365 desktop wins).

Code: `logical_nary` in `crates/rustsheet-engine/src/functions/logical/and_or_xor_not.rs`, branch
`CalcResult::String(..)`: it skips any string that is not "TRUE"/"FALSE" unless it comes from a
reference.

## Acceptance criteria
- [x] A probe case under eval/suites/probes pins the questions above (oracle-filled); the coercion.md
      entry is rewritten from it with Status VERIFIED.
- [x] `F	poi/FormulaEvalTestData_Copy.xlsx	EverythingTests!J136` disappears from eval/baseline/poi.txt
      (`make eval-baseline`), unless the probe shows Microsoft 365 differs from the Excel 15 cached
      value (then record it CONFLICTING per decision 0030 and leave the cell); no new F lines in any
      baseline, and the probe's cells pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- Error precedence across AND/OR/XOR arguments (T-0229).
- Array lifting of NOT (T-0225).

## Outcome
The probe probes/and-or-xor-text-arguments (84 cells: AND, OR and XOR x 28 argument shapes, excel-graph) settles the
rule. Microsoft 365 skips every text argument that is not "TRUE"/"FALSE", literal or computed, empty or not, in arrays
and in cells. Text "TRUE"/"FALSE" counts as the logical value, and a call left with only skipped text is `#VALUE!`. The
engine already passes all 84 cells, so no code changed. The Excel 15 cached `#VALUE!` for `AND("1",1)` (poi J136)
contradicts current Excel. Per decision 0030 2 (Excel 15 is below AppVersion 16) it is recorded as CONFLICTING in
coercion.md and added to T-1075 for a desktop answer, and J136 stays a known failure, as the acceptance criteria allow.
Eval delta: probes +1 case, 84 cells all passing; no other change; regressions 0.
Follow-ups: T-1075 item 7.
