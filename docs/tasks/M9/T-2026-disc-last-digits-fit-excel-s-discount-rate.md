---
id: T-2026
title: "DISC last digits: fit Excel's discount-rate evaluation order"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2023]
---

## Goal
DISC returns Excel's bits on the cached workbook, the day-count probes and the sweep, through a fitted
evaluation order of `(redemption - pr) / redemption * B / DSM`.

## Context
Gap triage 2026-10-19. All misses are 1-2 ulps (NumericUlp), on bases 0 and 1:
- eval/baseline/ironcalc-calc.txt, `ironcalc-calc/FINANCIAL/DISC.xlsx` Sheet1 F2, F6, F10 (Excel
  0.05207142857142884, ours ...883), F15, F19 (0.12828823250549048 / ...046).
- eval/baseline/probes.txt, probes/date-system-1900-financial-daycount S!A1
  `DISC(DATE(2024,1,31),DATE(2024,2,29),97,100,0)` (0.37241379310344863 / ...486), A2 (0.14210526315789485
  / ...487), A4 (0.25116279069767466 / ...746).
Candidates: `(1 - pr/red) * (B/DSM)`, `(red - pr)/red * B / DSM`, `(red - pr) * B / (red * DSM)`, with
the year basis B and DSM as `fn_disc` computes them (crates/rustsheet-engine/src/functions/financial/discount.rs);
the discount-security family (INTRATE, YIELDDISC, PRICEDISC, RECEIVED) shares the day count, and its
last digits are T-1374's: check those cells do not regress.
**Needs an oracle sweep:** fit against `probes-numeric/m9-disc-sweep` (T-2023), never blind; time-boxed
(decision 0056): record every candidate's bit-exact and within-4-ulp counts.

## Acceptance criteria
- [ ] The 8 lines above leave eval/baseline/ironcalc-calc.txt and eval/baseline/probes.txt, and the sweep's
      cells pass, or the Outcome lists the residue with the candidate table.
- [ ] No new F lines in any baseline.
- [ ] The fitted order is recorded in docs/excel-semantics/financial.md "Discount-security functions" with
      its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- INTRATE, YIELDDISC, PRICEDISC, RECEIVED last digits (T-1374).
