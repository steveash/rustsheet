---
id: T-3695
title: "Probe nested element errors in Excel and move the element-error tag to CalcResult (T-3080 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3080]
---

## Goal
Excel evidence for the nested element-error forms T-3080 extrapolated, and the element-error tag owned by `CalcResult`
instead of a statistical module.

## Context
- T-3080 (2026-11-01) drops the element-error tag (`functions/statistical/element_errors.rs`, the message "Error in
  array argument") where a function call's result leaves `Model::evaluate_node_in_context`. Its only oracle cell is
  gen-m9/1b31092a0d79e5f1 A363, `SMALL((SMALL({1,#N/A,3},2)/1.005),"a*")` = `#N/A`. These unit expectations are
  extrapolated, not probed: `SMALL(SMALL({1,#N/A,3},2),"x")`, `SMALL(LARGE({1,#N/A,3},2),"x")` (both `#N/A`).
- The other producers of the tag change the same way when nested, consistently with statistics.md but unevidenced:
  SUMX2MY2 and the paired sums (sum.rs), MODE (mode_functions.rs), CHISQ.TEST (chisq.rs), PROB (prob.rs), e.g.
  `SMALL(SUMX2MY2({1,#N/A},{1,2}),"x")` went from `#VALUE!` to `#N/A`. Also a LET or LAMBDA wrapping a tagged result
  (`LET(x,SMALL({1,#N/A,3},2),SMALL(x,"x"))`).
- T-3080's architecture review: the tag is a property of `CalcResult::Error`, so the constant, `array_element_error` and
  `is_array_element_error` (plus a stripping method) belong in calc_result.rs, which `model.rs` then calls without
  importing from `functions::statistical`. A flag or variant instead of a message string would remove the sentinel.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with the forms above, filled from the oracle; the engine matches, or the
      differences are filed.
- [ ] The tag lives in calc_result.rs; `element_errors` re-exports or is removed; no behaviour change (eval unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Array values with error elements from nested calls (still tagged by the outer reader, as intended).
