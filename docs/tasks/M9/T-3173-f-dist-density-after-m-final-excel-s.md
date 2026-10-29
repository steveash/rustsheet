---
id: T-3173
title: "F.DIST density after M-final: Excel's power term c·LN x where |c·LN x| >= 4 (about 18,070 unowned visibly-off probes-numeric cells, T-2586's open item)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2586]
campaign: visible
---

## Goal
`F.DIST(x,d1,d2,FALSE)` reproduces the stored-sum structure of Excel's power term P = c·LN x (c = d1/2 on side 1,
-d2/2 on side 2, as in M-final) wherever |P| >= 4, so the F density's visibly-off cells left in the T-2220, T-2513
and T-2586 sweeps after T-3034 fall to the share the new model measures, with no gated loss.

## Context
- Gap triage 2026-10-29 (`make eval` on main a2ecf005b, excel-graph; `python3 scripts/exit_owners.py --all-off`):
  18,293 visibly-off F.DIST density cells have no open owner. T-3034 (done) shipped T-2586's frozen model M-final
  (`crates/rustsheet-engine/src/functions/statistical/f_density.rs` `excel_f_density`) and took the F density's
  visibly-off count in probes-numeric from 38,027 to 18,722; what is left was "T-2586's open residue", which no
  engine task owned.
- Classified with M-final's own regions (`docs/research/t2464/r5-f-final/final_score.py` `region`: side and r from
  fl(m·x) against n, P = c·LN x, df class small min(d) < 5, mid < 16, large otherwise), the 18,293 cells are:
  |P| >= 32: 15,334 (large df 12,823, small 2,036, mid 475; 612 of them edge x below 1E-6 or above 1E6);
  |P| in [4, 32): 2,318 (large 1,899, small 231, mid 188); the 421 `F.DIST(...)*2^600` scaled edge cells of the
  r5-f-edges cases (|P| large); and 220 cells with |P| < 4, which are T-3174's (the constant K, not P). This task owns
  the other 18,073. Error size against Excel, all 18,293: 5,452 within 1 unit of the 15th digit, 4,970 within 2,
  4,514 within 5, 2,849 within 20, 508 beyond.
- This is the exact pattern r5-f-final's RESULTS.md section 4 measured on the research rows: parity 99-100% at
  |P| < 4, 87% in [4, 32), 36% in [32, 256), 10% above, with the error tracking ulp(P) at 1.35-2 times one
  rounding's spread, so at least one stored double at |P| scale is missing or misplaced. T-2586 item 6 and its round
  5 Progress line name the next step (Excel's exponential step for this density: exp2 split, x87 F2XM1/FSCALE,
  value-space x^(a-1)·(1+r)^-h with per-factor pow roundings, probed at (2,2) side 1 where E = LN(x) - 2·LN(fl(1+x))
  has no other rounding). That is research work with new probes; this task ports its result. Needs T-2586's model
  for the power term: no form fitted here blind (decision 0056).
- Examples (Excel vs ours):
  - probes-numeric/t2220-f-dist-sweep-pdf Calc!A1000 `F.DIST(100,100,100,FALSE)`: 9.32516532636366e-73 vs
    9.325165326363726e-73 (|P| 230)
  - probes-numeric/t2220-f-dist-sweep-pdf Calc!A711 `F.DIST(0.1,20,2,FALSE)`: 0.048828125000000173 vs
    0.04882812500000023 (exact 0.048828125: both sides are inaccurate, so the stage is `visible`)
  - probes-numeric/t2586-r5-f-edges-a Calc!B1037 `F.DIST(1E100,2,1,FALSE)`: 3.535533905932695e-151 vs
    3.5355339059327177e-151 (closed form (1+2x)^-1.5 = 3.5355339059327378e-151)
  - probes-numeric/t2513-r3-fdensity-hold Calc!B1401 `F.DIST(A1401,60,200,FALSE)`: 3.2775050873197982e-43 vs
    3.277505087319815e-43
- Rule: docs/excel-semantics/statistics.md "F density is not the beta density, R's df or a GAMMALN formula (T-2513)"
  and its T-2586 round 4 and 5 and T-3034 updates. Research: docs/research/t2464/r5-f-final/ (RESULTS.md sections
  4-6, `fdensity_excel.py`), r4-f-power, r5-f-smallr, r5-f-edges. Engine code must not read docs/ (decision 0066).

### Cases owned
F.DIST density cells per case at this triage, the visibly-off count without T-3174's |P| < 4 cells (of all
visibly-off F.DIST cells in the case):
- probes-numeric/t2220-f-dist-sweep-pdf: 52 (of 58); disjoint from T-2790's 89 named cells (the one
  shared address, A797, is one T-2790 lists as not its target)
- probes-numeric/t2513-r3-fdensity-hold: 176 (of 176)
- probes-numeric/t2513-verify-r3-fdensity-hold: 174 (of 174)
- probes-numeric/t2586-r4-f-brcomp-big: 570 (of 570)
- probes-numeric/t2586-r4-f-brcomp-hold: 302 (of 323)
- probes-numeric/t2586-r4-f-power-a: 2790 (of 2790)
- probes-numeric/t2586-r4-f-power-hold: 767 (of 767)
- probes-numeric/t2586-r4-f-small-df-a: 907 (of 944)
- probes-numeric/t2586-r4-f-small-df-hold: 285 (of 309)
- probes-numeric/t2586-r5-f-bigconst-a: 1625 (of 1625)
- probes-numeric/t2586-r5-f-bigconst-hold: 596 (of 596)
- probes-numeric/t2586-r5-f-combine-a: 1320 (of 1320)
- probes-numeric/t2586-r5-f-combine-hold: 236 (of 236)
- probes-numeric/t2586-r5-f-edges-a: 645 (of 645)
- probes-numeric/t2586-r5-f-edges-hold: 394 (of 412)
- probes-numeric/t2586-r5-f-final-hold: 813 (of 847)
- probes-numeric/t2586-r5-f-smallr-a: 3450 (of 3451)
- probes-numeric/t2586-r5-f-smallr-hold: 657 (of 657)
- probes-numeric/t2586-verify-r4-f-brcomp-hold: 161 (of 161)
- probes-numeric/t2586-verify-r4-f-power-hold: 480 (of 480)
- probes-numeric/t2586-verify-r4-f-small-df-hold: 6 (of 6)
- probes-numeric/t2586-verify-r5-f-bigconst-hold: 332 (of 335)
- probes-numeric/t2586-verify-r5-f-combine-hold: 247 (of 247)
- probes-numeric/t2586-verify-r5-f-edges-hold: 339 (of 391)
- probes-numeric/t2586-verify-r5-f-final-hold: 415 (of 439)
- probes-numeric/t2586-verify-r5-f-smallr-hold: 334 (of 334)

## Acceptance criteria
- [ ] `excel_f_density` computes the power term the way T-2586's model for it does, with unit tests pinning
      bit-exact rows from that model's untouched hold-out in a fixture.
- [ ] `make eval`: the visibly-off F.DIST density cells of the cases above fall by at least the margin T-2586's
      model shows over M-final on its hold-out (stated in the Outcome with both counts), with no new known-failure line
      in any eval/baseline file (`make eval-baseline`) and no gated loss (`make eval-check`). Any region where the new
      form loses visible cells against M-final keeps M-final, with the comparison in the Outcome.
- [ ] statistics.md's F density entry says which form the engine ships at |P| >= 4 and the measured counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The constant K at |P| < 4 (T-3174), the F cdf and tails (T-2508, T-2510, T-2219), huge df (T-2642), tiny x where
  Y = density·x is subnormal (T-2647).
- Bit-exactness: misses left within 15 digits go to a `defer: phase-1b` follow-up.

## Visible-parity stage
Decision 0114 `visible`: Excel is the inaccurate side (A711 above is 3.5E-15 off the exact 0.048828125), so the work is
reproducing Excel's chain, never a more accurate density. Target: the owned cells within half a unit of Excel's 15th
significant digit as far as T-2586's power-term model reaches; the measured remainder is named in the Outcome.
