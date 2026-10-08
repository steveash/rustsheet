---
id: T-1184
title: "Union argument to VARA, VARPA, STDEVA, STDEVPA (accept) and MODE, MODE.SNGL (#VALUE!)"
milestone: M1
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
VARA, VARPA, STDEVA and STDEVPA take a union argument and MODE / MODE.SNGL give `#VALUE!` for one, as Excel does.

## Context
probes/multiarea-foreach (excel-graph, T-1150), cells H13, H14, H8, H9 (engine gives `#VALUE!`, Excel 11.6667, 8.75, 3.41565, 2.95804 over `(A1,B2,C3,D4)`) and H17, H18 (engine `#N/A`, Excel `#VALUE!`). See docs/excel-semantics/references.md "Multi-area references in functions". Find the callers with `rg -n "fn_vara|fn_varpa|fn_stdeva|fn_stdevpa|fn_mode" crates/rustsheet-engine/src`; reuse `Model::evaluate_reference_arg`.

## Acceptance criteria
- [ ] Cells H8, H9, H13, H14, H17, H18 of probes/multiarea-foreach pass bit-exact and move into the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Aggregates that already pass over a union (HARMEAN, AVEDEV, SKEW, KURT, MINA, STDEV, STDEV.P, VAR, VAR.P and the rest of probes/multiarea-foreach).
