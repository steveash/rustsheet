---
id: T-3152
title: "YIELD and ODDFYIELD cells T-2974's correctly rounded start moved off the bit (decision 0162)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The 148 YIELD and ODDFYIELD cells that the correctly rounded start of T-2974 attempt 2 moved off the
bit are bit-exact again, or within 15 digits, while the cells it fixed stay fixed.

## Context
- Decision 0162 recorded these cells as known failures: probes 139, ironcalc-calc 7, gen-m9 2;
  77 YIELD and 71 ODDFYIELD. Nine of them are visibly off:
  - ironcalc-calc PRICE_YIELD!I31, ODDFPRICE_ODDFYIELD!K23 and K24;
  - oddfyield-solver-sweep S!C56 and C88;
  - yield-start-one-input S!A87 and B72;
  - yield-start-rate-term S!B30;
  - yield-start-term-basis S!C47.
- Code: `yield_start` in crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs.
- Semantics: financial.md "YIELD stops short of PRICE's root", T-2974 attempt 2. One-step cells
  show the start's last bit. On yield-solver-sweep C2, C7 and C8 (one step each, still known failures), Excel's start is one ulp
  above the correctly rounded one. First check whether the regressed cells are one-step or
  multi-step cells, and whether ODDFYIELD's start is rounded like YIELD's.

- T-2974 attempt 2 scored its start arrangements with scratch scripts, not a committed kernel.
  Add the rounded-once start (exact term) to the `yield-caret` fit as a named candidate first, so
  the financial.md table can be reproduced.

## Acceptance criteria
- [ ] The nine visibly-off cells are within 15 digits, and no known-failure line is added
      (`make eval-baseline`). Otherwise the task records the candidates tried with their lab
      counts (decision 0056).
- [ ] `fn_yield_start_term_whole_years_then_leftover_days` asserts A1's and A2's exact bits again.
- [ ] `uv run rustsheet-fitlab yield-caret` scores the rounded-once start.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-2974's own cells (yield-solver-sweep C7, yield-start-rate-term B39).

## Cells added by T-2974 attempt 4 (decision 0166, 2026-10-29)
The unrounded-term start (financial.md "YIELD stops short of PRICE's root", T-2974 attempt 4) moved 58 cells off the
bit (ulps in brackets). One is visibly off: probes/yield-one-step-start S!A37. The lab model gives the same distances
on the YIELD cells, so start from the lab, not the port. `fn_yield_start_term_whole_years_then_leftover_days` pins A1
(yield-start-term-basis B5) within one ulp too; its exact bits come back with this task.
- gen-m9/1b60bf99d71424e7: Calc!A112 (1)
- probes/date-system-1900-financial-readers: S!B19 (1)
- probes/date-system-1904-financial: S!A14 (1)
- probes/oddfyield-solver-sweep: S!B26 (1), S!B46 (1), S!B95 (1), S!B99 (1), S!C42 (2), S!C49 (1), S!C90 (1), S!E60 (3), S!F21 (1), S!F23 (1)
- probes/solver-iteration-traces: Y!C8 (2), Y!D8 (2), Y!E8 (2)
- probes/yield-one-step-start: S!A111 (1), S!A171 (1), S!A172 (1), S!A178 (1), S!A182 (1), S!A199 (1), S!A232 (1), S!A25 (1), S!A34 (1), S!A37 (32), S!A65 (1), S!A76 (1), S!A94 (1), S!B19 (1), S!B25 (1), S!B34 (1), S!B37 (1), S!C25 (1), S!C34 (1), S!C37 (1), S!D30 (1), S!D39 (1)
- probes/yield-start-rate-term: S!A26 (2), S!A5 (1), S!B26 (1), S!B5 (1), S!C26 (1)
- probes/yield-start-sweep: S!E1 (1)
- probes/yield-start-term-basis: S!A29 (1), S!A3 (1), S!A48 (1), S!A5 (1), S!A53 (1), S!A8 (1), S!B29 (1), S!B3 (1), S!B48 (1), S!B5 (1), S!B53 (1), S!B8 (1), S!C48 (1), S!C8 (1)
