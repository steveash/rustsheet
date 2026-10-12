---
id: T-1397
title: "xlsx import: count formula length in UTF-16 units and refuse defined-name bodies over 8,191 characters (T-1386 evidence)"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The xlsx importer refuses what Excel for the web refuses: a formula whose text is over 8,192 UTF-16 code units, and a defined-name body over 8,191 characters.

## Context
T-1386 probes (excel-graph): formula-length-nonbmp-33pieces loads (4,063 scalars, 8,023 UTF-16 units) and formula-length-nonbmp-34pieces is refused (4,186 scalars, 8,266 UTF-16 units; createSession 501), so Excel does not count Unicode scalars; the importer's `check_formula_length` (crates/rustsheet-xlsx/src/import/sheet_data.rs) does. name-body-length-8191 loads and name-body-length-8192 is refused. Exact UTF-16 boundary not probed (8,192 vs 8,193 units). See docs/excel-semantics/syntax.md "Formulas over 8,192 characters refuse the workbook".

## Acceptance criteria
- [ ] `check_formula_length` counts UTF-16 code units; a defined-name body over 8,191 characters is refused on import; unit tests at the limits; any importer deviation recorded per decision 0047.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
