---
id: T-2394
title: "Pivot import hardening: workbook-wide pivot cap, one log line per dropped batch, normalized part keys"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-2082]
---

## Goal
The pivot importer stays small in memory and in log output on hostile workbooks with many pivots, and treats equivalent part names as one part.

## Context
T-2082 security review minors, `crates/rustsheet-xlsx/src/import/pivots.rs`:
1. Every accepted pivot allocates a hidden bitset per field sized to its cache's items, and only a 64-per-sheet cap limits the count: one 1M-item cache and about 5,000 tiny pivot parts allocate about 600 MB. Add a workbook-wide pivot cap (for example 1,000) or a budget on the bitset words and fields allocated, as a SPEC-0015-style limit, and extend web.md's "Pivot caches past the import limits are dropped" DEVIATION entry.
2. `load_pivots` calls `log_skip` once for each pivot past the per-sheet cap. Log one summary line per sheet with the count dropped, and do the same anywhere else a file can trigger one log line per element.
3. The duplicate-part checks (`occurrences` over part names) compare exact strings, but `PartNames` matches entries case-insensitively and treats `\` as `/`. Count on the normalized key (`package.rs` `part_key`; reuse it rather than copying it).

## Acceptance criteria
- [x] Unit tests in `import/pivots/tests.rs`: pivots past the workbook cap are dropped; `xl/pivotTables/A.xml` and `xl/pivottables/a.xml` count as one part reached twice.
- [x] The DEVIATION entry in web.md names the new cap.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any change to what loads below the limits.

## Outcome
A workbook-wide cap of 1,000 pivots (`MAX_PIVOTS`, in sheet order) joins the 64-per-sheet cap; a sheet that loses pivots to either logs one summary line with the count. Duplicate-part checks for pivot tables and cache definitions count on `package::part_key` (now `pub(super)`), so case and `\\` variants are one part. Not done: the per-cache `log_skip` lines in `load_caches` stay one per listed cache (bounded by the workbook.xml size, not by a cap; no other per-element log is file-driven). web.md DEVIATION entry extended. No eval change.
