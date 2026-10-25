---
id: T-0473
title: "External links: O(1) per-sheet has-cells and extent lookups in the cache"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Reading a reference into a linked workbook costs O(1) to learn whether the cache holds cells of its sheet and what
extent they span, instead of a scan of every cached cell.

## Context
- Found in T-0457's review (security and quality reviewers, MINOR). `Model::cached_sheet`
  (crates/rustsheet-engine/src/external_links.rs) runs `book.cells.keys().any(..)` on every external reference
  evaluation; for a sheet with no cells it reads the whole map. `cached_extent` scans the same way for every whole
  column or row lookup and aggregate. A workbook with a large cache on one sheet and many formulas pointing at an
  uncached sheet costs formulas x cached cells.
- Fix: compute, per book, which sheet indexes hold cells and each one's extent once (at import, or lazily on the
  model), and read them in `cached_sheet` and `cached_extent`. `ExternalBook` is `Encode`/`Decode`; a lazily built
  side table on the model avoids changing its serialized form.

## Acceptance criteria
- [ ] `cached_sheet` and `cached_extent` no longer scan `book.cells`.
- [ ] Eval: 0 regressions (fixtures external-link-* and corpus-bench unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what a reference reads.
