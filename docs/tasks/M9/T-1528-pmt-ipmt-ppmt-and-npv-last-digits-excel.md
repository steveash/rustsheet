---
id: T-1528
title: "PMT, IPMT, PPMT and NPV last digits: Excel's annuity formulations (time-boxed)"
milestone: M9
lane: functions-financial
status: blocked
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
PMT, IPMT, PPMT and NPV match Excel bit for bit on the T-0942 sweeps, from a fitted formulation of
each (split from T-0901, which keeps the actionable NPER and accurate-PMT changes).

## Context
Kernel map rank 1 (`docs/research/numeric-kernel-map.md`); decision 0056. Sweeps (excel-graph,
2026-10-06): `eval/suites/probes-numeric/cases/m9-fin-pmt` (PMT, IPMT, PPMT, 308 cells each) and
`m9-fin-nper-npv` (NPV, 88 cells). Fit lab: `uv run rustsheet-fitlab annuity-pmt` (and `annuity-ipmt`,
`annuity-ppmt`, `annuity-npv`), `tools/src/rustsheet_tools/fitlab/kernels/annuity.py`. Findings:
docs/excel-semantics/financial.md "Annuity evaluation order"; numbers.md "x87 precision is per operator,
not per kernel (T-1524)". Excel's PMT lies up to 3.5 ulps from the exact value (the rate taken as a
double), so its formulation loses accuracy somewhere a half-ulp-accurate model does not; A7
`PMT(0.2,2,10000)` = -6545.454545454547 is 1.45 ulps from exact. Engine code:
`functions/financial/mod.rs` `compute_payment`, `compute_ipmt`, `compute_ppmt`;
`financial_util.rs` `compute_npv`.

**From T-0901 (2026-10-16):** eight PV/NPER cells of `ironcalc-calc/FINANCIAL/PMT.xlsx` (I16, I32,
I34, J12, J32, J35, K12, M12) are baselined failures because their PMT inputs fail (decision 0064):
they must pass once PMT matches. The accurate PMT (`-(pv*P+fv)/((P-1)/r*k)`, P-1 by expm1) fixes 30
Excel-cached cells and regresses 22, and IPMT's recurrence on the fitted FV regresses 4, so neither
shipped. IPMT still uses `FutureValueForm::IpmtRecurrence`.

## Acceptance criteria
- [ ] A PMT formulation matches >= 99.9% of m9-fin-pmt PMT cells bit for bit (today's best: 132/308).
- [ ] IPMT and PPMT built on it match >= 99.9% of their m9-fin-pmt cells (best: 219/308, 94/308).
- [ ] An NPV formulation matches >= 99.9% of m9-fin-nper-npv NPV cells (best: 27/88).
- [ ] Each is recorded in financial.md with the sweep cells and implemented in the engine with no new
      baseline lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
NPER and the accurate double PMT (`expm1`/`log1p`) engine change (T-0901); RATE/IRR solvers (T-0916).

## Blocked because: time-boxed (decision 0056)
T-1524 made two attempts (2026-10-14) and no candidate rose above the correctly rounded value:
- PMT (308): every 64-bit-intermediate model (P exact, by 64-bit square-and-multiply, or by
  fyl2x/f2xm1/fscale; 4 formula shapes) 132 = correctly rounded 132; doubles with P from
  square-and-multiply, libm pow or exp(n log1p r), 4 shapes, plain or twice-rounded operations: at
  most 91; doubles with P-1 = expm1(n log1p r), 3 shapes: at most 129; P rounded to a double before a
  64-bit formula: 96. The starting reference script's 10 operation orders at 64 bits reached at
  most 133.
- IPMT (308): PMT then the FV recurrence (ipmt = FV(r, per-1, pmt, pv)*r; type 1: per 1 is 0, else
  (FV(r, per-2, pmt, pv, 1) - pmt)*r), per power source and rounding model: correctly rounded 219,
  64-bit 218, doubles 195. PPMT = PMT - IPMT: 94, 85, 43.
- NPV (88): v/P with each power source, Horner from the last value, a running factor d *= 1+r; plus
  v*d with d /= 1+r, v*pow(1+r,-i), v/exp(i log1p r), v/exp(i log(1+r)), reverse sums and fsum: at most
  27 (correctly rounded 18).
Attempt 3 (2026-10-16, owner-directed, after T-0901 shipped PV/FV/NPER; scratch fits, same sweeps):
- PMT (308): new power sources were Excel's general `^` path, P = exp(n·ln(1+r)) (the product an Excel
  operator, and plain), and the discount form 1/P (Excel's `1/`). They were crossed with the earlier 4
  shapes plus 6 new ones (`-(pv+fv/P)*r/(k*(1-1/P))`, `-(pv+fv/P)/(k*((1-1/P)/r))`, PV's own factor
  `-(pv+fv/P)/((k*((P-1)/r))/P)`, and reorderings of `(pv*r*P+fv*r)/((1-P)*k)`), each in doubles and
  with Excel's twice-rounded operators. Best: 87, below the correctly rounded 132. The exp(n·ln) source
  is not in the top 15.
- NPV (88): each term multiplied by Excel's `(1+r)^-i` (the twice-rounded 1/P of square-and-multiply),
  with plain or Excel `*`, and terms by exp(i·ln(1+r)) with Excel's `*` and `/`, each summed in doubles,
  with Excel's `+` or at 64 bits. Best: 25 (`v/exp(i ln)` summed at 64 bits), below 27.
- Also found by T-0901 (financial.md): the accurate `expm1` PMT fixes 30 Excel-cached cells and breaks
  22, and IPMT's recurrence on the fitted FV breaks 4, so Excel's PMT is not the accurate value.
Still blocked: none of the three attempts raised the bit-exact count.
Reopens on a new hypothesis recorded here, for example: Excel's documented PMT formula evaluated in a
specific compiled order, a different power routine for PMT than for PV/FV, NPV accumulated in a
different order or with a different discount factor, or new oracle evidence (a probe with r chosen
so the candidate orders differ).
