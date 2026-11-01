---
id: T-3061
title: "Student-t quantile: Excel's T.INV is SQRT(df*(1/y-1)) of the rounded beta quantile y (reproduces 75 visibly-off T.INV/T.INV.2T/TINV/CONFIDENCE.T cells, df 3 to 1E10)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Away from the centre, T.INV, T.INV.2T, TINV and CONFIDENCE.T return Excel's quantile, the back-transform
`SQRT(df*(1/y-1))` of the rounded beta quantile y, and not the exact quantile. This brings the 75 cells listed
below within 15 digits, most of them bit-exact, from df 3 to 1E10.

## Visible-parity stage
`visible`: Excel is the inaccurate one. Ours is the exact quantile on these cells: 1.8E-17 relative at
`T.INV(0.9,1E10)` and 3E-17 at `T.INV(0.9,1E6)`, by mpmath. Excel is up to 3E-7 relative away. The fix reproduces
Excel's form; it is not an accuracy fix. Target: the 75 cells below (60 with the correctly rounded y, 15 needing Excel's own y) within half a unit of the 15th significant digit
of Excel's value. Time-boxed (decision 0056).

## Context
- Gap triage 2026-10-28 (opus queue refill, origin/main 3d4b480b). The rule is in docs/excel-semantics/statistics.md,
  "Student-t quantile is SQRT(df*(1/y-1)) of the rounded beta quantile, away from the centre". In short:
  - Take t* = the exact quantile (mpmath) and y = the double nearest df/(df+t*^2).
  - Then `fl(sqrt(fl(df*fl(fl(1/y)-1))))` gives Excel's value exactly (0 ulp) on 60 of the 75 cells.
  - The other 15 fit, at 0 ulp (at most 1 ulp on CONFIDENCE.T, whose `s/sqrt(n)` scaling rounds again), when y is
    one ulp from the nearest double. Excel's beta quantile is within an ulp of correctly rounded, not always
    correctly rounded:
    - y one ulp up: `T.INV.2T(0.1,1E10)` (m9-t-edges A38, m9-t-nonint-df A60, t1518-t-df-cut A17),
      `T.INV.2T(0.1,9999999999)` (m9-t-nonint-df A66), `T.INV.2T(0.9,1E6)` (m9-t-inv-2t A126),
      `CONFIDENCE.T(0.1,*,1000)` (m9-confidence-t A35 A40 A45);
    - y one ulp down: `T.INV(0.4,5)` (stat-argument-modes H518), `T.INV.2T(0.75,1000)` (m9-t-inv-2t A112);
    - nearest or one ulp down: `CONFIDENCE.T(0.05,*,1000)` (A5 A10 A15), `CONFIDENCE.T(1E-20,1,100001)`
      (t1531 A114), `CONFIDENCE.T(0.05,1,1E10)` (t1518 A23).

    At df 1E10 one ulp of y is about 3E9 to 4E9 ulps of t, so these cells need Excel's own y. Check first whether that y
    is the crossing of the ported incomplete beta (T-2511 inverts to that crossing for T.INV) or Excel's T.DIST
    kernel. If no rule fits them, ship the 60 and file the 15 as a follow-up with this evidence.
  - The script is not in the repo; rebuild it from this description.
  - One ulp of y moves t by about df·eps/t^2 relative, which is billions of ulps at df 1E10. So y has to be the
    correctly rounded value, for example from a double-double t*^2/df: n + t^2 in doubles is not enough at 1E10.
- Why no owner could take it: T-0967 owns most of these cells (Student-t quantile parity, P1). T-0967 depends on
  T-0969, which is blocked (time-boxed), and on T-2511 (todo). t1518-t-df-cut is owned only by T-0969 itself. The
  root cause here needs neither: it uses the exact quantile, which the engine already has at large df, plus Excel's
  back-transform. Coordinate with T-0967, which keeps its other cells (central last digits, tiny p, p near 1).
- Cells (excel-graph; `make eval-show CASE=<case>`), all fitted at 0 to 1 ulp:
  - probes-numeric/m9-t-inv.case.json: Calc!A108 A109 A113 A114 (df 1000), A118-A123 A125-A130 (df 1E6)
  - probes-numeric/m9-t-inv-2t.case.json: Calc!A35 A61 A74 A87 A99 A100 (p = 0.9 or 0.75 at df 3 to 100), A107
    A109 A110 A112 A113 (df 1000), A118-A126 (df 1E6)
  - probes-numeric/m9-confidence-t.case.json: Calc!A5 A10 A15 A35 A40 A45 (n = 1000)
  - probes-numeric/m9-t-nonint-df.case.json: Calc!A59 A60 A65 A66; probes-numeric/m9-t-edges.case.json: Calc!A38
  - probes-numeric/t1048-tinv.case.json: Calc!A21 B21; probes-numeric/t1518-t-df-cut.case.json: Calc!A14 A15 A17
    A20 A23
  - probes-numeric/t1531-t-tiny-p.case.json: Calc!A77 A78 A79 A88 A89 A90 A101 A102 A103 A112 A113 A114 A115 A116
    A117 (df 99999 and 1E5, p from 1E-20 to 1E-10)
  - probes/stat-argument-modes.case.json: H518 (`T.INV(0.4,5)`)
  - gen-m9: 1b31092a0d79e5f1 A275, 2138b909d8f810ea A241, 41248d1cb153c66e A113, 5919660b7bb4d195 A53,
    775d053de498b8c7 A176
- Examples, Excel vs ours vs exact:

  | cell | formula | Excel | ours | exact (mpmath) |
  |---|---|---|---|---|
  | t1518-t-df-cut A15 | `T.INV(0.9,10000000000)` | 1.2815518424115688 | 1.281551565629259 | 1.2815515656292591 |
  | m9-t-edges A38 | `T.INV.2T(0.1,1E10)` | 1.6448533237762966 | 1.6448536271038494 | 1.6448536271038492 |
  | t1518-t-df-cut A20 | `CONFIDENCE.T(0.05,1,10000000001)` | 1.9599644085189063E-5 | 1.9599639846792828E-5 | (the T.INV.2T quantile / sqrt(n)) |
  | m9-t-inv A126 | `T.INV(0.9,1E6)` | 1.2815524120978465 | 1.2815524121299386 | 1.2815524121299386 |
  | m9-t-inv A113 | `T.INV(0.9,1000)` | 1.2823987214609143 | 1.2823987214607573 | 1.2823987214609246 |

  At df 1000 ours is also 1.3E-13 off the exact value, so the y must come from an accurate quantile, not from today's
  statrs inverse.
- Where the form stops:
  - Near the centre it does not hold. At two-tailed p = 0.99 and 0.999 (one-sided q = 0.495 and 0.4995;
    m9-t-inv-2t A10, A23, A36, A49, A62, A75, A88, A101, A114, A115, A127, A128) Excel is within 1E-15 of the exact
    quantile, and the form is thousands of ulps off. The switch is between q = 0.45 (fits at every df) and 0.495.
    Keep today's path above q = 0.45. T-2724 owns the centre. If pinning the switch needs more points, file a
    research-lane sweep of T.INV(p, df) for p in (0.55, 0.995) at df 3, 30, 1000 and 1E6, and do not fit it blind.
  - Tiny q at a small df: at `T.INV.2T(1E-16,30)` (gen-m9 583ca2baea0bb30a A147) Excel's t is 2E-12 off (its y about 4E-12). That
    is T-1542, out of scope. The q = 1E-10 to 1E-20 cells at df 99999 and 1E5 do fit.
  - df 1 and 2: the form also gives the passing df 1 and 2 cells to 0 or 1 ulp (q up to 0.45). Do not let
    them regress.
- Engine: crates/rustsheet-engine/src/functions/statistical/t_dist.rs:
  - `t_quantile`: below `T_QUANTILE_SERIES_DF` = 1E5, statrs' `t_inverse_cdf` (inverse_beta.rs, which
    back-transforms with sqrt(df(1-y)/y)) or `t_tail_quantile` for q < `T_TAIL_P`; from 1E5, the normal quantile plus
    A&S 26.7.5.
  - `t_upper_quantile` serves T.INV.2T/TINV. CONFIDENCE.T is `fn_confidence_t` in normal.rs.
  - Keep directive 5: the work stays bounded, and the lift table charges `T_QUANTILE_WEIGHT`.

- T-2511 (2026-10-31) implemented this rule (`t_dist::t_rounded_y_quantile`): 65 of the 75 cells are within 15
  digits. Left: probes-numeric m9-t-inv-2t A112, A126; m9-confidence-t A35, A40, A45; m9-t-nonint-df A60, A66;
  m9-t-edges A38; t1518-t-df-cut A17; probes stat-argument-modes H518. At q = 0.05 Excel's y is about 2 ulps from the
  nearest crossing (statistics.md); the bit-level remainder is T-3399.

## Acceptance criteria
- [ ] At least the 60 cells that fit with the correctly rounded y are within half a unit of the 15th significant
      digit of Excel's value in `make eval`, bit-exact where the fit says 0 ulp. The 15 that need Excel's own y pass
      too, or go to a filed follow-up task that lists them with the y evidence.
- [ ] Their known-failure lines disappear from eval/baseline/probes.txt (H518) and eval/baseline/gen-m9.txt (the
      five gen-m9 cells). probes-numeric is ungated (`gate = false`), so its cells are checked in the report. No
      new known-failure line appears in any suite (`make eval-baseline` refuses regressions).
- [ ] Unit tests in t_dist.rs pin `T.INV(0.9,1E10)` = 1.2815518424115688, `T.INV.2T(0.1,1E10)` = 1.6448533237762966
      and `T.INV(0.9,1000)` = 1.2823987214609143 (excel-graph values above).
- [ ] statistics.md's entry gets "Implemented in" and the region actually used.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The centre (q above 0.45): T-2724.
- Tiny p where Excel's y is itself off, and df at least 99999 at p of 1E-200 or below: T-1542.
- The Student t cdf (T-0969) and the inversions of the other beta-family quantiles (T-2511).
