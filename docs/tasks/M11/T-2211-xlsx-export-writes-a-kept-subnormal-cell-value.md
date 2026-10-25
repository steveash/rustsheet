---
id: T-2211
title: "xlsx export writes a kept subnormal cell value as 0, as Excel does"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
When RustSheet saves an xlsx, a formula cell whose value is a non-zero subnormal (below 2.2250738585072014E-308) is
written as `<v>0</v>`, as Excel does, so a file RustSheet saves reads back like one Excel saves.

## Context
- Decision 0089 and docs/excel-semantics/numbers.md "Worksheet EXP (T-1931)": Excel for the web keeps `EXP(-745)`
  = 2^-1074 in memory (`(EXP(-745))=0` is FALSE) but writes `<v>0</v>` to the file; a normal value such as
  `EXP(-708)` is written in full (`<v>3.3075530036384078E-308</v>`). Evidence: probes/subnormal-saved-as-zero
  (excel-graph, 2026-10-20) and the saved workbook the oracle cached for it.
- Code: crates/rustsheet-xlsx/src/export/worksheets.rs (cell value writing). Search for an existing subnormal helper
  (`arithmetic::flush_subnormal` in the engine) before adding one.
- Only the cached `<v>` changes; the in-memory value and recalculation are untouched.
- Evidence gap (2026-10-25 steward spot-audit of #1374, fidelity and architecture reviewers): the
  only evidence is Excel for the web's save of positive kept subnormals (`EXP(-745)`, `EXP(-720)`, `=A1`
  in `probes/subnormal-saved-as-zero`). Before implementing, probe a negative one with a FALSE companion
  (`=-EXP(-745)` beside `=(-EXP(-745))=0`) and a stored constant subnormal; desktop Excel's save is
  unprobed (record that, decision 0039).

## Acceptance criteria
- [ ] An export test: a workbook with `=EXP(-745)` saves `<v>0</v>` for that cell, and `=EXP(-708)` saves its value
      in full; reloading the file and recalculating still gives 2^-1074.
- [ ] numbers.md "Worksheet EXP (T-1931)" names the export as implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Negative zero and other number spellings in the file.
