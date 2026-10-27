---
id: T-2803
title: "SPEC-0024 step 1: a per-formula work budget charged per node, per array cell built and per lifted kernel unit"
milestone: M7
lane: engine
status: blocked
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

## Blocked because
Blocked because: SPEC-0024 section 4's calibration rule — the heaviest eval formula spends more than 2^26 units, and no
weight for the per-call caps' charges meets both constraints. Resolve via T-2813 (spec amendment), then resume this branch.
T-2813 amends SPEC-0024 (branch `agent/engine/T-2813-per-call-budget-charge`): visit caps are charged 1 unit a visit or
kernel unit, everywhere (`VISIT_UNITS` stays 1, as committed here), and section 4's margin is split: the heaviest eval
formula stays within 2^26 outside the visit caps' charges and within 2^27 of them (measured 2026-10-27 on this branch with
main merged in: 8,388,620 and 118,751,450, 0 regressions); each visit cap's worst case, one call at its cap, runs within
5 s (RATE 2.1 s; YIELD and ODDFYIELD about 15-17 s are T-2879's, not this task's). Once T-2813 is `done` on main, unblock
this task on this branch (agent-workflow "Unblocking is an agent's job"), drop the `TODO(T-2813)` in area_budget.rs, update
the arrays.md DEVIATION entry's calibration to the amended rules, and continue at review.

What is on the branch (implementation of Design sections 1-5; fmt, clippy, engine tests, hygiene and eval check pass locally; no review yet):
- `AreaBudget` gains `work`/`work_exceeded`; `Model::spend_work` (`checked_sub`, `#[cold]` error), `spend_value`,
  `spend_cells`, `spend_visits`, `work_exceeded`; `with_area_budget` makes the formula `#VALUE!` (`TOO_MUCH_WORK`)
  whenever the budget was exceeded, so `IFERROR` cannot catch it; test-only `formula_work_limit`/`formula_work_spent`.
- Charges: `NODE_UNITS` (2) per node in `evaluate_node_in_context`, per operator-chain link and for the root `+`; a node's
  text value (1 unit per 4 bytes) and array value (`CELL_UNITS`, 2 per cell) at the same chokepoint (one definition
  instead of a charge at each `check_array_*` site, which also covers every array builder); `evaluate_range` per cell;
  the lift driver's up-front visits and output charges, `lift_text_call`, `lift_references`, ref_lift, GROUPBY/PIVOTBY
  visits (`VISIT_UNITS`, 1); lifted text results' bytes; REGEX* and regex XLOOKUP/XMATCH compile and search work.
- `call_prepared` refuses every LAMBDA call once the budget is exceeded, so the helpers' loops do no more work
  (counter test: a MAKEARRAY makes fewer than 1,000 cell reads at a 1,000-unit limit).
- Tests: src/test/test_formula_work.rs (units per node/cell/text and chain link, at-limit vs over, uncatchable,
  dependency budget, early stop, `u64::MAX` charge). The full engine suite passes at the committed weights;
  `rustsheet-eval check` has 0 regressions at them.
- DEVIATION entry in arrays.md with the measurements.

Measurements (release build, local instrumentation of `with_area_budget`, not committed):
- Eval (`rustsheet-eval check`): VISIT_UNITS 6 → 13 gen-m9 regressions; VISIT_UNITS 1 → 0 regressions, max 124,806,376
  units (gen-m9/0a614cfe974b59c3 Calc!A117; 984cba67bbe6972b Calc!A14 124,783,848; both one lifted call near the 2^27
  per-call cap); VISIT_UNITS 0 → 0 regressions, max 8,388,620.
- Time to the limit at the committed weights: SPEC-0024's three formulas 7.0 / 7.0 / 5.9 s; REDUCE 3000 9.5 s; nested
  MAKEARRAY of scalar bodies 6.2 s; text kinds 0.6-4.2 s; REGEXTEST 0.1 s; MAKEARRAY of lifted RATE about 50 s at
  VISIT_UNITS 1 (8.5 s at 6), because one RATE element is about 190 ns per kernel unit (T-2814).
- Text audit: SUBSTITUTE (`str::replace`, KMP on the UTF-16 path), FIND/SEARCH (greedy wildcard matcher, linear), TEXTSPLIT
  and REGEX* (per-call search budget) are linear in their input within one call; none needs a new per-call cap.
- Overhead: the repo has no criterion calc bench. Against an origin/main release build, alternating runs of
  `=SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z)))))` took 4.38/4.55/3.70 s on main and
  4.06/4.55/3.74 s on the branch (noise; the test suite ran alongside), and a MAKEARRAY of an IF body 0.20-0.21 s on both.
- Not done: review, PR checks.
