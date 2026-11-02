---
id: T-3411
title: "IRR on conventional flows: model what Excel does after its secant fails (Newton from v = 1 and the T-3406 candidates are refuted)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-3406]
---

## Goal
IRR returns Excel's type (number, `#NUM!` or `#DIV/0!`) on conventional cash flows where the fitted secant fails,
from a model of what Excel does there that holds on a fresh sweep. It replaces or reworks T-3347's `irr_conventional`
stage (PR #2529, blocked on its hold-out) and unblocks T-3347.

## Context
- Evidence: probes/t3406-irr-num-boundary (144 cells; financial.md "IRR on conventional flows where the secant fails:
  which return is path-sensitive"), plus T-3347's probe, sweep and hold-out (on its branch). The engine on main fails
  76 of the 144: level annuities `IRR({-P,1,...,1})` from r = -0.10 (n = 20), -0.04 (n = 30) and -0.02 (n = 40, 60)
  are `#NUM!` where Excel returns the root. That is an ordinary input, visibly wrong.
- Refuted (T-3406, type agreement on 450 cells): Newton on P(v) from v = 1 under any cap; the secant from 0.1 under
  a 50, 100 or 200-step cap (295, 301, 306); the secant carried past a flat step by a Newton step (304, 311, 317); a
  retry from any of 17 fixed guesses. The second family of the probe (an outlay scaled by 0.97 to 1.03) alternates
  between a number and `#NUM!`, so the decision depends on the exact path of an iteration: a model has to reproduce
  Excel's arithmetic step by step, as the secant already does with the engine's twice-rounded operators.
- Ideas not yet tried: run the T-3406 candidates through the engine's own `irr_npv` (extended operators) rather than
  a double replica, since a flat step in double may not be flat there; a secant in r rather than d after the first
  failure; a bisection or bracket walk like XIRR's (financial.md "XIRR is a bracketing bisection"); Excel's NPV
  overflow surfacing as `#DIV/0!` (T-3407) as a sign of which iteration runs.
- Type cells of T-3347's hold-out (Excel returns the root, the engine is `#NUM!`; gap triage 2026-11-01, `make eval` on
  origin/main 408444f20):
- probes-numeric/t3347-irr-conventional-holdout.case.json (30): P!A1, A3, A7, A13, A18, A21, A23, A39, A44, A45, A46, A47, A49, A50, A53, A54, A56, A57, A61, A66, A67, A69, A71, A90, A95, A103, A105, A107, A110, A119
- Decision 0188 R4: state the hold-out split and margin before scoring; fill a fresh sweep after the model is frozen.

## Acceptance criteria
- [ ] A model that gives Excel's type on every cell of probes/t3406-irr-num-boundary and T-3347's three cases
      (T-3407's `#DIV/0!` cells may stay `#NUM!` if T-3407 has not landed), with the R4 hold-out and a fresh
      probes-numeric sweep scored per stratum: 0 type mismatches, Gate A envelope.
- [ ] The engine implements it (in T-3347's PR, or a new one replacing it), with the rule in
      docs/excel-semantics/financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits (T-3408) and `#DIV/0!` (T-3407).

## R4 plan (fixed 2026-11-02 before the fresh sweep is generated or scored)
- Frozen model (commit "T-3411: freeze"): `irr_secant` runs up to `IRR_SECANT_STEPS` = 220 steps (was 50) and a
  stalled run (a zero step, or a flat secant after a step under 1e-7) returns its iterate when |NPV| < 1e-4 absolute
  (`IRR_STALL_RESIDUAL`; was 1e-7 of the largest flow, zero steps only). Both constants are bracketed by named
  cells, not chosen by maximizing a score: cap 213..=228 (t3347 holdout A90 stops exactly at step 213; t3478 r4-wide
  A240 would stop at 229 on digits Excel does not return), residual 8.4E-5..2.9E-4 (t3406 A123 returns; r4-wide A52
  is `#NUM!`).
- No hold-out split was fixed before the bracketing: every IRR case in eval was used. The fresh sweep below is the
  out-of-sample test.
- Fresh sweep: probes-numeric/t3411-irr-r4-fresh, 150 cells from docs/research/t3411/gen_sweep.py (seed
  "T-3411/r4-fresh"), generated after the freeze, oracle-filled from excel-graph, 50-digit roots in its math.tsv.
- Margin, stated now: on the fresh sweep the frozen model must (1) agree with Excel on the type (number vs error) on
  at least as many cells as main's model (cap 50) and (2) have a bit-exact rate within 10 points of its rate on
  t3478-irr-r4-wide (257/300 = 85.7%), the sweep that is generated the same way. Otherwise the model is not shipped.

## R4 round 1 result (fresh sweep t3411-irr-r4-fresh, scored once on the frozen model)
- Main's model 80/150, frozen model 93/150 (15 fixed, 2 new digit misses). Criterion (1) passes (type mismatches
  56 -> 44); criterion (2) fails: 62% is not within 10 points of 85.7%. Per the plan, the round-1 model is not shipped.
- The sweep refutes cap 220: A55's run from guess -0.99235 would stop at its 215th step on digits Excel does not
  return (Excel's are the retry's), so Excel's cap is 213 or 214.
- It refutes the stall rule as Excel's mechanism: A129 goes flat at |NPV| 2.3E-5 and Excel's digits are those of
  `irr_fallback` (Newton on the NPV polynomial from v = 0), not of the flat iterate. That fallback, uncapped, gives
  Excel's exact digits on 15 of the 22 stalled cells (up to 179 steps), but no fallback cap separates Excel's
  returns from its errors (returns and `#NUM!` interleave from 0 to 680 steps over the 285 cells where both
  secant runs fail).

## R4 plan, round 2 (fixed 2026-11-02 before the second sweep is generated or scored)
- Frozen model: main's IRR with the secant cap 50 -> `IRR_SECANT_STEPS` = 213 and nothing else (no stall rule).
- Fresh sweep 2: probes-numeric/t3411-irr-r4-fresh-2, 150 cells, gen_sweep.py with seed "T-3411/r4-fresh-2".
- Margin: on sweep 2 the frozen model (1) gets no cell wrong that main's model gets right (type or digits) and
  (2) has a pass rate within 10 points of its own pass rate on t3411-irr-r4-fresh (sweeps generated the same way).
  Otherwise nothing ships and the task is blocked with these notes.

## Landing notes
2026-11-01 23:40, opus run: resumed from these notes (step 0.2; branch idle 2 h).

2026-11-01, opus run (stopped at the session's landing window; no PR opened, no code changed).
- Branch head: the claim commit plus these notes. Nothing is implemented; CI and reviewers have not run.
- Scored on the type (number, `#NUM!`, `#DIV/0!` read as an error) of 288 cells: probes/t3406-irr-num-boundary (144)
  and probes-numeric/t3347-irr-conventional-holdout (144). T-3347's probe and sweep cases are on T-3347's branch, not
  main, so they were not scored. The scratch scorer was a plain-double replica of `irr_secant` plus candidate fallbacks,
  not the engine's twice-rounded operators.
  - Plain-double secant from 0.1 (retry 0.1), no fallback: 193/288.
  - Plus Newton in r from the guess (analytic derivative, |dr| < 1e-7 stop): 212 / 224 / 225 at 20 / 50 / 100 steps.
    It does not rescue the level annuities (t3406 A5-A10, root -0.10 to -0.20, n = 20): from r = 0.1 the NPV slope is
    small, so the first step lands below r = -1. It also returns 7 cells Excel is `#NUM!` on (t3406 A99-A106, n = 60,
    outlay about 7E+10). Refuted as the fallback.
  - Plus the secant in r (start 0.1 and 0.101): no change (193). Refuted.
  - The engine's own XIRR bracket walk (financial.md "XIRR is a bracketing bisection"), run as
    `XIRR(flows, 45000 + 365k, guess)` so the exponents are exactly k: it returns a number on all 288 cells. That covers
    every one of the 84 cells where Excel returns and engine IRR is `#NUM!`, but it also returns on 80 `#NUM!` and 31
    `#DIV/0!` cells (e.g. t3406 A12-A15, roots -0.24 to -0.30, n = 20; A66-A69, `#DIV/0!` in Excel). An unbounded
    bracket walk is refuted. Excel's pattern (returns down to a root of about -0.22 at n = 20, `#NUM!` below, `#DIV/0!`
    for larger outlays at n = 40 and 60) points to a walk or search whose reach in r is bounded and which, for long
    series, tries a rate whose discount leaves the double range (NPV's `#DIV/0!`, T-3407).
- Next step: model the bound. Find the most negative root Excel returns per n (t3406 gives -0.22 / -0.18 / -0.14 /
  -0.12 for n = 20 / 30 / 40 / 60) and test whether it is a fixed trial-rate set (for example rates stepped from the guess
  by a factor, as XIRR's trial chain does) whose NPV signs bracket the root, scoring with the engine's extended operators
  (`irr_npv`), not plain doubles.
- The level-annuity cutoffs in outlay P (t3406, flows `{-P,1,...,1}`): Excel returns up to P = 649.62 and is `#NUM!`
  from 1003.96 at n = 20; 2133.94 / 4033.97 at n = 30; 2970.92 / 6672.71 at n = 40; 17850.72 / 60801.44 at n = 60.
  Neither P/n nor the NPV at the guess is constant across n at the cutoff, and v^n at the last returned root is 144,
  391, 416 and 2140, so a cap on the discount power is not it either. `#DIV/0!` starts at P = 1.57E+8 for n = 40 and
  8.2E+5 for n = 60 (A83 at 1.35E+7 is `#NUM!` again), never for n = 20 or 30. That fits a trial rate near -1 whose
  n-th discount power leaves the double range (1 + r below 10^(-308/n): 2E-8 at n = 40, 7.6E-6 at n = 60, out of reach
  at n = 20). Untested hypothesis: after the secant fails, Excel's search moves r toward -1, and the outlay decides how far.
- Excel's returned level-annuity roots are 1E-18 to 9E-12 from the exact root (roots.tsv), and the outlay-family roots
  1E-18 to 2E-16: consistent with any fast solver stopped at |step| < 1e-7 with a 1e-7 residual, so the digits do not
  separate a Newton step from a secant step. The search's reach, not its final steps, is what the model must find.
- Do not repeat: Newton in r and the secant in r as fallbacks; an unbounded XIRR-style bracket walk.
