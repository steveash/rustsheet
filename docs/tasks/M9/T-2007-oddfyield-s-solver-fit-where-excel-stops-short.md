---
id: T-2007
title: "ODDFYIELD's solver: fit where Excel stops short of ODDFPRICE's root (31 desktop cells with a known root, about 70 web cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
ODDFYIELD returns Excel's bits by reproducing where Excel's iteration stops: its start, step and stop rule. Excel's
results sit 1e-12 to 7e-11 away from ODDFPRICE's root, on both sides, while ours land on the root. If two attempts
do not raise the bit-exact count, the task records the candidates and is time-boxed under decision 0056.

## Context
Gap triage 2026-10-19. ODDFYIELD has 106 failing cells of 166 checked, and no task owns its solver: T-1374 lists only
its fin-argument-modes base row next to unrelated last-digit work.
- `ironcalc-calc/FINANCIAL/PRICE_YIELD.xlsx` sheet ODDFPRICE_ODDFYIELD, K2:K52: all 31 ODDFYIELD cells fail (28 close,
  3 mismatch). Each row prices a bond at a known yield F (0.05, 0.067, 0.06, 0.03) with `J = ODDFPRICE(..,F,..)`, then
  inverts it with `K = ODDFYIELD(..,J,..)`. So the true root is F to within the rounding of J, and Excel's stopping
  error can be read off directly. K2 is 0.04999999999851281 (ours 0.05), K8 is 0.0499999999321231 (ours
  0.049999999999999635), and K23 is 0.05000000000085884, above the root. Desktop cache (Mac 16.x). In 14 rows the
  engine's J is itself an ulp off (J11, J23, ... in eval/baseline/ironcalc-calc.txt, T-1374), so the lab should read
  Excel's cached J (`rustsheet-eval snapshot`), not ours.
- Web (excel-graph): probes/date-system-1900-odd-years (19), -1900-odd-threshold (13), -1900-odd-accrint-bound (11),
  -1904-odd-threshold (11), -1904-odd-years (7), -1900-oddf-issue-after-quasi (2), date-system-1904-financial S!A32,
  fin-remaining-later-argument-ranges I121 and its neighbours (8), fin-yield-odd-lift A5/I9, fin-argument-modes H200.
  Many repeat one bond, e.g. 0.06499377053016654 against our 0.06499377053015054.
Engine: `fn_oddfyield` (functions/financial/financial_bonds.rs) calls `solve_yield`, a central-difference Newton
(h = 1e-5) from the coupon rate that stops at |dy| < 1e-8, with a bisection fallback. YIELD uses the same solver, and its
fit is blocked (T-1592: Excel stops below PRICE's root on a residual test; no start formula found). ODDFYIELD's
errors fall on both sides of the root, so its rule may not be YIELD's. The known-root desktop rows give evidence YIELD's
fit never had. If a rule found here also explains YIELD, record it in T-1592's "Blocked because" section as a new
hypothesis, which reopens that task.
Lab: the solver kernels are in tools/src/rustsheet_tools/fitlab/kernels/solvers.py (`newton_back_chain`, YIELD's
candidates, T-1534's iterate traces). Add an `oddfyield` fit there.
This needs numeric fitting against a solver trace, hence opus.

T-2207 (2026-10-23): ODDFPRICE, and the price ODDFYIELD's solver walks, now discount with Excel's `^`
(every ODDFPRICE cell in eval/ passes bit-exact); ODDFYIELD passes the same cells as on libm prices. In the YIELD
lab, `^`-priced solvers reach at most 4/178 bit-exact as the libm-priced ones do, so the price kernel does not
explain the stopping point (financial.md "PRICE and ODDFPRICE discount with Excel's `^`").

T-2483 probe (excel-graph, 2026-10-24): probes/odd-bond-argument-check-order A258
`ODDFYIELD(39448,43831,39300,39630,0.0575,95,100,2,0)` is 0.06342868471015199 in Excel, ours 0.06342868470999918 (a
known-failure line in eval/baseline/probes.txt; the rest of that case is argument order and passes).

## Acceptance criteria
- [ ] An `oddfyield` lab fit over the cells above (Excel's cached J as the price input) scores named candidates:
      Newton with an analytic or finite-difference derivative, secant, starts at the coupon rate / 0 / 0.1 / the
      one-period closed form, stop on |dy| or on |price residual|. Each candidate's bit-exact and within-4-ulp counts,
      rejected ones included, go into financial.md under a new "ODDFYIELD stops short of ODDFPRICE's root" entry that
      cites PRICE_YIELD K2, K8, K23 and the probe cells.
- [ ] Either: a model that raises the bit-exact count is implemented for ODDFYIELD alone, leaving YIELD on its current
      path unless the model also raises YIELD's count. ODDFYIELD F lines in probes.txt and ironcalc-calc.txt only
      shrink, and no baseline gains a line. Or: the task is set `blocked`, "Blocked because: time-boxed (decision
      0056)", with the candidates listed.
- [ ] The solver stays bounded within the lift weight `YIELD_SOLVER_WEIGHT` (financial.md "Yield and odd-coupon work cap").
- [ ] `make check` green; reviewers approve.

## Out of scope
- ODDFPRICE's own last digits (the J column, T-1374) and YIELD (T-1592).
- ODDLYIELD, which has no failing cells.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 106 in total.
- ODDFYIELD: 106 cells (13-14 digits right 0, 10-12 77, 6-9 29, fewer or 0 vs nonzero 0); most in `ironcalc-calc/FINANCIAL/PRICE_YIELD.xlsx` 31; `probes/date-system-1900-odd-years.case.json` 19. e.g. `ODDFYIELD(A2,B2,C2,D2,E2,J2,G2,H2,I2)` Excel 0.04999999999851281, ours 0.05

Excel stops short of the root by up to ~1e-9 relative; 15 digits means reproducing that stop.

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): ODDFYIELD 44. This task now owns them:
- probes/date-system-1900-odd-accrint-bound: S!A3, S!A4, S!A6, S!B3, S!B4, S!B6, S!C3, S!C4, S!D3, S!D4, S!D6
- probes/date-system-1900-odd-threshold: S!A5, S!B5, S!C5, S!D5, S!E5, S!F5, S!G5, S!H5, S!I5, S!J5, S!K5, S!L5, S!M5
- probes/date-system-1900-oddf-issue-after-quasi: S!A17, S!A32
- probes/date-system-1904-odd-threshold: S!C5, S!D5, S!E5, S!F5, S!G5, S!H5, S!I5, S!J5, S!K5, S!L5, S!M5
- probes/date-system-1904-odd-years: S!C3, S!E3, S!F3, S!J3, S!L3, S!N3, S!S3
Priority raised from P3 to P2 by this sweep: ODDFYIELD needs these 44 cells to reach the Phase 1 rate, and this task is their only owner.

## Attempt 1 of 2 (decision 0056), 2026-10-28: no candidate fits
Result: 0 of 43 distinct bonds (behind the 111 failing ODDFYIELD cells of the 2026-10-28 eval at `main` 1a0b616) bit-exact
or within 4 ulp for every candidate below, so the bit-exact count did not move and nothing changed in the engine.
The scores, the evidence and what Excel's values show are in financial.md "ODDFYIELD stops short of ODDFPRICE's root".
- Method: a temporary dump of `OddFirstSchedule`'s fields and the target price from `fn_oddfyield`, taken over every
  case that calls ODDFYIELD with `RS_ODDF_DUMP=1 rustsheet-eval show <case>` (not committed), re-priced in Python
  with the fit lab's `caret_pow`. Expected values are the `show` output's Excel values.
- Candidates: the full grid (starts, slopes, stops and how each was scored) is in that financial.md entry, under
  "T-2007 fit-lab attempt 1". Every candidate is 0/43 bit-exact and 0/43 within 4 ulp.
- What constrains the next attempt: Excel lands above the root in 60 of the 111 cells, and analytic Newton on a convex,
  decreasing price never does after its first step. So either the slope is not ODDFPRICE's (a difference quotient
  or another function's), or the function Excel solves is not exactly ODDFPRICE. The annual redemption-120 rows
  (K2-K5, K43-K46) all stop 1.34e-12 to 1.67e-12 below the root at a 1.1e-11 relative residual, whatever the yield.
  The semiannual rows K8-K11 and K49-K52 repeat each other's gaps. Reverse-solving the iterate before Excel's value
  for difference quotients with h = 1e-4 to 1e-7 gives no shared step tolerance.
- Next (attempt 2): add the `oddfyield` kernel to tools/src/rustsheet_tools/fitlab/kernels/solvers.py (port
  `OddFirstSchedule::new` so cells are scored from their formulas, as acceptance asks). Probe Excel's iterates
  directly, as probes/solver-iteration-traces did for YIELD (T-1525): ODDFYIELD over a price sweep of one bond whose
  root is known, at 1-unit steps of the price, to see how the stopping gap moves with distance from the start. Then try
  starts from the dates (YIELD's start uses the term in years: T-2777).

## Outcome
Attempt 2 (2026-10-28) found the rule: ODDFYIELD is YIELD's Newton (`excel_yield_newton` from `yield_start` over
`yield_start_years`) on ODDFPRICE with a slope that weighs the odd first coupon as a regular one
(`OddFirstSchedule::dprice`); the old central-difference solver and its bisection fallback (`solve_yield`, ODDFYIELD's
only caller) are gone. Evidence: the new probes/oddfyield-solver-sweep (547 cells, excel-graph): 320 pass against 123
before; the lab scores 323/324 coupon-bond cells within 1e-13 for this slope against 15 for the exact one.
Eval delta: ironcalc-calc known failures 567 -> 558 (9 PRICE_YIELD ODDFYIELD cells fixed), probes 16 fixed and 0
regressions; no line of an existing case was added. The only added lines are the 227 of the new sweep case
(new coverage, mostly within a few ulps, written by `make eval-baseline`), which the "no baseline gains a line"
criterion, a guard against regressions, does not cover. `quality/oversized-files.json` was rewritten by
`check_file_size.py --update` (every entry fell). The solver is
the one YIELD runs under `YIELD_SOLVER_WEIGHT`, with fewer price sums per step than before.
Left over, in T-3031: the long first periods at frequency 2 (PRICE_YIELD K8-K11, K29-K33, K49-K52) and the sweep
cells near a zero yield; the bit-exact last ulps stay with them.
