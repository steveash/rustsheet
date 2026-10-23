---
id: T-2503
title: "Beta density for min(a,b) <= 1: Excel's M1 composition (TOMS betaln, Excel log1p, extended exponent) (T-2464 E2)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2501]
---

## Goal
BETA.DIST(x,a,b,FALSE) and BETADIST's density, for min(a,b) <= 1, follow Excel's composition M1. The same function is
the prefactor density that bup needs (T-2506).

## Context
- T-2464 research, step E2 of docs/research/t2464/r2-impact/RESULTS.md section 4.
- The rule is docs/excel-semantics/statistics.md "Beta density for min(a,b) <= 1: TOMS betaln and an extended exponent
  (T-2464)":
  - lnx = LN(x); ly = Excel's log1p(-x) for x <= 0.375 (T-2501), LN(fl(1 - x)) above;
  - tx = fl(fl(a-1)·lnx) and ty = fl(fl(b-1)·ly);
  - lnB is TOMS 708 betaln from Excel's GAMMALN and LN:
    - for max <= 8 (8 inclusive), the GL form, smaller shape first;
    - above 8, GAMMALN(min) + algdiv;
    - for a = 1 and 2 < b < 8, -LN(b);
  - E = tx + (ty - lnB) held extended (stored only when b = 1);
  - the density is EXP of the extended E (T-2501).
- Models:
  - `docs/research/t2464/w1-density/model_final.py` (M1) and `toms_betaln.py`;
  - r2-bratio `bratio_excel.py` `density()`, which reproduces M1 exactly;
  - r2-log1p `dens_m1.py` (M1 with Excel's log1p).
- Scores:
  - old pdf cells: 685 of 2,035 (CR 337);
  - w1's probe1 (w1-density/probe/probe1.out.json, not in eval/suites): 10,301 of 16,000;
  - with Excel's log1p, the modelled probe1 shapes: 10,375/12,799/12,799 of 12,800.
- r2-impact's expected change:
  - -148 visibly-off pre-T2464 cells (-81 hold-out) and -410 in the w2f probes;
  - 1 broke;
  - 69 bit-exact cells lost, all in probes-numeric (none gated).
- Code: `incomplete_beta::beta_density`, today powf × tgamma ratio, and statrs above a + b = 171.6. F.DIST(.,FALSE)
  also calls `beta_density` for d <= 2. Keep F.DIST's density on the old path until it is measured, or measure it here.
- algdiv: port TOMS 708 algdiv with Excel's LN in double, as w1 did (`toms_betaln.py`). Its internals are open (T-2512),
  so the max > 8 region is a best-known model.

## Acceptance criteria
- [ ] On w1's probe1 (16,000 cells) the engine scores at least 10,301 bit-exact, and at least 10,375 on the 12,800
      modelled shapes, which are within 15 digits on at least 12,799.
- [ ] On the 2,035 old pdf cells of docs/research/t2464/beta_exact.tsv it scores at least 685 bit-exact.
- [ ] No F.DIST(.,FALSE) regression.
- [ ] No gated known-failure line is added. The probes-numeric bit-exact cells lost (r2-impact: 69) are listed in the
      Outcome.
- [ ] statistics.md's entry gains an "Implemented in" line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- min(a,b) > 1 (T-2507).
- Excel's algdiv internals (T-2512).

## Visible-parity stage
Decision 0083: this task is done when every BETA.DIST(.,FALSE) cell with min(a,b) <= 1 that it owns is within half a
unit of the 15th significant digit of Excel's value. r2-impact counts 155 such cells visibly off pre-T2464, and the
model leaves 6. Bit-level criteria still unmet then move to a follow-up task without the campaign key.
