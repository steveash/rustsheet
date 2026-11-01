---
id: T-2803
title: "SPEC-0024 step 1: a per-formula work budget charged per node, per array cell built and per lifted kernel unit"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2712, T-2923]
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
- [x] `=REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(SEQUENCE(100000))))`,
      `=SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAP(SEQUENCE(1000),LAMBDA(y,SUM(SEQUENCE(1000))))))))` and
      `=SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,SUM(SEQUENCE(1000)))))` finish within the harness timeout (`--timeout 30`, release)
      as `#VALUE!`. Engine tests assert the value and that the units spent are within the limit plus one loop iteration
      (counters, no timing).
- [x] Counter tests from SPEC-0024's Test plan: units of `=1+2` and `=SUM(SEQUENCE(10))`; at the limit versus one over it
      (test-only lower limit); `IFERROR(<over>,0)` is `#VALUE!`; an over-budget dependency cell caught by the outer
      `IFERROR(A1,0)` is 0; the next formula gets a full budget; past the limit a MAKEARRAY makes at most one more LAMBDA call.
- [x] No baseline gains a line (`make eval-check`: 0 regressions); the measured maximum is in the DEVIATION entry.
- [x] Per-kind timing (spec section 4, release build): each listed worst case (nodes, built cells, read cells, text, per-call
      caps) stops at the limit within 10 s. The PR records the timings and the chosen weights. The visit caps are judged
      by section 4 as amended by T-2813 (one call at its cap within 5 s, timed for each kernel family near the cap in eval);
      YIELD and ODDFYIELD are T-2879's and named in the DEVIATION entry until it merges.
- [x] `spend_work` uses `checked_sub`, and every charge is computed with saturating arithmetic. A unit test checks that a
      `u64::MAX` charge fails cleanly.
- [x] An audit of the text and regex functions (SUBSTITUTE with many matches, TEXTSPLIT, REGEX*, FIND/SEARCH on long text)
      for work super-linear in their input within one call. Any such function gets a per-call cap charged to the formula
      (SPEC-0024 "Known gaps"). The Outcome lists what was checked.
- [x] The criterion calc bench does not slow by more than 2% (PR notes the numbers).
- [x] `make check` green; reviewers approve.

## Out of scope
- Build-once, read-many shapes (the LET variant, whole-array readers, text elements of LET-bound arrays). They stay
  unbounded until T-2804 and T-2805 merge; land the three tasks in order.
- Charging array cells read by aggregates (T-2804) and by the other whole-array readers (T-2805).
- Dense single-range walks (T-2710, T-2711, T-2713).

## Outcome
Merged 2026-10-27 (three runs; blocked twice on SPEC-0024 calibration, unblocked by the T-2813 and T-2923 amendments).
- `AreaBudget` gains `work`/`work_exceeded`; `Model::spend_work` (`checked_sub`, `#[cold]` error), `spend_node`,
  `spend_value`, `spend_cells`, `spend_visits`, `spend_lambda_made`, `work_exceeded`. `with_area_budget` makes the formula
  `#VALUE!` (`TOO_MUCH_WORK`) whenever the budget ran out, so `IFERROR` cannot catch it; the test-only lower limit and the
  units spent are `AreaBudget.limit`/`last_spent` (`#[cfg(test)]`, no public surface; SPEC-0024 section 2 names them
  `formula_work_limit`/`formula_work_spent`).
- Charges and fitted weights: `NODE_UNITS` 6 per node (each operator-chain link one), `CELL_UNITS` 3 per array cell, charged
  once at the node chokepoint for every array a node gives (instead of at each `check_array_*` site; it covers every
  builder whose array surfaces as a node value) and in `evaluate_range`; text 1 unit per 4 bytes (SPEC-0024's 64 was
  provisional); `LAMBDA_UNITS` 512 per LAMBDA stored plus `NODE_UNITS` per node of each body clone (`LambdaDefKind`,
  `fn_lambda`, `prepare_lambda`; T-2923); visit caps 1 unit a visit (T-2813); REGEX* and regex XLOOKUP/XMATCH compile and
  search work, and the whole two budgets for a call that ran out of them (found in review). `call_prepared` refuses every
  LAMBDA call once the budget ran out.
- Calibration (release, idle 4-core container, local uncommitted instrumentation): `rustsheet-eval check` 0 regressions,
  no formula past the limit; heaviest 118,756,588 units (118,751,450 visit units, below 2^27); heaviest outside the visit
  caps 12,582,948 (below 2^26). Every timed kind stops within 0.8-7.9 s (table in the arrays.md DEVIATION entry); the T-2923
  nested worst case 2.9 s (124 s before); a MAP of REGEXTEST calls that each exhaust their budget 0.7-2.3 s.
- Text audit: SUBSTITUTE (`str::replace`, KMP on the UTF-16 path), FIND/SEARCH (greedy wildcard matcher), TEXTSPLIT and
  REGEX* (per-call search budget) are linear in their input within one call; none needs a new per-call cap.
- Bench: the repo has no criterion calc bench; alternating hand timings of a MAKEARRAY of MAP against origin/main were
  within noise (4.06-4.55 s vs 3.70-4.55 s).
- Tests: src/test/test_formula_work.rs (units per node, cell, text, chain link and LAMBDA made; at the limit vs one over;
  uncatchable; dependency budget; early stop; `u64::MAX` charge; the Problem formulas and the T-2923 one at a lowered
  limit; an exhausted regex call). Eval delta: none (0 fixed, 0 regressions).
