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
defer: phase-1b
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
shipped. Since T-2130 PMT, IPMT and PPMT are the exact payment (double-double, 15 digits on the
sweep; financial.md "PMT, IPMT and PPMT are the exact payment"), and T-2243 lists the 13 cells it
moved by 1 to 4 ulp.

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): NPV values of the financial sweep off in the 15th digit (`NPV(0.05,..,{-100,50,60,0,0})` Excel 0.7710730536356959, ours ...682), and the last-digit NPV, PPMT, IPMT and ISPMT cells.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 visibly off.
Examples:
- gen-m9/1cd3096e3e87c965 Calc!A345 `=NPV(0.05,Data!A7:A12,Data!A4:A11,{-100,50,60,0,0})`: Excel 0.7710730536356959, ours 0.771073053635682
- gen-m9/3aa168259335be54 Calc!A214 `=NPV((-0.05),{-100,300,-300,101},Data!E9:E11)`: Excel 2.671477233044049, ours 2.671477233044019
- gen-m9/abb7d565af89b849 Calc!A387 `=NPV(IRR(Data!B9:E10,(-0.5)),Data!C2:C8,Data!E10:E11,Data!E8:E9)`: Excel 0.00020006599755898017, ours 0.00020006599755891932
Cells:
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A345
- gen-m9/3aa168259335be54.case.json (1): Calc!A214
- gen-m9/abb7d565af89b849.case.json (1): Calc!A387

Last-digit cells of the same sweep (not visibly off; re-check them after the fix):
Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 65 known-failure lines in eval/baseline/gen-m9.txt, 65 last-digit (not visibly off).
Examples:
- gen-m9/1cd3096e3e87c965 Calc!A82 `=PPMT(0.1,7,95,234838)`: Excel -4.862617984267317, ours -4.862617984267313
- gen-m9/b6a02893a8d6b151 Calc!A133 `=IPMT(42,42,42,41647)`: Excel -1708495.5348837208, ours -1708495.534883721
- gen-m9/ba4ddb817a8e700e Calc!A42 `=NPV(0.5,Data!C9:C11)`: Excel 44.44444444444445, ours 44.44444444444444
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A59
- gen-m9/1b60bf99d71424e7.case.json (3): Calc!A108, A119, A168
- gen-m9/1cd3096e3e87c965.case.json (6): Calc!A51, A82, A100, A247, A278, A296
- gen-m9/1dce4774f5c0e736.case.json (2): Calc!A245, A343
- gen-m9/30e20ba3302fea8b.case.json (3): Calc!A146, A242, A342
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A203
- gen-m9/3aa168259335be54.case.json (1): Calc!A71
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A305
- gen-m9/5422bdbf173fc233.case.json (3): Calc!A138, A152, A334
- gen-m9/5c6a23c4f6597f1c.case.json (4): Calc!A106, A204, A225, A351
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A128
- gen-m9/6376e27be7a05871.case.json (3): Calc!A80, A277, A325
- gen-m9/67a4c03afd63f988.case.json (2): Calc!A128, A177
- gen-m9/762cd0e4ec9a7aa9.case.json (4): Calc!A96, A103, A163, A341
- gen-m9/79d87fc527927626.case.json (3): Calc!A50, A99, A150
- gen-m9/81bcdc0bec0c8e6b.case.json (2): Calc!A246, A344
- gen-m9/89f0aee953a55a45.case.json (2): Calc!A257, A278
- gen-m9/9a1fa5366bc50918.case.json (3): Calc!A27, A89, A383
- gen-m9/9d9ceba257f8001b.case.json (4): Calc!A32, A36, A85, A290
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A11
- gen-m9/abb7d565af89b849.case.json (2): Calc!A261, A289
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A133
- gen-m9/b72d952c5f52cb4d.case.json (3): Calc!A13, A21, A258
- gen-m9/b73c6cc96e093f03.case.json (3): Calc!A143, A339, A366
- gen-m9/ba4ddb817a8e700e.case.json (3): Calc!A20, A42, A216
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A392
- gen-m9/c0681a8b150ce301.case.json (2): Calc!A57, A351

Handed over by T-2734: `NPV(-0.9999999999999999,<19 ones>)` is 1.0153083415854513E+285, ours ...512E+285, and with 20 ones
1.016120504184532E+300, ours ...319E+300 (probes/t2493-npv-rate-edges A60, A61; the rate is cut to 15 digits, -0.999999999999999).

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 3 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The 65 T-2257 last-digit lines in Context are re-checked after the fix; the Outcome says how many pass.
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
- IPMT and PPMT were not re-fitted: they derive from PMT, so no PMT form means no new IPMT/PPMT form.
Still blocked: none of the three attempts raised the bit-exact count.
Reopens on a new hypothesis recorded here, for example: Excel's documented PMT formula evaluated in a
specific compiled order, a different power routine for PMT than for PV/FV, NPV accumulated in a
different order or with a different discount factor, or new oracle evidence (a probe with r chosen
so the candidate orders differ).

## New hypothesis (T-1405, 2026-10-19)
`PPMT(1,1,60,1)` is -8.67361737988405E-19 = -2^-60 in Excel (probes/pmt-overflow B8, excel-graph, unscored there),
where `pmt - ipmt` gives 0 (PMT -1, IPMT -1). That is exactly `-pv*rate/(P-1)` with P = 2^60, so Excel's first-period
PPMT is not `PMT - IPMT` but a closed form of its own; PPMT's last digits may follow from that form.

## New evidence (gap triage 2026-10-25)
NPV's visibly-off cells moved to T-2672 (visible-parity stage), which waits on T-2671's decomposition probe (one-term, zero-padded and two-term NPVs at rates where the candidate orders differ). The new evidence is NPV at IRR's root in ironcalc-calc/FINANCIAL/IRR.xlsx: 12 cells whose rate is bit-exact in ours still differ in the residue (financial.md "NPV's residue at IRR's root is not the double sum"). NPV's last-digit tail stays here.
Gap triage 2026-10-25 (second pass): that tail includes ironcalc-calc/FINANCIAL/NPV.xlsx, 15 known-failure lines, all 1-2 ulp
(none visibly off), which no task named: Sheet1!K2, K4, K5, K6, K7, L4, L7, M2, M4, M9, N2, N4, N5, N6, N7, e.g. K2
`NPV(A2,B2,C2,D2,E2:G2)` Excel 237.57926512731055, ours 237.5792651273106 (2 ulp). T-2671's probe should fit them too.

## New hypothesis (gap triage 2026-10-28): PMT through PPMT's discount, split to T-3014
PPMT's discount (T-2242: Excel's atanh-series L, `Q = EXP(-n*L)`, Kahan's expm1 for `1-Q`) in the form
`-(pv+fv*Q)/(k*((1-Q)/rate))` is bit-exact on 175/308 m9-fin-pmt PMT cells even with libm EXP and LN
(docs/research/t3014_pmt_discount_score.py), above this task's best 133. T-3014 (`campaign: visible`) fits and ships it
for PMT, because 113 RATE(n,PMT(..)) cells of probes-numeric/m9-fin-irr-rate are visibly off on PMT's last ulps. If
T-3014 lands, re-score IPMT on the new PMT here; if it reaches its time-box, its scores are recorded here.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).

## PMT fitted by T-3014 (2026-10-29)
PMT's bits are solved: `-(((pv+fv)/(1-Q) - fv) * d)` with PPMT's L and Q, 308/308 on m9-fin-pmt (financial.md "PMT is
a discounted payment"). This task keeps IPMT, PPMT and NPV; T-3171 tries the same `1-Q` switch in PPMT and IPMT.
