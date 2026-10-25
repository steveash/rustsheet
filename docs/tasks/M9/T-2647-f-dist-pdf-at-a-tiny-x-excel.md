---
id: T-2647
title: "F.DIST pdf at a tiny x: Excel's digits where (df1 x)^(df1/2) is subnormal (probes-numeric t1864-f-pdf-tiny-x)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1864]
---

## Goal
F.DIST's pdf at a tiny x gives Excel's value where Excel's density loses digits to a subnormal intermediate:
probes-numeric t1864-f-pdf-tiny-x A6, A10 and A17 match Excel (A5 to 15 digits too).

## Context
Found by T-1864's edge probes (probes-numeric t1864-f-pdf-tiny-x, t1864-f-pdf-edges, excel-graph, 2026-10-25).
Excel's F density is 0 once (df1 x)^(df1/2) underflows to 0 (T-1864 matches that, `fisher.rs` `f_pdf`), and
just above that, where the power is subnormal, Excel's value carries a large relative error that ours (an accurate
form: statrs/beta density, or Loader's form for a large df) does not:
- A6 `F.DIST(1E-215,3,1000,FALSE)`: Excel 6.42285339593616E-108, exact 6.5602056179812E-108 (2.1% low; power
  (3E-215)^1.5 = 1.6E-322).
- A10 `F.DIST(1E-160,4,1000,FALSE)`: Excel 4.007860519064344E-160, exact 4.008E-160 (3.5E-5 low; power 1.6E-319).
- A17 `F.DIST(1E-65,10,1000,FALSE)`: Excel 1.4821969375236873E-258, exact 1.3283E-258 (11.6% high; power 1E-320).
- A5 `F.DIST(1E-210,3,1000,FALSE)`: Excel 2.0745191673458065E-105, exact 2.07451916718532E-105 (7.7E-11).
The error is not simply the subnormal rounding of fl((df1 x)^(df1/2)) times the exact rest (that hypothesis misses
A6 by 1.3% and A17 by 11.6%), so Excel's factorisation is not yet known. At df2 = 5 our beta-density path happens to
match Excel to 15 digits (t1864-f-pdf-tiny-x A20, 1% below the exact density), which suggests the same
x^(a-1)·y·(1-y)/x-style subnormal product; probe more cells (df1 3..10, df2 5..1000, the power between 1E-308 and
5E-324) to pin the order of Excel's operations, then reproduce it for the subnormal range only.

## Acceptance criteria
- [ ] probes-numeric t1864-f-pdf-tiny-x A5, A6, A10, A17 match Excel (or are within 15 digits), and no passing F.DIST
      cell of any suite changes.
- [ ] The rule is recorded in docs/excel-semantics/statistics.md ("F density at large degrees of freedom (T-1864)").
- [ ] `make check` green; reviewers approve.

## Out of scope
- The bit-level last ulp at a huge df (T-2642); Excel's large-shape noise (T-2586).
