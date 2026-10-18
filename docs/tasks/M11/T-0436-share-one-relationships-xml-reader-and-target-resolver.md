---
id: T-0436
title: "Share one relationships-XML reader and target resolver in xlsx_snapshot"
milestone: M11
lane: harness
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`xlsx_snapshot` has one relationships-XML reader and one relationship-target resolver.

## Context
T-0434 added `read_sheet_tables` and `resolve_relative` (crates/rustsheet-eval/src/xlsx_snapshot/tables.rs), which repeat the `<Relationship>` loop of `read_workbook_rels` and overlap `normalize_target` (reviewer finding on T-0434).

## Acceptance criteria
- [ ] A shared `relationships(xml)` helper and one target resolver are used by the workbook and sheet readers; the duplicates are gone.
- [ ] No eval baseline change.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
