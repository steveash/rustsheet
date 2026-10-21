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
workbook, E12 `=POWER(C12/B12,1/A12)-1` is 0.7777777777777777. So RRI does not use the worksheet `^` here, which
contradicts financial.md "DURATION and RRI operation order".
`EXP(LN(fv/pv)/nper)-1`, computed with the engine's `excel_exp`/`excel_ln`, matches all 15 Excel points we have:
the 12 excel-graph RRI cells of probes/fin-mduration-rri-fit, the computed cells of probes/fin-rri, and D12.
The current `(fv/pv)^(1/nper)-1` (arithmetic.rs `power`) matches 14 of them. Only 3 points tell the two apart:
D12, fin-mduration-rri-fit A37 `RRI(10,1,200)` and A57 `RRI(0.75,1,2)`. `/nper` and `*(1/nper)` are not
told apart at all. The note is in docs/excel-semantics/financial.md "DURATION and RRI operation order".

Sweep, as stored inputs (probes-numeric case, `excel-probe` skill). Pick each point off-line so that the engine's
`EXP(LN(r)/n)-1`, `EXP(LN(r)*(1/n))-1` and `(r)^(1/n)-1` do not all agree (evaluate the candidates in a scratch
case with `rustsheet-eval --root <scratch> show`):
- `RRI(0.5,3,4)` (D12 on the web), plus `POWER(4/3,2)-1` and `EXP(LN(4/3)/0.5)-1` beside it.
- About 150 RRI calls: fractional nper (0.25 .. 0.95), small integer nper (2 .. 12) with ratios p/q of small
  integers, large nper (100 .. 1E+6), and ratios below 1. For each, add the worksheet forms
  `(fv/pv)^(1/nper)-1`, `EXP(LN(fv/pv)/nper)-1` and `EXP(LN(fv/pv)*(1/nper))-1` in the next columns.

## Acceptance criteria
- [ ] A probes-numeric case with an excel-graph oracle sidecar covers the list above, and financial.md's RRI
      paragraph links it with the counts of points each candidate matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `rri_of` (T-2217).
