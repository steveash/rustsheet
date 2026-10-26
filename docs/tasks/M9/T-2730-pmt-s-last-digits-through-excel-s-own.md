---
id: T-2730
title: "PMT's last digits through Excel's own L and expm1: the T-2242 sweeps (604 PMT and 631 PPMT cells within 15 digits)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
PMT returns Excel's bits on the two T-2242 sweeps by computing the payment from Excel's own `L = ln(1+rate)`
(`growth_log`) and Excel's EXP/expm1, as PPMT already does. PPMT's first period then follows, because Excel's PPMT
of period 1 is its own PMT times `EXP(-n*L)`. If two fit attempts do not raise PMT's bit-exact count on the sweeps,
the task records the candidates and is time-boxed (decision 0056).

## Context
Gap triage 2026-10-25. No open task names these two probes-numeric sweeps. T-2242 (done) added them and fitted PPMT,
but not PMT. T-1528 (blocked, time-boxed) holds the older PMT fit on m9-fin-pmt. It ran before T-2242 found Excel's
L, and none of its candidates used that L. This task is the new hypothesis. When it ends, record the result in
T-1528's "Blocked because" section.

Failing cells (excel-graph, 2026-10-24; probes-numeric is `gate = false`, so they are not baseline lines). All are
`numeric_ulp`, within 15 digits:
- probes-numeric/ppmt-first-period-sweep (`PMT(r,n,10000)` and `PPMT(r,1,n,10000)`, n = 1..400, r = 0.1 and 0.05):
  PMT 352 of 800 fail, PPMT 339 of 800.
- probes-numeric/ppmt-log-rate-sweep (126 rates from 0.001 to 2, six terms each with `n*L` near 30): PMT 252 of 756
  fail, PPMT 292 of 756.
- probes-numeric/m9-fin-pmt (T-1528's sweep, re-score only): PMT 176 of 308, PPMT 114, IPMT 89.

Examples (Excel vs ours):
- ppmt-first-period-sweep `PMT(0.1,1,10000)`: -11000.000000000002 vs -11000 (exact). A quick scratch check
  reproduces Excel's value with L one ulp below `log1p(0.1)` (financial.md: Excel's L is -1 ulp at 0.1) and
  `-pv*rate/(1-EXP(-n*L))`. With libm `exp`/`expm1` and plain doubles, that form is bit-exact on only 222/400 at 0.1
  and 195/400 at 0.05; the engine gets 448/800. So the operation order, Excel's EXP (x87_exp.rs), Kahan's
  expm1 (`excel_expm1`) and when the x87 register rounds still have to be fitted.
- `PMT(0.1,102,10000)`: -1000.0599752628509 vs -1000.0599752628508. `PMT(0.05,98,10000)` and
  `PMT(0.003,10015,10000)` also miss by 1 ulp.
- ppmt-first-period-sweep `PPMT(0.1,1,102,10000)`: -0.059975262850716686 vs -0.05997526285071669.
  financial.md "PPMT discounts with an atanh-series log": with Excel's own PMT cell as input, `PMT*EXP(-n*L)` is
  bit-exact on 282/400 (0.1) and 271/400 (0.05). The engine's discounted form `-(pv+fv)*rate*Q(-n)/(1-Q(-n))` passes
  461/800.

Semantics: financial.md "PPMT discounts with an atanh-series log" (L, the type-1 division, where the exact form still
applies). "PMT, IPMT and PPMT are the exact payment" records that Excel is up to 3.5 ulp off the exact payment, and
`PMT(0.2,2,10000)` is 1.45 ulp from exact. numbers.md has "Excel's expm1 is Kahan's formula" and "LN is FLDLN2 then
FYL2X".
Code: `fn_pmt`, `accurate_payment` (exact double-double), `growth_log`, `discounted_principal`
(crates/rustsheet-engine/src/functions/financial/annuity.rs); `excel_expm1` and Excel's EXP (x87_exp.rs);
`Register` (x87_register.rs). Lab: `uv run rustsheet-fitlab annuity-pmt`
(tools/src/rustsheet_tools/fitlab/kernels/annuity.py). Add candidates built on Excel's L there.
The sweeps hold enough points for fv 0, type 0. For fv and type 1, the lab re-scores m9-fin-pmt.
No oracle sweep is needed before the first attempt.

Also gen-m9 (gap triage 2026-10-26, `make eval` on 1c8b3d34, excel-graph): 7 PPMT and 1 CUMPRINC cells, each 1 ulp
off and the outer call, that no task listed; most at a large rate where the result is tiny (gen-m9/1dce4774f5c0e736
Calc!A112 `=PPMT(10,7,158,147532,763)`: Excel -7.575972059482481E-153, ours -7.57597205948248E-153), one plain
(gen-m9/3c06f9298f65a3ba Calc!A256 `=PPMT(0.05,3,114,302806)`: Excel -64.35825018498787, ours -64.35825018498785).
Re-score them with the model this task fits.
Cells:
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A112
- gen-m9/3c06f9298f65a3ba.case.json (2): Calc!A109, A256
- gen-m9/79d87fc527927626.case.json (1): Calc!A291
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A374
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A305
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A69
- gen-m9/d0854366830dcdee.case.json (1): Calc!A98

## Acceptance criteria
- [ ] Lab: PMT candidates built on Excel's L (`-pv*rate/(1-Q(-n))`, `-pv*rate*Q(n)/(Q(n)-1)`, expm1 via Excel's EXP or
      Kahan's formula, each with double and 64-bit intermediates) are scored on the two sweeps and m9-fin-pmt, including
      the rejected ones. The bit-exact and within-4-ulp counts go into financial.md under a new "PMT through Excel's
      own L" entry, which cites `PMT(0.1,1,10000)`.
- [ ] Either a model that raises PMT's bit-exact count on the two sweeps ships, and PPMT period 1 is re-derived from
      it where that raises PPMT's count, or the task is set `blocked`, "Blocked because: time-boxed (decision 0056)",
      with the candidates listed.
- [ ] If a model ships: no line is added to any `eval/baseline/` file, and T-1528's and T-2243's PMT, IPMT and PPMT
      lines in eval/baseline/ironcalc-calc.txt and gen-m9.txt only shrink (PMT.xlsx, IPMT_PPMT.xlsx,
      CUMPRINC_CUMIPMT.xlsx and the decision 0064 PV/NPER cells of PMT.xlsx re-checked). No cell of m9-fin-pmt or the
      two sweeps becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- NPV, IRR and XNPV (T-1528's NPV part, T-2493, T-2494, T-2672).
- PPMT at rates at or below -0.375 and at fractional periods (T-2588). IPMT's long-term and fractional cells (T-2587).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
