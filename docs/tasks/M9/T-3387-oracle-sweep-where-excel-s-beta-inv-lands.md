---
id: T-3387
title: "Oracle sweep: where Excel's BETA.INV lands at small shapes (tiny p, and p 0.01 to 0.5 at b 10 to 1000), with BETA.DIST at its own landing point"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The fit task T-3388 has dense excel-graph evidence of where Excel's BETA.INV lands at small and moderate shapes, and
of Excel's own cdf at that landing point, instead of the 33 informative rows of t1745-beta-inv-roundtrip it has
today.

## Context
- Gap triage 2026-10-31 (`make eval` on a64f963e, `make exit-owners MILESTONE=M9`): 12 visibly-off BETA.INV cells of
  probes-numeric/m9-beta-inv have no open owner (T-3388 lists them). They sit at the same (p, a, b) as rows of
  probes-numeric/t1745-beta-inv-roundtrip, which show two kinds of landing:
  - flat: Excel's BETA.DIST is the same double at x and both neighbours (rows 7, 14, 21, 22, 28, 29, 32, 33, 41),
    so there is no crossing and the landing point is wherever Excel's search stops, e.g. `BETA.INV(1E-10,1,1)` =
    9.999999999999996E-11 (m9-beta-inv A43; the uniform quantile is exactly 1E-10);
  - miss: x is 1 to 188 ulps of p off the crossing (rows 91, 98, 133), e.g. `BETA.INV(0.05,2.5,100)` =
    0.005668759702401242 (m9-beta-inv A101), where `BETA.DIST` of it is 0.05000000000000131.
- docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)" calls the stop "a
  tolerance stop, but it is not established". T-2555 (huge shapes) reads Excel's BETA.INV as AS 109's structure
  (p > 1/2 is 1 - BETA.INV(1-p,b,a)) and its tiny-quantile values as near the second Newton iterate from AS 109's
  start. Telling those hypotheses apart at small shapes needs more points than the 33 rows, and decision 0188 R4.1
  needs a hold-out split on top.
- Evidence today: probes-numeric m9-beta-inv (p in {1E-10, 0.001, 0.05, 0.5, 0.95, 0.999, 0.9999999999}, a and b in
  {0.5, 1, 2.5, 10, 100}), m9-legacy-betainv, t1745-beta-inv-roundtrip (its neighbour constants D and F are cell
  values, not formula literals).

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t3387-beta-inv-small-shape-landing.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in
      `m9-fin-power.case.json`), is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Its grid,
      for a in {0.5, 1, 2.5, 10} and b in {1, 2.5, 10, 100, 1000}:
      - `BETA.INV(p,a,b)` for p in {1E-10, 1E-8, 1E-6, 1E-4, 0.001, 0.01, 0.05, 0.1, 0.2, 0.35, 0.5};
      - `BETA.DIST(BETA.INV(p,a,b),a,b,TRUE)` for the same points (Excel's cdf at its own landing point, a double
        that never passes through a literal);
      - `1-BETA.INV(1-p,b,a)` and `BETA.INV(1-p,b,a)` for p in {0.05, 0.2, 0.5} (the AS 109 mirror);
      - p written as plain decimal literals (all have at most 15 significant digits, so no literal is cut; T-3175).
      Keep each call at valid arity (an invalid one makes Excel for the web reject the whole workbook, T-0040).
- [ ] The `.oracle.json` sidecar is committed with no pending cells.
- [ ] docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)" cites the case in
      its Evidence and adds a count: at how many points `BETA.DIST(BETA.INV(p))` equals p, is within one ulp of p, or
      is further (with the largest miss in ulps of p, per shape pair).
- [ ] T-3388 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-3388). Huge shapes (T-2555), p near 1 where the answer is 1 (T-3351), the crossing of
  the ported cdf (T-3398, from T-2511).
