---
id: T-0416
title: "Export PERCENTOF with _xlfn. without staling the bare-PERCENTOF oracle evidence"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`Function::Percentof` exports as `_xlfn.PERCENTOF`, and the oracle evidence for the bare form stays reproducible.

## Context
Found by T-0412. Excel stores bare `PERCENTOF` as #NAME? and `_xlfn.PERCENTOF` calculates (probes/catalog/function-name-forms A1/A2; docs/excel-semantics/xlsx-storage.md). T-0412's one-line arm was dropped after review: `tools/src/rustsheet_tools/spec2xlsx.py` reads the `_xlfn.` arms of xlsx_names.rs, so the arm would make the case builder prefix PERCENTOF everywhere. That would (a) make function-name-forms A1 unreproducible (it tests the bare form), (b) leave stale the gen-m5/m6/m7 oracle values of #NAME? for bare PERCENTOF (e.g. gen-m5/065c0542a4ce3d1c Calc!A11), and the 'PERCENTOF is #NAME? on excel-graph' remarks in numbers.md, references.md and coercion.md; (c) make the xlsx-storage.md line 'GROUPBY, PIVOTBY and PERCENTOF are not on it' wrong. Needs a builder opt-out for A1, a refill of the PERCENTOF gen cases and a re-probe of PERCENTOF snapping/overflow/unions.

## Acceptance criteria
- [x] The arm and a unit test land; function-name-forms A1 still tests the bare form.
- [x] PERCENTOF gen cases refilled; the stale #NAME? remarks in the semantics docs corrected.
- [x] `make check` green; reviewers approve.

## Out of scope
- The COS(huge) `#NUM!` quirk (T-1192), which gen-m6/6b2aa08e3b916da5 Calc!A32 now shows behind a PERCENTOF.

## Outcome
- `Function::Percentof` exports as `_xlfn.PERCENTOF` (unit test `newer_functions_export_with_xlfn_prefix`).
- Case builder: a sheet's `bare_function_names` list writes those formula cells without `_xlfn.`
  (spec2xlsx.py, mirrored as an ignored field in rustsheet-eval's `SheetSpec`; tests in
  test_spec2xlsx_prefixes.py). probes/catalog/function-name-forms lists A1, refilled: still `#NAME?`.
- Refilled the 11 gen-m5/m6/m7 cases that call PERCENTOF (excel-graph, 2026-10-13); new probe
  probes/percentof-snap-overflow-union (15 cells): PERCENTOF's sums snap near zero like SUM, overflow
  is `#NUM!`, unions work in both arguments, direct text and TRUE count. The engine now sums
  through `fn_sum` (snapped); `fn_sum_unsnapped` is gone. Semantics docs corrected (numbers.md,
  references.md, coercion.md, xlsx-storage.md).
- Eval delta: gated suites unchanged (0 fixed, 0 regressions; probes gains the new case, 15/15
  passing); non-gated gen-m5 392 -> 394, gen-m6 975 -> 979, gen-m7 932 -> 933 passing cells. The one
  PERCENTOF cell still failing (gen-m6/6b2aa08e3b916da5 Calc!A32) is COS of 2.4E+14, the known T-1192
  quirk.
