---
id: T-3566
title: "Oracle sweep: BETA.INV at shapes below 0.01 near p = 1/2 and at range-scaled quantiles, with BETA.DIST at Excel's landing point (for T-3562)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-3562 has dense excel-graph evidence of where Excel's BETA.INV lands in two regions no probe covers yet. One is shapes
below 0.01, where the cdf is almost flat around 1/2. The other is a quantile scaled to a range [A, B]. Each landing
point comes with Excel's own cdf at it.

## Context
- Gap triage 2026-11-01. Two gated cells of ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx are visibly off and owned
  by T-3562: H26 `BETA.INV(0.4998779049300665,0.0001,0.0001,0.15,1.2)`, which Excel gives as 0.2339999999999406,
  1.3E-11 below the 50-digit root; and H5 `BETA.INV(0.6734553797493481,3,600,2000,5489582)`, which Excel gives as
  33587.000000000226, 6.6E-15 above the root. A fit needs more than two points and a hold-out (decision 0188 R4.1).
- T-3387 sweeps BETA.INV at shapes 0.5 to 10 against b 1 to 1000 on [0, 1], and T-3388 fits from it. Neither has a
  shape below 0.5 or bounds A, B.
- Rule: docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)".

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t3566-beta-inv-tiny-shape-and-range.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in
      `m9-fin-power.case.json`), is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Its grid:
      - tiny shapes: a = b in {1E-5, 1E-4, 1E-3, 0.01}, plus (a, b) in {(1E-4, 0.01), (0.01, 1E-4)}, with p in
        {0.4998, 0.4999, 0.49995, 0.5, 0.50005, 0.5001, 0.5002, 0.45, 0.55, 0.1, 0.9}. Each gets `BETA.INV(p,a,b)`,
        `BETA.INV(p,a,b,0.15,1.2)` and `BETA.DIST(BETA.INV(p,a,b),a,b,TRUE)`;
      - range-scaled: `BETA.INV(p,3,600,A,B)` and `BETA.INV(p,3,600)` with (A, B) in {(0, 1), (2000, 5489582), (-1, 1),
        (0.15, 1.2)} and p in {0.05, 0.3, 0.5, 0.6734553797493481, 0.9}; and `BETA.INV(p,2,2.5,A,B)` for the same A,
        B and p;
      - p written as literals of at most 15 significant digits. The exception is 0.6734553797493481, which has 16
        digits and would be cut (T-3175): take it from a cell holding `=BETA.DIST(33587,3,600,TRUE,2000,5489582)`,
        which Excel returns as exactly that double (BETADIST_BETAINV G5). Keep each call at valid arity (T-0040).
- [ ] The `.oracle.json` sidecar is committed with no pending cells.
- [ ] docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)" cites the case and
      adds counts: at how many points `BETA.DIST(BETA.INV(p))` equals p, is within one ulp of p, or is further. It also
      says whether the scaled quantile is A + (B - A)·BETA.INV(p,a,b) bit for bit.
- [ ] T-3562 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-3562, T-3388). Shapes from 0.5 up on [0, 1] (T-3387).
