---
id: T-2821
title: "PIVOTBY as a TTEST array2: ours 0.3720 against Excel's 0.3685 (gen-m9 c2684066d8df263e A214)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
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
