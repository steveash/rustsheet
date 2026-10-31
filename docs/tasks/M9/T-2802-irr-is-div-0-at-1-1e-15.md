---
id: T-2802
title: "IRR is #DIV/0! at {-1,1E+15}, {-1,1E+300} and a zero-first-flow gen-m9 set where ours is #NUM!"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IRR returns Excel's `#DIV/0!` where Excel gives it instead of our `#NUM!`.

## Context
Cut from T-2727 (2026-10-26), which fixed IRR's far-root bound and its fallback but found no rule for the `#DIV/0!`
cells (financial.md "IRR's far-root bound", the "Not explained" paragraph):
- probes/irr-guess-edges-2 A20, A28, A29: `IRR({-1,1E+15})` from guesses 0.1, 1E+15 and 5E+14; probes/irr-guess-edges A38
  (guess 0.05) and A39 `IRR({-1,1E+300})`. Neighbours `IRR({-1,9E+14})` and `IRR({-1,2E+15})` are `#NUM!`.
- probes/irr-guess-edges A27-A29 and gen-m9/bc04c1f478c9896c Calc!A56: IRR over Data!A1:D12 (first flow 0, values 1E+15
  among them) from guesses 0.05, 0.1, 0.5; `IRR({0,1E+15,2,42,1E+15,-0.0001})` (A34, A35) is `#NUM!`.
A likely shape: a division by an exact zero (a discount factor of 0, 1/v, or a flat step) in Excel's steps; it needs a
probe of nearby flows (X = 1E+15 +- a few ulp, other zero-first-flow sets).
Code: `compute_irr`, `irr_secant`, `irr_fallback` in crates/rustsheet-engine/src/functions/financial/financial_util.rs.

Cell added by gap triage (2026-10-31, `make eval` on origin/main, excel-graph; `scripts/exit_owners.py --all-off`
listed it as unattributed because BITXOR is not glue, and no open task named it):
- gen-m9/530c636749a85209 Calc!A338 `=HYPGEOM.DIST(IRR(Data!C2:D3,BITXOR(235,Data!E4)),3,2,19,NOT(..))`: Excel `#DIV/0!`,
  ours `#NUM!`. Data!C2:D3 is " x ", -1; 1E+15, "x", so the IRR is `IRR({-1,1E+15},143)` (texts skipped,
  BITXOR(235,100) = 143): the same `{-1,1E+15}` `#DIV/0!` as above, from a guess of 143. HYPGEOM.DIST passes the error.
Cells:
- gen-m9/530c636749a85209.case.json (1): Calc!A338

## Acceptance criteria
- [ ] The cells above pass (`make eval-baseline` drops them) with no new known-failure lines; the rule is in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR's returned digits (T-2131).
