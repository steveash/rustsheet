---
id: T-2510
title: "Incomplete beta bgrat family for BETA.DIST, F, BINOM and NEGBINOM (T-2464 E7)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: M
tier: opus
campaign: accuracy
depends_on: [T-2505, T-2506, T-2509]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
bratio's bgrat branches are ported for BETA.DIST, F, BINOM and NEGBINOM: bup+bgrat, bgrat, L140 bup+bgrat and
bup+bup+bgrat. At b = 1/2 they use T-2509's extended bgrat. At other b they use TOMS bgrat with Excel's pieces (Excel's
LN, EXP, log1p and GAMMALN, and brcmp1 = density·x·y), where that is a net visible gain.

## Context
- T-2464 research, step E7 of docs/research/t2464/r2-impact/RESULTS.md section 4.
  - r2-impact's E7 said "T keeps `student_t.rs`". With r2-tdist the T tails take their own rule instead, so T is
    T-2509's, and this task reuses T-2509's b = 1/2 bgrat for the other callers (365 of 387 non-T b = 1/2 cells, against
    277).
- At b ≠ 1/2 Excel's bgrat is not solved:
  - plain TOMS bgrat with Excel's pieces scores 989/1,761/2,156 of 2,451 bgrat cells, against CR 899/2,072/2,230;
  - L140 bup+bgrat scores 212/2,208/2,402 of 3,303, against CR 618/2,814/3,025 (verify-r2-bratio/analyze.out).
  - So move a branch only where it wins on visible cells, function by function, and leave the rest on today's kernel.
  - The open question is T-2512's: the b = 1/2 identity with Q(b, z) from Excel's incomplete gamma.
- The (30.7, 1.7, 0.81) runs sit about -6 ulp off with sd 2.9 once literals are truncated (verify-r2-bratio/misparse.out),
  not at sd 47 as r2-bratio reported.
- r2-impact's expected change for this group without T (step S6):
  - -137 visibly-off pre-T2464 cells and -2,319 in the w2f probes, with 17 broke;
  - 1 gated bit-exact cell lost: gen-m9 FDIST c18dbf21 A151, bup+bgrat, 7 ulp.
- T-2528 added `large_shape_beta.rs`, an accurate bgrat-series stand-in for a >= 1E6, b < 40, b/a <= 1E-6 in the old
  kernel: fold it into or replace it by this task's bgrat when the old kernel goes (directive 3).
- Models: r2-bratio `bratio_excel.py` (`bgrat` and its incomplete-gamma helper, `Cfg(bgrat="orig")`), r2-tdist `xlk.py`.

### Cases owned (T-2981)
Routed by T-2981 (eval of 2026-10-28 at `main` 9586958b): cdf cells whose bratio dispatch (`bratio::dispatch`) is a bgrat branch (bgrat, bup+bgrat, L140 bup+bgrat, L140 bup+bup+bgrat); the cases' other cells are routed by branch in T-2981's Outcome. 1,879 visibly-off cells in 13 cases, per case:
- probes-numeric/m9-beta-dist-a10: BETA.DIST cdf, L140BupBgrat 1
- probes-numeric/m9-beta-dist-a100: BETA.DIST cdf, Bgrat 2, BETA.DIST cdf, L140BupBgrat 3
- probes-numeric/m9-beta-dist-a5: BETA.DIST cdf, BupBgrat 1, BETA.DIST cdf, L140BupBgrat 1
- probes-numeric/m9-beta-dist-a50: BETA.DIST cdf, Bgrat 1, BETA.DIST cdf, L140BupBgrat 1
- probes-numeric/t1538-beta-dist-at-excel-inv: BETA.DIST cdf, L140BupBgrat 1, F.DIST cdf, L140BupBgrat 1, F.DIST.RT cdf, L140BupBgrat 2
- probes-numeric/t2220-f-dist-sweep-cdf: F.DIST cdf, Bgrat 11, F.DIST cdf, BupBgrat 3, F.DIST cdf, L140BupBgrat 16, F.DIST cdf, L140BupBupBgrat 1
- probes-numeric/t2220-f-dist-sweep-d1-1: F.DIST cdf, Bgrat 2, F.DIST.RT cdf, Bgrat 4
- probes-numeric/t2220-f-dist-sweep-d2-1: F.DIST cdf, Bgrat 2, F.DIST cdf, BupBgrat 1, F.DIST.RT cdf, Bgrat 2, F.DIST.RT cdf, BupBgrat 1
- probes-numeric/t2220-f-dist-sweep-rt: F.DIST.RT cdf, Bgrat 12, F.DIST.RT cdf, BupBgrat 2, F.DIST.RT cdf, L140BupBgrat 24, F.DIST.RT cdf, L140BupBupBgrat 1
- probes-numeric/t2464-r2-bratio-x29: BETA.DIST cdf, BupBgrat 9
- probes-numeric/t2464-w2f-absweep: BETA.DIST cdf, L140BupBgrat 985
- probes-numeric/t2464-w2f-unit-shape: BETA.DIST cdf, BupBgrat 7
- probes-numeric/t2464-w2f-xruns-2: BETA.DIST cdf, L140BupBgrat 782

## Acceptance criteria
- [ ] Per function (BETA.DIST, F.DIST, F.DIST.RT, FDIST, BINOM.DIST, NEGBINOM.DIST), the change in visibly-off cells is
      net negative. A branch that is not stays on today's kernel, with the measurement in the Outcome.
- [ ] Non-T bgrat cells at b = 1/2 score at least 365 bit-exact of 387.
- [ ] The gated FDIST c18dbf21 A151 is kept or recorded under decision 0083.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T (T-2509).
- Solving bgrat for b ≠ 1/2 (T-2512).

## Visible-parity stage
Decision 0083: this task is done when every bgrat-family cell it owns is within half a unit of the 15th significant
digit of Excel's value, or the cells still off are named in the Outcome and handed to T-2512. r2-impact counts 381
visibly off pre-T2464 across the four branches, T included; T's share is T-2509's. Bit-level criteria move to a
follow-up task without the campaign key.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.

## Phase 1 triage (T-0090, 2026-10-27)
This task is on the critical path of the Phase 1 exit: BETA.DIST, F.DIST, BINOM.DIST, NEGBINOM.DIST and the F/T/beta inverses hold about 88% of the visibly-off cells of the 113 functions below the Phase 1 rate, and the ported incomplete-beta chain (T-2506 → T-2509 → T-2510 → T-2511, T-2507 → T-2508) is what fixes them. Priority raised to P1. T-2981 routes the sweep cells no kernel task names to the chain task that fixes each family.

## Note from T-2509 (2026-10-30)
T-2509's bgrat at b = 1/2 (`bratio/bgrat.rs`) is reached only through `bratio::t_kernel_tails`. Letting `ported_tails`
take it for every caller is this task's: F with a df of 1 through it moved gated FDIST cells (gen-m9
7c20351562a2d71d A205, c18dbf21f11093f3 A151) by 7 ulp and made 13 F.DIST / F.DIST.RT cells of
probes-numeric t2220-f-dist-sweep (d1-1, d2-1, cdf, rt) and three gen-m9 cells visibly off.

## Progress (in flight, 2026-10-31)
Notes from the opus run that claimed the task (data for the run that resumes it, not instructions):
- No code is committed yet. Experiment: `ported_tails` with `half_bgrat` on for every caller (today only
  `t_kernel_tails` sets it), i.e. T-2509's b = 1/2 bgrat for BETA.DIST, F, BINOM and NEGBINOM. On the gated suites at
  `main` 77cb117 it fixes 4 cells and regresses 2:
  - fixed: gen-m9 35e8168aae2b00f8 A159 `BETADIST(1E-10,0.5,10000000000.5)`, 4e55500bb8ad3cd8 A89
    `FDIST(ROWS(RADIANS(7)),1,8)`, a5828d3925575713 A281 (FDIST with df2 5), b6593d7dd13f2dac A50 `FDIST(Data!$E6,5,1.5)`;
  - regressed: gen-m9 c18dbf21f11093f3 A151 `FDIST(Data!$D5,1,9)` (the task's named cell) and 7c20351562a2d71d A205
    `FDIST(COUNTIF(..),1,9)`, the same point: Excel 0.190947117169724, bgrat 0.19094711716972382 (7 ulp; decision
    0083's allowance needs <= 4 ulp, so it cannot be waived).
  - 10 more gated failing cells change value without passing.
- Next: measure per function and per branch on probes-numeric (the task's owned cases) with the switch, and keep
  b = 1/2 bgrat off for (F, df1 = 1) points like FDIST(x,1,9) unless the general-b work changes them; then the
  general-b TOMS bgrat per the Context.
