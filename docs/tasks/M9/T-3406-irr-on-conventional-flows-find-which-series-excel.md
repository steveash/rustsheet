---
id: T-3406
title: "IRR on conventional flows: find which series Excel leaves #NUM! where the secant fails (the v = 1 Newton split misses 4 hold-out cells)"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A rule, backed by oracle cells, for which conventional cash-flow series (one outlay, then flows of the other sign)
Excel's IRR returns a number for and which it leaves `#NUM!`, where the secant model fails, that holds on a hold-out
sweep drawn after it is frozen. It unblocks T-3347, whose `irr_conventional` stage failed its R4 hold-out.

## Context
- T-3347 fitted "Newton on P(v) from v = 1, 50-step cap" on 162 cells (train: probes/t3347-irr-conventional-num,
  probes-numeric/t3347-irr-conventional-sweep): it returned the 159 numbers and left the three `#NUM!` cells (sweep
  A111, A115, A117: 128, 160 and 65 steps). On the hold-out (probes-numeric/t3347-irr-conventional-holdout, 144 cells,
  seed 334702) it turns 29 `#NUM!` cells into Excel's numbers but returns a number on three where Excel is `#NUM!`:
  A26 (near0-neg, 60 flows, root -2.84e-4, 3 steps from v = 1), A70 (near-10pct, 39 flows, root -0.0961, 17 steps),
  A142 (neg-long, 22 flows, root -0.1916, 24 steps); and it leaves A61 (near-10pct, 55 flows, root -0.0983) `#NUM!`
  where Excel returns a number, because the run needs 54 steps, past the cap. No step cap separates them (A54: 25
  steps, returns; A142: 24, `#NUM!`; A61: 54, returns).
- The secant's own failure reason does not separate them either (scratch classification in T-3347's Outcome): A26
  hits the 50-step cap like A44 and A45, which return; A70 and A142 go flat at step 3 like A66, A67, A69, A71 and
  A54, which return.
- Candidates to test: Excel retries from other guesses than 0.1 (a fixed list), a different start or stop for the
  last stage, a bound on |NPV| or on the iterate path; probe single-flow perturbations of A26, A70 and A142 to find
  the boundary. Code: `compute_irr`, `irr_conventional`, `irr_newton` in
  crates/rustsheet-engine/src/functions/financial/financial_util.rs. Score with
  docs/research/t3347_irr_holdout_score.py.
- Decision 0188 R4: the rule is chosen on the train and hold-out cells above, then a fresh sweep is filled after it
  is frozen; the margin is stated before scoring.

## Acceptance criteria
- [x] (time box: no rule found; candidates and counts below and in financial.md) A rule that gives the right type (number or `#NUM!`) on all 306 cells of the three t3347 cases (the 9 `#DIV/0!` cells of T-3407 aside), or, if none is
      found within the time box, the candidates tried and their counts in this file and in
      docs/excel-semantics/financial.md "IRR on conventional flows near 0 and -10%".
- [ ] (moved to T-3411: no rule to freeze) A fresh probes-numeric sweep filled after the rule is frozen, scored per stratum (Gate A envelope, type
      mismatches), with 0 type mismatches.
- [ ] (moved to T-3411) T-3347's PR (#2529) updated to the rule, or T-3347 re-scoped with an owner ruling.
- [x] `make check` green; reviewers approve.

## Out of scope
- The `#DIV/0!` cells of the hold-out (T-3407) and the last digits (T-3408).

## Outcome
- Probe: probes/t3406-irr-num-boundary (excel-graph, 2026-10-31; 144 cells; generator
  docs/research/t3406_irr_num_boundary.py, 50-digit roots in its `.roots.tsv`). A1-A96 level annuities (n = 20, 30,
  40, 60; root -0.02 to -0.48), A97-A144 the outlays of t3347-irr-conventional-holdout A26, A70, A142 and A61 scaled
  by 0.97 to 1.03. The hold-out case is copied from T-3347's branch unchanged (the generator reads its flows).
- Found: (1) Excel returns level annuities down to r = -0.22 (n = 20), -0.18 (30), -0.14 (40), -0.12 (60), then
  `#NUM!`, then `#DIV/0!` from about -0.36 (n = 40) and -0.18 (n = 60); (2) a sub-1% change of the outlay flips
  Excel between a number and `#NUM!` back and forth (A97-A132), so the decision is path-sensitive, not a smooth
  criterion.
- Refuted, by type agreement on 450 cells (this probe and T-3347's three cases; a double replica of the secant): Newton
  from v = 1 under any cap (T-3347's stage; n = 60 returns at 137 steps, n = 20 is `#NUM!` at 33); the secant from 0.1
  with a 50/100/200 cap (295/301/306); the secant continued past a flat step by a Newton step (304/311/317); a retry
  from any of 17 fixed guesses in -0.9 to 2. Recorded in docs/excel-semantics/financial.md "IRR on conventional flows
  where the secant fails: which return is path-sensitive".
- Eval: the new case adds 76 known failures to probes (28 + 26 `#NUM!` where Excel returns the root, 22 `#NUM!` where
  Excel is `#DIV/0!`); no other cell moves (no engine change).
- Follow-up: T-3411 (the model; owns the remaining criteria and unblocks T-3347). T-3407 gains 22 `#DIV/0!` cells
  (A66-A72, A81, A82, A84-A96) of this probe.
