---
id: T-2639
title: "Probe ODDLPRICE, ODDLYIELD and ACCRINT schedules for a 29th/30th date across a February (does the day drift?)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for whether the ODDLPRICE, ODDLYIELD and ACCRINT coupon walks keep a 29th/30th date's day across a
February, as the COUP*, PRICE and ODDF* schedules do, recorded in financial.md; a task to implement it if ours differs.

## Context
T-2486 found that the COUP*, PRICE, YIELD, DURATION, MDURATION and ODDF* schedules move each coupon date back whole
periods from the maturity, so the day does not drift to the 28th (probes/coup-calendar-edges B1-B14, J1-J14;
docs/excel-semantics/financial.md "Coupon dates keep the maturity's day"). The ODDL* walk (`change_month` in
crates/rustsheet-engine/src/functions/financial/financial_bonds.rs) and ACCRINT's (accrint.rs) still step from date to
date, which the entry marks unprobed. Write a probe case (excel-probe skill): ODDLPRICE/ODDLYIELD with a `last_interest`
or maturity on the 29th or 30th of a month that is not its last, frequencies 2 and 4, so the walk passes a February;
ACCRINT with `first_interest` on such a day and an issue several periods before it, `calc_method` TRUE and FALSE.

## Acceptance criteria
- [ ] A probe case with excel-graph values; the finding recorded in financial.md (that entry's "unprobed" line replaced).
- [ ] If ours differs, a follow-up implementation task filed with `make new-task`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the change (unless the probe shows ours already matches).

## Outcome
Probe `probes/oddl-accrint-february-day` (49 cells, excel-graph): ODDLPRICE/ODDLYIELD ignore whether the last
interest is the 29th or 30th (single and multi-period spans across a February), and ACCRINT with a first interest on
the 29th or 30th across a February matches Excel on every cell. The engine matched all 49, so no follow-up
implementation task. Finding recorded in financial.md (the "unprobed" line replaced). Eval delta: +49 passing probe cells.
