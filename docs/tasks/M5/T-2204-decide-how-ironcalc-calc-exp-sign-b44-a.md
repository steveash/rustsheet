---
id: T-2204
title: "Decide how ironcalc-calc EXP_SIGN B44 (a cached 0 where Excel keeps 2^-1074) counts for the M5 exit"
milestone: M5
lane: research
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A decision record that settles `ironcalc-calc/MATH_AND_TRIGONOMETRY/EXP_SIGN.xlsx!EXP_SIGN!B44` for the M5 ironcalc-calc
criterion. The cell is one of the 9 cells left in the M5 row's "failing excl. exceptions". No engine change can fix it:
the engine already gives Excel's in-memory value.

## Context
- The cell: B44 is `=EXP(A44)` with A44 = -745 (stored). Its cached value is 0 (source excel-cached, "Microsoft Excel
  Online"). Ours is 5E-324 = 2^-1074 (x87 model V6, T-1931). `make eval-show CASE=ironcalc-calc/MATH_AND_TRIGONOMETRY/EXP_SIGN.xlsx`.
- Excel's own evidence says the value is not 0. probes/subnormal-function-results C4 `=(EXP(-745))=0` is FALSE, and C6
  (`EXP(-720)`) too (excel-graph). docs/excel-semantics/numbers.md "Worksheet EXP (T-1931)" says a subnormal result is
  kept, not flushed. A4 `=EXP(-745)` passes only by decision 0042, which scores an excel-graph 0 against a kept
  subnormal as a pass. That rule needs an excel-graph sidecar and a FALSE `(x)=0` companion in the same case, so it never
  reaches an excel-cached cell. B44 is the only excel-cached cell in ironcalc-calc with an expected 0 and a subnormal
  engine value (planner scan of eval/baseline/ironcalc-calc.txt, 2026-10-20). poi was not checked.
- Likely cause: the excel-graph oracle reads values from the xlsx that Excel for the web saves (tools
  `oracle/cli.py` `_recalc_cached` → `checked_values`). So "Graph reports a kept subnormal as 0" may really be "Excel writes
  a kept subnormal to the file as 0", the same artifact as B44's cached value. Check that from the saved workbook (the
  oracle cache, or `uv run rustsheet-oracle recalc -b excel-graph <case>` on a one-cell case) before the decision rests on it.
- Measures that bind the choice: decision 0085 item 3 (a cell further than 4 ulp, or not numeric, always blocks; 0 and
  2^-1074 are adjacent doubles, 1 ulp apart); decision 0087 (named exit exceptions live in
  eval/milestones/exit-exceptions.tsv, added only in the PR that takes the decision); decision 0052 (accepted deviations
  are directive-5 limits only, so this cannot be a DEVIATION); prime directive 2 (no gate weakens, no hand-edited baseline).
- Options to weigh, at least:
  (a) a named exit exception for B44 under the new decision;
  (b) extend decision 0042 to excel-cached expectations when the same formula over the same input has a FALSE
      `(x)=0` companion elsewhere in Excel evidence, with the harness change filed as a harness task;
  (c) leave B44 blocking until something else changes, and say what that would be.
- T-1944 (functions-math) owns the EXP edges. With this task, B44 leaves T-1944's scope.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) chooses among the options with the evidence above. It says whether the
      saved xlsx holds 0 for a kept subnormal, citing the file or case checked, and what the decision means for any
      later excel-cached subnormal cell.
- [ ] If the decision names B44 as an exception: the same PR adds its line to eval/milestones/exit-exceptions.tsv
      (decision 0087 format, milestone M5), and the cumulative M5 row's "failing excl. exceptions" drops by 1 in
      `make eval`. If it calls for a harness rule: a harness task is filed with the rule and its test cells, and B44 is
      named as that task's acceptance cell.
- [ ] docs/milestones.md M5 names the outcome next to the other exceptions, if any. numbers.md "Worksheet EXP (T-1931)"
      records the file-serialization finding. T-1944's Context points to the decision, and B44 leaves T-1944's scope.
- [ ] Eval: no baseline change (B44 stays an F line in eval/baseline/ironcalc-calc.txt unless a harness rule passes it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- EXP's overflow and underflow edges (T-1944, T-2203).
- Changing EXP or V6.
