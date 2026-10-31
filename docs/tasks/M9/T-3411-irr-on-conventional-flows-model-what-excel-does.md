---
id: T-3411
title: "IRR on conventional flows: model what Excel does after its secant fails (Newton from v = 1 and the T-3406 candidates are refuted)"
milestone: M9
lane: functions-financial
status: todo
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
