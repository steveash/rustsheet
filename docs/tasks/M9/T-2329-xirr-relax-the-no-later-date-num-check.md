---
id: T-2329
title: "XIRR: relax the no-later-date #NUM! check for a zero first value, apply T-2328's stop bound"
milestone: M9
lane: functions-financial
status: todo
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
- [ ] Visible parity (gen-m9, T-2257): the 1 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A28-A30 pass (done, branch); the nonzero-first-value no-later-date cases (xirr-mismatched-empties X9, X12, X13) stay `#NUM!`.
- [ ] A11, A14, A20 pass once T-2328 has fitted the bound.
- [ ] `make check` green; reviewers approve.

## Out of scope
Probing the exact condition (a zero first value with all other dates equal): file a research probe if needed.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.

## Handoff (resumed)
Tried: skipped `xirr_dates`' no-later-date `#NUM!` when the first value is 0 (functions/financial/mod.rs), with a unit test. Result: probes/xirr-span-zero-first A28-A30 now pass (3 baseline lines removed from eval/baseline/probes.txt). Still failing: gen-m9/e168be7f6d3f84c2 A125 stays `#NUM!` (Excel 2.5849043512437796e-09), and A11, A14, A20 still fail (expected values were not checked against T-2328's Outcome).
Why A125 fails: all five dates are equal, so XNPV is constant (sum of values) at every rate and `xirr_bracket` never sees a sign change, so it returns `None`. Excel returns a tiny value that looks guess-dependent (2.5849e-9 * 2^25 is about 0.0867; the other zero-first cells give guess-bracket/2^25, financial.md). Next: read the guess value of A125 (`make eval-show`, Data!D8/B7/B8), then probe equal-date zero-first cells at several guesses (excel-probe skill) to find the rule for a flat XNPV, then implement it in `xirr_bracket`/`compute_xirr`. Check A11, A14, A20 against T-2328's Outcome.
