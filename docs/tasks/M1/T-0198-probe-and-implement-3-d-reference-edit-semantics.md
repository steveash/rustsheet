---
id: T-0198
title: "Probe and implement 3-D reference edit semantics (sheet insert/delete/move inside a span, row and column edits)"
milestone: M1
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-0122]
---

## Goal
3-D references follow Excel when sheets inside or at the ends of a span are inserted, deleted or moved, and when rows or columns are inserted or deleted on a sheet of the span.

## Context
T-0122 added `Node::SheetSpanKind` (crates/rustsheet-engine/src/expressions/parser/mod.rs). Today
(docs/excel-semantics/references.md "3-D references when sheets or cells change", UNVERIFIED):
renaming an end sheet renames it in the formula; moving a sheet re-resolves the span by name;
deleting an end sheet makes the reference `#REF!`; row and column edits do not shift the span
(stringify passes `DisplaceData::None` to the inner reference). Excel's behavior is not probed and
edits cannot be probed with a static workbook, so this needs evidence from desktop Excel or
documentation plus a design note first.

## Acceptance criteria
- [ ] Evidence for each edit recorded in docs/excel-semantics/references.md (or an explicit
      decision when none can be had).
- [ ] The engine matches it, with unit tests.
- [ ] `make check` green; reviewers approve.

## Out of scope
- 3-D references inside INDIRECT text.
