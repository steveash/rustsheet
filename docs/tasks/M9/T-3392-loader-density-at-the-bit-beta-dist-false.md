---
id: T-3392
title: "Loader density at the bit: BETA.DIST(.,FALSE) for min(a,b) > 1 and the BINOM/NEGBINOM pmf, the 55,363 last-digit cells T-2507 left with no owner (T-2513 loader probes, T-2464 density-large)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2586]
defer: phase-1b
---

## Goal
The BETA.DIST density cells with min(a,b) > 1 and the BINOM.DIST/BINOMDIST and NEGBINOM.DIST/NEGBINOMDIST pmf cells
named below, all already within 15 significant digits of Excel, match Excel to the bit once T-2586's model of the
Loader constant and bd0 series is ported, or the Outcome records the bit-exact counts of each candidate (decision 0056
time box).

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 6a26b992a, excel-graph). An ownership check over every failing cell
  (`scripts/exit_owners.py`'s `owned` rule over the task files of origin/main and of every pushed branch, not only the
  visibly-off cells `make unowned-off` lists) found these cells named by no open task. T-2507 (done) owned the whole
  cases ("Cases owned (T-2981)") and shipped Excel's Loader saddle point (`loader.rs` `excel_beta_density`). Its
  Visible-parity stage said bit-level criteria move to a follow-up without the campaign key, and none was filed. This
  is that follow-up.
- Neighbouring owners, none of which lists these cells: T-2586 (in progress; research: Loader's constant, the bd0
  series, BINOM's interior pmf and NEGBINOM's final step), T-3220 (the 661 cells the port newly moved visibly off,
  `docs/research/t2507/newly_visibly_off.tsv`), the 2026-10-30c gap-triage branch's tasks for the visibly-off
  large-shape and a = 1025 cells (`agent/functions/tasks-gap-triage-opus-2026-10-30c`, unmerged), T-2821 (min(a,b) <= 1,
  deferred). Visibly-off cells of these cases stay theirs.
- Every cell is 1 to 36 ulp off and none is visibly off, so this is Phase 1b work and stays deferred. It depends on
  T-2586 because the fit is its model; re-run the cell counts after T-2586 and T-3220 land, before any port.
- Per case: failing cells no task names, median / max ulp from Excel (`make eval-show`):
  - probes-numeric/t2464-r2-density-large-p1: 9,950, 3 / 7
  - probes-numeric/t2513-r3-loader-const-sweep1: 8,144, 2 / 12
  - probes-numeric/t2513-r3-loader-const-diverse1: 8,116, 2 / 21
  - probes-numeric/t2513-r3-loader-bd0-hold: 7,955, 2 / 22
  - probes-numeric/t2513-r3-loader-bd0-a: 7,911, 3 / 36
  - probes-numeric/t2513-r3-loader-const-grid1: 5,210, 1 / 9
  - probes-numeric/t2513-r3-loader-bd0-b: 3,084, 1 / 8
  - probes-numeric/t2513-r3-loader-const-hold1: 2,236, 2 / 14
  - probes-numeric/t2513-verify-r3-loader-bd0-hold: 1,434, 2 / 20
  - probes-numeric/t2513-verify-r3-loader-const-hold: 1,281, 3 / 21
  - the m9 shape sweeps: 14 densities of m9-beta-dist-a2p5 (3 / 16) and 9 of m9-beta-dist-a2 (1 / 5), listed below
  - probes-numeric/m9-legacy-binomdist: 12 BINOM pmf cells (Calc!A8, A15, A16, A19, A20, A21, A22, A39, A40, A42, A50,
    A54); probes-numeric/t1048-negbinomdist: 7 NEGBINOM pmf cells (Calc!A3, A6, A8, B3, B6, B7, B8)
- Examples, Excel vs ours:
  - t2513-r3-loader-bd0-a Calc!A4962 `=BETA.DIST((13901607*2^-25+46333451*2^-51),5.5,12.5,FALSE)`: 2.0052546808124876
    vs 2.005254680812488
  - t2464-r2-density-large-p1 Calc!A2560 (shapes 2.5, 3.5): 1.5313510992408923 vs 1.5313510992408916
  - m9-beta-dist-a2p5 Calc!A36 `=BETA.DIST(0.9,2.5,5,FALSE)`: 0.005007824901642813 vs 0.005007824901642812
  - m9-legacy-binomdist Calc!A19 `=BINOMDIST(50,100,0.3,FALSE)`: 1.3026227131445354e-05 vs 1.3026227131445342e-05
  - t1048-negbinomdist Calc!A6 `=NEGBINOMDIST(20,4,0.25)`: 0.021938345093172287 vs 0.021938345093172273
- Rule of record: docs/excel-semantics/statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point". The
  BINOM and NEGBINOM pmf share `excel_dbinom_exponent` and `excel_bd0` (T-2507 Outcome).

Owned: every failing cell of the ten T-2513/T-2464 density cases listed above that no other open task lists, plus
these density and pmf cells:
- probes-numeric/m9-beta-dist-a2p5.case.json (14): Calc!A20, A22, A26, A28, A30, A32, A34, A36, A38, A40, A42, A44, A48, A50
- probes-numeric/m9-beta-dist-a2.case.json (9): Calc!A24, A28, A30, A36, A38, A40, A44, A46, A50
- probes-numeric/m9-legacy-binomdist.case.json (12): Calc!A8, A15, A16, A19, A20, A21, A22, A39, A40, A42, A50, A54
- probes-numeric/t1048-negbinomdist.case.json (7): Calc!A3, A6, A8, B3, B6, B7, B8

## Acceptance criteria
- [ ] With T-2586's model ported into `loader.rs`, the Outcome reports per case the bit-exact count before and after,
      and every owned cell is bit-exact or named with the candidate forms tried (decision 0056).
- [ ] No case listed gains a visibly-off cell, and no known-failure line is added to any `eval/baseline/` file
      (ironcalc-calc BETADIST_BETAINV.xlsx and BINOM.xlsx are the gated cells on this kernel).
- [ ] docs/excel-semantics/statistics.md's Loader section records the final score.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off cells of these cases (T-3220 and the 2026-10-30c triage tasks), min(a,b) <= 1 (T-2821, T-3035), the
  T and F densities (T-2585, T-3173, T-3174).
- Cracking the model (T-2586): this task ports it.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1
exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off
(then give it a `campaign:` stage instead).
