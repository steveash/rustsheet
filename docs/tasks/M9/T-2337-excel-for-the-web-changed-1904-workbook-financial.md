---
id: T-2337
title: "Excel for the web changed 1904-workbook financial results after 2026-10-12: re-fill the 1904 probes and re-derive the 1904 financial rules"
milestone: M9
lane: research
status: in-progress
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
- [ ] Re-fill every 1904 probe case through excel-graph again. Confirm that the 2026-10-21 values hold on a second
      day (if they flip back, record that and stop). Then commit the new sidecars, with the change and its date in
      the Outcome.
- [ ] dates.md and financial.md describe the current 1904 financial behavior: which functions still read dates
      2·1462 days on, which do not, and the epoch cuts (COUPNUM's and ACCRINT's from the T-2125 probes). Superseded
      entries are rewritten, not left contradicting the new ones.
- [ ] The engine matches. The probes cases above and the re-filled 1904 cases pass, or each remaining failure has
      a filed task. T-2125's COUPNUM check lands here or in T-2125 once the rule is settled.
- [ ] `make eval-baseline` records the change. Cells whose committed expectation changed are explained in the
      Outcome, and any regression goes through a decision record (directive 2).
- [ ] docs/research/excel-1904-drift-2026-10.md is deleted (its findings moved into dates.md and financial.md, its
      COUPNUM draft landed or dropped), and dates.md and financial.md stop citing it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1900 date system.
- The 1904 schedule break near 9999 (T-1440), unless the re-fill changes it.
