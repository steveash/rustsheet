# 0131 — AMORDEGRC below period 1 reads calculation-order state

- **Status:** accepted (2026-10-27, T-2825)
- **Context:** Excel's AMORDEGRC with 0 < period < 1 computes no charge of its own: it returns a value the last
  AMORDEGRC or AMORLINC Excel ran before it left behind, 0 in a fresh workbook (docs/excel-semantics/financial.md
  "AMORDEGRC below period 1"). T-2825 offered two options: keep the cells that read a neighbour's value as known
  failures, or carry the state and evaluate in Excel's order. Its probes (probes/amordegrc-fraction-state-h to -y,
  excel-graph 2026-10-27) show the state follows exactly the fresh-load order SPEC-0017 already reproduces for
  WEEKNUM's stale `return_type` (recalc.md "The order Excel calculates a freshly loaded workbook in"): last sheet
  first (-n), bottom row first and each row right to left (-p, -q), a precedent first (-o). What each call leaves is
  a short table: an AMORDEGRC from period 1 on its value, at period 0 a 0, an AMORLINC its rounded first-period
  charge; errors and periods past the asset life leave it as it was, and other functions do not touch it.
- **Decision:** model it. The per-recalc state SPEC-0017 introduced for WEEKNUM becomes one `OrderState` (WEEKNUM's
  start weekday and AMORDEGRC's left-over value), reset, saved and restored as a unit by the passes and the
  Excel-order chain; a read of either part triggers the Excel-order pass. A lifted AMORDEGRC's fractional elements are
  0 (-x). No other ordering or design changes.
- **Consequences:** the 23 cells T-2825 listed (probes/amordegrc-fractional-period, amor-argument-check-order,
  fin-amor-period-bound, amordegrc-fraction-alone-f) pass, and so do the new probes; gen-m9 d912671baa1364aa Calc!A76
  stays a known failure, unexplained by the table. Workbooks without a fractional-period AMORDEGRC run as before (no
  extra pass). Further calculation-order quirks found later join `OrderState` rather than adding a mechanism of their
  own; the SPEC-0017 move budget and its DEVIATION (dates.md) apply to them as they do to WEEKNUM.
