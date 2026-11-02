---
id: T-3806
title: "IRR's secant runs up to 213 steps, not 50 (Excel's cap is 136..214)"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
IRR's secant takes as many steps per run as Excel's does, so the slow runs Excel finishes (long level annuities,
long series near a root of 0) return Excel's digits instead of falling through to `#NUM!`.

## Context
- Split out of T-3411 (IRR on conventional flows after the secant fails), whose full model is still open: this is
  the one part of it the evidence pins. Findings and refuted candidates: T-3411's task file and financial.md
  "IRR's secant runs up to 214 steps".
- `irr_secant` capped a run at 50 steps as a directive-5 bound. In an engine-faithful replica (twice-rounded
  operators), runs of 51 to 136 steps stop on Excel's exact digits (7 probe cells), and three cells put Excel's cap
  below 215 (t3411-irr-r4-fresh A55, t3478-irr-r4-wide A240, m9-fin-irr-rate A125: Excel returns the retry's digits).

## Acceptance criteria
- [x] `IRR_SECANT_STEPS` replaces the 50-step cap, with its evidence and FITTED range.
- [x] No eval cell regresses; the R4 guards are met and recorded below.
- [x] financial.md records the rule and T-3411's other findings.
- [ ] `make check` green; reviewers approve.

## Out of scope
- What Excel does after both secant runs fail (T-3411), `#DIV/0!` (T-3407), the last digits (T-3408).

## Outcome
- `irr_secant` runs up to 213 steps (`IRR_SECANT_STEPS`, was 50). Excel's cap is bracketed at 136..=214
  (t3406-irr-num-boundary A5 stops at step 136 on Excel's digits; t3411-irr-r4-fresh A55 would stop at 215 on digits
  Excel does not return).
- Eval delta (full `rustsheet-eval run`, against origin/main's engine): +34 cells, 0 regressions: gen-m9/31474c20fc1dafd6
  +1, gen-m9/ba4ddb817a8e700e +1, probes/t3406-irr-num-boundary 68 -> 70, probes-numeric/t2927-irr-fallback-sweep
  40 -> 43, t3347-irr-conventional-holdout 101 -> 106, t3478-irr-r4-wide 229 -> 251 (of 300). New cases:
  probes-numeric/t3411-irr-r4-fresh 88/150 and t3411-irr-r4-fresh-2 83/150 (excel-graph, 2026-11-02); their other
  cells are T-3411's open model (unexplained Excel errors on conventional flows); probes-numeric has no eval/baseline
  file, so they show only in the eval report's gap list.
- R4 guards (decision 0188; plans fixed in T-3411's task file before each sweep was generated):
  - Round 1 froze cap 220 plus a stall rule (return a stalled iterate at |NPV| < 1E-4). Its fresh sweep
    t3411-irr-r4-fresh: 93/150 against main's 80, but 62% missed the stated margin (within 10 points of 85.7% on
    t3478-irr-r4-wide), and the sweep refuted both constants (A55 bounds the cap at 214; A129 shows Excel takes the
    fallback's digits, not the stalled iterate). Not shipped.
  - Round 2 froze cap 213 alone (from round 1's bracket; its lower end leaned on the refuted stall rule, and the
    honest bracket is 136..=214). Fresh sweep t3411-irr-r4-fresh-2: 83/150 against main's 76, no cell main gets right
    is lost (criterion 1); pass rate 55% against 59% on t3411-irr-r4-fresh (criterion 2: within 10 points). Passed.
  - Hold-out (R4.1): no split was fixed before the round-1 bracketing (every IRR case was used); the two fresh sweeps
    stand in. Correctly rounded candidate: not applicable (a step cap, not a rounding).
  - Gate A envelope (R2, against each sweep's 50-digit math.tsv; inside = Excel's value to 15 digits or within 1e-9 of
    the math): t3411-irr-r4-fresh 94/150 inside, t3411-irr-r4-fresh-2 95/150. Every point outside is a type or error-kind
    mismatch (T-3411's open model) or a digit miss on a long run; none was moved out by the cap (criterion 1): sweep 1
    A1-A6, A8, A9, A11, A12, A15, A17, A18, A20, A22, A23, A25-A27, A44, A46, A51, A59, A66, A69, A70, A72, A73,
    A76-A86, A88, A89, A91, A93-A97, A99, A102, A107, A115, A126, A128, A137, A145, A150; sweep 2 A1-A3, A5-A9, A11,
    A12, A14-A21, A23, A29, A33, A35, A40, A41, A51, A52, A56, A58, A61, A62, A67-A70, A76, A80-A85, A87, A88, A91-A93,
    A95, A97-A100, A106, A109, A127, A130.
  - Reproducibility: `cd tools && uv run python -I ../docs/research/t3411/gen_sweep.py <repo> [case name]` regenerates
    both cases' cells (seeds "T-3411/r4-fresh", "T-3411/r4-fresh-2").
- Directive 5: the cap stays a bound (213 steps of an O(n) NPV per run, two runs).
