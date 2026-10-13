---
id: T-1474
title: "Implement the shared aggregate-argument walker and move SUM and the AVERAGE family onto it"
milestone: M2
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1473]
---

## Goal
The walker that T-1473's spec defines exists once. SUM and every current `for_each_value*` consumer (AVERAGE
family, GEOMEAN, VAR/STDEV, DEVSQ, MEDIAN and the rest) use it, with no change in any result.

## Context
See T-1473 for the four duplicated argument dispatches and the rules they share. This task replaces
`Model::for_each_value`, `for_each_value_skip_empty` and `for_each_value_a` (functions/statistical/count_and_average.rs)
and the dispatch loop in `Model::sum_with_last_addition` (functions/math_and_trigonometry/sum.rs) with the
spec's walker. SUM's last-addition tracking for near-zero snapping stays SUM's own (docs/excel-semantics/numbers.md
"Near-zero snapping").

## Acceptance criteria
- [ ] One walker, as the spec specifies. `rg "let mut reference_error"` finds it only in the walker, product.rs and mathematical.rs (those two are T-1476's).
- [ ] `make eval` shows 0 regressions and 0 fixes in every suite (a pure refactor). probes/aggregate-error-precedence, probes/sum-error-precedence and probes/average-value-args still pass.
- [ ] scripts/check_duplication.py does not rise; record any drop with `--update` per decision 0010.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MIN, MAX, PRODUCT, COUNT, COUNTA (T-1476).
- Settling T-0236 (empty arguments of DEVSQ/SUMSQ); keep today's behavior.
