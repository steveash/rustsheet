---
id: T-0458
title: "Calipers: 35 PPMT/IPMT-fed cells moved by ulps after T-2242"
milestone: M11
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Calipers' Excel-saved goldens and the probes behind T-2242 agree on how PPMT and IPMT discount.
The calipers baseline then records its fixes with no regressions.

## Context
Merging main into the T-0455 branch brought T-2242 ("PPMT discounts with Excel's EXP and an
atanh-series log of 1+rate"). `rustsheet-eval update-baseline --suite calipers` now reports 591
fixed cells and 35 regressions. All 35 are last-digit (ulp) moves in cells fed by PPMT/IPMT:
- pv_goldens/pv23_007 "Pro Forma Cash Flow Statement" AF13, AV13, BG15, G15, G24, N13 and others;
- pv23_008 "Debt Financing"!E231, "Est Property Value"!B47:C47, IRR!B26;
- pv23_009 "Equity Waterfall Model"!T14;
- pv23_056 "Monthly Cash Flow Statement" row 27.
They matched Excel bit for bit before T-2242. Calipers is not gated in CI, so the move went
unnoticed. The calipers baseline was left as it was, because recording it needs
`--allow-regressions`.

## Acceptance criteria
- [ ] Find which PPMT/IPMT arguments the moved cells use, and probe them in Excel
      (excel-probe skill). Decide whether T-2242's formula or the earlier one matches Excel for
      them, citing evidence, and update docs/excel-semantics accordingly.
- [ ] `rustsheet-eval update-baseline --suite calipers` succeeds without `--allow-regressions`,
      recording the 591 fixed cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other Calipers failures.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
