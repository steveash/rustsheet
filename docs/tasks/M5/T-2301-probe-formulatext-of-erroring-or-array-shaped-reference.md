---
id: T-2301
title: "Probe FORMULATEXT of erroring or array-shaped reference-returning calls"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's FORMULATEXT result is known for the untested reference-returning forms, with oracle values in a probe case.

## Context
T-2137 made FORMULATEXT(INDEX(..)) return INDEX's error or `#VALUE!` for a picked value (probes/index-reference-args A4, A5). Unverified: `INDEX(A1:A3,NA())`, `INDEX({1,2,3},9)`, `INDEX({1,2;3,4},0,1)`, and erroring OFFSET, INDIRECT, CHOOSE and IF arguments (engine gives `#ERROR!`). See docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] A probe case holds each form above with Excel values (excel-probe skill), and references.md records the results.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe case probes/formulatext-erroring-references holds every form with excel-graph values (2026-10-23); results are in docs/excel-semantics/references.md. The engine fails 22 of its cells; fix filed as T-2523.
