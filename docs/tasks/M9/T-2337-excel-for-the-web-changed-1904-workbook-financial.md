---
id: T-2337
title: "Excel for the web changed 1904-workbook financial results after 2026-10-12: re-fill the 1904 probes and re-derive the 1904 financial rules"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The 1904-workbook financial rules in docs/excel-semantics, the committed 1904 probe evidence and the engine all
agree with today's Excel for the web again.

## Context
- On 2026-10-21, re-filling the 24 committed 1904 probe cases through excel-graph changed 111 of their 1,041
  cells, all of them in financial functions. The committed sidecars date from 2026-10-12 to 2026-10-20. The
  date readers and the one-day-late COUPPCD (`date-system-1904-financial-shift` A1) did not change.
- The full cell list, both values for each and the parked COUPNUM patch are in
  docs/research/excel-1904-drift-2026-10.md; the summary is in dates.md "Excel for the web changed 1904 financial
  results (2026-10)". The biggest groups:
  - ACCRINT: settlement-bound values that were 2·1462 days too large are now sane, and many former `#NUM!`
    cells compute;
  - ACCRINT near 9999 computes;
  - COUPNUM is `#NUM!` when the previous coupon is before 1904-01-01;
  - several day counts moved by about a day;
  - TBILLPRICE, TBILLYIELD and TBILLEQ over a 29 February maturity are now `#NUM!`.
- The engine models the old behavior: every financial date reads as the 1900 serial + 2·1462
  (`DateSystem::financial_1900_serial`, formatter/dates.rs; dates.md "The 1904 date system: financial day counts
  read a 1904 serial as the 1900 serial + 2·1462"). Parts of that model may now be wrong.
- New evidence committed with this task: probes/fin-coupnum-1904-epoch (41 cells) and
  probes/fin-coupnum-1904-maturity-sweep (79 + 21 cells), excel-graph 2026-10-21.
- Desktop Excel was not checked. If it still shows the old behavior, record that as an M14 question (decision 0039)
  rather than holding this task: excel-graph is the oracle of record.

## Acceptance criteria
- [x] Re-fill every 1904 probe case through excel-graph again. Confirm that the 2026-10-21 values hold on a second
      day (if they flip back, record that and stop). Then commit the new sidecars, with the change and its date in
      the Outcome.
- [x] dates.md and financial.md describe the current 1904 financial behavior: which functions still read dates
      2·1462 days on, which do not, and the epoch cuts (COUPNUM's and ACCRINT's from the T-2125 probes). Superseded
      entries are rewritten, not left contradicting the new ones.
- [x] The engine matches. The probes cases above and the re-filled 1904 cases pass, or each remaining failure has
      a filed task. T-2125's COUPNUM check lands here or in T-2125 once the rule is settled.
- [x] `make eval-baseline` records the change. Cells whose committed expectation changed are explained in the
      Outcome, and any regression goes through a decision record (directive 2).
- [x] docs/research/excel-1904-drift-2026-10.md is deleted (its findings moved into dates.md and financial.md, its
      COUPNUM draft landed or dropped), and dates.md and financial.md stop citing it.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 1900 date system.
- The 1904 schedule break near 9999 (T-1440), unless the re-fill changes it.

## Outcome
- Re-filled all 27 1904 probe cases through excel-graph on 2026-10-27 (the 24 of the drift report plus
  coup-calendar-edges-1904 and the two fin-coupnum-1904 cases). The 111 cells that changed on 2026-10-21 hold on this
  second day, bit for bit, and no other cell moved; the new sidecars are committed.
- The rule: the change is per function. ACCRINT, COUPNUM, TBILLPRICE, TBILLYIELD and TBILLEQ now read a 1904 serial as
  the date itself (s + 1462): every such cell in the 27 cases equals our 1900 evaluation of the same formula, except
  cells with literal serials. Every other financial function and YEARFRAC still reads s + 2·1462. The COUPNUM and
  ACCRINT epoch cuts (T-2125, T-2835) are the 1900 calendar-start rules at 1904-01-01; an ACCRINT issue on serial 0 is
  `#NUM!` in 1904 too. New quirk found on the way: the TBILL functions are `#NUM!` for any settlement in 9999, in both
  systems (financial.md "T-bills settled in 9999").
- Engine: `DateReading` (Shifted / Calendar) in functions/financial/dates.rs with `DateSystem::calendar_1900_serial` and
  `first_coupon_1900_serial`; ACCRINT, COUPNUM and `tbill_arguments` read Calendar. Removed the now-dead ACCRINT 1904
  quirks (`accrint_settlement_limit`, the settlement-slice override, ACCRINT's `coupon_schedule_breaks` check) and
  `from_excel_date_up_to`.
- New probes: fin-1904-calendar-readers (25 cells), fin-1900-tbill-last-day (19), accrint-calendar-start-1904 (50, T-2835's
  probe). dates.md: new entry "The 1904 date system: ACCRINT, COUPNUM and the TBILL functions read the date itself"
  replaces the drift entry, the 2·1462 entry marks the old behavior of the five; financial.md updated (COUPNUM in a 1904
  workbook VERIFIED, ACCRINT calendar start, T-bill range). docs/research/excel-1904-drift-2026-10.md deleted (its
  COUPNUM draft dropped for the general rule). T-2125 and T-2835 closed as landed here.
- Eval: probes known failures 3021 -> 2921 (100 fixed: 98 fin-coupnum-1904-* cells, fin-coupon-walk-bounds-1904 A5, A6),
  3 new cases (94 cells) all pass, no regressions. The 111 cells whose committed expectation changed all pass with the
  new values (they passed with the old ones before), so they move no baseline line.
- Desktop Excel unchecked (M14 question, decision 0039).
