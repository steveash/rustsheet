---
id: T-1895
title: "AMORDEGRC below period 1 and at a cost of 1E308 match Excel (probes fin-amor-period-bound B17, -b B2)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1841]
---

## Goal
AMORDEGRC returns Excel's value for a period between 0 and 1 and for a cost whose charges round past 1E293.

## Context
Found by T-1841's probes; both are base values (the period walk is not involved), so T-1841 left them as known
failures:
- probes/fin-amor-period-bound B17 `AMORDEGRC(2400,39679,39813,300,0.5,0.15,1)` is 485 in Excel, ours 0. 485 is the
  value of period 2 (`AMORDEGRC(..,2,..)` and `..,2.5,..` are 485, B18; period 1 is 776, probes/fin-argument-modes
  H18; period 0 is the first period's charge). Main's walk runs `floor(period)` periods, none for 0.5, and
  returns 0. AMORLINC at 0.5 is a full period's charge in Excel and ours (A8, A9: 360, 2.4).
- probes/fin-amor-period-bound-b B2 `AMORDEGRC(Z7,39679,39813,300,1E5,0.000001,1)` with Z7 = 1E308 is
  1.94700443465919E302 in Excel (15 significant digits), ours 0: `amor_round` pre-rounds `x*1E13`, which overflows
  to infinity past about 1.8E295, so the first period's charge is infinite and every later one 0
  (functions/financial/amortization.rs). Excel's value suggests its rounding keeps 15 significant digits there.
Probe more points first (periods 0.1, 0.9, 1.5, 3.5 at rate 0.15; costs 1E290 to 1E308 at periods 0, 1 and 2).

## Acceptance criteria
- [ ] fin-amor-period-bound B17 and fin-amor-period-bound-b B2 pass (baseline lines removed), with no new failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The AMORDEGRC rate thresholds (T-1764).
