---
id: T-0193
title: "Union printing and parsing: keep nested union parentheses and read (-A1,B1) as a union of -A1 (FORMULATEXT)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT (and formula printing) of unions matches Excel: nested unions keep their parentheses, and a negated operand stays inside the union.

## Context
T-0187 probes (excel-graph, 2026-10-06): `FORMULATEXT` of `=((Data!A2,Data!E8),Data!B6)` is `=((Data!A2,Data!E8),Data!B6)` (probes/union-accepted H7; the engine prints `=(Data!A2,Data!E8,Data!B6)`), and of `=(-A1,B1)` it is `=(-A1,B1)` (probes/union-neg-operand H2; the engine parses `-(A1,B1)`). Excel loads `(-A1,B1)` and gives #VALUE! (H1). See docs/excel-semantics/references.md, "Union: parsing and evaluation". Code: expressions/parser/reference_operators.rs (`Parser::parse_union`), the stringify module.

## Acceptance criteria
- [ ] probes/union-accepted H7 and probes/union-neg-operand H2 pass; baseline shrinks.
- [ ] Unit tests for both round-trips.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unions as function arguments (T-0133).
