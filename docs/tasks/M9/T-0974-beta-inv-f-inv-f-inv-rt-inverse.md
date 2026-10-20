---
id: T-0974
title: "BETA.INV/F.INV/F.INV.RT: inverse incomplete beta parity (28 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-0970, T-1044, T-1745]
---

## Goal
BETA.INV, F.INV and F.INV.RT return Excel's values from one inverse-incomplete-beta routine built on
the T-0970 kernel.

## Context
Kernel map (T-1522, decision 0056): rank 5 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle targets: probes-numeric cases `m9-beta-dist-a*`, `m9-f-dist`, `m9-f-dist-rt`, `m9-beta-inv`, `m9-f-inv`, `m9-f-inv-rt` (T-1044).
Triage T-0903. Kernel: `statrs::function::beta::inv_beta_reg` via `Beta::inverse_cdf`
(`functions/statistical/beta.rs` `fn_beta_inv`, then `A + (B-A) t`) and `FisherSnedecor::inverse_cdf`
(`functions/statistical/fisher.rs` `fn_f_inv`, `fn_f_inv_rt` which inverts `1.0 - p`).
28 baseline lines in `eval/baseline/ironcalc-calc.txt` (13 NumericClose, 13 NumericUlp, 2 NumericMismatch):
- `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx` `Sheet1`: H3, H4(C), H5(C), H11(M), H15(C), H16(M), H19, H22, H24(C), H25, H26(C), H28(C), H29(C), H31(C), H32, H43(C), H45(C), H60, H61, H63, H64, H65, H72(C)
- `ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx` `BETA_DIST`: J3
- `ironcalc-calc/STATISTICAL/F_DIST.xlsx` `Sheet1`: J2, L2, J3(C), L3(C)
(C = NumericClose, M = NumericMismatch, unmarked = NumericUlp.)

Examples (expected vs actual):
- H11 `BETA.INV(BETA.DIST(1E-12,2,2.5,TRUE,0,1.2),2,2.5,0,1.2)`: expected 1.0000000000000012e-12, actual
  1.0000000337481705e-12 (Mismatch, 3.4e-8: statrs loses the deep lower tail).
- H16 (x = A = -1): expected -2.220446049250313e-16, actual -1.1102230246251565e-16 (Excel does not land
  on 0; see `docs/excel-semantics/statistics.md`, "Inverse distribution functions do not round-trip exactly").
- H5: expected 33587.000000000226, actual 33586.999999993524 (2e-13); H24 (alpha = 1e-4): expected
  0.23400000000003063, actual 0.23399999999973403.
- `F.INV(F.DIST(0.7,4,2.4,FALSE)...)` J3: expected 0.8753979410454287, actual 0.8753979410454386.

Hypothesis (the oracle is the judge): Excel solves I_t(a,b) = p iteratively on its own BETA.DIST
kernel (T-0970) and maps back with `A + (B-A) t`; F.INV.RT solves on the upper tail directly.
**Needs an oracle sweep (T-0018):** the last digits depend on the solver, so fit against the
T-1044 incomplete-beta sweep, never blind.

## Acceptance criteria
- [ ] Both NumericMismatch lines (H11, H16) and all NumericClose lines above leave those classes, at
      least 15 of the 28 lines leave `eval/baseline/ironcalc-calc.txt`, no new lines in any baseline;
      remaining lines listed in the PR with expected/actual.
- [ ] No statrs `inverse_cdf` left for Beta or FisherSnedecor (`rg inverse_cdf`); bounded iteration.
- [ ] `make check` green; reviewers approve.

## Out of scope
The CDF kernel (T-0970); T.INV (T-0967); legacy BETAINV/FINV evidence (T-1047).

## Fit result (T-1745)
Excel's BETA.INV returns a crossing point of its own BETA.DIST at 155 of 188 probe points (statistics.md, "BETA.INV solves Excel's own CDF to the crossing"). Solve the engine's BETA.DIST to adjacent doubles rather than to a loose tolerance. With the correctly rounded I_x that gets 87 of 212 cells against 34 today; 11 of Excel's points miss the crossing by 1 to 188 ulps of p, so the stop rule is still open. The fit's "mid" end is the even-significand end of the final bracket, not a rule to copy. The same solver through the F and t mappings beats the engine on F.INV, F.INV.RT, T.INV and T.INV.2T (fits `f-inv`, `f-inv-rt`, `t-inv`, `t-inv-2t`). Parity improves as T-0970's forward kernel does.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 381 in total.
- BETA.INV: 249 cells (13-14 digits right 208, 10-12 26, 6-9 1, fewer or 0 vs nonzero 14); most in `probes-numeric/t1745-beta-inv-roundtrip.case.json` 109; `probes-numeric/m9-beta-inv.case.json` 102. e.g. `BETA.INV(0.5,1,10)` Excel 0.06696700846319258, ours 0.06696700846319285
- BETAINV: 7 cells (13-14 digits right 6, 10-12 1, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-legacy-betainv.case.json` 7; `gen-m9/288b99797f58f149.case.json` 0. e.g. `BETAINV(0.3,10,1.5)` Excel 0.8361662533038416, ours 0.8361662533038301
- F.INV: 43 cells (13-14 digits right 29, 10-12 5, 6-9 0, fewer or 0 vs nonzero 9); most in `probes-numeric/m9-f-inv.case.json` 20; `probes-numeric/t1049-finv.case.json` 13. e.g. `F.INV(0.4,5,5)` Excel 0.7878805312664828, ours 0.7878805312664846
- F.INV.RT: 57 cells (13-14 digits right 37, 10-12 7, 6-9 8, fewer or 0 vs nonzero 5); most in `probes-numeric/m9-legacy-finv.case.json` 21; `probes-numeric/m9-f-inv-rt.case.json` 19. e.g. `F.INV.RT(0.5,2,5)` Excel 0.7987697769322353, ours 0.7987697769322422
- FINV: 25 cells (13-14 digits right 14, 10-12 4, 6-9 4, fewer or 0 vs nonzero 3); most in `probes-numeric/m9-legacy-finv.case.json` 23; `probes-numeric/t1530-f-df-cut.case.json` 2. e.g. `FINV(0.5,2,5)` Excel 0.7987697769322353, ours 0.7987697769322422

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.
