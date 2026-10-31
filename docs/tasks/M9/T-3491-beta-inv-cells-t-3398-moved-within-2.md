---
id: T-3491
title: "BETA.INV cells T-3398 moved within 2 ulp: Excel's end of the final bracket"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3398]
---

## Goal
The BETA.INV cells T-3398 moved by 1 or 2 ulp, all still within 15 digits, return Excel's bits.

## Context
- T-3398 (2026-10-31) made BETA.INV cross BETA.DIST's own kernel and take the even-significand end of the final
  bracket. That fixed 9 visibly-off cells and moved these known-good cells by 1 or 2 ulp (decision 0083's allowance,
  condition 4: this task names them):
  - gated, ironcalc-calc STATISTICAL/BETADIST_BETAINV.xlsx: Sheet1!H2 (Excel 3.0, 2 ulp), H25 (0.23400000000000004,
    1 ulp), H63 (2.8388452645039353, 2 ulp), H64 (3.0, 2 ulp), H67 (3.0, 1 ulp);
  - ungated, probes-numeric: m9-beta-inv Calc!A128 and t1745-beta-inv-roundtrip J124 `BETA.INV(0.001,10,10)` (Excel
    0.18903711590479788, 2 ulp); m9-beta-inv A130 and t1745 J126 `BETA.INV(0.5,10,10)` (Excel 0.5, ours
    0.4999999999999999, 2 ulp).
- The end of the bracket is a scored fit (statistics.md "BETA.INV solves Excel's own CDF to the crossing", T-3398
  paragraph): the nearer end fixes more but makes BETADIST_BETAINV.xlsx H16 visibly off. Excel's own stop rule is
  T-3387 (sweep) and T-3388 (fit); this task is their bit-level remainder for these cells.

## Acceptance criteria
- [ ] The cells above return Excel's bits, or the Outcome shows the rule that would and why it is not taken.
- [ ] No cell becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F quantiles (T-3485).
