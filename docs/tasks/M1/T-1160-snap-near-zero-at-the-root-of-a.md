---
id: T-1160
title: "Snap near-zero at the root of a defined-name body (nz, nz+0, SUM(nz))"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A defined name whose body ends in a near-zero `+`/`-` evaluates to exactly 0 wherever it is used, as in Excel.

## Context
T-1137 oracle evidence (probes/near-zero-snap-groups, excel-graph 2026-10-08): with `nz` = `=0.1+0.2-0.3` and `nzb` = `=Sheet1!$A$1-0.3` (A1 = `=0.1+0.2`), `=nz`, `=nzb`, `=nz+0`, `=nz*1` and `=SUM(nz)` all give 0, so the name body is snapped at its own root. The engine gives 5.551115123125783e-17 for all five (cells C1-C5, known failures in eval/baseline/probes.txt). Name bodies are parsed with `Parser::parse`, not `Parser::parse_cell_formula`, and never reach `Model::evaluate_formula_root`'s snapping. See docs/excel-semantics/numbers.md "Near-zero snapping". Keep the single-level grouped-root rule (a body `=(X-b)` is unprobed: file a probe if needed).

## Acceptance criteria
- [x] probes/near-zero-snap-groups C1-C5 pass and leave eval/baseline/probes.txt.
- [x] Cell formulas `=(0.1+0.2-0.3)+0` etc. (B7, B8) still pass unsnapped.
- [x] `make check` green; reviewers approve.

## Out of scope
- FORMULATEXT keeping every parenthesis level (B6; known gap in references.md).

## Outcome
`evaluate_defined_name_formula` now evaluates the body with `evaluate_formula_root`, so a name body is snapped at its own root. probes/near-zero-snap-groups C1-C5 pass (probes known failures 171 -> 166); B7/B8 unchanged. A grouped body `=(X-b)` stays unprobed.
