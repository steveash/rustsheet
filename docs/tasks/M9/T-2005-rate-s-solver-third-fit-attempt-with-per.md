---
id: T-2005
title: "RATE's solver: third fit attempt with per-iterate traces, the nested RATE(n,PMT(..)) cells and Excel's #NUM! failure rule"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
RATE returns Excel's bits on more cells than the engine's Newton solver does (12 of the lab's 136 literal cells
today), using a fitted start, update and stop rule. It converges where Excel converges (guess 0) and is `#NUM!` where
Excel gives up. Or, if this third attempt also fails to raise the count, the task records the candidates tried and is
time-boxed under decision 0056.

## Context
Gap triage 2026-10-19. RATE has more failing cells than any function without an open task: 435 of 517 checked. The
solver was split out of T-0916 (done), and no open task owns RATE now.
- `probes-numeric/m9-fin-irr-rate` (excel-graph, T-1009; not gated): 321 failing RATE cells. 274 are close (e.g. Calc!A217
  `RATE(1,PMT(0.2,1,10000),10000)` expected 0.19999999999999993, ours 0.20000000000000018), 28 ulp and 3 mismatch.
  16 have the wrong type: 3 are `#NUM!` in Excel where we converge (e.g. A272
  `RATE(360,PMT(0.1,360,10000,-500,1),10000,-500,1)`), and 13 converge in Excel where we give `#NUM!`.
- `probes/solver-iteration-traces` sheet R (excel-graph, T-1525): 77 baseline lines, e.g. R!B10 `RATE(10,0,-1,2)`
  expected 0.07177346253644193, ours 0.07177346253629326.
- `probes/rate-empty-arguments` (9 baseline lines): A1, A3 and A8 have guess 0 (empty, `0` or an empty cell). Excel
  returns 0.008052981923885287; ours is `#NUM!` because Newton's f(r) divides by r. A2/A4 (guess 0.1) and A9/A10
  (guess 1) differ in the 12th digit, so the stopping point depends on the start (financial.md "RATE: an empty guess is
  guess 0").
- `ironcalc-calc/FINANCIAL/RATE.xlsx` (desktop cache): 26 baseline lines, e.g. Sheet1!F2 expected 0.008052981923906556,
  ours 0.008052981923938503 (the same value as rate-empty-arguments A2, so desktop and web agree).
- Single cells: probes/fin-argument-modes H282, gen-m9/eefcc3989f31de76 A5.

What was tried (financial.md "RATE iterates on the rate itself"): Newton in r, at most 11/136 bit-exact. Finite-difference
Newton gave 7/136. IRR's secant in 12 variants (T-1533) gave at most 19/136 bit-exact and 40 within 4 ulp. On the guess
sweep R!C1:C31, Excel's error is not smooth in the guess and grows about as (guess - root)^26.

New hypothesis material that no attempt has used yet (the reason for this third attempt, decision 0056):
1. T-1534's lab tooling, now done: `Candidate.trace` and `Score.iterate_hits`. With these, candidates can be scored on
   whether Excel's value is any iterate of the candidate, not only its final value. T-1533 named this as the next step.
2. The 277 nested `RATE(n,PMT(r,...),...)` cells of m9-fin-irr-rate, which the lab does not parse. Each has a known
   root r, so the stopping distance can be measured exactly. The 31 ironcalc ODDFYIELD cells (T-2007) have the same
   property.
3. The guess-0 convergence and the 16 cells where Excel and the engine disagree on `#NUM!`, which constrain how the start and the failure rule work. Excel's
   iteration variable may not be r itself (for example v = 1/(1+r), or the payment form multiplied out to remove 1/r).
Lab: `uv run rustsheet-fitlab rate` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py `rate_f`, `_rate`).
Engine: `compute_rate` (functions/financial/financial_util.rs), called from `fn_rate` (functions/financial/annuity.rs).
RATE's f(r) evaluates the annuity (1+r)^n, so it depends on kernel-map rank 1 (T-1528, time-boxed). If a candidate
matches the iterates but misses the final bits only through (1+r)^n, record that and stop there.

## Acceptance criteria
- [ ] The lab parses the nested `RATE(n,PMT(..),..)` cells and scores every candidate on all RATE cells of
      m9-fin-irr-rate, solver-iteration-traces R and rate-empty-arguments, with iterate hits. Each candidate's
      bit-exact and within-4-ulp counts go into financial.md "RATE iterates on the rate itself", rejected ones included.
- [ ] Either: a model that raises the bit-exact count above 19/136 (and above the engine's count on the full set) is
      implemented in `compute_rate`. It converges for guess 0, agrees with Excel on which m9-fin-irr-rate cells are `#NUM!` as far as
      the model explains them, and RATE baseline lines in probes.txt and ironcalc-calc.txt only shrink, with no new line in any
      baseline. Or: the task is set `blocked` with a "Blocked because: time-boxed (decision 0056)" section listing
      the candidates.
- [ ] Iteration stays bounded (directive 5); any cap that differs from Excel's gets a DEVIATION entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The annuity evaluation order of PMT/IPMT/PPMT/NPV (T-1528) and IRR's remaining cells, which need NPV's order.
- YIELD (T-1592) and ODDFYIELD (T-2007), though a shared stop rule found here should be written down for both.
