---
id: T-3516
title: "Oracle sweep: F.INV, F.INV.RT and FINV at one df from 1E5 to 1E10 against a partner df of 1 to 10, with BETA.INV at the mapped shapes"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A committed, excel-graph-filled probes-numeric case gives T-3501 the Excel points it needs to model the F quantile
at a huge df: F.INV, F.INV.RT and FINV where one df is 1E5 to 1E10 (odd and even, integer and fractional) and the
other is 1 to 10, with BETA.INV at the shapes F.INV maps to.

## Context
- Gap triage 2026-10-31: gen-m9/f34f66fb7f81e673 Calc!A209 `F.INV.RT(Data!E7,2,(Data!$C9+Data!E2))`, which is
  `F.INV.RT(0.5,2,123456789.1229)` (df truncated to 123456789), is 0.6931471803615475 in Excel and 0.6931471872147749
  in the engine: 1E-8 apart, and the engine is 4E-9 off the 50-digit math (0.69314718445161477), outside Gate A's
  envelope. The cached evidence (probes-numeric/t1538-f-inv-huge-df, t1538-f-quantile-switchover; statistics.md
  "F quantile at a huge df goes through Excel's BETA.INV (T-1538)") has only round dfs (1E5, 1E6, 1E8, 1E10, and
  99999 at p = 1E-12), and there the engine matches Excel within a few ulps: `F.INV.RT(0.5,2,1E8)` is Excel
  0.6931471826011306, ours 0.6931471826011304 (t1538-f-inv-huge-df A10). An odd df (a half-integer shape d2/2) or a
  df between the probed decades is the untested axis.
- T-3501 owns the fit and depends on this sweep (gap-triager rule: a numeric fit that needs more Excel points than the
  cached workbooks give is not attempted blind).

## Acceptance criteria
- [ ] A new case `eval/suites/probes-numeric/cases/t3500-f-inv-huge-df-sweep.case.json` (+ `.oracle.json`, filled
      with `cd tools && uv run rustsheet-oracle fill`, excel-graph) with, for each huge df in {99999, 100000, 1000001,
      12345678, 123456789, 123456790, 1E8+1, 987654321, 1E9+1, 4999999999, 1E10} and each partner df in
      {1, 2, 3, 5, 10}: `F.INV(p,d1,d2)` and `F.INV.RT(p,d1,d2)` with the huge df on each side and `FINV(p,·,·)` with
      the huge df second, at p in {1E-6, 0.025, 0.3, 0.5, 0.975, 0.999999}, plus `BETA.INV(p,d1/2,d2/2)` and
      `BETA.INV(p,d2/2,d1/2)` at the same points (about 2,000 cells; keep it under the oracle's per-workbook limit or
      split it into two cases).
- [ ] A separate hold-out case `t3500-f-inv-huge-df-holdout` of about 300 points with the huge df drawn log-uniform
      from 1E5 to 1E10 (a fixed seed, fractional values included) and the partner df from 1 to 10, filled the same way,
      for T-3501's R4 hold-out and fresh-sweep guards. T-3501 must not tune on it.
- [ ] `make eval` runs both cases (probes-numeric is gate-free, so nothing is added to eval/baseline), and the
      Outcome records their within-15-digit and bit-exact counts on current main, with no regression elsewhere.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-3501).
- The tiny-p quantiles (decision 0188 R3 case 1, T-3358).
