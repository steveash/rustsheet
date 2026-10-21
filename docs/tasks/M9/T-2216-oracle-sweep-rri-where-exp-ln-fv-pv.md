---
id: T-2216
title: "Oracle sweep: RRI where exp(ln(fv/pv)/nper)-1 and (fv/pv)^(1/nper)-1 differ (RRI.xlsx D12)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph oracle sweep gives T-2217 enough RRI points to tell the two formulations apart, and settles
whether Excel for the web agrees with the Mac value of `RRI(0.5,3,4)`.

## Context
Gap triage 2026-10-21 (`make eval` on main d48456cd). The one RRI known failure is
`ironcalc-calc/FINANCIAL/RRI.xlsx` Sheet1!D12 `=_xlfn.RRI(A12,B12,C12)` with A12 = 0.5, B12 = 3, C12 = 4. The
cached value (Microsoft Macintosh Excel 16.x) is 0.7777777777777775; ours is 0.7777777777777777. In the same
workbook, E12 `=POWER(C12/B12,1/A12)-1` is 0.7777777777777777. So RRI does not use POWER here (POWER and `^` are the
same in the engine; Excel has not been shown to agree), which contradicts financial.md "DURATION and RRI operation order".
`EXP(LN(fv/pv)/nper)-1`, computed with the engine's `excel_exp`/`excel_ln`, matches all
25 numeric Excel RRI cells: the 12 excel-graph cells of probes/fin-mduration-rri-fit, the 8 numeric cells of probes/fin-rri
(A1, A9-A14, A16), and the Mac cells RRI.xlsx D3-D6 and D12; the special-case cells (fin-rri A5/A6, RRI.xlsx D2/D8)
  and error cells (RRI.xlsx D7, D9-D11) are not counted.
The current `(fv/pv)^(1/nper)-1` (arithmetic.rs `power`) misses only D12. D12 is the only cell where the engine's
forms differ at all. Plain libm `exp(log(r)/n)-1` also fits all 25 (only libm `pow` misses, at A37 and A57), so the
evidence does not yet tell `excel_exp`/`excel_ln` from libm either. `/nper` and `*(1/nper)` are not told apart at all. The note is in docs/excel-semantics/financial.md "DURATION and RRI operation order".

Sweep, as stored inputs (probes-numeric case, `excel-probe` skill). Pick each point off-line so that the engine's
`EXP(LN(r)/n)-1`, `EXP(LN(r)*(1/n))-1` and `(r)^(1/n)-1` do not all agree, and so that libm `exp(log(r)/n)-1`
differs from the engine's EXP/LN form on some of them. Screen a few thousand candidates in a scratch case with
`rustsheet-eval --root <scratch> show` (hand-picked points have agreed so far). The error edges are the exception:
they are included as they are.
- `RRI(0.5,3,4)` (D12 on the web), plus `POWER(4/3,2)-1` and `EXP(LN(4/3)/0.5)-1` beside it.
- About 150 RRI calls: fractional nper (0.25 .. 0.95), small integer nper (2 .. 12) with ratios p/q of small
  integers, large nper (100 .. 1E+6), ratios below 1, and the error edges (nper 0, pv or fv 0, negative ratio). For each, add the worksheet forms
  `(fv/pv)^(1/nper)-1`, `POWER(fv/pv,1/nper)-1`, `EXP(LN(fv/pv)/nper)-1` and `EXP(LN(fv/pv)*(1/nper))-1` in the next columns.

## Acceptance criteria
- [ ] A probes-numeric case with an excel-graph oracle sidecar covers the list above, and financial.md's RRI
      paragraph links it with the counts of points each candidate matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `rri_of` (T-2217).
