---
id: T-2248
title: "Generator edge pools for M9 financial params: rates, periods, signs, settlement and maturity dates, basis and frequency"
milestone: M9
lane: harness
status: in-progress
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
- [ ] The pools are added to `tools/src/rustsheet_tools/gen/pools.py`, and `pool:` is set on the
      catalog params listed above (`eval/catalog/functions.yaml`). Each pool's edges are listed in
      a comment, with the Excel reason for each.
- [ ] Tools tests: every pool value is a valid literal under 255 characters, and the draw keeps
      each param's `domain`.
- [ ] Smoke check: a scratch suite with one formula per pool value, filled through excel-graph,
      returns a value for every cell. Delete the scratch suite afterwards, as T-2110 did.
- [ ] No eval change: the sweep tasks regenerate the suites.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Generating or filling suites (the sweep tasks).

## Visible-parity stage
Prerequisite for decision 0094's sweeps; its acceptance criteria are the whole stage.
