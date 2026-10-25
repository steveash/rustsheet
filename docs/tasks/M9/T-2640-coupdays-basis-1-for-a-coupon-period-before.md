---
id: T-2640
title: "COUPDAYS basis 1 for a coupon period before 1900-01-01: Excel's irregular lengths (gen-m9, 1 cell)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
COUPDAYS on basis 1 gives Excel's length for a coupon period that starts before 1900-01-01, including the semiannual
month-end schedules where Excel differs from the real-calendar length and settlement 0 differs from settlement 1.

## Context
Split from T-2486, which probed it (probes/coup-calendar-edges C1-C21, D1-I12; docs/excel-semantics/financial.md
"COUPDAYS basis 1 for a period before 1900-01-01"). Ours counts the real calendar (`coupdays_actual` in
crates/rustsheet-engine/src/functions/financial/financial_bonds/coupon_schedule.rs). Excel matches that on quarterly,
annual and mid-month schedules but gives 181 for a March/September or May/November month-end semiannual schedule (real
182) and 184 for a June/December one at settlement 0 or 5E-05 (181 at settlement 1). Find the rule first (more probes:
other maturity years, the 1904 date system, settlements 0.5 and 1 against each month), then implement it.

Owned gen-m9 cell (T-2257 triage, excel-graph, 2026-10-23):
- gen-m9/b72d952c5f52cb4d Calc!A88 `=COUPDAYS((Data!B7*(-0.5)),2958465,Data!B1,1)`: Excel 184.0, ours 181.0

## Acceptance criteria
- [x] Visible parity: the gen-m9/b72d952c5f52cb4d Calc!A88 known-failure line disappears from eval/baseline/gen-m9.txt,
      and probes/coup-calendar-edges C1-I12 pass, with no new known-failure lines in any baseline.
- [x] The rule is written into financial.md (Status no longer "unexplained") with its probe evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 1904 coupon rules (T-2125, T-2337).

## Outcome
The rule is not specific to 1900. Excel's basis-1 COUPDAYS is not COUPNCD - COUPPCD. It walks the schedule back from
maturity without the end-of-month rule, takes the first date after settlement (B), and counts from B moved back one
period, landing on a month end only when B is one. If that start is after settlement it steps back once more, and a
pre-1900 date compares one higher than its serial. That one rule also covers the old 29 February special case, and it
reproduces all 7,422 basis-1 COUPDAYS cells of the 1900-system probes. New probes: coupdays-pre1900-control (1,728
cells), -sweep (6,316) and -period (148). financial.md "COUPDAYS basis 1 counts from the chained next coupon" replaces
"COUPDAYS basis 1 around 29 February" and "COUPDAYS basis 1 for a period before 1900-01-01". `coupdays_actual` is
rewritten. Eval: gen-m9 known failures 4216 -> 4215 (b72d952c5f52cb4d Calc!A88). Probes 2644 -> 2629 (coup-calendar-edges
C/D/F cells). The 3 new cases pass in full, with no regressions.
