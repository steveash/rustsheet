---
id: T-0163
title: "Drop the facade's zip pre-pass now that the importer enforces the byte budget"
milestone: M1
lane: api
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-0121]
---

## Goal
`Workbook::from_xlsx_bytes` inflates a workbook once, not twice, with the same limits enforced.

## Context
T-0121 made `rustsheet-xlsx` charge every part it reads against `ImportLimits` (entries, inflated
bytes), and the facade passes its `LoadOptions` sizes through. `check_zip_limits` in
`crates/rustsheet/src/workbook.rs` still inflates every zip entry into a sink first, so loading pays
for decompression twice. The difference: the pre-pass also inflates entries the importer never
reads (the `zip_bomb_is_rejected_by_actual_inflated_size` test zips an unreferenced `bomb.xml`).

## Acceptance criteria
- [ ] `check_zip_limits` removed (or reduced to the cheap entry-count / declared-size checks); the importer's typed `XlsxError::Limit` is the enforcement.
- [ ] The facade zip-bomb tests rewritten so the bomb is a part the importer reads (e.g. `xl/sharedStrings.xml`) and still fail with `LoadError::Options`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the limit values.
