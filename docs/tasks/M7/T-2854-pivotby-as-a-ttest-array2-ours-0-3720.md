---
id: T-2854
title: "PIVOTBY as a TTEST array2: ours 0.3720 against Excel's 0.3685 (gen-m9 c2684066d8df263e A214)"
milestone: M7
lane: functions-lookup
status: done
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
- [x] gen-m9/c2684066d8df263e Calc!A214 disappears from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
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

## Outcome
- A blank key stays a true blank (`ArrayNode::Empty`) inside GROUPBY and PIVOTBY results (`shown_key` removed,
  grouping.rs, pivotby.rs): COUNT and TTEST skip it, INDEX gives TYPE 1; spilled, it still displays 0.
- IFERROR and IFNA read a blank array element as 0, as they already read a blank cell of a multi-cell range
  (`if_error_like`, logical/mod.rs). That is the handoff's regression (`TEXTJOIN(IFERROR(GROUPBY(..)))` shows "0"), now
  fixed with no regressions.
- Eval: gen-m9 3630 -> 3626 (A214 here plus T-2460's 8d3588d8df0812df A240, d37313406b9f6e55 A182, e85daedce5c2675e A35);
  probes 2655 -> 2626 (33 fixed, T-2460's cells among them, so T-2460 is closed here). The handoff's new probe
  pivotby-ttest-array2 adds 4 known failures (TTEST 1-2 ulp off), filed as T-2993.
