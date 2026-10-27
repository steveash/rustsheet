---
id: T-0463
title: "Calipers baseline is stale: 36 PPMT, NPV and SUM cells in pv23_007/008/009/056 fail on main by a few ulp"
milestone: M11
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The calipers cell baseline is current again: the 36 cells that fail on `main` while the baseline still lists them as passing
match Excel or are recorded, and `rustsheet-eval update-baseline --suite calipers` can ratchet the suite without
`--allow-regressions`.

## Context
Found by T-0461 (2026-10-25). `rustsheet-eval check --suite calipers` with a binary built from `main` 583620cc reports
`known failures 3461 -> 2897 (fixed 600, REGRESSIONS 36)`. The calipers suite is not in `make check` (`gate = false`,
eval/suites/calipers/suite.toml), so merges since its last ratchet went unchecked there. The 600 cells fixed meanwhile
cannot be recorded until these 36 are dealt with. The check prints the first 25:
- pv23_007 `Pro Forma Cash Flow Statement`: AF13, AV13, BG15, BI13, BI15, BO13, G15, G24, N13, N15, N24 (e.g. G15
  `=SUM(G12:G14)`: Excel 3867.435351082244, ours 3867.4353510822402);
- pv23_008: `Debt Financing!E231`, `Est Property Value!B47` and `C47` (`=NPV(A47,B$41:K$41)`: Excel 1846873.2673480364,
  ours ...362), `IRR!B26` (`=NPV(B25,C23:L23)+B23`);
- pv23_009 `Equity Waterfall Model!T14` (PPMT: Excel -21525.32858453232, ours ...315);
- pv23_056 `Monthly Cash Flow Statement`: AE27, AH27, AK27, AQ27, AR27, AV27, AY27, BB27, BE27 (PPMT: Excel
  -92319.543415507, ours ...98).

They are last-digit PPMT/IPMT and NPV results and their sums. pv23_007, 008 and 009 run with iteration on. The likely
cause is a recent change to the annuity or NPV kernels (T-0458's calipers PPMT/IPMT cells, T-1528). Find the commit
with `git bisect` on `check --suite calipers` before fixing anything.

- T-2688 (2026-10-26) fixed pv23_056 Summary!F23 (XIRR empty guess); its stale `F` line is left for this task's
  ratchet.

## Acceptance criteria
- [ ] The 36 cells are fixed, or recorded through a reviewed decision if the change behind them is kept.
- [ ] `rustsheet-eval update-baseline --suite calipers` succeeds without `--allow-regressions` and records the fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Gating calipers in CI (decision of T-0446).
