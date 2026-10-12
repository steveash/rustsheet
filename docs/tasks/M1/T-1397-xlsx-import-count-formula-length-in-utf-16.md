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
The xlsx importer refuses what Excel for the web refuses for formula text with non-BMP characters and for defined-name bodies, once the exact boundaries are probed.

## Context
T-1386 probes (excel-graph): formula-length-nonbmp-33pieces loads (4,063 scalars, 8,023 UTF-16 units) and formula-length-nonbmp-34pieces is refused (4,186 scalars, 8,266 UTF-16 units; createSession 501), so Excel does not count Unicode scalars; the importer's `check_formula_length` (crates/rustsheet-xlsx/src/import/sheet_data.rs) does. name-body-length-8191 loads and name-body-length-8192 is refused. Exact UTF-16 boundary not probed (8,192 vs 8,193 units). See docs/excel-semantics/syntax.md "Formulas over 8,192 characters refuse the workbook".

## Acceptance criteria
- [ ] First, a boundary probe (excel-graph, mixed ASCII and emoji pieces, each literal 255 units or fewer): a formula of exactly 8,192, 8,193 and 8,194 UTF-16 units, and a name body with non-BMP text, recorded in syntax.md. Then `check_formula_length` uses the probed count (UTF-16 units if the boundary is 8,192); a defined-name body over the probed limit (at most 8,191 ASCII characters) is refused on import; unit tests at the limits; any importer deviation recorded per decision 0047.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
