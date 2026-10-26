---
id: T-2803
title: "SPEC-0024 step 1: a per-formula work budget charged per node, per array cell built and per lifted kernel unit"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2712]
---

## Goal
One formula evaluation spends at most `MAX_FORMULA_WORK` work units (SPEC-0024). Past that, the formula is an uncatchable
`#VALUE!` ("Formula needs too much work to calculate"), and evaluation stops at once. The three T-2712 formulas finish within
the harness timeout, and no eval cell changes.

## Context
SPEC-0024 rollout step 1, Design sections 1–5. Read the spec first. Extend `AreaBudget` (area_budget.rs) and do not add a new
struct: add `work`, `work_exceeded`, `Model::spend_work` and `Model::work_exceeded`. Make the error uncatchable in
`with_area_budget`, like `seen_ranges_exceeded`. Charge:
- 1 per node in `evaluate_node_in_context`, with a `#[cold]` error path (frame size, T-0444);
- rows × columns where an array is built (SEQUENCE, RANDARRAY, MAKEARRAY, MAP/SCAN/BYROW/BYCOL results, `evaluate_range`,
  `broadcast_arrays` and the IF/operator broadcasts, `broadcast_text`, the lift driver's result,
  HSTACK/VSTACK/EXPAND/WRAP*/TEXTSPLIT, MMULT), at the existing `check_array_*` call sites;
- text values at the node chokepoint, ⌈UTF-8 bytes / `TEXT_BYTES_PER_UNIT`⌉ (spec section 1), and `broadcast_text`'s byte total;
- every per-call cap's spend, again, to the formula: the lift driver's `kernel_units`, the *IF(S) cell visits, and the regex
  compile and match budgets.
The LAMBDA helpers (MAP, REDUCE, SCAN, BYROW, BYCOL, MAKEARRAY, GROUPBY/PIVOTBY loops) break on `work_exceeded()`.
Calibrate per the spec's section 4: measure the heaviest eval formula's units with a local, uncommitted instrumentation run,
and stop and report if it is above 2^26. Add the DEVIATION entry from section 5 to docs/excel-semantics/arrays.md, with the
measurement filled in. Must keep probes/lambda-call-depth H3, H6 and probes/union-walk-repeated-in-lambda passing.

## Acceptance criteria
- [ ] `=REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(SEQUENCE(100000))))`,
      `=SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAP(SEQUENCE(1000),LAMBDA(y,SUM(SEQUENCE(1000))))))))` and
      `=SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,SUM(SEQUENCE(1000)))))` finish within the harness timeout (`--timeout 30`, release)
      as `#VALUE!`. Engine tests assert the value and that the units spent are within the limit plus one loop iteration
      (counters, no timing).
- [ ] Counter tests from SPEC-0024's Test plan: units of `=1+2` and `=SUM(SEQUENCE(10))`; at the limit versus one over it
      (test-only lower limit); `IFERROR(<over>,0)` is `#VALUE!`; an over-budget dependency cell caught by the outer
      `IFERROR(A1,0)` is 0; the next formula gets a full budget; past the limit a MAKEARRAY makes at most one more LAMBDA call.
- [ ] No baseline gains a line (`make eval-check`: 0 regressions); the measured maximum is in the DEVIATION entry.
- [ ] Per-kind timing (spec section 4, release build): each listed worst case (nodes, built cells, read cells, text, per-call
      caps) stops at the limit within 10 s. The PR records the timings and the chosen weights.
- [ ] `spend_work` uses `checked_sub`, and every charge is computed with saturating arithmetic. A unit test checks that a
      `u64::MAX` charge fails cleanly.
- [ ] An audit of the text and regex functions (SUBSTITUTE with many matches, TEXTSPLIT, REGEX*, FIND/SEARCH on long text)
      for work super-linear in their input within one call. Any such function gets a per-call cap charged to the formula
      (SPEC-0024 "Known gaps"). The Outcome lists what was checked.
- [ ] The criterion calc bench does not slow by more than 2% (PR notes the numbers).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Build-once, read-many shapes (the LET variant, whole-array readers, text elements of LET-bound arrays). They stay
  unbounded until T-2804 and T-2805 merge; land the three tasks in order.
- Charging array cells read by aggregates (T-2804) and by the other whole-array readers (T-2805).
- Dense single-range walks (T-2710, T-2711, T-2713).
