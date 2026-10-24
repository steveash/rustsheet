---
id: T-2329
title: "XIRR: relax the no-later-date #NUM! check for a zero first value, apply T-2328's stop bound"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2328]
campaign: visible
---

## Goal
XIRR returns Excel's value, not `#NUM!`, for a zero first value whose date is after every other date, and applies the T-2328 stop bound once it lands.

## Context
probes/xirr-span-zero-first A28-A30 (engine `#NUM!`, Excel 2.98e-9 / 3.73e-9) contradict the engine's no-later-date check (`xirr_dates`, functions/financial/mod.rs). A11, A14, A20 need T-2328's stop bound. See financial.md "XIRR is a bracketing bisection" and "XIRR reads empty cells as 0".

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): `XIRR({0,0,-100,50,60},{39448,39448,39448,39448,39448},..)` (all dates equal, zero first values) is 2.5849043512437796E-09 in Excel, ours `#NUM!`: the no-later-date check again.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 visibly off.
Examples:
- gen-m9/e168be7f6d3f84c2 Calc!A125 `=XIRR({0,0,-100,50,60},{39448,39448,39448,39448,39448},(F.DIST(0.5,6,Data!D8,Data!B$8)^(Data!$B7-(-2.5))))`: Excel 2.5849043512437796e-09, ours #NUM!
Cells:
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A125

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2257): the 1 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] A28-A30 pass (done, branch); the nonzero-first-value no-later-date cases (xirr-mismatched-empties X9, X12, X13) stay `#NUM!`.
- [x] ~~A11, A14, A20 pass once T-2328 has fitted the bound.~~ Moved to T-2566: T-2328 found they need XNPV to discount with Excel's `^`, which T-2566 owns.
- [x] probes/xirr-zero-first-bracket (new, excel-graph): a zero first value brackets at [0, first chain point] whatever the flows, and a negative guess is `#NUM!`; all 16 cells pass.
- [x] `make check` green; reviewers approve.

## Out of scope
Probing the exact condition (a zero first value with all other dates equal): file a research probe if needed.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.

## Outcome
A zero first value skips XIRR's later-date input check (`xirr_dates`) and its bracket walk: `xirr_bracket` returns
[0, s] (s = the guess, 1e-5 for 0) whatever the flows, and `#NUM!` for a negative guess. New probe
probes/xirr-zero-first-bracket (excel-graph, 16 cells) shows the flows do not matter (roots at 1.0, 3, -0.1, a flat
XNPV of 10 or 0) and a negative guess is `#NUM!` even with a root on the downward chain; recorded in financial.md
("XIRR is a bracketing bisection", T-2329 bullet). The sonnet handoff (skipping the date check) fixed A28-A30; the flat
equal-dates cell needed the bracket rule. A11, A14, A20 belong to T-2566 (Excel's `^` in XNPV, per T-2328).
Eval: gen-m9 known failures 6723 -> 6722 (e168be7f6d3f84c2 A125), probes -3 (xirr-span-zero-first A28-A30), the new
probe's 16 cells all pass; 0 regressions. Tier moved sonnet -> opus by the handoff.
