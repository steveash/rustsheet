---
id: T-3117
title: "Probe IRR's fallback under power-of-two scaling and negation: separate its absolute residual test from its arithmetic"
milestone: M9
lane: research
status: blocked
priority: P0
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Excel values for IRR's fallback path on flows scaled by exact powers of two and negated, so that T-2928 can tell
an absolute residual test (which sees the scale) from the iteration's own arithmetic (which a power-of-two scale
leaves bit-for-bit unchanged) and from a step whose direction depends on the sign of the NPV.

## Context
T-2928 is blocked after two fit attempts (decision 0056); this probe is the new evidence its "Blocked because"
section asks for. The T-2927 sweep (probes-numeric/t2927-irr-fallback-sweep) scales by 1E6 and negates. A decimal
scale changes the flows' bits, so it cannot separate a stop test |P| < 1E-7 in the flows' units from rounding.
Scaling by 2^k changes the bits of no intermediate value: Newton or a secant in v gives the same bits at every k
unless a test compares an absolute quantity (|P| or |NPV|) with a constant. Negation also leaves every Newton
iterate bit-for-bit unchanged, so a difference between a row and its negation proves a step that depends on the
sign of P, such as a secant's signed first offset (financial.md "IRR's fallback when the secant fails", Fit, attempt 2).
Write the case with the generator tools/src/rustsheet_tools/fitlab/irr_fallback_sweep.py (extend `families()` or add a
second case next to it), then fill it on the oracle and write the roots TSV:
- `{k,1,1,-1}` for k = 4, 5, 6, 7, 42, 70 and `{-N,1,1,1,1,1}` for N = 284, 400, 496: each multiplied by 2^j for
  j = -30, -20, -10, -5, 0, 5, 10, 20, 30, 40, and each of those negated;
- `{0.3,-2.5,1.005}` times 2^j for the same j, and negated;
- column B: the same with guess 0.5 on the j = 0 and j = 20 rows.
Fewer than 300 cells. The flows are written as `{flows}*2^j`, exact in the formula (a typed 17-digit literal is cut to 15 digits by
Excel), so the case's values are exactly c·2^j.

## Acceptance criteria
- [x] The case is in eval/suites/probes-numeric with Excel's values (excel-graph) and a roots TSV with each value's
      signed offset in ulps from the exact root.
- [x] financial.md "IRR's fallback when the secant fails" records, per family, the j at which Excel's bits change
      (or that they never do), and whether each negated row equals its positive row.
- [x] T-2928's "Blocked because" section is updated to name this evidence, so the steward can reopen it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing the engine (T-2928).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 29 probes-numeric cells that no open task listed: 21 with the wrong type or error, 8 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t2927-irr-fallback-sweep.case.json (29): P!A24, A26-A31, A44, A47, A60, A62, A74, A101-A112, A138, A144, B31, B101, B111

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).

## Outcome
Extended tools/.../fitlab/irr_fallback_sweep.py (`pow2-case`, `pow2-roots`); case
probes-numeric/t3117-irr-pow2 (240 cells: 10 flows x 10 values of j x 2 signs, column B at j = 0 and 20) filled on
excel-graph, with math sidecar and roots TSV. The flows are written `{flows}*2^j` (exact in the formula) instead of
literals of the scaled doubles, because Excel cuts a typed literal to 15 digits and a 17-digit literal would change
the bits. Findings are in financial.md "IRR's fallback when the secant fails", finding 7; T-2928's "Blocked because"
names them. No engine change, so the 29 decision-0267 cells of the t2927 sweep are unchanged (the fit is T-2928's;
this probe is its new evidence; see "Blocked because"). Eval: new case only, no fixes, no
regressions. The new probes-numeric case serves the owned t2927 cells only indirectly (decision 0267 item 5).

## Blocked because: time-boxed (decision 0056, 0267)
The probe is done (the deliverables above), but it changes no engine value, so none of the 29 decision-0267 cells
moved; the task's own rule for that is to block it as time-boxed, and decision 0267 then accepts the cells that are
left. T-2928 (also blocked, time-boxed) is where a fit would start from finding 7. A steward or opus run may land
this branch's evidence (PR #3428) with the task in that state, or reassign the cells by decision record.
