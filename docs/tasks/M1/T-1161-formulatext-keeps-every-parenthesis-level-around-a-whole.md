---
id: T-1161
title: "FORMULATEXT keeps every parenthesis level around a whole formula"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT (and the stored formula) of a cell whose whole formula is wrapped in several parentheses returns every written level, as in Excel.

## Context
Oracle (probes/near-zero-snap-groups B6, excel-graph 2026-10-08): `FORMULATEXT(B3)` with B3 = `=((0.1+0.2-0.3))` is `=((0.1+0.2-0.3))`; the engine returns `=(0.1+0.2-0.3)`. See docs/excel-semantics/numbers.md "Near-zero snapping" (`Node::GroupedRootKind`, `Parser::parse_cell_formula`).

## Acceptance criteria
- [ ] probes/near-zero-snap-groups B6 passes and leaves eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Near-zero snapping (T-1160).
