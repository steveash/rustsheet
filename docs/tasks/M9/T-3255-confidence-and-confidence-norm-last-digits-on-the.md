---
id: T-3255
title: "CONFIDENCE and CONFIDENCE.NORM last digits on the m9-legacy probe once T-0925's inverse normal lands (10 cells within 15 digits no task names)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-0925]
defer: phase-1b
---

## Goal
The 10 CONFIDENCE and CONFIDENCE.NORM cells of probes-numeric/m9-legacy-confidence listed below, all 1 ulp off and
within 15 significant digits of Excel, match Excel to the bit, or the Outcome records the formulations tried with their
bit-exact counts (decision 0056 time box).

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 6a26b992a, excel-graph). An ownership check over every failing cell
  (`scripts/exit_owners.py`'s `owned` rule over the task files of origin/main and of every pushed branch) found these
  cells named by no open task. T-0925 (campaign `visible`, in flight on `agent/functions/T-0925-inverse-normal`) owns
  the inverse normal behind them and the CONFIDENCE.NORM ulp lines of ironcalc-calc, but does not list these probe
  cells, and this triage does not edit a task another branch is working.
- CONFIDENCE(alpha, sigma, n) is NORM.S.INV(1 - alpha/2) · sigma / SQRT(n). The pairs agree (CONFIDENCE and
  CONFIDENCE.NORM give the same value at the same arguments, rows 1-2, 3-4, 7-8, 11-12), so the miss is in that
  expression or in the quantile, not in either name. It depends on T-0925 because the quantile is what T-0925 changes:
  re-run the list after it before fitting the operation order (sigma/SQRT(n) first or the product first, a division
  by SQRT(n) or a product with its reciprocal).
- A19 (`CONFIDENCE(0.05,1,100.9)`) truncates n to 100 and A29 (`CONFIDENCE("0.05",1,100)`) coerces text, and both
  equal A1, so neither coercion is involved.
- None is visibly off, so this is Phase 1b work and stays deferred.

Examples, Excel vs ours (`make eval-show CASE=probes-numeric/m9-legacy-confidence.case.json`):
- Calc!A1 `=CONFIDENCE(0.05,1,100)`: 0.19599639845400538 vs 0.19599639845400535
- Calc!A11 `=CONFIDENCE(0.999,3,7)`: 0.0014211250243260142 vs 0.0014211250243260137
- Calc!A7 `=CONFIDENCE(0.5,1,2)`: 0.47693627620447 vs 0.4769362762044699

Owned cells:
- probes-numeric/m9-legacy-confidence.case.json (10): Calc!A1, A2, A3, A4, A7, A8, A11, A12, A19, A29

## Acceptance criteria
- [ ] The owned cells match Excel to the bit (`make eval-show`; probes-numeric is ungated), with no new known-failure
      line in any `eval/baseline/` file (ironcalc-calc's CONFIDENCE.NORM cells must not regress); or the Outcome lists
      the operation orders tried with their bit-exact counts.
- [ ] docs/excel-semantics/statistics.md records CONFIDENCE's operation order, citing A1 and A11.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The inverse normal itself (T-0925); CONFIDENCE.T (the Student t quantile).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1
exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off
(then give it a `campaign:` stage instead).
