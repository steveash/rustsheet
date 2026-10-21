---
id: T-2257
title: "Sweep gen-m9 financial: the 49 functions under 400 Excel cells to at least 500"
milestone: M9
lane: research
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0063, T-2248]
---

## Goal
The 49 M9 financial functions under 400 Excel cells reach at least 500. Each function gets enough Excel-checked cells, with edge values, that a zero
visibly-off count means something (decision 0094).

## Context
- Functions and their current Excel cells (eval of 2026-10-21, `main` 7dab5361), 49 in all:
  ACCRINTM (77), AMORDEGRC (83), AMORLINC (58), COUPDAYBS (187), COUPDAYS (208), COUPDAYSNC (227), COUPNCD (221), COUPNUM (248), COUPPCD (266), CUMIPMT (71), CUMPRINC (65), DB (48), DDB (52), DISC (68), DOLLARDE (27), DOLLARFR (26), DURATION (134), EFFECT (60), FVSCHEDULE (47), INTRATE (54), IPMT (388), IRR (379), ISPMT (46), MDURATION (79), NOMINAL (51), NPER (266), NPV (198), ODDFPRICE (315), ODDFYIELD (174), ODDLPRICE (209), ODDLYIELD (132), PDURATION (25), PPMT (387), PRICE (215), PRICEDISC (42), PRICEMAT (58), RECEIVED (49), RRI (54), SLN (29), SYD (29), TBILLEQ (90), TBILLPRICE (90), TBILLYIELD (78), VDB (116), XIRR (305), XNPV (53), YIELD (293), YIELDDISC (41), YIELDMAT (54).
- Template: T-2114. It swept gen-m5 to 22,400 Excel cells with `--focus` and edge pools, filled
  them through excel-graph, gated the suite, and triaged 260 new failures into tasks.
- Generate with `cd tools && uv run rustsheet-gen --milestone M9 --focus --category financial --count <n>
  --formulas 400 --seed <new>`. `--functions` and `--category` come from T-0063. Pick `--count`
  so that each function is the outer call of about 250 formulas.
- Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9` and `make eval-baseline`.
- Dates must be valid security dates for the bond functions (settlement < maturity) on most draws, and invalid on a deliberate minority, so that both the values and the #NUM! rules get coverage.
## Acceptance criteria
- [ ] In `make eval`'s "Excel evidence per function", every function listed above has at least
      400 Excel-backed cells, and at least 200 of them are new generated formulas with the
      function as the outer call. Aim for 500. Count from T-0062's checkpoint section once it has
      landed, and from the suite's cases until then.
- [ ] Every new cell is Excel-backed (excel-graph sidecars; the fill reports 0 failures, or the
      Outcome explains each refused case), and the suite is gated.
- [ ] `make eval-baseline` records the new failures as new coverage only: 0 regressions, and the
      decision 0088 lock moves only by new coverage.
- [ ] The gap-triager traces the new failures (as T-2114 did). Every visibly-off family gets a
      `campaign: visible` task, new or extended, listed in the Outcome with its cell count.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any function: the triaged tasks do that.
- New edge pools, which go to the pool tasks. If a parameter has no pool, note it in the Outcome
  rather than inventing values ad hoc.

## Visible-parity stage
Coverage half of decision 0094's checkpoint: this task is done when its functions meet the
400-cell bar and their visibly-off cells all have tasks.
