---
id: T-1493
title: "1900 financial readers: COUPDAYS basis 1 across 2100-02-28 and YIELD last-digit gaps"
milestone: M1
lane: functions-financial
status: todo
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
- [ ] The listed cells, in both 1900 cases, leave eval/baseline/probes.txt via `make eval-baseline`, with no new F line in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1904 reading (T-1492).

## Handoff
Escalated by a sonnet run without implementing anything (work-loop 5b, "numeric fitting"). Why: the YIELD cells (A8-E8, A19-E19, A30-E30) differ in the 10th digit or later, E8/E30 at 1e-9 relative, which the context ties to the solver's stopping rule; matching that is numeric fitting against oracle values, and the DURATION 1-ulp cells likely need the same care. Not tried: the COUPDAYS basis 1 case (E15, 184 vs 182 across the non-leap 2100-02-28/29 boundary) looks separable and probably smaller. Next: read the YIELD iteration rule in docs/excel-semantics/financial.md, diff `make eval-show CASE=probes/date-system-1900-financial-readers`, fix E15 first, then the YIELD stopping rule.
