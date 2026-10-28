---
id: T-2951
title: "PMT at rates -0.9 and 0.2 to Excel's bits so PMT.xlsx's 10 visibly-off chained NPER, PV and FV cells pass (parked in deferred T-2730)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
The 10 visibly-off NPER, PV and FV cells of `ironcalc-calc/FINANCIAL/PMT.xlsx` agree with Excel to 15 significant
digits. Each one takes its pmt (or nper) argument from a PMT (or NPER) cell that is 1 or 2 ulp off Excel's cached
value, and the rate of -0.9 or the long term at 0.2 magnifies that ulp into the 12th to 14th digit. So the target is
PMT's own last bit on 8 cells, plus NPER's on one, where Excel is the inaccurate one.

## Context
Gap triage 2026-10-27 (`make eval` on origin/main cc64bd91, poi and calipers fetched). These 10 cells are visibly
off, yet their only open owners are T-2243 and T-2730, both `defer: phase-1b`, and T-1528 (blocked, time-boxed).
Decision 0114 does not let a visibly-off cell wait for Phase 1b. T-2733 did the same for PRICE's cells parked in
deferred T-1374.

Owned cells (Excel's cached value vs ours; Sheet1 of ironcalc-calc/FINANCIAL/PMT.xlsx, Mac desktop cache):

| cell | formula | Excel | ours | input it reads |
|---|---|---|---|---|
| H10 | `NPER(A10,F10,C10,D10,E10)` | 3.0000000000001865 | 2.999999999999991 | F10 |
| J10 | `PV(A10,B10,F10,D10,E10)` | 119.99999999977724 | 120.00000000000462 | F10 |
| H11 | `NPER(A11,F11,C11,D11,E11)` | 3.000000000000222 | 2.9999999999999485 | F11 |
| J11 | `PV(A11,B11,F11,D11,E11)` | 99.99999999973858 | 99.99999999996595 | F11 |
| J18 | `PV(A18,B18,F18,D18,E18)` | 99.99999999999832 | 100.00000000000003 | F18 |
| J23 | `PV(A23,B23,F23,D23,E23)` | 99.99999999999832 | 100.00000000000003 | F23 |
| H27 | `NPER(A27,F27,C27,D27,E27)` | 30.000000000000217 | 29.999999999999915 | F27 |
| I27 | `NPER(A27,G27,C27)` | 30.000000000000064 | 29.999999999999915 | G27 |
| L32 | `FV(A32,I32,F32,C32,E32)` | 10.000000000000114 | 10.000000000000057 | F32 |
| L33 | `FV(A33,I33,F33,C33,E33)` | 10.000000000000114 | 10.0 | F33, I33 |

The inputs, all known-failure lines in eval/baseline/ironcalc-calc.txt (rows: rate, nper, pv, fv, type):

| cell | arguments | Excel | ours | Excel vs exact | ours vs exact |
|---|---|---|---|---|---|
| F10 | -0.9, 3, 120, 300, 1 | -2703.783783783783 | -2703.7837837837847 | +3.5 ulp | -0.5 |
| F11 | -0.9, 3, 100, 300, TRUE | -2703.603603603602 | -2703.6036036036044 | +4.8 ulp | -0.2 |
| F18, F23 | -0.9, 1, 100, 300, 1 | -3099.999999999999 | -3100.0000000000005 | +3.5 ulp | +0.5 |
| F27 | 0.2, 30, -300, 0, 1 | 50.2115271162167 | 50.21152711621671 | -0.8 ulp | +0.2 |
| G27 | 0.2, 30, -300 | 60.25383253946004 | 60.25383253946005 | -0.8 ulp | +0.2 |
| F32 | 0.2, 3, -200, 10, 5 | 76.83150183150182 | 76.83150183150184 | -0.5 ulp | +0.5 |
| F33 | 0.2, 4, -200, 10, -1 | 62.82911077993046 | 62.82911077993045 | +1.0 ulp | 0.0 |
| G33 | 0.2, 4, -200 | 77.25782414307007 | 77.25782414307005 | +1.5 ulp | +0.5 |
| I33 | `NPER(0.2,G33,-200)` | 3.9999999999999996 | 4.000000000000001 | | |

"Exact" is the payment of the stored double arguments at 50 digits (mpmath, triage scratch). The engine's PMT is
correctly rounded on all eight; Excel is 0.5 to 4.8 ulp off it. So this is Excel's algorithm, not accuracy.

The chained functions are already right. Fed Excel's cached PMT and NPER doubles as cell values, the engine's PV,
NPER and FV give Excel's H10, J10, H11, J11, J18, H27, I27, L32 and L33 bit for bit (triage check through a scratch
spec case). Decision 0064 found the same for the eight PV and NPER cells of rows 12, 16, 32, 34 and 35. A typed
literal will not do for that check: a constant typed with 16 digits is cut to 15, so the input must be a cell value.
The textbook form `-(pv*(1+r)^n+fv)*r/((1+r*t)*((1+r)^n-1))` in plain doubles gives Excel's F33 only. At -0.9,
`1+rate` is 0.09999999999999998, so the order of the `(1+r)^n - 1`, `1 + r*t` and `fv` terms decides the last bits.

Prior work: T-1528 (blocked, time-boxed under decision 0056) fitted PMT on probes-numeric/m9-fin-pmt, which has
rates from -0.02 to 0.3 with fv and type variants (154 cells with an fv) but no rate below -0.02. T-2730 (deferred)
holds the hypothesis that PMT goes through Excel's own L = ln(1+rate) (`growth_log`) and EXP/expm1, as PPMT does
(financial.md "PPMT discounts with an atanh-series log"). That was fitted on fv 0, type 0 sweeps. This task tries
that hypothesis and the operation-order candidates on these fv and type-1 cells. Record any model that also moves
T-2730's sweeps in T-2730.
Semantics: financial.md "Annuity evaluation order and `(1+r)^n`" and "PMT, IPMT and PPMT are the exact payment"
(Excel is up to 3.5 ulp off the exact payment; F11 here is 4.8).
Code: `fn_pmt`, `compute_payment`, `payment_pair`, `accurate_payment`, `growth_log`
(crates/rustsheet-engine/src/functions/financial/annuity.rs). For L33's I33, `fn_nper` in the same module. Lab:
`uv run rustsheet-fitlab annuity-pmt` (tools/src/rustsheet_tools/fitlab/kernels/annuity.py).

Evidence limits: the -0.9 rows have three distinct Excel PMT values (F10, F11, F18 = F23), plus G10's 1-ulp miss. The
0.2 rows sit in m9-fin-pmt's grid. The first attempt needs no oracle sweep. If no candidate fits both the 0.2 sweep
cells and the three -0.9 values, the next step is a research sweep of PMT at rates -0.95 to -0.5 with fv and type 1
(T-2588's PPMT probe covers PPMT only).

## Acceptance criteria
- [ ] The lab scores named candidates on the 8 PMT cells above, PMT.xlsx's other PMT cells and m9-fin-pmt, and reports
      bit-exact counts for each, rejected candidates included: the textbook order, the fv-first order, the
      `fv*r/((1+r)^n-1)` split, the Excel-L/EXP form of T-2730, and `1+r*t` applied before and after the division.
      financial.md "Annuity evaluation order" records the counts.
- [ ] Either: the chosen model makes F10, F11, F18, F23, F27, G27, F32, F33 and G33 bit-exact, and with I33 fixed
      too the 10 owned cells pass. Their lines leave eval/baseline/ironcalc-calc.txt. PMT's bit-exact count on
      m9-fin-pmt, ppmt-first-period-sweep and ppmt-log-rate-sweep does not drop, and no baseline gains a line.
      Or: the task is set `blocked` with "Blocked because: time-boxed (decision 0056)", and the candidates and the
      sweep it needs are filed as a research task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PMT's last bits on the T-2242 sweeps at fv 0, type 0 (T-2730) and IPMT/PPMT (T-2243, T-2587, T-2588).
- PV, NPER and FV formulations: they already reproduce Excel on Excel's inputs (decision 0064, this triage).

## Visible-parity stage
Decision 0083/0114, stage `visible`: Excel is the inaccurate one here (the engine's PMT is correctly rounded). The task
is done when the 10 owned cells are within half a unit of the 15th significant digit of Excel's value, which needs the
PMT inputs bit-exact. PMT cells left 1 ulp off after that move to T-2730 (deferred), named in the Outcome.
