---
id: T-3311
title: "Gamma quantiles in the search region of the deep lower tail: 7 visibly-off gen-m9 cells left by T-2561 (CHISQ.INV(1E-200,5), (1E-20,6), GAMMAINV(1E-16,2.5,1))"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The seven deep-tail gamma-quantile cells T-2561 left visibly off are within half a unit of Excel's 15th digit, or
time-boxed with the forms tried.

## Context
T-2561 matched Excel where the quantile is tiny: the leading-factor root `incomplete_gamma::excel_tiny_quantile`.
Away from that, Excel's value is neither the engine's crossing nor the root with Newton steps
(docs/excel-semantics/statistics.md, "Deep lower tail at a shape above 1"). The cells, from gen-m9 (excel-graph):
- 06b4e8f5de409d5e A128 and 623663cacea53bbc A98 `CHISQ.INV(1E-200,5)`: Excel 3.233407780583136E-80, ours 20 ulp
  below. The root is 236 ulp below, and our cdf at the root is 6.24E-14 off p.
- d8266bc15533702a A212 `CHISQ.INV(1E-200,10)`: Excel 5.210342169394733E-40, ours 36 ulp below, the root 111 below.
- a612e89321a6df9c A169 `CHISQ.INV(1E-20,6)`: Excel 7.829736048647269E-7, ours 5 ulp below.
- c2974c061384fdd9 A355 `CHISQ.INV(1E-20,10)`: Excel 0.0005210568413974334, ours 0.0005210568413974328.
- a5828d3925575713 A338 `GAMMAINV(1E-16,2.5,TRUE)`: Excel 6.436215297435573E-7, ours 6.436215297435568E-7.
- ecc87f80077959e0 A301 `CHISQ.INV.RT(1-1E-16,3)`: Excel 5.5854857570344876E-11, ours 11 ulp below.

Leads:
- At `CHISQ.INV(1E-16,3)` (a7bf0d8d81d33ba6 A216) Excel is one Newton step from the root on the leading-factor series
  form (T-3013's step rule), but that rule is 71 ulp off at A169.
- A probe of GAMMA.DIST at Excel's quantiles would show whether Excel lands on a crossing of its own cdf there.

Cells (the seven above, listed so `scripts/exit_owners.py` credits them; gap triage 2026-10-31):
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A128
- gen-m9/623663cacea53bbc.case.json (1): Calc!A98
- gen-m9/d8266bc15533702a.case.json (1): Calc!A212
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A169
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A355
- gen-m9/a5828d3925575713.case.json (1): Calc!A338
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A301

## Acceptance criteria
- [ ] The seven cells reach visible parity, or the Outcome lists the candidates with their bit-exact and visible
      counts (decision 0056).
- [ ] No new known-failure line in any eval/baseline file; no cell T-2561 fixed gets worse.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value;
bit-level misses left then go to a `defer: phase-1b` follow-up.
For these seven cells Excel is the side off the true quantile; ours is the correctly rounded crossing. So
reproduce Excel's landing, never a more accurate kernel.

## Out of scope
- Shape 20 and above (T-3294); the p = 0.999 band (T-3146).
