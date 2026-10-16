---
id: T-1701
title: "Probe a sheet rename under an undefined qualified name and a workbook name read through a sheet prefix"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Know, from Excel, what a sheet rename does to the prefix of (a) a qualified name that resolves to nothing (`=Data!Missing`) and (b) a workbook name read through a sheet prefix (`=Feb!loc`), and make the engine match.

## Context
T-1693 made a rename rewrite the prefix of a name scoped to the renamed sheet. The engine currently rewrites (a) (the parser gives it scope Some(Data) with an empty formula) but keeps the prefix of (b) (scope None); neither has oracle evidence. See `rename_defined_name_sheet` in `expressions/utils/mod.rs` and `prefixed_name_operand` in `parser/operands.rs`. Use the `excel-probe` skill; model the case on `probes/sheet-ops/sheet-ops-rename-sheet-scoped-name`.

## Acceptance criteria
- [ ] Probe cases for (a) and (b) with oracle values, in `eval/suites/probes/cases/sheet-ops/`.
- [ ] The engine matches them, or a follow-up task is filed; the finding is in `docs/excel-semantics/references.md`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Workbook-scoped names read without a prefix.
