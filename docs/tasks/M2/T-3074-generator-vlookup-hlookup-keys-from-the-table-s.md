---
id: T-3074
title: "Generator: VLOOKUP/HLOOKUP keys from the table's first column or row, and VLOOKUP/HLOOKUP/INDEX indexes inside the drawn range"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-gen` draws VLOOKUP and HLOOKUP calls whose lookup_value usually comes from the table's first column (first
row for HLOOKUP) and whose col_index_num/row_index_num usually lies inside the table, and INDEX calls whose row_num and
column_num usually lie inside the array, so generated cells exercise the search and the pick instead of `#N/A`/`#REF!`.

## Context
- The stored gen-m2 oracle cells (43 cases) show the lookups are mostly errors at the outer call: VLOOKUP 413 of 473
  (228 `#N/A`, 118 `#VALUE!`, 57 `#REF!`), HLOOKUP 167 of 191, INDEX 311 of 404 (211 `#REF!`), against SUMIF,
  COUNTIF, COUNT 0 and MATCH 297 of 480 before T-2719's top-up. (Counted from `eval/suites/gen-m2/cases/*.oracle.json`
  by the outer function name of each `Calc` formula.)
- Cause, in eval/catalog/functions.yaml: VLOOKUP/HLOOKUP's lookup_value is a free `value` (no `probe:`), their index is
  `domain: { int: true, min: 1, max: 4 }` whatever the table's width or height, and INDEX's row_num `max: 6` and
  column_num `max: 4` ignore the array's shape.
- T-2719 fixed the same gap for MATCH with `probe: lookup_array` (tools/src/rustsheet_tools/gen/generator.py, the
  `drawn`/`PROBE_HIT_RATE` path, `cell_in`). Extend that mechanism; do not add a second one. The catalog header comment
  (functions.yaml lines 30-60) documents each knob and must document the new ones.
- Keep the error edges covered: like `PROBE_HIT_RATE` (95%) and `SHAPE_MISMATCH_RATE`, a minority of draws stays free
  so `#N/A`, `#REF!` and `#VALUE!` from a bad key or index remain in the stream.

## Acceptance criteria
- [ ] A catalog knob (for example `probe_edge: first_column | first_row` on the `probe` param, and an index bound tied
      to a range param's width or height) is documented in the functions.yaml header and used by VLOOKUP, HLOOKUP and
      INDEX; MATCH's existing `probe:` behaves as before.
- [ ] tools/tests/test_generator.py: over a seeded run of 200 VLOOKUP, HLOOKUP and INDEX calls each, at least 80% of
      keys come from the first column/row of the drawn table and at least 80% of indexes lie inside it, and some draws
      are still outside (both asserted); any recorded generator digest is re-recorded in the same PR.
- [ ] `make py-check` (ruff, mypy strict, pytest) passes; `make check` green; reviewers approve.

## Out of scope
- Generating and oracle-filling the new gen-m2 batch (T-3075).
- XLOOKUP, LOOKUP and other M6 lookups.
