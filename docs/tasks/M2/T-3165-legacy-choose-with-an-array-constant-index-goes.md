---
id: T-3165
title: "Legacy CHOOSE with an array-constant index: @ goes on the call, not the constant (=@CHOOSE({1,2},INDEX(B1:B6,3),5))"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A legacy CHOOSE whose index is an array constant shows the `@` on the call, as Excel does:
`=@CHOOSE({1,2},INDEX(B1:B6,3),5)` and `=ABS(@CHOOSE({1,2},INDEX(B1:B6,3),5))`, not `=CHOOSE(@{1,2},...)`.

## Context
- Known failures probes/t3134-legacy-if-index-arm-shapes I54 and K54 (excel-graph, T-3134). Values (3, 3; SUM 8)
  already pass; only FORMULATEXT differs.
- docs/excel-semantics/references.md, IF/IFS arm section (T-3134 paragraph): an array condition or index keeps
  the `@` on the call (`=@IF({TRUE,FALSE},INDEX(B1:B6,3))` already passes, I50).
- The `@` on the constant comes from the array-constant scalar-argument rule (T-1633, `add_array_at` in
  crates/rustsheet-engine/src/expressions/parser/static_analysis.rs), applied to CHOOSE's index (legacy.rs
  `index == 0` for CHOOSE). Check which other probes pin `CHOOSE(@{..},...)` before changing it.

## Acceptance criteria
- [ ] I54 and K54 pass; every other cell of probes/t3134-legacy-if-index-arm-shapes and the legacy CHOOSE probes keeps
      passing (`make eval-check`: 0 regressions, `make eval-baseline` run).
- [ ] `make check` green; reviewers approve.

## Out of scope
- IF and IFS conditions (already evidenced, T-3134).
