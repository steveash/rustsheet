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

More cells (gap triage 2026-10-30c, `make eval` on main 40ca52179, `python3 scripts/exit_owners.py --all-off`):
212 visibly-off F.DIST density cells of the F-density probe at large df ((100,100), (50,100), (20,100), (50,50),
(2,10), (10,30)). Their only owner was T-2821, which is `defer: phase-1b`. Every one has |P| = |c·LN x| >= 4 by
M-final's side rule, so they are this task's, not T-3174's:
- probes-numeric/t2513-r3-fdensity-a.case.json (212): Calc!B695, B696, B697, B698, B699, B712, B713, B714, B715, B716, B2125, B2126, B2127, B2153, B2154, B2155, B2156, B2157, B2356, B2392, B2394, B2395, B2404, B2405, B2406, B2407, B2413, B2414, B2415, B2416, B2417, B2418, B2419, B2420, B2421, B2422, B2423, B2424, B2430, B2431, B2432, B2433, B2434, B2435, B2436, B2437, B2441, B2442, B2443, B2444, B2445, B2446, B2447, B2448, B2657, B2659, B2661, B2662, B2684, B2694, B2695, B2696, B2697, B2698, B2699, B2700, B2701, B2702, B2703, B2710, B2711, B2712, B2713, B2714, B2715, B2716, B2717, B2724, B2725, B2726, B2727, B2728, B2729, B2730, B2731, B2732, B2733, B2737, B2743, B2744, B2745, B2752, B2753, B2759, B2763, B2764, B2771, B2772, B2779, B2780, B2781, B2833, B2834, B2835, B2836, B2842, B2843, B2844, B2845, B2846, B2847, B2848, B2853, B2854, B2855, B2856, B2857, B2858, B2863, B2864, B2865, B2866, B2867, B2873, B2874, B2875, B2876, B2877, B2878, B2879, B3025, B3026, B3027, B3028, B3029, B3031, B3032, B3034, B3035, B3036, B3037, B3038, B3042, B3043, B3044, B3047, B3048, B3049, B3054, B3055, B3056, B3057, B3059, B3060, B3061, B3062, B3063, B3064, B3066, B3067, B3068, B3069, B3070, B3072, B3078, B3079, B3080, B3081, B3082, B3083, B3084, B3085, B3086, B3099, B3100, B3101, B3102, B3103, B3104, B3106, B3107, B3114, B3124, B3125, B3126, B3127, B3128, B3129, B3130, B3131, B3132, B3133, B3134, B3135, B3136, B3137, B3138, B3139, B3140, B3141, B3142, B3143, B3144, B3145, B3146, B3154, B3155, B3156, B3157, B3158, B3159, B3168

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
