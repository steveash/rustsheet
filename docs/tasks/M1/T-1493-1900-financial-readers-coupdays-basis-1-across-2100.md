---
id: T-1493
title: "1900 financial readers: COUPDAYS basis 1 across 2100-02-28 and YIELD last-digit gaps"
milestone: M1
lane: functions-financial
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The 1900 twin probe (probes/date-system-1900-financial-readers, T-1489) passes: `COUPDAYS(DATE(2096,2,15),DATE(2100,2,28),2,1)` gives 184 (ours 182), and YIELD agrees with Excel in the last digits.

## Context
These are 1900-calendar gaps found by T-1489, independent of the 1904 reading (T-1492). Failing cells (also the same YIELD and DURATION cells of probes/date-system-1900-financial-readers-shifted, 18 in all): E15 (COUPDAYS basis 1 across the non-leap 2100-02-28/29 boundary), and YIELD on bases 0, 1 and 4 (A8-E8, A19-E19, A30-E30; most differ in the 10th digit or later, E8 and E30 at 1e-9 relative, which looks like the solver's stopping rule). DURATION A20/C20 are 1 ulp. Check docs/excel-semantics/financial.md for the YIELD iteration rule before changing it.

## Acceptance criteria
- [x] The listed cells, in both 1900 cases, leave eval/baseline/probes.txt via `make eval-baseline`, with no new F line in any suite.
  (E15 did; the YIELD and DURATION cells moved to T-1374 on 2026-10-13, see Outcome.)
- [x] `make check` green; reviewers approve.

## Out of scope
- The 1904 reading (T-1492).

## Handoff (resumed)
Escalated by a sonnet run without implementing anything (work-loop 5b, "numeric fitting"). Why: the YIELD cells (A8-E8, A19-E19, A30-E30) differ in the 10th digit or later, E8/E30 at 1e-9 relative, which the context ties to the solver's stopping rule; matching that is numeric fitting against oracle values, and the DURATION 1-ulp cells likely need the same care. Not tried: the COUPDAYS basis 1 case (E15, 184 vs 182 across the non-leap 2100-02-28/29 boundary) looks separable and probably smaller. Next: read the YIELD iteration rule in docs/excel-semantics/financial.md, diff `make eval-show CASE=probes/date-system-1900-financial-readers`, fix E15 first, then the YIELD stopping rule.

## Outcome
COUPDAYS: probed the shape (probes/coupdays-feb-month-end, 296 cells, excel-graph). The 184 is not a century-year
quirk: COUPDAYS on basis 1 counts the schedule walked without the end-of-month rule whenever the period around
settlement starts or ends on a 29 February that is not the maturity (maturity 2025-02-28, settlement 2023-11-15: 184).
Implemented as `coupdays_actual`; recorded in docs/excel-semantics/financial.md ("COUPDAYS basis 1 around 29 February").
E15 passes, and so do the 292 non-DURATION cells of the new probe.
YIELD and DURATION: not fitted here. probes/yield-solver-sweep (66 cells, excel-graph) shows Excel's YIELD stops
1e-11 to 1e-9 short of PRICE's root, the same bits for the same bond moved by four-year blocks, but different bits for
two bonds with equal coupon count and day counts (financial.md, "YIELD stops short of PRICE's root"). The probe is
the sweep T-1374 asked for, so the YIELD cells (A8-E8, A19-E19, A30-E30 of both 1900 cases) and DURATION A20/C20
(1 ulp; no reordering of the sum matched all 43 DURATION oracle cells) moved to T-1374.
Eval delta: probes fixed 1 (date-system-1900-financial-readers E15), 0 regressions; 2 new probe cases (362 cells)
with 70 new-coverage known failures (the 66 YIELD sweep cells and 4 DURATION ulps, T-1374).
