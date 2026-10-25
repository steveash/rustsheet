---
id: T-2722
title: "T.TEST and TTEST: two gen-m9 cells left from T-2469 (A15 unexplained NUM, A214 PIVOTBY sample numeric)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Two gen-m9 cells T-2469 left match Excel: `TTEST(CELL("col",Data!D4),Data!A5:B10,2,2)` (a5342a7e24fd803e A15, Excel `#NUM!`, ours `#DIV/0!`) and c2684066d8df263e A214 (a PIVOTBY sample; Excel 0.36852939060019746, ours 0.3719598969194891).

## Context
Left over from T-2469 (statistics.md "T.TEST: an error sample first..."). A15 needs a probe of its sub-expressions; A214 is probably the PIVOTBY result's shape, not T.TEST. Code: `fn_t_test` in functions/statistical/t_dist.rs.

## Acceptance criteria
- [ ] Both cells' known-failure lines disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), no new ones.
- [ ] Probe and settle the two unevidenced T-2469 rules: `T.TEST(<range with an error>,NA(),1,9)` (ours `#N/A`, array2's error value first) and a non-empty typed text sample `T.TEST("a",A1:A4,1,3)` / `(..,1,1)` (ours: a sample with no number); update statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
