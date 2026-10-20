---
id: T-0969
title: "T.DIST/T.DIST.RT/T.DIST.2T/T.TEST: accurate Student-t CDF with direct upper tail (55 known failures)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1744]
---

## Goal
T.DIST (cumulative and density), T.DIST.RT, T.DIST.2T and T.TEST return Excel's values from one
Student-t CDF kernel that computes the upper tail directly instead of as `1 - CDF`.

## Context
Kernel map (T-1522, decision 0056): rank 3 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Triage T-0903. Kernel: `statrs::distribution::StudentsT::cdf`, i.e. `0.5 * beta_reg(df/2, 0.5, df/(df+x^2))`
(statrs' Numerical-Recipes continued fraction, ~1e-14..1e-10 relative error) and `StudentsT::pdf`
(statrs Lanczos `ln_gamma`). Code: `functions/statistical/t_dist.rs` (`fn_t_dist`, `fn_t_dist_2t`
and `fn_t_dist_rt` compute `1.0 - dist.cdf(x)`; `fn_t_test` computes `1.0 - cdf` too).
55 baseline lines in `eval/baseline/ironcalc-calc.txt` (40 NumericClose, 14 NumericUlp, 1 NumericMismatch):
- `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST T.INV T.DIST.RT`: D2, F2(C), D3(C), F3(C), F4(C), D5, D6, F6(C), D7, F7(C), D8, F8(C), D10(C), D12, F13(C), D14(C), D15, F15, D16(C), F16(C), D17(C), F17(C), F18(M), D19(C), D20(C), F20(C), D21(C), F21(C), D28, F28, D32(C), D38, F38, D40(C), D52(C)
- `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST.2T T.INV.2T`: C2(C), C3(C), C4(C), C6(C), C7(C), C8(C), C13(C), C15, C16(C), C17(C), C18(C), C20(C), C37
- `ironcalc-calc/STATISTICAL/T_TEST.xlsx` `Sheet1`: F2(C), F3(C), F4(C), F5(C), F6(C), F7(C), F8(C)
(C = NumericClose, M = NumericMismatch, unmarked = NumericUlp.)

Examples (expected vs actual):
- `T.DIST.RT(1E6,10)` F18: expected 1.230468749943611e-56, actual 0.0 (Mismatch: catastrophic `1 - cdf`).
- `T.DIST(1E-6,11,TRUE)` D20: expected 0.500000389989757, actual 0.5000003900287391 (7.8e-11 relative,
  statrs' incomplete beta near h = 1).
- `T.DIST(1,100,TRUE)` D17: expected 0.8401379221079384, actual 0.8401379221078716 (7.9e-14).
- `T.DIST(1,1,TRUE)` D15: expected exactly 0.75, actual 0.7499999999999998; `T.DIST.RT(1,1)` F15:
  expected 0.25000000000000006, actual 0.2500000000000002.
- `T.TEST(...)` F3: expected 0.3017830576436469, actual 0.3017830576436453 (5e-15).

Excel quirk (recorded in `docs/excel-semantics/statistics.md`, "T.DIST.RT and T.DIST.2T are not
1 - T.DIST"): T.DIST(1,1) = 0.75 but T.DIST.RT(1,1) = 0.25000000000000006, so the tails are computed
separately; `T.DIST(1,2,TRUE)` = 0.7886751345948129 = 0.5 + 1/(2*sqrt(3)).

Hypothesis (the oracle is the judge): T.DIST/T.DIST.RT/T.DIST.2T truncate df to an integer, so
Excel may use the closed-form finite series for integer df (Hill 1970 ACM 395 / AS 3: atan series for
odd df, sqrt series for even df), with the upper tail evaluated directly from the same series; T.TEST
type 3 has a non-integer (Welch) df, so it needs an incomplete-beta path too (share T-0970's kernel).
Try the closed form first on the cached cells; keep whichever reproduces them; never `1 - cdf` for a
tail. Put the kernel in `functions/statistical/special.rs` (one definition; T-0967 builds T.INV on it).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/4df1b22d3ed63f58 Calc!A9 `=CHIINV(T.DIST(0.1,7,Data!D1),2)`: Excel 1.2382105318705625, engine 1.2382105318705663 (engine T.DIST(0.1,7,TRUE) = 0.5384259701976026); shared with T-0972.

Oracle sweep (T-1043): probes-numeric cases `m9-t-dist-cdf`, `m9-t-dist-pdf`, `m9-t-dist-rt`, `m9-t-dist-2t`, `m9-t-nonint-df`, `m9-t-edges`; findings in docs/excel-semantics/statistics.md "Student t sweep".

## Acceptance criteria
- [ ] No T.DIST/T.DIST.RT/T.DIST.2T/T.TEST line listed above remains with a NumericClose or
      NumericMismatch verdict (`rustsheet-eval show` on both cases), at least 30 of the 55 lines
      leave `eval/baseline/ironcalc-calc.txt`, and no new lines appear in any baseline.
- [ ] Every remaining line is <= 4 ulp and listed in the PR with expected/actual; they stay for the
      oracle-sweep follow-up (T-1043).
- [ ] `T.DIST.RT(1E6,10)` returns 1.230468749943611e-56 (unit test), no `1 - cdf` in the tail paths.
- [ ] Findings (which formulation matched) added to the `docs/excel-semantics/statistics.md` entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
T.INV/T.INV.2T/CONFIDENCE.T (T-0967); the incomplete-beta kernel for BETA.DIST/F.DIST (T-0970);
legacy TDIST/TTEST evidence (T-1048).

## Fit result (T-1744)
Through I_x (mapped via the smaller of n/(n+t^2) and t^2/(n+t^2), which large n needs, but I_y/2 directly for a lower tail under 0.1: 0.5 - I_z/2 cancels to 0 at T.DIST(-10,1000,TRUE) = 8.3E-23), Cephes incbet is bit-exact on 105 of 150 T.DIST cells and the correctly rounded value on 113, against 78 today; no candidate is Excel's algorithm yet.
