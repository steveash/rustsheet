---
id: T-2503
title: "Beta density for min(a,b) <= 1: Excel's M2 composition (lnB from Excel's GAMMALN, a double-double exponent split) (T-2464 E2, T-2513)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: M
tier: opus
campaign: accuracy
depends_on: [T-2645]
---

## Goal
BETA.DIST(x,a,b,FALSE) and BETADIST's density, for min(a,b) <= 1, follow Excel's composition M2 (T-2513, replacing
T-2464's M1). The same function is the prefactor density that bup needs (T-2506).

## Context
- T-2464 research, step E2 of docs/research/t2464/r2-impact/RESULTS.md section 4; refined by T-2513 round 3
  (docs/research/t2464/r3-m1-residual, verifier-confirmed in verify-r3-m1-residual).
- The rule is docs/excel-semantics/statistics.md "Beta density for min(a,b) <= 1: lnB and a double-double exponent split
  (T-2464, T-2513)":
  - lnx = LN(x); ly = Excel's log1p(-x) for x <= 0.375 (T-2645), LN(fl(1 - x)) above;
  - tx = fl(fl(a-1)·lnx) and ty = fl(fl(b-1)·ly), x87 ops stored to doubles;
  - lnB:
    - a = 1 or b = 1: -LN(the other shape), at any size (a < 1 with b = 1 rests on one shape, (0.37, 1));
    - min < 1, max <= 8 (8 inclusive): fl(GL(min) + fl(GL(max) - GL(a+b))), GL = GAMMALN;
    - min < 1, max > 8: fl(GL(min) + algdiv(min, max)), TOMS 708 algdiv in doubles with Excel's LN (best known; T-2512);
  - hi = fl(fl(tx + ty) - lnB), lo = the exact remainder (tx + ty - lnB) - hi (two-sum of the two additions);
  - density = fl(EXP(hi)·fl(1 + lo)), Excel's EXP of the double hi. No extended EXP is needed here, and b = 1 is no
    longer a special case.
- Models: `docs/research/t2464/r3-m1-residual/final_score.py` (`lnb_m2`, frozen M2) on `m1lib.py`; verifier's
  `verify-r3-m1-residual/vlib.py`.
- Scores, exact / 4 ulp / 15 digits (verifier `vscore_all.out`), with Excel's measured GAMMALN, then with the engine's
  GAMMALN model:
  - w1's probe1 modelled shapes (12,800): 12,796/12,799/12,799 both ways (M1 10,375);
  - old pdf cells with min <= 1 (1,107 of beta_exact.tsv's 2,035): 815/1,101/1,103, engine GL 806 exact (M1 566);
  - worker hold-out t2513-r3-m1-residual-hold (4,096): 3,106/3,991/4,062, engine GL 2,804 exact;
  - verifier hold-out t2513-verify-r3-m1-residual-hold (864): 649/816/846, engine GL 562 exact.
- **GAMMALN dependency.** The bit-exact counts need Excel's GAMMALN at the shape arguments. The engine model differs on
  14 of 47 arguments below 1 (probe t2513-r3-m1-residual-gl; verifier 24 of 66), which is T-2122's K3 and (1.5, 2]
  work. The 15-digit counts do not change. Ship with the engine's GAMMALN; the measured-GL counts become reachable
  when T-2122's coefficient swap lands.
- r2-impact's expected change (for M1; M2 only adds bit-exact cells): -148 visibly-off pre-T2464 cells (-81 hold-out)
  and -410 in the w2f probes; 1 broke.
- Code: `incomplete_beta::beta_density`, today powf × tgamma ratio, and statrs above a + b = 171.6. F.DIST(.,FALSE)
  calls `beta_density` through `fisher.rs` `f_pdf`, and T-2513 found that Excel's F density is not the beta density
  times a Jacobian: keep F.DIST's density on the old path, or measure it here.
- Code must not read docs/ (decision 0066): copy the rows the tests need into a fixture under the engine crate.

## Acceptance criteria
- [ ] On w1's probe1 modelled shapes the engine scores at least 12,796 bit-exact and 12,799 within 15 digits of 12,800.
- [ ] On the 1,107 min(a,b) <= 1 pdf cells of docs/research/t2464/beta_exact.tsv at least 806 bit-exact and 1,103
      within 15 digits (815 once Excel's GAMMALN is exact at the shape arguments).
- [ ] On t2513-r3-m1-residual-hold at least 2,804 bit-exact and 4,060 within 15 digits, and on
      t2513-verify-r3-m1-residual-hold at least 562 and 846.
- [ ] No F.DIST(.,FALSE) regression.
- [ ] No gated known-failure line is added. Bit-exact probes-numeric cells lost are listed in the Outcome.
- [ ] statistics.md's entry gains an "Implemented in" line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- min(a,b) > 1 (T-2507).
- Excel's algdiv internals (T-2512).

## Visible-parity stage
Decision 0083: this task is done when every BETA.DIST(.,FALSE) cell with min(a,b) <= 1 that it owns is within half a
unit of the 15th significant digit of Excel's value. r2-impact counts 155 such cells visibly off pre-T2464, and M1
left 6 (M2 leaves 4 of the 1,107 old cells). Bit-level criteria still unmet then move to a follow-up task without the
campaign key.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.
