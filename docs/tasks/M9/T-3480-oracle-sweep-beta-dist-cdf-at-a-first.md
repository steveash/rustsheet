---
id: T-3480
title: "Oracle sweep: BETA.DIST cdf at a first shape within a few hundred ulps of 1 (both sides) against b 1.5 to 20 and x 0.01 to 0.5"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The fit task T-3481 has dense excel-graph evidence of BETA.DIST's cdf at a first shape a few ulps either side of 1,
enough for a train set and a hold-out split (decision 0188 R4.1). Today it has only the 594 cells of
probes-numeric/t2464-w2f-unit-shape, and only a handful of those sit at the shapes and x where Excel leaves the math.

## Context
- Gap triage 2026-10-31 (`make eval` on main b9558688e): 6 visibly-off cells of probes-numeric/t2464-w2f-unit-shape
  (Calc!A211, A407, A409, A411, A421, A423) have no open owner, and T-3481 now owns them. Their first shape is 1 - k·2^-52 for
  small k (`67108863*2^-26+67108859*2^-52` is 1 - 5·2^-52), or 0.999999999999999, with b = 2.5 or 5 and x = 0.1.
  Excel is 9E-16 to 3.1E-15 relative off the 50-digit math, and ours is 1.7E-16 to 9E-16. Excel's value
  moves with the shape's last bits: `BETA.DIST(0.1,a,5,TRUE)` is 0.4095099999999998 at three of these shapes,
  0.4095099999999996 at another and 0.40951000000000004 at a = 0.999999999999999. So it is not the a = 1 closed form
  (`1-(1-x)^b` in doubles is 0.40950999999999993). The differences are far above the oracle's 4-ulp floor (R4.4).
- statistics.md: "Incomplete beta at a shape of 1 outside the closed forms" (T-3007) and the shape-of-1 w that T-3170
  ported (Excel's expm1 and log1p). T-3150 probes w1 below 2^-53 at a shape of exactly 1. This sweep is about shapes
  next to 1.

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t3480-beta-dist-near-unit-shape.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in
      `m9-fin-power.case.json`), is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Grid:
      - first shape a = 1 + s·k·2^-52 for s in {-1, +1} and k in {1, 2, 3, 5, 8, 16, 64, 256, 1024}, written as exact
        dyadic sums like the w2f case (`67108863*2^-26+...`), never as rounded decimals; plus a = 1 itself;
      - b in {1.5, 2.5, 5, 10, 20}; x in {0.01, 0.05, 0.1, 0.2, 0.35, 0.5};
      - each point as `BETA.DIST(x,a,b,TRUE)`, and the swapped call `BETA.DIST(1-x,b,a,TRUE)`, in separate cells.
      That is 19 × 5 × 6 × 2 = 1,140 cells. Keep each call at valid arity: an invalid one makes Excel for the web
      reject the whole workbook (T-0040).
- [ ] The `.oracle.json` sidecar is committed with no pending cells.
- [ ] A second, smaller case `t3480-beta-dist-near-unit-shape-hold.case.json` (300 points, k drawn log-uniformly from
      1 to 4096, b and x uniform over the same ranges, seed stated in its `hypotheses`) is filled in the same PR for
      T-3481's hold-out. It is never used for fitting.
- [ ] statistics.md "Incomplete beta at a shape of 1 outside the closed forms" cites both cases in its Evidence, with a
      count of how many cells are bit-exact, within 4 ulp and within 15 digits against today's engine.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-3481).
