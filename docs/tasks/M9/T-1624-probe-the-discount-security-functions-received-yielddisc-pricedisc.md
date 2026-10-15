---
id: T-1624
title: "Probe the discount-security functions RECEIVED, YIELDDISC, PRICEDISC, INTRATE, PRICEMAT, YIELDMAT to the evidence floor"
milestone: M9
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): RECEIVED (10), YIELDDISC (11), PRICEDISC (12), INTRATE (16), PRICEMAT (17), YIELDMAT (13) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
every basis 0-4 and invalid basis, settlement >= maturity, issue after settlement, fractional dates (truncation), non-positive rate/price/investment, dates as text, leap-year and month-end day counts. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
(`make new-task`); fixing the engine is out of scope.

## Acceptance criteria
- [ ] Each listed function has ≥ 20 Excel-evidence cells in the report.
- [ ] `make eval-baseline` lowers quality/evidence-floor.json by the number of functions lifted
  over the floor; new failing cells are recorded as new-coverage failures with fix tasks.
- [ ] Quirks recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the mismatches found (filed as tasks).
