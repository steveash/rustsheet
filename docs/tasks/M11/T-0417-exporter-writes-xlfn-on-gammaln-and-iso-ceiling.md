---
id: T-0417
title: "Exporter writes _xlfn. on GAMMALN and ISO.CEILING that Excel-authored files omit"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The exporter writes GAMMALN and ISO.CEILING the way Excel does, or the evidence says it should not change.

## Context
Found by T-0412's review. Excel-authored crates/rustsheet-xlsx/tests/calc_tests/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx stores `GAMMALN(E:E)` and MATH_AND_TRIGONOMETRY/FLOOR_CEILING.xlsx stores `ISO.CEILING(...)` bare, yet `Function::to_xlsx_string` writes `_xlfn.GAMMALN` and `_xlfn.ISO.CEILING`. Excel accepts `_xlfn.` on classic names (SUM, NORMSDIST, CEILING probed), so values are probably unaffected; GAMMALN and ISO.CEILING were not probed. The case builder also writes them prefixed into oracle workbooks.

## Acceptance criteria
- [ ] Probe `_xlfn.GAMMALN` and `_xlfn.ISO.CEILING` (excel-probe skill); remove the two arms with tests citing the workbooks, or record why they stay.
- [ ] xlsx-storage.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
