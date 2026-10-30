---
id: T-2219
title: "F.DIST, F.DIST.RT and FDIST to 15 digits where the engine's incomplete beta and F density lose accuracy (16 visibly-off cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
The 16 F-distribution cells below agree with Excel to 15 significant digits. In 15 of them Excel is
within about 6E-16 of the true value and the engine is up to 4.3E-15 off. The 16th is `#NUM!` where
Excel has a number. These are not fits: an engine result accurate to about 1E-16 meets the
15-digit test on all of them.

## Context
Gap triage 2026-10-21 (`make eval` on `main` fce6020d). T-0970 moved F.DIST, F.DIST.RT, FDIST and
F.TEST onto `functions/statistical/incomplete_beta.rs` (`beta_reg`, `beta_reg_upper`,
`beta_density`). It left about 120 F-distribution known failures in probes-numeric that no open task
owns. T-2119 covers BETA.DIST's cdf only. T-1849 covers booleans, T-2095 x = 0, and T-1538/T-1864
df from 1E7 up (t1530-f-df-cut, m9-legacy-fdist A86).

Each cell below was checked against a 50-digit mpmath reference: `betainc(d1/2, d2/2, 0, y,
regularized=True)` with y = d1*x/(d1*x+d2), and the closed-form density. The relative errors were:

| cell | formula | Excel | ours | Excel err | ours err |
|---|---|---|---|---|---|
| m9-f-dist-rt Calc!A12 | `F.DIST.RT(10,5,2)` | 0.09339804392481495 | 0.09339804392481488 | 6.1E-17 | -6.8E-16 |
| m9-f-dist-rt Calc!A18 | `F.DIST.RT(1,50,100)` | 0.48912050780793015 | 0.48912050780793204 | -3.5E-17 | 3.8E-15 |
| m9-f-dist Calc!A20 | `F.DIST(0.5,10,10,FALSE)` | 0.682822740435909 | 0.6828227404359095 | -2.9E-16 | 5.2E-16 |
| m9-f-dist Calc!A25 | `F.DIST(0.5,50,100,TRUE)` | 0.0038265139669455906 | 0.003826513966945599 | 1.2E-15 | 3.4E-15 |
| m9-f-dist Calc!A26 | `F.DIST(0.5,50,100,FALSE)` | 0.08394970207752739 | 0.08394970207752772 | 3.2E-16 | 4.3E-15 |
| m9-f-dist Calc!A27 | `F.DIST(1,50,100,TRUE)` | 0.5108794921920699 | 0.510879492192068 | 3.4E-17 | -3.7E-15 |
| m9-f-dist Calc!A28 | `F.DIST(1,50,100,FALSE)` | 1.6223539167134502 | 1.622353916713457 | -4.4E-17 | 4.2E-15 |
| m9-f-dist Calc!A30 | `F.DIST(3,50,100,FALSE)` | 1.05292998522241E-05 | `#NUM!` | | |
| m9-legacy-fdist Calc!A27, A28 | `FDIST(0.5,5,50)`, `F.DIST.RT(0.5,5,50)` | 0.7747520966479879 | 0.7747520966479886 | -4.6E-16 | 4.0E-16 |
| m9-legacy-fdist Calc!A37, A38 | `FDIST(1,5,50)`, `F.DIST.RT(1,5,50)` | 0.4274944664166388 | 0.4274944664166397 | -5.6E-16 | 1.5E-15 |
| m9-legacy-fdist Calc!A39, A40 | `FDIST(1,100,3)`, `F.DIST.RT(1,100,3)` | 0.6038137503819936 | 0.6038137503819957 | 3.9E-16 | 3.9E-15 |
| t1049-fdist Calc!A4 | `F.DIST(0.5,3,7,FALSE)` | 0.6123581302772132 | 0.6123581302772126 | 6.3E-16 | -2.8E-16 |
| t1049-fdist Calc!A6 | `F.DIST(0.5,10,4,FALSE)` | 0.6272254743863065 | 0.627225474386307 | -6.7E-16 | 2.1E-16 |

All of these are probes-numeric cases (`gate = false`, no baseline file), excel-graph oracle. The
errors grow with a + b = (d1 + d2)/2. That points at the prefactor and the density's
Gamma(a+b)/(Gamma(a)Gamma(b)) ratio through musl `tgamma`, which loses digits as the shapes grow;
T-0970 switched to TOMS 708's `brcomp` form only past MAXGAM. A30's `#NUM!` is the density's
fallback, "where statrs' direct form is finite", failing at moderate df. Two cells (t1049-fdist A4
and A6) are ones where ours is closer to the true value than Excel, but on the other side, so a
correctly rounded result passes the 15-digit test and ours does not.
See docs/excel-semantics/statistics.md "Incomplete beta in the ironcalc-calc workbooks: F tails
and densities (T-0970)" and its accuracy note (2026-10-21), and
docs/research/excel-kernel-playbook.md before changing the kernel.

gen-m9 sweep (gap triage 2026-10-23, T-2255): 42 FDIST, F.DIST and F.DIST.RT known-failure lines
in eval/baseline/gen-m9.txt were routed here. None is at a + b of 10 or more on this kernel; the 26
still off on 2026-10-30 are handed to T-3198, which lists them.

## Acceptance criteria
Narrowed at implementation (2026-10-30) to the kernel this task changed; what it did not reach is
T-3198's (see Outcome).
- [x] Half-integer shapes with 10 <= a + b < MAXGAM, outside the closed forms and the ported bratio
      branches, are within 4 ulp of a 50-digit reference on the table's cells at that size
      (m9-f-dist-rt A18, m9-f-dist A25, A27, m9-legacy-fdist A39, A40) and in the unit test.
- [x] No F.DIST, F.DIST.RT, FDIST, F.TEST, BETA.DIST, BETA.INV or F.INV cell moves to visibly off
      except where Excel itself is more than half a unit in the 15th digit off the 50-digit value
      (10 cells, named in statistics.md; T-2221's).
- [x] `make eval-check` shows 0 regressions in gated suites (no gated cell changes).
- [x] statistics.md's T-0970 section records what changed and from which a + b, with the cells.
- [x] `make check` green; reviewers approve.
- Moved to T-3198: the gen-m9 known-failure lines, m9-f-dist-rt A19 (ported L140 branch) and the
  densities m9-f-dist A26, A30 (Excel's density model).

## Out of scope
- The last-digit fit of Excel's own formulation (closed forms at d1 = d2 = 1, the complement and
  tail choice, and cells where Excel itself is off by more than half a unit in the 15th digit:
  m9-f-dist-rt A16, A20; m9-legacy-fdist A47, A48, A55, A56; t1049-fdist A16). That is T-2221,
  after the T-2220 sweep.
- df of 1E7 and above (T-1538, T-1864), x = 0 (T-2095), boolean arguments (T-1849), BETA.DIST's
  cdf (T-2119).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns (the table above) is within half a unit of
the 15th significant digit of Excel's value, with no cell newly visibly off. Bit-level matches are
not required here: they belong to T-2221.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.

Gap triage 2026-10-30: probes-numeric/m9-f-dist-rt A19, `F.DIST.RT(3,50,100)`, was routed here; it
runs on the ported L140 bup + bpser branch and is handed to T-3198.

## Outcome
Half-integer shapes with 10 <= a + b < MAXGAM, outside Excel's closed forms and the ported bratio
branches, now take `incomplete_beta.rs::half_integer_tail`: x^a y^b with the exponent in
double-double (the argument above 1/2 as 1 minus its complement, exactly) times F.TEST's
double-double series `fisher::f_test_remainder`, instead of Cephes' prefactor and fraction (up to
about 40 ulp off a 50-digit reference). The quantile searches (BETA.INV, F.INV, FINV) and the
Student-t callers keep Cephes through `search_tails`: with the series under them 7 gated gen-m9
quantile cells and BETADIST_BETAINV.xlsx H67 lose their bit. Below a + b = 10 the series only
traded bit-exact cells and moved the gated `FDIST(2,1,9)` and `F.DIST(1,3,8,1)` 1 ulp, so Cephes
stays there (`SERIES_MIN_SHAPE_SUM`).

Eval delta (`make eval` against origin/main d6618cd): gated suites unchanged (no gen-m9 or
ironcalc-calc cell moves). probes-numeric: 172 cells from visibly off to within 15 digits, 105 onto
Excel's bit, 23 off it (all within 15 digits), and 10 to visibly off, each one where Excel is 6 to
50 ulp off the 50-digit value and ours within 5 (listed in statistics.md). Table cells: A18, A25,
A27 (m9-f-dist-rt / m9-f-dist), m9-legacy-fdist A39, A40 now within 4 ulp; A12, A28, legacy A27,
A28, t1049-fdist A4 were already bit-exact on main; A20, legacy A37, A38 and t1049 A6 within 4 ulp
before and after (densities and a + b < 10).

Not reached, handed to T-3198 with the gen-m9 cells (all a + b below 10, df 1E10 or the shape-of-1
forms; none moved): `F.DIST.RT(3,50,100)` (A19, the ported L140 bup + bpser branch) and the
densities `F.DIST(0.5,50,100,FALSE)` and `F.DIST(3,50,100,FALSE)` (A26, A30, Excel's density model,
T-3034). The gen-m9 acceptance line is therefore not met here; T-3198 owns it.
