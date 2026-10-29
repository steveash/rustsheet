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
- [ ] `fn_yield_start_term_whole_years_then_leftover_days` asserts A2's exact bits again.
- [ ] `uv run rustsheet-fitlab yield-caret` scores the rounded-once start.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-2974's own cells (yield-solver-sweep C7, yield-start-rate-term B39).
