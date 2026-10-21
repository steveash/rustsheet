---
id: T-2217
title: "RRI matches Excel's last ulp at RRI(0.5,3,4) (candidate: exp(ln(fv/pv)/nper)-1, not ^)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2216]
---

## Goal
RRI matches Excel to the last bit on every RRI cell we have Excel evidence for, so its one ironcalc-calc known
failure goes away.

## Context
Gap triage 2026-10-21 (`make eval` on main d48456cd; `make eval-show CASE=ironcalc-calc/FINANCIAL/RRI.xlsx`):
- `ironcalc-calc/FINANCIAL/RRI.xlsx` Sheet1!D12 `=_xlfn.RRI(A12,B12,C12)` (0.5, 3, 4): Excel (Mac 16.x cached)
  0.7777777777777775, ours 0.7777777777777777 (NumericUlp, 2 ulp; not visibly off). In the same workbook, E12
  `=POWER(C12/B12,1/A12)-1` is 0.7777777777777777. So Excel's RRI is not POWER here (the engine's `^` and POWER agree).
- `rri_of` (crates/rustsheet-engine/src/functions/financial/annuity.rs) computes
  `extended_sum(power(extended_quotient(fv, pv), extended_quotient(1.0, nper)), -1.0)` (T-1975), and its comment
  says Excel uses its own `^`. That fits the 12 excel-graph RRI cells of probes/fin-mduration-rri-fit A37-A59 and
  probes/fin-rri, but not D12.
- Candidate (triage check in a scratch case): `EXP(LN(fv/pv)/nper)-1` and `EXP(LN(fv/pv)*(1/nper))-1`, with the
  engine's `excel_exp` (x87_exp.rs) and `excel_ln` (x87_log10.rs), match all
  27 Excel RRI cells: the 12 excel-graph cells of probes/fin-mduration-rri-fit, the 10 numeric cells of probes/fin-rri
  (A1, A5, A6, A9-A14, A16), and the Mac cells RRI.xlsx D3-D6 and D12. D12 is the only cell where they differ from the current form; every other one
  agrees under every form. Plain libm `exp(log(r)/n)-1` fits too, so T-2216 must also tell it apart.
- **Needs an oracle sweep** (T-2216): one discriminating point, and a desktop-only one, is too few to fit
  blind. Decision 0030 makes the desktop value the target if the web disagrees on `RRI(0.5,3,4)`.
- docs/excel-semantics/financial.md "DURATION and RRI operation order" records the D12/E12 quirk.

## Acceptance criteria
- [ ] The line `F	ironcalc-calc/FINANCIAL/RRI.xlsx	Sheet1!D12` disappears from eval/baseline/ironcalc-calc.txt
      (`make eval-baseline`), with no new F lines in any baseline. In particular, probes/fin-rri, fin-rri-vdb-edges,
      fin-mduration-rri-fit and T-2216's sweep pass in full. If no single formulation fits every point, the PR
      lists the points that still miss and files a follow-up.
- [ ] If the EXP/LN form lands, `nper` 0, `fv/pv` 0 and a negative ratio still return the same errors as today
      (probes/fin-rri, fin-rri-vdb-edges), with no panic.
- [ ] `rri_of` cites the fitted points in its comment; financial.md's RRI paragraph states the rule and is
      marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PDURATION, DURATION and MDURATION (already fitted by T-1975).
