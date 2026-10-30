---
id: T-2511
title: "BETA.INV, F.INV, F.INV.RT and T.INV invert the ported incomplete beta to the crossing (T-2464 E8)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2505, T-2506, T-2508, T-2509, T-2510]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
BETA.INV, BETAINV, F.INV, F.INV.RT, FINV, T.INV, T.INV.2T and TINV find the crossing of the engine's ported CDF. The
crossing is T-1745's rule: Excel's x is a crossing of its own CDF. This replaces statrs' AS 64/109 Newton on statrs'
`checked_beta_reg`.

## Context
- T-2977: F.INV, F.INV.RT and FINV search `fisher.rs::f_quantile_tail` (the unoriented tail,
  `unoriented_shape_one_tail` at a shape of 1); delete both once they invert the ported F tail. Searching the ported
  tail as of T-2977 fixes 9 gen-m9 cells and moves 9 gated cells 2 to 8 ulp off (T-2977 Outcome).
- T-2464 research, step E8 of docs/research/t2464/r2-impact/RESULTS.md section 4 (`inverse.py` → `inverse.out`).
- Rule: docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)". That holds at 155
  of 188 rows; 11 miss by 1 to 188 ulps of p, which suggests a tolerance stop that is not established.
- r2-impact's estimate on 1,207 numeric inverse cells (778 visibly off in the engine):
  - a bisection to the crossing of the BEST+log1p CDF fixes 418 and breaks 115, and is bit-exact on 310;
  - per function: BETA.INV 185/243, BETAINV 10/10, F.INV 43/100, F.INV.RT 56/102, FINV 47/80, T.INV 29/97,
    T.INV.2T 43/125, TINV 5/21.
  - The estimate is rough: the model CDF was wrong on the T bgrat cells then (T-2509 fixes them), and Excel's start
    and stop rule is unknown.
- Overlap:
  - T-0974 (done 2026-10-23) already solves F.INV, F.INV.RT and FINV as the crossing of F.DIST's own tails with
    `special::refine_increasing` (statistics.md "F.INV is a crossing of F.DIST's tails"). Reuse that helper for
    BETA.INV and T.INV* on the ported CDF instead of writing a second search; F.INV moves when its CDF does.
  - T-0967 (T.INV, T.INV.2T, CONFIDENCE.T) and T-1871 (statrs saturation and panics) own the remaining cells.
    Re-triage them in the Outcome and close what this settles.
  - T-2119 (done) already refines BETA.INV to its crossing on today's kernel; its leftover chained cells are T-2633's
    (decision 0111).
  - T-3007 (2026-10-29): at a shape of 1 outside the closed forms BETA.DIST's w1 is now (0.5 - w) + 0.5, a step of one
    ulp of w, so the crossing BETA.INV searches is flat there: `BETA.INV(0.999,1,100)`, `BETA.INV(0.001,100,1)`
    and `BETA.INV(0.95,1,100)` (probes-numeric/m9-beta-inv A69, A149, A68; t1745-beta-inv-roundtrip J67, J144, J66) went
    from -4, +1 and -16 ulp to -40, -5 and -6 ulp off Excel's quantile, while Excel's BETA.DIST at those quantiles
    became bit-exact. Find where in the step Excel's search stops
    (statistics.md "Incomplete beta at a shape of 1 outside the closed forms").
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - gen-m9/eced4bb6a0234c9e.case.json: Calc!A291
  - gen-m9/320d51765af9b6b7.case.json: Calc!A379, A15
  - gen-m9/589f42e67f4b4b8d.case.json: Calc!A365
  - gen-m9/60b0b6e69ad35a45.case.json: Calc!A197, A150
  - gen-m9/c18dbf21f11093f3.case.json: Calc!A92
  - gen-m9/e6cf3b20a014e6a3.case.json: Calc!A135
  - gen-m9/f34f66fb7f81e673.case.json: Calc!A209
  - gen-m9/7674c89c824aca43.case.json: Calc!A212
  - gen-m9/40425341c5034eed.case.json: Calc!A108
  - gen-m9/a63d1c7d6f96c56d.case.json: Calc!A320
  - gen-m9/c6cfda6bd10cae93.case.json: Calc!A183

## Acceptance criteria
- [ ] Every listed inverse calls one crossing search on the ported CDF (no statrs inversion left).
- [ ] T-1745's probes-numeric t1745-beta-inv-roundtrip: at least 155 of 188 rows land on a crossing of the engine's
      CDF.
- [ ] The search is bounded, with no panics, on any input (directive 5; T-1871's cells).
- [ ] The Outcome reports fixed and broke counts per function on the 1,207 numeric inverse cells, against r2-impact's
      418 / 115.
- [ ] No gated line is added without decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting Excel's start, step and stop rule beyond the crossing. File a research task if the Outcome shows it is the
  limit.

## Visible-parity stage
Decision 0083: this task is done when every numeric inverse cell it owns is within half a unit of the 15th significant
digit of Excel's value, or the cells still off are named in the Outcome with T-0974, T-0967 or a follow-up as owner.
Bit-level criteria move to a follow-up task without the campaign key.

## Phase 1 triage (T-0090, 2026-10-27)
This task is on the critical path of the Phase 1 exit: BETA.DIST, F.DIST, BINOM.DIST, NEGBINOM.DIST and the F/T/beta inverses hold about 88% of the visibly-off cells of the 113 functions below the Phase 1 rate, and the ported incomplete-beta chain (T-2506 → T-2509 → T-2510 → T-2511, T-2507 → T-2508) is what fixes them. Priority raised to P1. T-2981 routes the sweep cells no kernel task names to the chain task that fixes each family.

## Cells added by the Phase 1 triage (T-0090, 2026-10-27)
Deep-tail quantiles handed from the parked T-2695 (visibly off at the 2026-10-27 eval): F.INV gen-m9/0a614cfe974b59c3 Calc!A268 `F.INV(1E-200,8,10)` and gen-m9/85f48d5cdfb24555 Calc!A12 `F.INV(1E-300,9,3)`; FINV gen-m9/85f48d5cdfb24555 Calc!A10 `FINV(1E-300,5,10)`.

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): F.INV 7, F.INV.RT 11, FINV 6. This task now owns them:
- gen-m9/4ab2ba98d26c8a5e: Calc!A390
- probes-numeric/m9-f-inv: Calc!A1, Calc!A4, Calc!A7, Calc!A21, Calc!A25
- probes-numeric/m9-f-inv-rt: Calc!A1, Calc!A7, Calc!A15, Calc!A25
- probes-numeric/m9-legacy-finv: Calc!A1, Calc!A2, Calc!A3, Calc!A4, Calc!A9, Calc!A10, Calc!A37, Calc!A38, Calc!A39, Calc!A40, Calc!A47, Calc!A48
- probes-numeric/t1049-finv: Calc!A7
- probes-numeric/t1049-finvrt: Calc!A14
