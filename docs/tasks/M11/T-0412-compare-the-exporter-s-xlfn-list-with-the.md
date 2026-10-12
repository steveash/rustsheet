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
Compared the arm list with XlsxWriter 3.2.9 and the catalog: every XlsxWriter name with an engine function was already prefixed, but PERCENTOF (not on XlsxWriter's list; oracle: function-name-forms A1) was written bare; added it with a unit test. The other XlsxWriter names (FILTERXML, QUERYSTRING, WEBSERVICE, GROUPBY, PIVOTBY) have no engine function yet. Recorded in docs/excel-semantics/xlsx-storage.md. Eval delta: none.
