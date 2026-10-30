---
id: T-3229
title: "Probe the 65-call nesting cap for names defined through the API and for a name's body counted in the formula that uses it"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for two unprobed corners of the 65-call cap on names, and the engine matching it.

## Context
- T-3143 (docs/excel-semantics/syntax.md "Formula nesting depth cap") found a stored name body of 66 nested calls
  refuses the workbook and 65 loads (probes/t3143-name-body-65); the importer checks bodies in
  `reparse_formula_hack` (rustsheet-xlsx import/mod.rs).
- Unprobed: (1) whether a cell formula's calls plus those of a name it uses add up (`N` = 40 calls, `=ABS^30(N)`
  in a cell); (2) a name defined through the API (Graph `names/add` with a 66-call body) and the engine's
  name-definition API for the same body.
- Probe at the boundary on both sides; refused inputs are dropped after the run (T-1091, T-3097).

## Acceptance criteria
- [ ] excel-graph probes (milestone M8) for both questions; syntax.md records the result.
- [ ] Where Excel refuses what the engine accepts (or the reverse), the engine matches, with tests at the boundary.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional formats and data validation (T-3225).
