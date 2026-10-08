---
id: T-1161
title: "FORMULATEXT keeps every parenthesis level around a whole formula"
milestone: M1
lane: engine
status: done
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
- [x] probes/near-zero-snap-groups B6 passes and leaves eval/baseline/probes.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
- Near-zero snapping (T-1160).

## Outcome
The parser counts the levels of the group that wraps a whole cell formula (`root_group`) and `parse_cell_formula` nests one `GroupedRootKind` per level; stringify already printed each. probes/near-zero-snap-groups B6 passes (probes known failures 166 -> 165). SUBTOTAL's nested-formula check now unwraps every level.
