---
id: T-2854
title: "PIVOTBY as a TTEST array2: ours 0.3720 against Excel's 0.3685 (gen-m9 c2684066d8df263e A214)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`TTEST(Data!C10:D12,IF(FALSE,Data!A4,PIVOTBY(Data!C1:D12,Data!D1:D12,Data!B1:D12,MIN)),1,2)` is Excel's 0.36852939060019746; ours is 0.3719598969194891.

## Context
gen-m9/c2684066d8df263e Calc!A214, moved from T-2469 (T.TEST itself is right; array2 is the PIVOTBY result, so the grid PIVOTBY returns for this call differs). Trace the argument with the T-2524 method (arguments evaluated by the engine) before changing PIVOTBY.

## Acceptance criteria
- [ ] gen-m9/c2684066d8df263e Calc!A214 disappears from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Cause found (probe added: eval/suites/probes/cases/pivotby-ttest-array2, excel-graph 2026-10-28): the PIVOTBY grid
matches Excel cell for cell as spilled, but Excel's blank row/column keys (blank source cells) are true blanks
inside the array: COUNT(PIVOTBY(..)) is 108 in Excel, 114 ours (INDEX row/column counts BA1:BB32 pin the six cells:
row keys A12, A13, B11 and the column-key header AD1:AF1). TTEST/COUNT skip them; ours holds Number(0.0)
(`shown_key`, grouping.rs). Same cause as T-2460 (GROUPBY too).
Tried: `shown_key` returning the key unchanged (blank stays `ArrayNode::Empty`). That fixes the probe but regresses
probes/groupby-sort-filter-args(2) G!B10 etc. (about 20 cells): `TEXTJOIN("|",FALSE,IFERROR(r,"#"))` over a blank key
gives "0" in Excel, ours "" , so IFERROR (and other element-wise consumers) must read an Empty array element as 0 while
INDEX/`&""`/COUNT keep it blank. Next: find which lifts/consumers turn array blanks into 0 in Excel (probe
IFERROR/IF/N/TEXTJOIN over a VSTACK or GROUPBY blank), implement that in the shared array-element read, then apply the
`shown_key` change. Rule goes in docs/excel-semantics/arrays.md. Check the eval gate for regressions with
`rustsheet-eval check` before pushing.
