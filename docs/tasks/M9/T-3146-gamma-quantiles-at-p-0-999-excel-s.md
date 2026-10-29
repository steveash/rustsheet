---
id: T-3146
title: "Gamma quantiles at p = 0.999: Excel's landing point in the flat band of its double cdf (12 unowned visibly-off GAMMA.INV, GAMMAINV, CHISQ.INV and CHISQ.INV.RT probe cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3145]
campaign: visible
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV and CHISQ.INV.RT at p = 0.999 return Excel's quantile to half a unit of the 15th
significant digit: the point Excel's search lands on inside the band of doubles whose double-valued cdf rounds to p,
not the true quantile the engine returns today.

## Context
Gap triage 2026-10-29 (`make eval` on main d57e9e9a2, excel-graph; `python3 scripts/exit_owners.py --all-off`): no open
task names these cells. Ours is within an ulp of the true quantile at every one; Excel is 18 to 106 ulp off it
(50-digit mpmath bisection on the regularized incomplete gamma). Excel vs ours (Excel's offset from the truth):
- probes-numeric/m9-chisq-inv Calc!A20 `CHISQ.INV(0.999,3)`: 16.266236196237987 vs 16.266236196238125 (-40 ulp)
- probes-numeric/m9-chisq-inv Calc!A34 `CHISQ.INV(0.999,10)`: 29.58829844507459 vs 29.588298445074415 (+48)
- probes-numeric/m9-chisq-inv Calc!A41 `CHISQ.INV(0.999,50)`: 86.6608151904034 vs 86.66081519040313 (+18)
- probes-numeric/m9-chisq-inv-rt Calc!A20 `CHISQ.INV.RT(0.999,3)`: 0.024297585815692378 vs 0.024297585815692746 (-106)
- probes-numeric/m9-chisq-inv-rt Calc!A27 `CHISQ.INV.RT(0.999,5)`: 0.210212602629218 vs 0.21021260262921926 (-44)
- probes-numeric/m9-chisq-inv-rt Calc!A34 `CHISQ.INV.RT(0.999,10)`: 1.478743463835678 vs 1.4787434638356651 (+57)
- probes-numeric/m9-chisq-inv-rt Calc!A41 `CHISQ.INV.RT(0.999,50)`: 24.67390527187741 vs 24.67390527187726 (+41)
- probes-numeric/m9-chisq-inv-rt Calc!A48 `CHISQ.INV.RT(0.999,100)`: 61.917939206936786 vs 61.91793920693662 (+23)
- probes-numeric/m9-gamma-inv Calc!A34 `GAMMA.INV(0.999,2,1)`: 9.233413476451513 vs 9.233413476451585 (-40)
- probes-numeric/m9-gamma-inv Calc!A41 `GAMMA.INV(0.999,2,1000000)`: 9233413.476451514 vs 9233413.476451587 (-38)
- probes-numeric/t1048-gammainv Calc!A7 `GAMMAINV(0.999,5,1)` and Calc!B7 `GAMMA.INV(0.999,5,1)`: 14.794149222537294 vs
  14.794149222537207 (+48)

docs/excel-semantics/statistics.md (the "Excel's gamma quantiles (T-0972 ...)" bullet) already reads these as Excel
solving the tail it is given on a double-valued cdf: the lower one for CHISQ.INV and GAMMA.INV, the upper one for
CHISQ.INV.RT. That explains the RT cells: `CHISQ.INV.RT(0.999,3)` is not Excel's own `CHISQ.INV(0.001,3)`
(0.02429758581569273 in the same probe, 1 ulp from the truth), because a cdf near 0.999 has 2^-53 absolute resolution
and the band of x it cannot tell apart is about 1E-13 relative wide. The engine instead always crosses the smaller tail
on the accurate pair (`incomplete_gamma::gamma_quantile`, `refine_increasing`). Where in the band Excel lands depends
on its starting point and stopping test, which the cached probes (p only 0.95, 0.999 and 1 - 1E-10) cannot pin down:
Needs an oracle sweep, T-3145, which also gives Excel's cdf at the doubles beside each quantile above. At 1 - 1E-10
the same probes are within 2 ulp, so the rule changes between those p.

T-3016 (`CHISQ.INV(0.999,5)` in the m9 and T-1049 CHISQ.INV probes) and T-3131 (`CHISQ.INV.RT(0.999,5)` and
`(0.999,20)` in the T-1049 CHISQ.INV.RT probe) own cells with the same cause. Whichever task lands the landing rule
first takes them all; the others then only check their cells. `GAMMA.INV(0.999,9,1)` and `(0.999,9,1000000)` are 2-3E-15
relative off, within 15 digits; it should follow the same rule.

## Acceptance criteria
- [ ] The 12 cells above are within half a unit of Excel's 15th significant digit (`python3 scripts/exit_owners.py
      --all-off` no longer lists them), with no new known-failure line in any eval/baseline file
      (`make eval-baseline`) and no p = 0.999 cell of T-3016 or T-3131 made worse.
- [ ] The landing rule and the forms tried, with their bit-exact and visible-parity counts on T-3145's case and the
      cached probes, are in docs/excel-semantics/statistics.md (decision 0056: two attempts, then time-box).
- [ ] Iteration stays bounded on untrusted input (the shared `MAX_TERMS` budget; huge shape or scale; p next to 1).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The deep tails (T-2561), shapes above 1E9 (T-3070), the statrs search start (T-3131) and the 1-3 ulp lines of
  T-3016.
- The CDF kernel (T-2638, T-3147).

## Visible-parity stage
Decision 0083/0114: Excel is the inaccurate side here (18 to 106 ulp from a 50-digit reference) and the engine is
within an ulp of the truth, so the stage is `visible`: reproduce Excel's search, never a more accurate kernel. The
target is each owned cell within half a unit of Excel's 15th significant digit; bit-level misses left then go to a
`defer: phase-1b` follow-up.
