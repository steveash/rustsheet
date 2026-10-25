---
id: T-0466
title: "Cells that call a function of another workbook ([1]!NAME) are out of scope and not scored, like _xll. calls (poi HsGetVal, 56502)"
milestone: M11
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-0462]
---

## Goal
A cell whose formula calls a function defined in another workbook through an external-link index
(`[1]!HsGetValue(...)`, `[1]!LUCANET(...)`) is out of scope, as decision 0027 puts code and add-ins out
of scope. Such a cell is reported as out of scope and not scored, through the same harness path T-0462
builds for `_xll.` add-in calls.

## Context
Gap triage 2026-10-25 (`rustsheet-eval run --suite poi` on main c30bebf7). Three poi known failures
(eval/baseline/poi.txt) have no owner. Each calls a VBA add-in through the workbook's external link 1,
whose target is an `.xla` add-in (xl/externalLinks/_rels/externalLink1.xml.rels):
- `poi/HsGetVal.xlsx` Sheet1!B5 and C5 `=[1]!HsGetValue("HSACTIVE","Year#Jan",...)`: Excel's cached
  value is "#Missing" (text), ours `#NAME?`. The link target is `/Oracle/SmartView/bin/HsTbar.xla`, the
  Oracle Smart View add-in.
- `poi/56502.xlsx` Tabelle1!A4 `=[1]!LUCANET(...)` (one text argument): Excel's cached value is "pls. update", ours
  `#NAME?`. The link target is `.../LucaNet/Excel-Add-In/lucanet.xla`.
No engine can compute these values: they come from the add-in's code. They count against the poi suite
and hide real misses, as the `_xll.` cells of calipers pv26_003 do (T-0462).

T-0462 (in progress on origin/agent/harness/T-0462-xll-out-of-scope) decides how an add-in call is left
out of scoring and listed in report.md's out-of-scope section (crates/rustsheet-eval/src/report.rs,
eval/catalog/scope.json). This task extends that predicate to a function name prefixed by an
external-workbook index (`[n]!`). It is the same rule with one more formula shape, so it needs no
new decision.

## Acceptance criteria
- [ ] A cell whose formula calls a function written as `[n]!NAME(` is not scored (neither failing
      nor passing) and is counted in report.md's out-of-scope section with the `_xll.` cells, or
      beside them.
- [ ] The three lines `poi/HsGetVal.xlsx Sheet1!B5`, `Sheet1!C5` and `poi/56502.xlsx Tabelle1!A4`
      leave eval/baseline/poi.txt (`make eval-baseline` after `rustsheet-eval fetch --suite poi`),
      no other poi or calipers cell changes verdict, and no new known-failure lines appear.
- [ ] A unit test in crates/rustsheet-eval covers a `[1]!NAME(...)` formula. It also covers a
      `[1]Sheet1!A1` cell reference to another workbook, which stays scored: external references
      are not code.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Evaluating add-in or external-workbook functions.
- External cell references (`[1]Sheet1!A1`), which the engine already reads from the cached
  external-link values.
