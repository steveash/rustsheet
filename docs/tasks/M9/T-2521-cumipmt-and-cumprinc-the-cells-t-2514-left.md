---
id: T-2521
title: "CUMIPMT and CUMPRINC: the cells T-2514 left (type 1 long ranges, log1p at 0.1, 1E-15 cancellation, periods near 1E15, balance-band edge probes)"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: [T-2514]
campaign: visible
---

## Goal
CUMIPMT and CUMPRINC are within half a unit of the 15th significant digit of Excel on the cells T-2514
left visibly off, or each one is documented in financial.md with the forms tried.

## Context
T-2514 made the short principal one geometric sum (`GEOMETRIC_MIN_DISCOUNT`) and type 1's tiny-rate interest
the payments less the principal; docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over many periods"
(T-2514 notes) has the fit and what was tried. Left, by cause:
- Type 1 over a long range: b73c6cc96e093f03 A62 `CUMPRINC(2,151,319288,3,42,1)` -1.049166891970377E-47 (40
  periods, so the closed form; ours ...70404E-47). The closed form is 103 ulps off and the geometric sum 245.
- 9d9ceba257f8001b A372 `CUMPRINC(0.1,1023,478581,7,24,1)`: the geometric sum is 67 ulps off. It is bit-exact
  with `L = log1p(0.1)` one ulp low. No log1p rewrite tried reproduces that: `log(u)-((u-1)-x)/u` and
  `log(u)·x/(u-1)` give the correctly rounded value, and `log(1+x)` is 5 ulps high.
- 79d87fc527927626 A352 `CUMIPMT(1E-15,283,(1.005+3.14159+1E15)%,2,16,0)` is -0.1456298828125 and ours is
  -0.14581298828125. That is 3 ulps of `pmt·m` (5.3E11). No pairing of 10 payment forms with 5 principal
  forms (sum, balance drop, geometric, closed, `pow`) gives Excel's value.
- e168be7f6d3f84c2 A379 `CUMIPMT(1E-9,235,379138,7,14,1)`: neither the IPMT sum nor the payments less the
  principal fits (T-2363 notes).
- Periods near 1E15 (untouched by T-2514): probes fin-remaining-later-argument-ranges K3,
  fin-cumipmt-overflow-and-forms A9-A11, fin-cumipmt-period-bound A18 (type 1) and A6.
- Gate edges with no oracle cell nearby; probe cells that bracket each edge:
  - type 0's balance-form band [2^-10, 2^10] (T-2363);
  - `GEOMETRIC_MIN_DISCOUNT` = 8 (between `CUMPRINC_CUMIPMT.xlsx` H25 at 3.96 and H4 at 8.9);
  - `TINY_RATE_GROWTH` = 2^-10.
- Type 1 from period 1 (gap triage 2026-10-27, `make eval` on 7dba3bfe). Only deferred tasks named these cells (T-2243,
  T-2730), but they are visibly off: ironcalc-calc CUMPRINC_CUMIPMT.xlsx Sheet1!G2 = probes/fin-cumipmt-overflow-and-forms
  A7 `CUMIPMT(0.0083,36,8000,1,10,1)`, Excel -520.1181191021385, ours -520.1181191021378. T-2363 found that Excel's
  CUMIPMT is `-pmt·m - CUMPRINC` and that G2 is bit-exact from Excel's own H2. So G2 is H2's 1-ulp miss
  (`CUMPRINC(0.0083,36,8000,1,10,1)`) magnified by the cancellation. Fit H2's type 1 principal and G2 follows.
Code: functions/financial/cumulative.rs.

## Acceptance criteria
- [x] Each cell above is visibly right (no new known-failure lines, `make eval-baseline`), or it is documented in
      financial.md with the forms tried.
- [x] Probe cells (excel-probe skill) bracket the three gate edges, and the gates move to the evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- Bit-exact fits of cells that are already visibly right (T-1374).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns the visibly-off cells of CUMIPMT (6 of 9) and CUMPRINC (3). Fixing them lifts CUMPRINC to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells. CUMIPMT also needs its other 3 cells owned (T-0087 sweep).

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): CUMIPMT 1. This task now owns them:
- probes/fin-cumipmt-overflow-and-forms: Sheet1!A10

## Outcome
- New probe case probes/fin-cumipmt-gate-edges (87 cells, excel-graph 2026-10-28) brackets the three gates and type 1
  from period 1.
- Excel's short principal discounts with its own `ln(1+rate)`, PPMT's `growth_log`, not `log1p`: A372 is now to the
  bit (A62 too as a geometric sum, but its 40 periods still take the closed form). `GEOMETRIC_MIN_DISCOUNT` moved
  from 8 to 3.6 on the evidence (gate-edges C/D 3.62 to 9.34 against fin-argument-modes H80 at 3.58).
- Type 0's short interest is the payments less the principal wherever the principal is no larger than the interest
  (gate-edges B1-B11 across 2^10, H72). `BALANCE_GROWTH_MIN` and `TINY_RATE_GROWTH` stay: the probes across them are
  off with both forms.
- Eval: 11 known failures fixed (gen-m9 4: 79d87fc527927626 A156, 89f0aee953a55a45 A152, 9d9ceba257f8001b A372,
  b6a02893a8d6b151 A305; probes 7: fin-argument-modes H72, fin-remaining-later-argument-ranges I65-I73 and K11),
  no regressions; visibly-off in-scope cells 2620 to 2615 with the new case's 19 (the new case adds 52 known
  failures, 35 of 87 to the bit).
- Left, documented in financial.md with the forms tried, for T-3058: G2 (= A7, type 1 from period 1), A62, A352,
  A379, the tiny-rate interest cells of both types (gate-edges A2-A8, E1-E10), and the periods near 1E15.
