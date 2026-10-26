---
id: T-2756
title: "ROUNDUP of a number at the smallest normal is 0 (probes/t2715-roundup-near-smallest-normal)"
milestone: M2
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ROUNDUP of a positive or negative number just above the smallest normal double gives Excel's value. Excel for the web
gives 0 where the engine gives one step, and the threshold is fitted from oracle evidence.

## Context
Found by T-2715 (its ROUNDUP/ROUNDDOWN controls). probes/t2715-roundup-near-smallest-normal (excel-graph,
2026-10-25): ROUNDUP(±2^-1022, d) is 0 for d in 0, -1, -2, -10, -300 (C11-C20, the case's 10 known failures). But
ROUNDUP(±1E-307, d) is one step (1, 10, 100, 1E10, 1E300), as the engine already gives, and so is ROUNDUP(±1E-300, d).
ROUNDDOWN is 0 throughout. The calipers golden's `ROUNDUP(10^-307,-2)` is 100 (formulas_math_basic C143).

So ROUNDUP has a threshold somewhere in (2.2250738585072014E-308, 1E-307]. It is not the subnormal flush of the
CEILING family (numbers.md "CEILING and FLOOR of a subnormal ratio are 0"): there, a normal number divided by a
significance of 1 keeps its step. Bisect the threshold with a probe (for example 2^-1022·(1+k·2^-52), 2^-1021, 2^-1020,
1E-307·(1-…)), check whether it depends on num_digits (positive digits too, such as 2 and 300), and look for a scaling
mechanism (`x * 10^d` underflow, a 15-digit cut of a value below some magnitude). Code: ROUNDUP in
crates/rustsheet-engine/src/functions/math_and_trigonometry/ (search `fn_roundup`).

## Acceptance criteria
- [x] A probe case tagged M2 (excel-graph) bisects the threshold and covers positive and negative num_digits.
- [x] The engine matches that probe and probes/t2715-roundup-near-smallest-normal (its 10 lines leave eval/baseline/probes.txt),
      except the 7 read-back cells A52-A58 of the new probe (3 ulps; T-2771).
- [x] numbers.md gets an entry with the fitted rule, the evidence and `Status: VERIFIED`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The CEILING/FLOOR family (T-2715, done).

## Outcome
- New probe probes/t2756-roundup-smallest-normal-threshold (541 cells, excel-graph, with `=(...)=0` companions,
  decision 0042). The threshold is the 15-digit cut, rounded to nearest: for x = 2^-1022·(1+k·2^-52), k ≤ 7 (cut
  2.22507385850720E-308, subnormal) ROUNDUP, ROUNDDOWN and TRUNC return that subnormal cut at every num_digits; from
  k = 8 (cut 2.22507385850721E-308) ROUNDUP takes its step. ROUND's result, ROUNDDOWN's result and ROUNDUP's rounded-down
  part and unit are each flushed when subnormal (so `ROUNDUP(x,308)` is 0 for k = 8).
- `round_function` (new, ROUND family entry point) and the flushes in `round_decimal`, crates/rustsheet-engine/src/decimal_round.rs.
- probes/t2715-roundup-near-smallest-normal gets E/F companions (refilled): its 10 known failures are fixed.
- numbers.md "ROUNDUP and ROUNDDOWN of a number whose 15-digit cut is subnormal".
- Eval delta: probes known failures 2599 → 2596 (fixed 10; new coverage 7: A52-A58 read the subnormal back through
  `*2^100`, and Excel's is 3 ulps below the nearest double the engine returns). Regressions 0.
- Follow-up: T-2771 (the conversion behind those 3 ulps; time-boxed here).
