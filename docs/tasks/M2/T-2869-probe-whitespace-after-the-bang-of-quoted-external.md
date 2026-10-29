---
id: T-2869
title: "Probe whitespace after the bang of quoted, external and R1C1 sheet prefixes"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whitespace after the `!` of quoted (`='My S'! B1`), external (`[1]S! A1`) and R1C1 sheet prefixes, with a line feed after the `!`; adjust the lexer if Excel differs.

## Context
T-2785 skips whitespace after every sheet prefix's `!` (lexer `consume_range`) but only the unquoted `=S! B1` form has oracle evidence (probes/t2776-ws-after-bang). Reviewer finding, excel-fidelity.

## Acceptance criteria
- [ ] Probe cases with oracle values for each form above; engine matches; syntax.md entry updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
