---
id: T-1107
title: "Typed-input oracle path: what Excel stores when text is typed into a cell"
milestone: M1
lane: harness
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The oracle can answer "what does Excel store when this text is typed into a cell?", so cell-entry parsing (`set_user_input`) gets Excel evidence as formulas already do.

## Context
Every oracle backend today is xlsx in, xlsx out (`recalc` in tools/src/rustsheet_tools/oracle/backends.py):
values in a workbook are stored, not typed, so an xlsx probe cannot show how Excel parses typed text
(T-1100's handoff). The excel-graph backend could PATCH a range's `values` with raw strings, as a user typing,
and read back `values`/`valueTypes`/`numberFormat`. This is a new oracle capability: first write a short spec
(docs/specs/) covering the case format for typed cells (e.g. a `typed` map beside `cells`), the sidecar
shape, how the harness replays the same input through `set_user_input`, and the canary and credential
rules of the existing backend (docs/architecture/oracle.md). Never print or commit RUSTSHEET_ORACLE_* values.
First users: T-1100 (`1e-310`, `1e-309`, `2e-308`, `0e309`), and any date or percent entry questions.

## Acceptance criteria
- [ ] A spec in docs/specs/ for typed-input probes, reviewed.
- [ ] A typed-input probe case is filled by excel-graph and scored by the harness through `set_user_input`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
