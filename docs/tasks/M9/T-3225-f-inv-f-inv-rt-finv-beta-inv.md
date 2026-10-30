---
id: T-3225
title: "F.INV, F.INV.RT, FINV, BETA.INV and BETAINV last digits on the probes-numeric quantile sweeps (120 cells no task names)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2500, T-2511]
defer: phase-1b
---

## Goal
The 120 F and beta quantile cells of the probes-numeric sweeps (and two probes/t2436 cells) listed below, all already
within 15 significant digits of Excel, match Excel to the bit, or the task records the search and stop rules tried with
their bit-exact counts (decision 0056 time box).

## Context
Gap triage 2026-10-28 (`make eval` on 40844417, excel-graph evidence). An ownership check over every failing cell found
these cells named by no open task, here or on a pushed `agent/*` branch. T-2767 (deferred) is the same work for the
gen-m9 sweep's 28 F quantile cells; T-2500 owns the F quantile cells about a unit of the 15th digit off; T-2511 re-bases
BETA.INV, F.INV, F.INV.RT and T.INV on the ported incomplete beta; T-2555, T-2808 and T-2936 own BETA.INV's tiny-quantile
and huge-shape quirks. None lists these cells. Every cell is 1 to 9 ulp off and none is visibly off, so this is Phase 1b
work and stays deferred. It depends on T-2500 and T-2511 because both move the crossing these cells sit on: re-run the
list after them before any fit.

Root cause (suspected): each value is a crossing of Excel's own F or beta cdf, which is a few ulp from the exact one, or
the point where Excel's quantile search stops (T-1745 found BETA.INV is a crossing of Excel's cdf; statistics.md). The
cells also show that Excel's left-tail and right-tail F inverses differ at small degrees of freedom, not only at df 1E10
(statistics.md "At df = 1E10 the left and right tail inverses are not mirror images"). The two cells below have the
same quantile, yet Excel's two values differ:
- m9-f-inv Calc!A11 `=F.INV(0.5,2,5)` is 0.7987697769322357, and m9-f-inv-rt Calc!A11 `=F.INV.RT(0.5,2,5)` is
  0.7987697769322353. The engine gives 0.7987697769322355 for both. Against a 50-digit mpmath root, Excel is +0.64 and
  -3.36 ulp, and ours is -1.36 for both.
- m9-f-inv Calc!A5 `=F.INV(0.95,1,1)` is 161.44763879758827 and m9-legacy-finv Calc!A26 `=F.INV.RT(0.05,1,1)` is
  161.44763879758855 (mpmath: Excel +2.15 and +12.15 ulp, ours +5.15 and +15.15).
- m9-legacy-finv Calc!A33 `=FINV(0.5,1,1)` is exactly 1 in Excel (the exact root), and ours is 1.0000000000000009 (4 ulp).
- m9-legacy-betainv Calc!A21 `=BETAINV(0.5,0.5,0.5)` is 0.49999999999999994 in Excel, and ours is 0.5000000000000002.
So the fit has to model F.INV and F.INV.RT as separate searches (or separate cdf sides), not as one quantile.
Code: crates/rustsheet-engine/src/functions/statistical/inverse_beta.rs, beta.rs and fisher.rs (the F quantile
search over `incomplete_beta.rs` and `bratio/`).

Examples (`make eval-show CASE=probes-numeric/<case>.case.json`), Excel vs ours:
- t1049-finv Calc!A6 `=F.INV(0.25,10,4)`: 0.6270117302682932 vs 0.6270117302682927
- m9-legacy-betainv Calc!A15 `=BETAINV(0.3,0.5,0.5)`: 0.20610737385376338 vs 0.20610737385376326
- probes/t2436/05d44f6dd6cfc567 Calc!A46 `=FINV(1E-20,1,6)`: 18898810.498423174 vs 18898810.49842316

Owned cells (F.INV, F.INV.RT, FINV, BETA.INV or BETAINV is the call that misses; A61 and A91 wrap it in DEGREES and
GAMMALN.PRECISE):
- probes/t2436/05d44f6dd6cfc567.case.json (2): Calc!A46, A61
- probes/t2436/a612e89321a6df9c.case.json (2): Calc!A76, A91
- probes-numeric/m9-f-inv-rt.case.json (22): Calc!A5, A6, A8, A11, A13, A16, A17, A18, A20, A22, A23, A24, A26, A27, A28, A29, A30, A31, A32, A33, A34, A35
- probes-numeric/m9-f-inv.case.json (21): Calc!A5, A6, A8, A11, A13, A16, A17, A19, A20, A22, A23, A24, A26, A27, A28, A29, A31, A32, A33, A34, A35
- probes-numeric/m9-legacy-betainv.case.json (20): Calc!A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A21, A22, A25, A26
- probes-numeric/m9-legacy-finv.case.json (38): Calc!A5, A6, A8, A11, A13, A16, A17, A18, A19, A20, A22, A23, A24, A26, A29, A30, A31, A32, A33, A34, A35, A36, A41, A42, A43, A44, A45, A46, A49, A50, A51, A52, A53, A54, A55, A56, A69, A72
- probes-numeric/t1049-finv.case.json (8): Calc!A5, A6, A8, A11, A13, A22, A23, A28
- probes-numeric/t1049-finvrt.case.json (7): Calc!A5, A8, A11, A13, A22, A23, A28

The sweeps hold about 60 distinct (p, d1, d2) points with both tails at most of them. If they cannot separate the
candidate stop rules, file an oracle sweep first (research lane, `excel-probe` skill: F.INV and F.INV.RT over a dense p
grid at these shapes, with F.DIST and F.DIST.RT evaluated at each Excel quantile and its neighbours one ulp away) and
depend on it. Do not fit blind.

## Acceptance criteria
- [ ] The owned cells match Excel bit for bit in `make eval`, or the Outcome lists each candidate search or stop rule with
      its bit-exact count over the owned cells and T-2767's (decision 0056).
- [ ] The two probes/t2436 cells fixed leave eval/baseline/probes.txt, and no new known-failure lines appear in any
      `eval/baseline/` file (probes-numeric is ungated, so its cells are checked with `make eval-show`).
- [ ] docs/excel-semantics/statistics.md records that F.INV and F.INV.RT differ at small df (m9-f-inv A11 against
      m9-f-inv-rt A11, m9-f-inv A5 against m9-legacy-finv A26), next to the df 1E10 entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The gen-m9 F quantile cells (T-2767, T-2500) and BETA.INV's tiny-quantile and huge-shape cells (T-2555, T-2808, T-2936).
- T.INV and the Student-t quantiles (T-0967, T-3061).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
