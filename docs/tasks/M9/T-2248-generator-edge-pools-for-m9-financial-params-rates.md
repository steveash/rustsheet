---
id: T-2248
title: "Generator edge pools for M9 financial params: rates, periods, signs, settlement and maturity dates, basis and frequency"
milestone: M9
lane: harness
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063]
---

## Goal
M9 financial params draw from edge pools. The decision 0094 sweeps then hit the inputs where Excel's behavior changes, not just
random interior values.

## Context
- T-2109 and T-2110 added pools for M2–M6 (`number_edges`, `unit_interval`, `angle`,
  `code_point`, `text_edges`, `numeric_text`, `number_formats`, `errors`). The M7 and M9 params
  have none (0 pooled params among 263 M9 and 32 M7 functions).
- Edges to cover:
  - rates: 0, -1, just above -1, 1E-10, 1, 10, negative;
  - nper: fractional, 0, huge (1E6), negative;
  - pv/fv/pmt sign combinations (all positive, mixed, zeros);
  - `type` 0/1 and invalid;
  - settlement and maturity dates: leap days, month ends, 1900-02-28/29/03-01, the same day, the
    wrong order;
  - basis 0–4 and invalid; frequency 1, 2 and 4 and invalid;
  - cash-flow arrays with no sign change, many sign changes, and leading or trailing zeros.
- Source edges from docs/excel-semantics/*.md (the cuts and caps already found) and from the
  existing probes, so the sweep reproduces known quirks at scale.

## Acceptance criteria
- [x] The pools are added to `tools/src/rustsheet_tools/gen/pools.py`, and `pool:` is set on the
      catalog params listed above (`eval/catalog/functions.yaml`). Each pool's edges are listed in
      a comment, with the Excel reason for each.
- [x] Tools tests: every pool value is a valid literal under 255 characters, and the draw keeps
      each param's `domain`.
- [x] Smoke check: a scratch suite with one formula per pool value, filled through excel-graph,
      returns a value for every cell. Delete the scratch suite afterwards, as T-2110 did.
- [x] No eval change: the sweep tasks regenerate the suites.
- [x] `make check` green; reviewers approve.

## Out of scope
- Generating or filling suites (the sweep tasks).

## Visible-parity stage
Prerequisite for decision 0094's sweeps; its acceptance criteria are the whole stage.

## Outcome
Added twelve financial pools to `gen/pools.py`, each edge commented with its financial.md reason: `rate` (rates, guesses,
yields, discounts, coupons), `nper` (nper, life), `period` (per, period, start/end period), `cash_amount` (pv, fv, pmt,
cost, salvage, principal, par, investment; independent draws give every sign combination), `price` (pr, redemption),
`annuity_type`, `security_date` (settlement, maturity, issue and the coupon/purchase dates: serials 0, 0.5, -1, 1, 59-61,
leap days and February ends, month ends, 2958465/2958466, a text date; settlement and maturity share it, so the same day
and the wrong order occur), `basis`, `frequency`, `npery`, `cash_flows` (no, one and several sign changes, leading and
trailing zeros, text, booleans and an error inside) and `cash_dates`. `pool:` is set on 257 params of 53 financial
functions (calc_method and no_switch take the existing `flag` pool). Tools tests: every pool's string literals stay under
255 characters (now checked over all pools), and every pooled financial param with a soft int or enum domain still draws
in-domain at least 60% of the time and reaches its pool's out-of-domain edges.

Smoke check: a scratch suite with one formula per entry of the twelve pools (169 cells) filled through excel-graph
returned a value for all 169; the engine matches 127. Besides the known PMT/IPMT/YIELD last-digit misses it found three
mismatches, filed as T-2390 (IRR over an array with an error element is `#VALUE!`), T-2391 (YIELD at a price of 1E6 is
`#NUM!`) and T-2392 (EFFECT at npery 1E10); the observations are in financial.md "Financial edge-pool smoke check
(T-2248)". The scratch suite was deleted. No eval change (the sweep tasks regenerate gen-m9).
