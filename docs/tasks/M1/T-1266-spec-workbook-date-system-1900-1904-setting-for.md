---
id: T-1266
title: "Spec: workbook date system (1900/1904) setting for the formatter and date functions"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0005-workbook-date-system.md
---

## Goal
An accepted spec for carrying the workbook date system (1900/1904) through import, export, the
facade and the formatter, with an ordered S-sized rollout, so T-1105 can be implemented.

## Context
T-1105 (negative serials under elapsed formats: `#VALUE!` in 1900, minus kept in 1904) cannot be
fixed without date-system awareness, and the engine has none (`rg -i 1904 crates` finds only the
TODO). Adding it changes `WorkbookSettings` (bitcode format), xlsx import/export and the formatter
signature, which is architectural (CLAUDE.md directive 4).

## Acceptance criteria
- [ ] docs/specs/SPEC-0005-workbook-date-system.md covers model, import/export, formatter, facade,
      alternatives, tests and rollout; architecture and performance reviewers approve.
- [ ] T-1105 links the spec and depends on the implementation tasks; rollout tasks filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing any rollout step.
