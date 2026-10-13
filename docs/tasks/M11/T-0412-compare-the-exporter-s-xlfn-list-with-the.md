---
id: T-0412
title: "Compare the exporter's _xlfn. list with the function catalog"
milestone: M11
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Every engine function Excel stores with `_xlfn.` is written with it by `Function::to_xlsx_string`.

## Context
T-0411 added five missing arms (DAYS, GAUSS, IMSECH, GAMMA.DIST, FORECAST.LINEAR) but did not compare the whole arm list in crates/rustsheet-engine/src/functions/xlsx_names.rs with the function catalog (eval/catalog) and XlsxWriter's future-function list. Other newer functions may still be written bare, which Excel reads as #NAME? (docs/excel-semantics/xlsx-storage.md).

## Acceptance criteria
- [ ] The comparison is done; every missing function gets an arm and a unit test, or a task if the evidence is missing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Compared the exporter's arm list with XlsxWriter 3.2.9 and the catalog. Every XlsxWriter name with an engine function already has an arm. The only engine function missing an arm is PERCENTOF, but adding it also changes the case builder and stales existing oracle evidence, so it is T-0416. Review also found two arms that add a prefix Excel omits (GAMMALN, ISO.CEILING): T-0417. No code change; recorded in docs/excel-semantics/xlsx-storage.md. Eval delta: none.
