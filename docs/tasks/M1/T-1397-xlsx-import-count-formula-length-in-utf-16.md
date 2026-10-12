---
id: T-1397
title: "xlsx import: match Excel's formula-length count for non-BMP text and refuse over-limit defined-name bodies (probe first)"
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
T-1386 probes (excel-graph): formula-length-nonbmp-33pieces loads (4,063 scalars, 8,023 UTF-16 units) and a 34-piece variant (4,186 scalars, 8,266 UTF-16 units) is refused on load (dropped after the run, createSession 501, so it has no case file; re-create it as LEN of 34 `&`-joined 120-emoji literals), so Excel does not count Unicode scalars; the importer's `check_formula_length` (crates/rustsheet-xlsx/src/import/sheet_data.rs) does. The counting unit and the boundary are open: anywhere in 8,024 to 8,265 UTF-16 units. Name bodies: name-body-length-8191 loads and an 8,192-character `1+1+...+11` body is refused on load (also dropped, no case file); non-BMP name bodies are unprobed. See docs/excel-semantics/syntax.md "Formulas over 8,192 characters refuse the workbook".

## Acceptance criteria
- [ ] First, a boundary probe (excel-graph, mixed ASCII and emoji pieces, each literal 255 units or fewer): a formula of exactly 8,192, 8,193 and 8,194 UTF-16 units, and a name body with non-BMP text, recorded in syntax.md. Then `check_formula_length` uses the probed count (UTF-16 units if the boundary is 8,192); a defined-name body over the probed limit (at most 8,191 ASCII characters) is refused on import; unit tests at the limits; any importer deviation recorded per decision 0047.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
