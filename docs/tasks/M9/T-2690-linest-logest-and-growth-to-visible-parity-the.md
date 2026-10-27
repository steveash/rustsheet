---
id: T-2690
title: "LINEST, LOGEST and GROWTH to visible parity: the 113 visibly-off cells of the least-squares solve (visible stage of T-2006)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Every LINEST, LOGEST and GROWTH cell of the least-squares solve is within half a unit of the 15th significant
digit of Excel's value, or has Excel's type. That includes the F statistic that Excel computes from its solve's
residual noise, where ours is `#NUM!`. Excel's bits stay T-2006's target.

## Context
Gap triage 2026-10-25 (`make eval`, agent/engine/T-2632-typed-arg-counts at 4dbb3c62). T-2006 (the
least-squares kernel fit, ~300 cells) carries `defer: phase-1b`, but 113 of its cells are visibly off. Per decision
0114 a group with visibly-off cells gets a campaign stage, and only the within-15-digits residue is deferred. No
pickable task owns these cells today: T-2569 is a probe, and T-2574, T-2556, T-2557 and T-2584 are argument and
check-order work. This task is that visible stage. T-2006 stays the bit-level follow-up, and its fit lab work
(if started) is reused here.

Visibly-off cells (all are known-failure lines in eval/baseline/probes.txt or ironcalc-calc.txt):
- probes/linest-modes, 42 cells: L1, L8, L15, L22, L29, L36, K51, L52, K53, L54, L64, L71, L78, L99, K106, M106, P106,
  K107, L107, M107, N107, P107, L108, K109, L110, K134, L134, L138, L148, L169, L190, K191, L191, L192, K193, L194,
  L197, K198, L198, L199, K200, L201.
  Three are the wrong type: K109 (collinear `LINEST(B2:B9,A2:E9,TRUE,TRUE)`, F: Excel 3.1115766120637556E+34, ours
  `#NUM!`), K193 and K200 (y in x, F: Excel 1.3629779253653922E+32, ours `#NUM!`). K106 is coefficient noise: Excel
  1.4342828472961902E-16, ours 0. K134 is the two-point fit: Excel exactly 0.5 and -1.5, ours 0.49999999999999867.
- probes/logest-modes, 51 cells: K9, L10, L12, K30, L31, L33, L54, K72, L73, L75, K79, L80, L82, K85, L85, M85, N85, K86,
  L86, M86, N86, L87, K88, L89, L92, M92, K93, L93, M93, L94, K95, L96, L115, L117, K135, L136, L138, K151, L152, L155,
  L159, K170, L170, L171, K172, L173, K198, L198, L199, K200, L201.
  K172 and K200 are the wrong type in the other direction: Excel's exact-fit residual is exactly 0, so F is `#NUM!`,
  where ours is a number (1.05E+31, 6.0E+30). K170: Excel exactly 0, ours 2.1E-16. K85: Excel 0.982454941667086, ours
  0.9824549416670796.
- probes/growth-modes, 15 cells: K3, K15, K27, K123, K145-K152, K157, K169, K219. For example K145
  `GROWTH(D2:D9,A2:C9,A2:C9)`: Excel 2.943203437996074, ours 2.943203437996122.
- gen-m9/589f42e67f4b4b8d Calc!A193 `LOGEST(Data!A10:B11)`: Excel 2.220601291150046, ours 2.2206012911500514 (gap triage
  2026-10-25: visibly off, owned until now only by the deferred T-2006; it counts with the 113 below).
- ironcalc-calc/STATISTICAL/LINEST_LOGEST.xlsx (Mac 16.x cache), 5 cells: E2, E3, E5, E6, G6. For example E3
  `LINEST(A2:A6)` intercept: Excel -0.031000000000000028, ours -0.03100000000000025.

So Excel's solve leaves rounding noise where ours is exact, and is exact where ours leaves noise. Matching the
numbers to 15 digits and the zero-or-not of residuals and noise coefficients means reproducing Excel's
decomposition, not solving more accurately. Hence `visible`, not `accuracy`, and the wrong-type cells go with
the numbers because they share the root cause. Leads, all UNVERIFIED:
- Microsoft's LINEST help describes QR since Excel 2003;
- statistics.md "LINEST, LOGEST and GROWTH: cells left after T-1741" infers a pivoted QR that keeps the
  largest-norm column first (K106, 285d87cea1aaa73c A293);
- the engine forms normal equations and solves them with Gauss-Jordan (`linear_regression`, `solve_system` in
  functions/statistical/linest.rs).
statistics.md also notes that the singular test should be relative to scale. Read T-2006's Context for the full
cell census and the time-box rule (decision 0056). There are several hundred Excel cells in the repo, so no
oracle sweep is needed to start.

Semantics: statistics.md "LINEST, LOGEST, GROWTH: probe findings", "LINEST and LOGEST: shapes, one point, collinear
columns, exact fits and check order (T-1741)", "LINEST, LOGEST and GROWTH: cells left after T-1741 (T-1742)".

## Acceptance criteria
- [ ] A fit-lab kernel (`uv run rustsheet-fitlab ...`, a new `kernels/` module) scores candidate decompositions on
      the cells above and on the within-15-digit cells T-2006 lists. The chosen one is documented in statistics.md
      with its score and the rejected candidates.
- [ ] Visible parity: the 113 known-failure lines above disappear from eval/baseline/probes.txt and
      eval/baseline/ironcalc-calc.txt (`make eval-baseline`), or each one left is documented in statistics.md with
      the forms tried (decision 0056 time box). No new known-failure lines appear in any baseline, and no
      TREND_GROWTH.xlsx, regression-trend or gen-m9 LINEST/LOGEST/TREND/GROWTH cell becomes visibly off.
- [ ] T-2006's Outcome or Context names this task as its visible stage when this one lands (edit T-2006 in this
      task's PR).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-exact last digits of cells already within 15 digits (T-2006, `defer: phase-1b`).
- Argument reading, check order, lifting and size bounds (T-2556, T-2557, T-2574, T-2584), and the collinear-pair
  probe (T-2569).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of
Excel's value (an expected 0 must be exactly 0), or has the right type, error and text. Criteria above that ask
for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in
the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-25 eval run (agent/engine/T-2632-typed-arg-counts, 4dbb3c62): 113 in total.
- LINEST: probes/linest-modes 42 (3 wrong type), ironcalc-calc LINEST_LOGEST.xlsx 5.
- LOGEST: probes/logest-modes 51 (2 wrong type).
- GROWTH: probes/growth-modes 15.

Stage `visible`: Excel's solve is the less exact one here, so the target is its algorithm (time-boxed, decision 0056).
