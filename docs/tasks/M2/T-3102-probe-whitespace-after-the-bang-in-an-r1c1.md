---
id: T-3102
title: "Probe whitespace after the bang in an R1C1-mode formula (formulasR1C1) and what we do with S! R1C2 typed in A1 mode"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whitespace after `!` in a formula entered in R1C1 mode (`S! R1C2`, via `formulasR1C1`), and what RustSheet does with `=S! R1C2` typed in A1 mode (Excel refuses it, 400 InvalidArgument); adjust the lexer if it differs.

## Context
T-2869 probed the A1-mode forms only (docs/excel-semantics/syntax.md, "drop after `!`"). Fidelity-review follow-up.

## Acceptance criteria
- [ ] Probe case(s) with oracle values; engine matches; syntax.md entry updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
