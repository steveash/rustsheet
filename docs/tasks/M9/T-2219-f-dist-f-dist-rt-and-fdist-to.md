---
id: T-2219
title: "F.DIST, F.DIST.RT and FDIST to 15 digits where the engine's incomplete beta and F density lose accuracy (16 visibly-off cells)"
milestone: M9
lane: functions-stat
status: todo
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

gen-m9 sweep (gap triage 2026-10-23, T-2255). FDIST, F.DIST and F.DIST.RT values visibly off (42 cells).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 42 known-failure lines in eval/baseline/gen-m9.txt, 42 formulas.
Examples:
- gen-m9/146b46e74f2dac3b Calc!A329 `=F.DIST.RT(10,6,2)`: Excel 0.09368601255412709, ours 0.09368601255412701
Cells:
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A329
- gen-m9/2801c1c11c21e134.case.json (3): Calc!A130, A219, A245
- gen-m9/285d87cea1aaa73c.case.json (2): Calc!A34, A125
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A259
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A123
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A49, A181
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A141
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A124
- gen-m9/623663cacea53bbc.case.json (1): Calc!A365
- gen-m9/68d8ffb69ff739d8.case.json (4): Calc!A3, A29, A64, A273
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A349
- gen-m9/7c20351562a2d71d.case.json (2): Calc!A251, A343
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A144, A395
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A92
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A127
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A220, A225
- gen-m9/a5828d3925575713.case.json (1): Calc!A235
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A70
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A142
- gen-m9/b6593d7dd13f2dac.case.json (2): Calc!A50, A91
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A22
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A344
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A297
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A289
- gen-m9/cde551ea787f713a.case.json (2): Calc!A88, A359
- gen-m9/d8266bc15533702a.case.json (3): Calc!A120, A337, A364
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A118
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A250

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 42 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The 16 cells in the table agree with Excel to 15 significant digits. For A30 that means a
      number, not `#NUM!`.
- [ ] No F.DIST, F.DIST.RT, FDIST, F.TEST, BETA.DIST, BETA.INV or F.INV cell in ironcalc-calc,
      probes or probes-numeric moves from bit-exact or 15-digit-equal to visibly off. Ulp-level moves
      among cells that stay within 15 digits are allowed (decision 0083) and are listed in the Outcome
      with their before/after ulp distance.
- [ ] `make eval-check` shows 0 regressions in gated suites (ironcalc-calc F_DIST.xlsx, F_TEST.xlsx,
      BETADIST_BETAINV.xlsx, BETA_GAMMA.xlsx).
- [ ] statistics.md's T-0970 section records what changed (prefactor or density form and from which
      a + b), with the cells as evidence.
- [ ] `make check` green; reviewers approve.

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
