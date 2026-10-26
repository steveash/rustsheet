---
id: T-2671
title: "Probe NPV's discount and summation order: one-term, zero-padded and two-term NPVs where the candidate orders differ"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel-graph values that take NPV apart one rounding at a time: the discount factor of each period on its own, then
how two and three discounted terms are added. With them, T-2672 can fit NPV's formulation instead of guessing it
from whole sums, which T-1528's three attempts could not do.

## Context
- Gap triage 2026-10-25. 37 NPV cells are visibly off, and T-1528, which owns NPV's last digits, is blocked
  (time-boxed, decision 0056) and deferred. Its reopen condition asks for "a probe with r chosen so the candidate
  orders differ". This task is that probe.
- The strongest evidence so far is NPV at IRR's root, where every rounding shows in the residue
  (docs/excel-semantics/financial.md "NPV's residue at IRR's root is not the double sum"). In
  ironcalc-calc/FINANCIAL/IRR.xlsx, General!M2 `NPV(L2,A2:F2)` with L2 = 0.07930826116039724 (bit-exact in ours)
  over {-20,5,5,5,5,5} is 6.4212018552224666E-12 in Excel. Ours is 6.418865439172805E-12, and the exact value at
  that rate is 6.421923509333131E-12. 12 such cells share a rate with ours and still differ (list in T-2672).
- T-1528's rejected candidates (in doubles, at 64 bits, plain or twice-rounded): v/P from each power source, Horner
  from the last value, a running factor d *= 1+r, v*d with d /= 1+r, v*pow(1+r,-i), v/exp(i*log1p r),
  v/exp(i*ln(1+r)), reverse sums, fsum, and Excel's `^` with plain or Excel `*`. The best was 27/88 on
  probes-numeric/m9-fin-nper-npv (the correctly rounded value gets 18). A triage scan on 2026-10-25 also matched none
  of the residue cells with a pow, Horner, running-factor or exp/log1p sum in doubles (Horner matched M8 alone).
- Lab: tools/src/rustsheet_tools/fitlab/kernels/annuity.py (`uv run rustsheet-fitlab annuity-npv`). Excel cuts formula
  literals to 15 digits, so a probe rate needs at most 15 significant digits, or it must come from a cell or a
  formula such as `1+2^-40`.

## Acceptance criteria
- [ ] A new probes case `probes/npv-term-order` (excel-graph fill, canary-verified sidecar) holds at least 300 cells:
  - one-term NPVs `NPV(r,v)`, which isolate the first division;
  - zero-padded `NPV(r,0,..,0,v)` for periods 2 to 12, which isolate each period's discount factor and show whether
    it is (1+r)^i through Excel's `^`, a running product, or something else;
  - two- and three-term NPVs with values of opposite sign and close magnitude, which isolate the summation order and
    precision;
  - each at rates where the T-1528 candidates give different doubles. The case or its README records the candidate
    values each rate was chosen to separate.
  Rates include IRR.xlsx's L2, L3, L8 and Three Parameters D2, and 0.05, 0.07, 0.1, 0.125 and 0.2 from
  m9-fin-nper-npv.
- [ ] financial.md's entry "NPV's residue at IRR's root is not the double sum" is updated with what the probe shows:
  the per-period discount rule, if one fits, and the summation order. Each candidate's hit count goes in, the rejected
  ones too.
- [ ] Newly failing cells are known failures in eval/baseline/probes.txt owned by T-2672. No other baseline gains a
  line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's NPV (T-2672).
- NPV's error order and its #DIV/0! at rate -1 (T-2493, T-2618).
- PMT, IPMT and PPMT (T-1528, T-2243).
