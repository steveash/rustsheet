---
id: T-1148
title: "Oracle-load exported vm errors on formula and dynamic-array cells; pin the rich-error-export fixture to the exporter"
milestone: M1
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel is shown to load every shape of exported rich error, and the committed evidence keeps describing what the exporter writes.

## Context
T-1014 review (excel-fidelity, minor). fixtures/rich-error-export.xlsx (excel-graph) proves Excel loads rich errors
written by `crates/rustsheet-xlsx/src/export/rich_errors.rs` as constants (`ErrorCell`). Not yet oracle-loaded: a
formula cell carrying `vm` (`<f>` + `t="e"` + `vm`), a dynamic-array anchor carrying `cm="1"` and `vm` together, and
a workbook mixing rich-error cells with spilling `cm` cells (e.g. `=SEQUENCE(2)` beside a FILTER with no match).
Excel writes that combination itself (tests/calc_tests/LOOKUP_AND_REFERENCE/DROP_TAKE.xlsx), so the risk is low.
Also nothing ties the fixture to the exporter: a later change to the parts would leave the evidence stale.

## Acceptance criteria
- [ ] An exported workbook with the three shapes above is loaded by excel-graph (`rustsheet-oracle recalc`), with ERROR.TYPE of each cell as expected; committed as a fixture with its sidecar.
- [ ] A test exports the fixture's model and asserts `xl/metadata.xml`, `xl/richData/*`, the `vm` attributes and the content-type overrides equal the fixture's.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
New fixture `rich-error-export-formulas.xlsx` (+ oracle sidecar, excel-graph 2026-10-08): a plain formula with `vm`
(`IMAGE` → #BLOCKED!), array anchors with `cm` and `vm` (`FIELDVALUE`, `FILTER` no match, blocked `SEQUENCE`) and a
spilling `cm` cell, in one workbook. Excel for the web loads it and returns #FIELD!, #BLOCKED!, #CALC!, #SPILL! with
ERROR.TYPE 13, 11, 14, 9. Test `rich_error_fixtures_match_the_exporter` pins both rich-error fixtures to the exporter
(`vm` cells, metadata, richData parts, content-type overrides); changing `fCollapsed` in the exporter fails it.
Eval delta: fixtures +1 case (15 cells), all passing, 0 regressions.
