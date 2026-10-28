---
id: T-2217
title: "RRI matches Excel's last ulp at RRI(0.5,3,4) (exp(ln(fv/pv)*(1/nper))-1, not ^)"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: S
tier: opus
campaign: visible
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
- Candidate (triage check; T-2216 sweep settled it for the second form: `/nper` misses 49 of 136 web points, `*(1/nper)` misses none): `EXP(LN(fv/pv)/nper)-1` and `EXP(LN(fv/pv)*(1/nper))-1`, with the
  engine's `excel_exp` (x87_exp.rs) and `excel_ln` (x87_log10.rs), match all
  25 numeric Excel RRI cells: the 12 excel-graph cells of probes/fin-mduration-rri-fit, the 8 numeric cells of probes/fin-rri
  (A1, A9-A14, A16), and the Mac cells RRI.xlsx D3-D6 and D12; the special-case cells (fin-rri A5/A6, RRI.xlsx D2/D8)
  and error cells (RRI.xlsx D7, D9-D11) are not counted. D12 is the only cell where they differ from the current form. Plain libm
  `exp(log(r)/n)-1` also fits all 25 (libm `pow` misses fin-mduration-rri-fit A37 and A57, fin-rri A14 and RRI.xlsx D12), so T-2216 must also tell it apart.
- **Needs an oracle sweep** (T-2216): one discriminating point, and a desktop-only one, is too few to fit
  blind. If the web disagrees with the Mac value of `RRI(0.5,3,4)`, the web value is the target (decision 0039 item 4;
  decision 0074 is the precedent): D12 then stays a known failure, recorded in numbers.md as a web-vs-desktop
  last-digit conflict with an M14 item under T-1075, and the first acceptance criterion's D12 clause does not apply.
- docs/excel-semantics/financial.md "DURATION and RRI operation order" records the D12/E12 quirk.

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): RRI of the financial sweep: `RRI(0.4168699705898027,8370,40876)` Excel 43.89282054161517 (ours ...116) and `RRI(0.2,4299,42330)` 92554.9018325151 (ours ...501) are visibly off; three more are last-digit.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 visibly off.
Examples:
- gen-m9/01523d247594952e Calc!A218 `=RRI(T.DIST.RT(TIME(5,2,16),1E+8),8370,40876)`: Excel 43.89282054161517, ours 43.892820541615116
- gen-m9/5ded7594e597e31e Calc!A51 `=RRI(Data!B9,4299,42330)`: Excel 92554.9018325151, ours 92554.90183251501
Cells:
- gen-m9/01523d247594952e.case.json (1): Calc!A218
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A51

Last-digit cells of the same sweep (not visibly off; re-check them after the fix):
Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 last-digit (not visibly off).
Examples:
- gen-m9/01523d247594952e Calc!A176 `=RRI(0.5,3085,21048)`: Excel 45.54910480733617, ours 45.549104807336164
- gen-m9/23d82b62e9a85bdd Calc!A261 `=RRI(0.5,1675,28713)`: Excel 292.8512342169747, ours 292.8512342169748
- gen-m9/a91bad4e84f49739 Calc!A73 `=RRI(0.5,1293,38207)`: Excel 872.1499369859362, ours 872.1499369859359
Cells:
- gen-m9/01523d247594952e.case.json (1): Calc!A176
- gen-m9/23d82b62e9a85bdd.case.json (1): Calc!A261
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A73

T-2216's sweep cells the engine misses today (gap triage 2026-10-28, `make eval` on dea28aa5; all 1 or 2 ulp, none
visibly off; no task named them). The acceptance criterion "T-2216's sweep pass in full" already covers them, e.g. D1
`RRI(0.5,3,4)` Excel 0.7777777777777775, ours 0.7777777777777777, and D4 Excel 8.000000000000002, ours 8:
- probes-numeric/m9-fin-rri-sweep.case.json (19): Calc!D1, D4, D21, D22, D23, D42, D70, D84, D93, D98, D104, D109,
  D115, D119, D120, D122, D125, D126, D129

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 2 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The 3 T-2257 last-digit lines in Context are re-checked after the fix; the Outcome says how many pass.
- [ ] If the web agrees with the Mac D12 value: the line `F	ironcalc-calc/FINANCIAL/RRI.xlsx	Sheet1!D12` disappears from eval/baseline/ironcalc-calc.txt
      (`make eval-baseline`), with no new F lines in any baseline. In particular, probes/fin-rri, fin-rri-vdb-edges,
      fin-mduration-rri-fit and T-2216's sweep pass in full. If no single formulation fits every point, the PR
      lists the points that still miss and files a follow-up.
- [ ] If the EXP/LN form lands, the edges keep today's values (probes/fin-rri, fin-rri-vdb-edges, RRI.xlsx): nper <= 0
      and a negative ratio are `#NUM!`, fv = 0 is -1, pv = fv = 0 is 0, and the overflow `RRI(1E-5,1,2)` (fin-rri A15)
      is `#NUM!`; no panic.
- [ ] `rri_of` cites the fitted points in its comment; financial.md's RRI paragraph states the rule and is
      marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PDURATION, DURATION and MDURATION (already fitted by T-1975).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.

## Note from T-2216
The sweep (financial.md "DURATION and RRI operation order") fits `EXP(LN(fv/pv)*(1/nper))-1` on 134 of 136 numeric points; `/nper` fits only 87. Implement the `*(1/nper)` form.

Gap triage 2026-10-25 (eval on origin/main becefc6f3): the task was deferred by decision 0114's adoption, but an eval shows a cell it owns visibly off, so, as its Deferred note asked, the `defer` key gave way to `campaign: visible`. Visibly off: gen-m9/01523d247594952e Calc!A218 `=RRI(T.DIST.RT(TIME(5,2,16),1E+8),8370,40876)`: Excel 43.89282054161517, ours 43.892820541615116; gen-m9/5ded7594e597e31e Calc!A51 `=RRI(Data!B9,4299,42330)`: Excel 92554.9018325151, ours 92554.90183251501.

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): RRI 1. This task now owns them:
- probes-numeric/m9-fin-rri-sweep: Calc!D140

## Outcome
`rri_of` computes `EXP(LN(fv/pv)*(1/nper))-1` with the engine's `excel_ln`, `extended_product`, `excel_exp` and
`extended_sum`, and `fv/pv-1` when nper is exactly 1. That nper-1 case is new: `RRI(1,826,40633)` (gen-m9/1cd3096e3e87c965
and b73c6cc96e093f03 Calc!A291) passed before and regressed under the plain EXP/LN form, so this task probed it
(probes-numeric/m9-fin-rri-nper-one, excel-graph, 38 rows: all 32 nper-1 rows are `fv/pv-1`, while nper 1 ± 1 ulp follow
EXP/LN). financial.md records the rule.
- Eval: gen-m9 known failures 3334 -> 3330, ironcalc-calc 556 -> 555 (RRI.xlsx D12; the web agrees with the Mac value),
  no regressions. probes/fin-rri, fin-rri-vdb-edges, fin-mduration-rri-fit, RRI.xlsx and probes-numeric m9-fin-rri-sweep
  (including D140) and m9-fin-rri-nper-one pass in full. The edges keep their values.
- Visible parity: gen-m9/5ded7594e597e31e A51 passes. 01523d247594952e A218 still misses, and RRI is not the cause: its
  nper `T.DIST.RT(TIME(5,2,16),1E+8)` is about 2 ulp above the value Excel's RRI needs. The cell is recorded on T-2126 (the
  Student-t centre above df 1000), which owns that error.
- T-2257 last-digit cells: all 3 pass (01523d247594952e A176, 23d82b62e9a85bdd A261, a91bad4e84f49739 A73).

