---
id: T-3636
title: "corpus-bench 13894 F20:F25: a legacy CSE array formula's cells past its 6-row result are #N/A (M11 owner of T-0950's corpus cells)"
milestone: M11
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0950]
---

## Goal
The six extra cells of corpus-bench/13894/1_13894_input.xlsx's legacy array formula, Sheet1!F20:F25, are `#N/A`
as Excel shows them, so M11 has no unowned exit cell.

## Context
- The CSE array formula over F14:F25 returns a 6-row result (`ucode` = J3:J8). Excel fills the rest of the range
  with `#N/A`; ours gives `#VALUE!` in corpus-bench/13894/1_13894_input.xlsx Sheet1!F20, F21, F22, F23, F24 and F25.
  The formula is `IFERROR(INDEX(ucode,SMALL(IF(MATCH(ucode,ucode,0)=ROW(INDIRECT("1:"&ROWS(ucode))),...)),"")`.
- T-0950 (M9) implements the rule: docs/excel-semantics/arrays.md, "Legacy (CSE) array formula over a larger range".
  It already lists these cells, but as an M9 task it cannot own an M11 exit cell (decision 0096). This task is
  their M11 owner. Once T-0950 lands, confirm they pass and close it; if they still fail, fix what remains here.
- Found by `make exit-owners MILESTONE=M11` on 2026-11-01. M11's exit row is met (corpus-bench is within its
  allowance), so only the owners criterion of the Phase 1 exit waits on these cells.

## Acceptance criteria
- [ ] corpus-bench/13894/1_13894_input.xlsx Sheet1!F20:F25 pass, and the case leaves eval/baseline/corpus-bench.txt
      if no other cell fails.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The rest of T-0950 (poi/MatrixFormulaEvalTestData.xlsx).
