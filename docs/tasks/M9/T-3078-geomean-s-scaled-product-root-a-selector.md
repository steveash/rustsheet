---
id: T-3078
title: "GEOMEAN's scaled-product root: search for a selector between the plain and log-domain root models (17 probe cells one ulp off)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
GEOMEAN's over/underflow path (a plain product outside the normal range) returns Excel's root on the 17 probe cells
listed below, each one ulp off today, with no probed GEOMEAN cell regressing. Otherwise the task records, under the
decision 0056 time box, that no evidence-backed selector exists.

## Context
Gap triage 2026-10-28 (`make eval` on 40844417, excel-graph evidence). An ownership check over every failing cell found
these 17 cells named by no open task. T-1099 (blocked) owns the same class of cells in its two probes (a 600-value
underflowing set and 17 random product-order sets). T-1284 and T-2794 own the in-range root (a normal plain product), which is a different
path in `fn_geomean`. All 17 cells are one ulp off and none is visibly off, so the task stays deferred.

What is known (docs/excel-semantics/numbers.md "GEOMEAN root rounding", T-1108, T-0290):
- The engine's root, `(M * 2^r)^(1/n) * 2^q`
  (crates/rustsheet-engine/src/functions/statistical/geomean.rs), matches 21 of 29 earlier over/underflow cells.
- No single tried model reproduces all of them: glibc `pow`, Excel's own `^` on the same X (14 of 17 in
  probes/geomean-root-models), `EXP(LN(X)/n)`, `2^(log2 X/n)` and the x87 compositions all fail.
- T-0290's log-domain root `exp((ln M + r ln 2)/n) * 2^q` fixes 16 known failures and regresses 9. The 16 include
  10 of this task's cells (pow2-count L4, M1, M4; stored-3ulps-low R1; exponent-sum C2; exponent-sum-2 C28;
  overflow-count C1; overflow-cut C1; overflow-cut-2 C9; overflow-cut-3 C13) and six of T-1099's. The 9 regressions are
  three product-order sets and two 600-value sets of T-1099's probes, stored-3ulps-low R2, exponent-sum C9, overflow-cut
  C26 and overflow-cut-2 C8 (numbers.md lists them by cell).
- T-0290 left open whether a per-cell selector between the two models exists ("would need a selector backed by
  evidence, which was not searched for"). That search is this task's new hypothesis, so T-1284's reopen condition
  (a new root hypothesis) does not apply to it.
Approach: over every cached over/underflow GEOMEAN cell (the probes above, the mixed-magnitude probe and
T-1099's two probes, about 60 cells), compute both models' roots and features
available before rounding: the remainder r, n, r/n, the distance of each model's root from the exact one, whether the
two models agree, and the significand M. Then look for a rule that picks Excel's value on every cell. If one exists,
check it on cells it was not fitted on before shipping: hold out a third, or file an `excel-probe` case of new sets
chosen to separate the rule from its nearest alternative. That probe is a research-lane task; depend on it.

Examples (`make eval-show CASE=probes/<case>.case.json`), Excel vs ours:
- geomean-pow2-count Sheet1!L4 `=GEOMEAN(IF(SEQUENCE(3000),2^-1000*1.001))`: 1.4994830372284005E-301 vs
  1.4994830372284007E-301
- geomean-overflow-count Sheet1!A1 `=GEOMEAN(A2:A1002)`: 5.0000000000000006E+204 vs 5E+204
- geomean-root-models Pin!C60 `=GEOMEAN(C1:C24)`: 3.313292825330382E+223 vs 3.3132928253303826E+223

Owned cells (known-failure lines in eval/baseline/probes.txt):
- probes/geomean-exponent-sum.case.json (2): Sheet1!C2, C7
- probes/geomean-exponent-sum-2.case.json (2): Sheet1!C24, C28
- probes/geomean-overflow-count.case.json (3): Sheet1!A1, B1, C1
- probes/geomean-overflow-cut.case.json (1): Sheet1!C1
- probes/geomean-overflow-cut-2.case.json (2): Sheet1!C2, C9
- probes/geomean-overflow-cut-3.case.json (1): Sheet1!C13
- probes/geomean-pow2-count.case.json (3): Sheet1!L4, M1, M4
- probes/geomean-root-models.case.json (2): Pin!C60, Pin!F60
- probes/geomean-stored-3ulps-low.case.json (1): Sheet1!R1

## Acceptance criteria
- [ ] Either a selector ships: the owned cells it fixes leave eval/baseline/probes.txt, no new known-failure lines appear
      in any `eval/baseline/` file, and numbers.md "GEOMEAN root rounding" states the rule and the cells it was checked on
      (held out or newly probed). Or the Outcome lists each selector feature tried with its count over the roughly 60
      cells and numbers.md records that none separates them (decision 0056).
- [ ] If the selector also fixes T-1099's cells, the Outcome says so, so the steward can close T-1099.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The in-range root (T-1284, T-2794) and the significand cut and 2048 * k wrap (T-1112, T-1120, done).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
