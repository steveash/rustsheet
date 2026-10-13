---
id: T-0416
title: "Export PERCENTOF with _xlfn. without staling the bare-PERCENTOF oracle evidence"
milestone: M11
lane: engine
status: in-progress
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
- [ ] The arm and a unit test land; function-name-forms A1 still tests the bare form.
- [ ] PERCENTOF gen cases refilled; the stale #NAME? remarks in the semantics docs corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
