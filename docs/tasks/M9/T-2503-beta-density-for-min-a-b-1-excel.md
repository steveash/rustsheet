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
- [x] On w1's probe1 modelled shapes the engine scores at least 12,796 bit-exact and 12,799 within 15 digits of 12,800.
- [x] On the 1,107 min(a,b) <= 1 pdf cells of docs/research/t2464/beta_exact.tsv at least 806 bit-exact and 1,103
      within 15 digits (815 once Excel's GAMMALN is exact at the shape arguments).
- [x] On t2513-r3-m1-residual-hold at least 2,804 bit-exact and 4,060 within 15 digits, and on
      t2513-verify-r3-m1-residual-hold at least 562 and 846.
- [x] No F.DIST(.,FALSE) regression.
- [x] No gated known-failure line is added. Bit-exact probes-numeric cells lost are listed in the Outcome.
- [x] statistics.md's entry gains an "Implemented in" line.
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
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up. Owner ruling 2026-10-25 (this task's session, recorded in decision 0115 by T-2504): for the beta family a caller ports Excel's pieces and may take a correctly rounded piece wherever that scores better on the oracle's visible cells; this task's M2 scores better than the accurate path (Outcome).

## Outcome
2026-10-25. `functions/statistical/beta_small_shape.rs` `small_shape_density` is M2 as statistics.md states it: tx, ty,
a-1, b-1, a+b and the GL sums are x87 ops stored to doubles, hi and lo come from two `two_sum`s, the density is
EXP(hi)·fl(1 + lo), and algdiv is TOMS 708 in doubles with Excel's LN. lnB is -LN(the other shape) whenever a = 1 or
b = 1 (the verifier's refinement). With the engine's GAMMALN, (0.37, 1) gets the same lnB either way. BETA.DIST's pdf
calls it first (`beta.rs`). F.DIST's `f_pdf_accurate` still calls `incomplete_beta::beta_density`, which is unchanged,
so no F.DIST cell moved (every changed cell below is a BETA.DIST cell).
- Fixture `test/fixtures/beta_small_shape.fixture` (the rows copied from the cases, as decision 0066 requires) and the
  test `scores_on_excels_cells`, bit-exact / within 15 digits: probe1 12,796 / 12,799 of 12,800, old cells 806 / 1,103
  of 1,107, t2513-r3-m1-residual-hold 2,810 / 4,065 of 4,096, t2513-verify-r3-m1-residual-hold 562 / 846 of 864. Each
  meets the criteria. `pinned_cells` pins one probe1 cell per lnB branch.
- Gated suites: no change (`make eval-check`: 0 fixed, 0 regressions). Nothing to record with `make eval-baseline`.
  probes' "2 new-coverage failures" is the unrelated union-walk-repeated-in-lambda timeout, and main shows it too.
- probes-numeric, per cell, main against HEAD: 40,188 fixed, 732 lost (730 within 4 ulp, 2 within 15 digits; none visibly
  off, and no type or error changed), 4,880 changed but still failing. 1,003 cells left the visibly-off set and none entered it.
  - Every lost bit-exact cell (all BETA.DIST; the tdensity, fdensity and tdist cases lose only their BETA.DIST
    cells), with ours minus Excel in ulps. By |ulp|: 1: 651, 2: 59, 3: 15, 4: 5, 5: 1, 6: 1. The two above 4 ulp are
    still within 15 digits:
    - t2464-w2f-xruns-2 (sheet Calc), 472: A12290 +1, A12294 +1, A12296 +1, A12298 +1, A12314 +1, A12316 +1, A12320 +1, A12332 +1, A12334 +1, A12336 +1, A12358 +1, A12360 +1, A12362 +1, A12372 +1, A12384 +1, A12394 +1, A12396 +1, A12424 +1, A12426 +1, A12436 +1, A12448 +1, A12464 +1, A12466 +1, A12486 +1, A12488 +1, A12490 +1, A12492 +1, A12514 +1, A12544 +1, A12554 +1, A12574 +1, A12594 +1, A12614 +1, A12616 +1, A12618 +1, A12620 +1, A12634 +1, A12638 +1, A12658 +1, A12716 +1, A12740 +1, A12742 +1, A12744 +1, A12746 +1, A12780 +1, A12832 +1, A12836 +1, A12848 +1, A12862 +1, A12874 +1, A12888 +1, A12912 +1, A12914 +1, A12940 +1, A12952 +1, A12954 +1, A12956 +1, A12964 +1, A12980 +1, A12982 +1, A13020 +1, A13032 +1, A13034 +1, A13036 +1, A13038 +1, A13042 +1, A13044 +1, A13060 +1, A13062 +1, A13064 +1, A13082 +1, A13084 +1, A13086 +1, A13098 +1, A13102 +1, A13112 +1, A13116 +1, A13124 +1, A13126 +1, A13128 +1, A13142 +1, A13162 +1, A13164 +1, A13166 +1, A13168 +1, A13172 +1, A13192 +1, A13196 +1, A13206 +1, A13222 +1, A13224 +1, A13232 +1, A13234 +1, A13236 +1, A13244 +1, A13254 +1, A13260 +1, A13262 +1, A13286 +1, A13288 +1, A13296 +1, A13300 +1, A13316 +1, A13318 +1, A13322 +1, A13324 +1, A13340 +1, A13350 +1, A13352 +1, A13364 +1, A13366 +1, A13382 +1, A13384 +1, A13388 +1, A13390 +1, A13396 +1, A13404 +1, A13416 +1, A13422 +1, A13426 +1, A13428 +1, A13430 +1, A13444 +1, A13446 +1, A13452 +1, A13468 +1, A13480 +1, A13482 +1, A13486 +1, A13510 +1, A13512 +1, A13514 +1, A13516 +1, A13520 +1, A13524 +1, A13546 +1, A13550 +1, A13576 +1, A13580 +1, A13606 +1, A13640 +1, A13644 +1, A13650 +1, A13654 +1, A13662 +1, A13680 +1, A13684 +1, A13694 +1, A13696 +1, A13706 +1, A13708 +1, A13710 +1, A13724 +1, A13726 +1, A13730 +1, A13734 +1, A13736 +1, A13748 +1, A13760 +1, A13766 +1, A13770 +1, A13772 +1, A13774 +1, A13790 +1, A13792 +1, A13804 +1, A13824 +1, A13826 +1, A13830 +1, A13854 +1, A13856 +1, A13864 +1, A13886 +1, A13890 +1, A13894 +1, A13920 +1, A13934 +1, A13984 +1, A14014 +1, A14024 +1, A14036 +1, A14050 +1, A14084 +1, A14110 +1, A14114 +1, A14130 +1, A14132 +1, A14144 +1, A14154 +1, A14170 +1, A14174 +1, A14194 +1, A14196 +1, A14234 +1, A14236 +1, A14238 +1, A14250 +1, A14252 +1, A14264 +1, A14274 +1, A14276 +1, A14278 +1, A14304 +1, A14316 +1, A14328 +1, A14330 +1, A14338 +1, A14382 +1, A14406 +1, A14412 +1, A14414 +1, A14416 +1, A14428 +1, A14468 +1, A14474 +1, A14508 +1, A14514 +1, A14516 +1, A14518 +1, A14534 +1, A14536 +1, A14546 +1, A14564 +1, A14566 +1, A14568 +1, A14570 +1, A14572 +1, A14574 +1, A14578 +1, A14580 +1, A14598 +1, A14604 +1, A14608 +1, A14610 +1, A14612 +1, A14614 +1, A14616 +1, A14618 +1, A14620 +1, A14622 +1, A14624 +1, A14634 +1, A14636 +1, A14638 +1, A14640 +1, A14642 +1, A14644 +1, A14646 +1, A14648 +1, A14666 +1, A14668 +1, A14672 +1, A14674 +1, A14676 +1, A14696 +1, A14698 +1, A14700 +1, A14710 +1, A14728 +1, A14732 +1, A14734 +1, A14788 +1, A14794 +1, A14796 +1, A14806 +1, A14828 +1, A14844 +1, A14846 +1, A14848 +1, A14850 +1, A14868 +1, A14870 +1, A14874 +1, A14880 +1, A14906 +1, A14912 +1, A14926 +1, A14932 +1, A14934 +1, A14936 +1, A14938 +1, A14948 +1, A14964 +1, A14966 +1, A14968 +1, A14970 +1, A14974 +1, A14978 +1, A14980 +1, A14982 +1, A14988 +1, A14994 +1, A15000 +1, A15004 +1, A15008 +1, A15010 +1, A15026 +1, A15032 +1, A15036 +1, A15038 +1, A15040 +1, A15042 +1, A15054 +1, A15056 +1, A15066 +1, A15086 +1, A15088 +1, A15090 +1, A15098 +1, A15100 +1, A15102 +1, A15128 +1, A15130 +1, A15132 +1, A15134 +1, A15136 +1, A15164 +1, A15188 +1, A15192 +1, A15194 +1, A15196 +1, A15198 +1, A15206 +1, A15208 +1, A15252 +1, A15254 +1, A15266 +1, A15270 +1, A15294 +1, A15308 +1, A15314 +1, A15316 +1, A15326 +1, A15328 +1, A15332 +1, A15334 +1, A15336 +1, A15338 +1, A15344 +1, A15348 +1, A15364 +1, A15366 +1, A15376 +1, A15378 +1, A15394 +1, A15396 +1, A15408 +1, A15410 +1, A15414 +1, A15424 +1, A15446 +1, A15458 +1, A15464 +1, A15478 +1, A15480 +1, A15508 +1, A15512 +1, A15528 +1, A15538 +1, A15540 +1, A15544 +1, A15546 +1, A15554 +1, A15560 +1, A15568 +1, A15574 +1, A15576 +1, A15578 +1, A15582 +1, A15588 +1, A15590 +1, A15592 +1, A15604 +1, A15610 +1, A15636 +1, A15644 +1, A15658 +1, A15674 +1, A15676 +1, A15684 +1, A15690 +1, A15704 +1, A15706 +1, A15708 +1, A15710 +1, A15712 +1, A15718 +1, A15724 +1, A15734 +1, A15738 +1, A15740 +1, A15758 +1, A15760 +1, A15762 +1, A15768 +1, A15772 +1, A15774 +1, A15776 +1, A15788 +1, A15798 +1, A15800 +1, A15804 +1, A15806 +1, A15814 +1, A15820 +1, A15848 +1, A15854 +1, A15874 +1, A15876 +1, A15888 +1, A15890 +1, A15904 +1, A15906 +1, A15918 +1, A15934 +1, A15936 +1, A15938 +1, A15940 +1, A15942 +1, A15950 +1, A15956 +1, A15964 +1, A15966 +1, A15968 +1, A15970 +1, A15972 +1, A15978 +1, A15984 +1, A15994 +1, A16018 +1, A16020 +1, A16048 +1, A16054 +1, A16080 +1, A16086 +1, A16108 +1, A16114 +1, A16118 +1, A16120 +1, A16128 +1, A16134 +1, A16136 +1, A16148 +1, A16150 +1, A16166 +1, A16172 +1, A16178 +1, A16184 +1, A16210 +1, A16216 +1, A16244 +1, A16248 +1, A16250 +1, A16258 +1, A16264 +1, A16278 +1, A16280 +1, A16298 +1, A16300 +1, A16302 +1, A16308 +1, A16314 +1, A16328 +1, A16330 +1, A16332 +1, A16340 +1, A16346 +1, A16348 +1, A16350 +1, A16352 +1, A16362 +1, A16374 +1
    - t2513-r3-tdensity-a2 (sheet Calc), 71: F810 +1, F811 +1, F812 +1, F813 +1, F814 +1, F815 +1, F816 +1, F817 +1, F818 +1, F906 +1, F907 +1, F910 +1, F911 +1, F920 +1, F946 +1, F947 +1, F953 +1, F974 +1, F979 +1, F980 +1, F986 +1, F1005 +1, F1056 +1, F1073 +1, F1087 -1, F1088 -2, F1095 +1, F1097 -3, F1108 -2, F1115 -2, F1149 -1, F1169 -1, F1180 -2, F1195 -3, F1196 +2, F1221 +3, F1222 -2, F1229 +1, F1252 -2, F1288 -1, F1291 +1, F1293 -3, F1299 +3, F1320 -1, H331 -1, H705 +1, H789 +1, H791 +1, H803 +1, H804 +1, H809 +1, H810 +1, H814 +1, H825 +1, H831 +1, H836 +1, H906 +1, H920 +1, H939 +1, H940 +1, H946 +1, H991 +1, H992 +1, H1084 +1, H1146 +2, H1174 +1, H1183 -1, H1195 -3, H1272 +1, H1314 +1, H1320 -1
    - t2513-r3-m1-residual-hold (sheet Calc), 52: D1012 +1, D1023 +1, D1027 +2, D1034 +3, D1046 +2, D1539 +3, D1541 +2, D1572 +2, D1582 +4, D1880 -2, D1900 -1, D1906 -1, D1920 -2, D2044 -2, D2128 -1, D2134 -2, D2142 -1, D2146 -2, D2152 -1, D2158 -1, D2563 +1, D2565 +1, D2567 +1, D2571 +1, D2572 +1, D2573 +1, D2574 +1, D2581 +1, D2589 +1, D2591 +1, D2592 +1, D2593 +1, D2594 +1, D2596 +1, D2598 +1, D2600 +1, D2791 +1, D2983 +2, D3042 -1, D3046 -1, D3047 -1, D3051 -1, D3058 -1, D3060 -1, D3063 -1, D3076 -1, D3619 +1, D3628 +2, D3645 +1, D3651 +1, D3681 +2, D3818 -1
    - t0969-beta-switch (sheet Pdf), 30: C63 -2, C68 +2, C69 -1, C76 -1, C80 +1, C84 -2, C86 -3, C94 -2, C147 -1, C150 -1, C152 +1, C160 -1, C247 -2, C259 -2, C261 -3, C287 -3, D53 -1, D55 +2, D56 +2, D65 -1, D70 +2, D85 +2, D149 +1, D154 +1, D159 +1, D176 +1, D188 +2, D193 +1, D244 -2, D285 +1
    - t2464-r2-tdist-p1 (sheet Calc), 30: A87 +1, A91 +1, A93 +1, A101 +1, A113 +1, A117 +1, A129 +1, A132 +1, A137 +1, A165 +1, A182 +2, A184 +1, A188 +1, A227 +1, A235 +1, A247 +1, A248 +1, A266 +2, A268 +1, A282 +1, A681 +2, A693 +1, A695 +1, A704 +1, A709 +1, A710 +1, A711 +1, A713 +1, A714 +1, A715 +1
    - t2513-r3-m1-residual-a (sheet Calc), 25: D1013 +1, D1029 +1, D1039 +1, D1045 +2, D1076 -1, D1138 -4, D1468 +2, D1474 +1, D1504 -4, D2707 -1, D2720 -1, D3229 +1, D3232 +1, D3401 +4, D3424 +2, D3906 +2, D3907 +1, D3912 +2, D3913 +2, D3926 +2, D3927 +2, D3933 +2, D3962 +1, D3968 +2, D3969 +2
    - t2513-verify-r3-m1-residual-hold (sheet Calc), 20: B82 +1, B86 +1, B91 +1, B96 +1, B98 +1, B102 +1, B104 +1, B105 +1, B108 +1, B197 -1, B200 -1, B203 -2, B205 -2, B206 -1, B210 -1, B213 -1, B215 -1, B216 -1, B603 -1, B611 -1
    - t0969-beta-density-b (sheet Pdf), 9: C87 +2, C101 +1, C104 -1, C109 +2, C120 +1, C132 -1, C141 +1, C170 -1, C216 -2
    - t2513-r3-tdensity-hold (sheet Calc), 9: F1168 +6, F1173 +1, F1174 -2, F1176 -3, F1178 +2, F1182 +1, F1228 +4, F1250 +2, H1214 +2
    - t2464-w2f-absweep (sheet Calc), 6: A2118 +1, A2300 +1, A2588 +2, A2812 -1, A3026 +1, A6226 +5
    - t2513-r3-fdensity-a (sheet Calc), 5: C373 +2, C385 +3, C386 +3, D373 +2, D386 +3
    - t0969-t-beta-link (sheet Link), 2: K142 +1, K161 +2
    - t0969-beta-prefactor (sheet Pref), 1: E113 -1
  - By lnB branch: GAMMALN form 549 lost (12,928 fixed), algdiv 181 lost (2,322 fixed), min = 1 none lost (24,793
    fixed), 2 unclassified.
  - Cause: the engine's GAMMALN. (0.5, 0.7), 472 of the losses, needs GAMMALN(1.2), and the engine's value is 1 ulp
    above Excel's (-0.08537409000331582 against ...583, T-2122's K3). A scratch build that used Excel's measured
    GAMMALN at the 569 arguments of t2513-r3-m1-residual-gl leaves 208 losses (GL branch 25, algdiv 181) and 41,700
    fixed. What remains is algdiv (T-2512): (15, 0.5) 53, (0.5, 15) 20, (0.5, 25) 5 and others, plus GL shapes whose
    GAMMALN arguments were not measured, such as (0.48, 1.05) and (0.36, 0.82).
- Visible parity: the only BETA.DIST(.,FALSE) cells with min(a,b) <= 1 still visibly off in probes-numeric are 3 cells
  of t2513-r3-m1-residual-hold (D863, D875, D957, a = 1 with b = 9.1e6 or 4e8). There Excel's graph value is 0 and ours
  is a subnormal. Under decision 0042 the graph reports a kept subnormal as 0, and these cells have no (x)=0 companion,
  so whether Excel flushes them is not known. Left as is. (main had 1,007 such cells.)
- Not done: reviewers, PR, `make check` in CI.

