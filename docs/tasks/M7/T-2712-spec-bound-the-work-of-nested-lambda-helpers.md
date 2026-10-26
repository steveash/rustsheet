---
id: T-2712
title: "Spec: bound the work of nested LAMBDA helpers (three formulas under 100 characters run past the 30 s case timeout)"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ (template docs/specs/TEMPLATE.md) that bounds the CPU work one formula
evaluation can do through repeated LAMBDA calls. Short formulas must stop being able to run for hours
(directive 5), with no formula Excel computes in reasonable time losing its value. The spec is reviewed
and splits the work into S/M tasks.

## Context
Gap triage 2026-10-25 (robustness, engine-level). Each array is capped at 2^20 cells (arrays.md "Array
results above 1,048,576 cells are #ERROR!"), and LAMBDA call depth by a stack share (lambda.rs
`lambda_stack_base`, arrays.md "How deep LAMBDA calls nest"). But nothing bounds the product of
iterations and per-call work. One formula each, release build, harness `--timeout 30`, all three
`timeout` at 30 s:
- `=REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(SEQUENCE(100000))))` (1E10 element steps)
- `=SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAP(SEQUENCE(1000),LAMBDA(y,SUM(SEQUENCE(1000))))))))` (1E9)
- `=SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,SUM(SEQUENCE(1000)))))` (1E9)
`=REDUCE(0,SEQUENCE(3000),LAMBDA(a,b,a+SUM(SEQUENCE(100000))))` (3E8) takes 20 s, about 67 ns per
element step. Each extra nesting level multiplies the work by up to 2^20, so a few more characters
reach years.

Existing budgets to build on or unify, not copy (directive 3):
- SPEC-0011 "Security rule for kernel work": `KernelWork` Constant/Weighted/ByValue, `kernel_units`
  in lift/driver.rs. It covers lifted scalar kernels, not LAMBDA bodies.
- area_budget.rs `MAX_AREA_WALK` (2^24 walk steps per formula evaluation, T-1497) and
  `MAX_AREA_PAIRS`. These are per-formula budgets that every LAMBDA call of the formula draws on.
- T-2607 (lifted-quantile work charging) and T-2518 (D-function per-record re-evaluation cap) are
  local caps of the same kind.

Questions the spec must answer:
1. What is charged: LAMBDA invocations, array cells built (SEQUENCE, MAKEARRAY, arithmetic over
   arrays), cells walked, or one step unit for all, and where the counter lives (per formula
   evaluation like `MAX_AREA_WALK`, reset per cell formula, conditional format and `evaluate_formula`).
2. The limit and the result past it. `#CALC!`, `#NUM!` or `#VALUE!` with a message, chosen from what
   Excel shows, if anything, for an over-resource formula; probe with the excel-probe skill if needed.
   Also how it composes with IFERROR (an error a budget makes should not be catchable into a wrong
   value without a DEVIATION entry saying so).
3. Calibration: the limit must sit above every Excel-evidenced formula in eval (the
   union-walk-repeated-in-lambda shapes, 2E8 walked cells, run in Excel), measured in the same units.
4. The DEVIATION entry (docs/excel-semantics/README.md, decision 0014) and the test strategy:
   counter-based tests, not wall-clock limits (T-2282).

## Acceptance criteria
- [x] docs/specs/SPEC-XXXX answers questions 1-4, with the three formulas above as worked examples and
      the eval formulas that must stay computed.
- [x] Implementation tasks are filed (S/M each). Their acceptance criteria say the three formulas finish
      within the harness timeout and no baseline gains a line.
- [x] Reviewers approve the spec.

## Out of scope
- The implementation (the follow-up tasks).
- Dense single-range walks (T-2710, T-2711, T-2713), which are a cost per walk, not a repetition bound.

## Outcome
docs/specs/SPEC-0024-formula-work-budget.md answers questions 1–4:
1. **What is charged.** A work unit is one node evaluated, one array cell built, or one array cell read whole. The lifted
   kernels' own charges are drawn again. The counter is a field of the existing per-formula `AreaBudget`, reset by
   `with_area_budget`.
2. **The result past the limit.** It is `#VALUE!` ("Formula needs too much work to calculate"), set formula-wide when the run
   ends, so `IFERROR` cannot catch it. The LAMBDA helpers stop at once.
3. **Calibration.** The provisional limit is 2^28. The upper bound is the harness timeout. The lower bound is the eval gate,
   plus a measured maximum that must sit under 2^26.
4. **DEVIATION and tests.** The DEVIATION text is drafted, and the tests use counters only.

Follow-ups: T-2803 (step 1, M, opus), T-2804 (step 2, S, sonnet), T-2805 (step 3, M, opus). No eval change (docs only).
