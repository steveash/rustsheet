---
id: T-2320
title: "Probe which of a table's name and displayName structured references resolve through when they differ"
milestone: M8
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know, from an Excel probe, which of a table part's `name` and `displayName` structured references resolve through when the two differ, and make the importer/engine match.

## Context
T-2208 made `name` and `displayName` stand in for each other when one is missing. When both exist and differ the engine still keys tables by `name` (`expressions/parser/operands.rs`), and nothing reads `Table::display_name`; no oracle evidence says which Excel uses. The exporter also writes no `displayName` (`crates/rustsheet-xlsx/src/export`); check a displayName-only round trip. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] An oracle probe case shows which attribute `Name[col]` resolves through when they differ; evidence recorded in docs/excel-semantics/xlsx-storage.md.
- [ ] The exporter writes the attributes Excel needs, or a task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
