---
id: T-1714
title: "Probe A/P and locale bracket inferences"
milestone: M2
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Every inference T-1702 made about `A/P` and `[$-...]` is probed in Excel, and the engine matches.

## Context
T-1702 lexes any `[$-...]` as a silent locale and treats `A/P` unlike `AM/PM` in a fourth section. See the "Inferred, not probed" paragraph in docs/excel-semantics/number-formats.md. Code: formatter/lexer.rs, formatter/complexity.rs.

## Acceptance criteria
- [ ] Probe `[$-zzz]0`, `[$-]0`, `[$-40C]0`, `0;0;0;x@AM/PM`, `0;0;0;A/P`, `A/P` and `[$-409]` length-limit costs, and `A/P` in cell display; oracle values committed.
- [ ] The engine matches and number-formats.md drops the inferred note.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
