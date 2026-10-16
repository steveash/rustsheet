---
id: T-1681
title: "Fuzz: PERCENTRANK.INC over SE:RCH aborts with a 3 GB allocation: bound values_from_range on whole-column ranges"
milestone: M9
lane: functions-stat
status: todo
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
A formula whose range argument covers whole columns of an almost empty sheet (`SE:RCH`, 7,000+
columns × 1,048,576 rows) never aborts the process by allocation (directive 5). The
order-statistics and paired-list functions give Excel's result in bounded memory.

## Context
Nightly fuzz run 38847367745 (2026-10-16, `main` 1aebd11c), target `formula_eval`, artifact
`oom-628770d99ec69d8b480f8269cc4c0d84b3ca8105`. The input is the 33 bytes
`PERCENTRANK.INC(SE:RCH,"İİx",2)` (the text holds two U+0130; base64
`UEVSQ0VOVFJBTksuSU5DKFNFOlJDSCwixLDEsHgiLDIp`). The target sets it as `=...` in `C3` of its small
model (`fuzz/fuzz_targets/formula_eval.rs`).

The steward reproduced it on a18a7329 with a release-build facade test (`Workbook::set("C3", ...)`,
`ulimit -v 4000000`): `memory allocation of 3019898896 bytes failed`, then `rust_oom` aborts.
The text argument has nothing to do with it:
- `PERCENTRANK.INC(SE:RCH,2)`, `PERCENTILE.INC(SE:RCH,0.5)` and `QUARTILE(SE:RCH,1)` abort the same way.
- `MEDIAN(SE:RCH)` and `SUM(SE:RCH)` return in 4 ms, because they use the clipping walker.

Root cause: `collect_sorted_values` (`functions/statistical/percentile.rs`) calls
`values_from_range` (`functions/statistical/chisq.rs:262`). On one sheet, that function builds one
`Option<f64>` per cell of the range on purpose, unclipped, because paired callers (CHISQ.TEST,
SUMXMY2, COVARIANCE and others) match two lists by position. Other callers of `values_from_range`
may have the same exposure: `mode_functions.rs`, `fisher.rs`, `t_dist.rs`, `frequency.rs`,
`prob.rs`, `z_test.rs`, `covariance.rs` and `math_and_trigonometry/sum.rs`.

Fix direction:
- Callers that only need the numbers, like the percentile and quartile family, go through the
  existing clipping walk (`for_each_cell_or_error_in_areas`, as MEDIAN does).
- Paired callers clip both ranges to the same used-area window, or read blanks past the used area
  without allocating them.
- Search for an existing helper first (`rg`). Never add a second copy.

Excel's values for whole-column order statistics: check probes for existing evidence. If none
exists, write a probe case (`excel-probe` skill).

## Acceptance criteria
- [ ] The reproducer is a unit test (decision 0035). It passes with no huge allocation, for example
      by asserting it returns quickly, or by running under a bounded allocator.
- [ ] Every caller of `values_from_range` is bounded by the used area on whole-column and
      whole-row ranges. Each caller kind gets a test: order statistics, paired lists, and
      MODE/FREQUENCY.
- [ ] The values match excel-graph evidence. Otherwise the case is left pending and the task is
      blocked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Performance work on bounded ranges beyond removing the per-cell allocation (perf lane).
