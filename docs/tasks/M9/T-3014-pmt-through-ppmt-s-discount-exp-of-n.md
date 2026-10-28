---
id: T-3014
title: "PMT through PPMT's discount (EXP of n·L with the atanh-series L, Kahan's expm1): the 113 RATE(n,PMT(..)) cells of m9-fin-irr-rate that ride on PMT's last bits"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
PMT returns Excel's bits on the m9-fin-pmt sweep wherever Excel's own discount (the one PPMT uses) explains them,
so the 113 `RATE(n,PMT(..),..)` cells of probes-numeric/m9-fin-irr-rate, which are visibly off only because the
engine's PMT differs from Excel's by an ulp or two, reach 15-digit parity.

## Context
Gap triage 2026-10-28 (eval on origin/main abefb50).
- **The cells.** 113 RATE cells of probes-numeric/m9-fin-irr-rate (excel-graph) are visibly off and attributed to no
  single function (the formula calls RATE and PMT). The only open task naming the case, T-2131, is IRR's and lists
  RATE as out of scope, so nobody owned them. Examples (Excel, ours):
  - A218 `RATE(1,PMT(0.2,1,10000,-500,1),10000,-500,1)`: 0.19999999999999138, 0.20000000000000448
  - A386 `RATE(1,PMT(0.01,1,10000,-500,1),10000,-500,1)`: 0.010000000000102807, 0.009999999999867651
  - A497 `RATE(1,PMT(-0.02,1,10000),10000)`: -0.020000000000003015, -0.01999999999999864
  Owned cells (all `Calc!`): A218, A246, A247, A249, A250, A256, A273, A274, A276, A278, A280, A282, A283, A284, A286,
  A287, A288, A303, A310, A312, A314, A315, A316, A318, A334, A336, A338, A385, A386, A387, A388, A389, A390, A393,
  A394, A396, A397, A399, A400, A401, A402, A404, A407, A408, A413, A414, A415, A416, A417, A418, A419, A420, A421,
  A422, A423, A424, A425, A426, A427, A428, A429, A430, A431, A432, A433, A434, A435, A436, A437, A438, A443, A444,
  A445, A455, A456, A458, A460, A461, A463, A464, A469, A470, A472, A473, A474, A475, A476, A477, A479, A480, A481,
  A482, A483, A485, A486, A487, A488, A489, A490, A493, A494, A497, A498, A500, A502, A504, A505, A508, A509, A511,
  A512, A514, A523.
- **Root cause: PMT's bits, not RATE.** T-2527 showed RATE's secant is bit-exact on all 308 nested cells when the PMT
  is replaced by Excel's own PMT bits (probes/rate-excel-pmt-bits, gated, passes), and that the RATE cells still off
  are off by the engine's PMT, which differs from Excel on 176 of the 308 inner calls (financial.md "RATE is a secant
  on the FV residual"). RATE amplifies a 1-ulp PMT difference to the 13th digit (its residual cancels near the root).
  The engine's PMT is the exact payment (`accurate_payment`, crates/rustsheet-engine/src/functions/financial/annuity.rs,
  T-2130), within 4 ulp of Excel everywhere; Excel is not the exact payment: even at nper 1,
  `PMT(0.05,1,10000)` is -10499.999999999998 in Excel (m9-fin-pmt A169) where the exact payment is -10500.
- **New hypothesis (not tried by T-1528).** T-1528's three PMT attempts (2026-10-14/16, all before T-2242) found no
  form above the correctly rounded 132/308 bit-exact. T-2242 later found PPMT's discount: Excel's own
  `L = ln(1+rate)` (an atanh series held at 64 bits below 0.38, Excel's LN of `1+rate` from 0.38 on),
  `Q(x) = EXP(x·L)` and `1-Q(-n)` by Excel's expm1, Kahan's formula (financial.md "PPMT discounts with an
  atanh-series log"; numbers.md "Excel's expm1 is Kahan's formula"). The same discount in PMT,
  `-(pv+fv·Q(-n))/(k·((1-Q(-n))/rate))` with `k = 1+rate·type`, scores **175/308** bit-exact on m9-fin-pmt even
  with libm EXP/LN and plain double operators (`python3 docs/research/t3014_pmt_discount_score.py`; the
  `·rate/(k·(1-Q))` order 169, the P-1 form 131), and its inner PMT matches Excel on 62 of the 113 RATE cells. With
  Excel's EXP/LN models (`x87_exp.rs`, `x87_log10.rs`; docs/research/t2464/excelmath.py evaluates them through the
  python bindings) and Excel's twice-rounded operators (numbers.md) it should go higher.
- Evidence is all in eval/: m9-fin-pmt (308 PMT cells, excel-graph), probes/rate-excel-pmt-bits, the Excel-cached
  ironcalc-calc FINANCIAL/PMT.xlsx, IPMT_PPMT.xlsx and CUMPRINC_CUMIPMT.xlsx. Use the fit lab
  (`cd tools && uv run rustsheet-fitlab`) and docs/research/excel-kernel-playbook.md. Decision 0056 applies: two
  attempts, then time-box; if it does not reach the bar, record the scores in financial.md and in T-1528's
  "Blocked because" and leave the cells to T-1528.
- Interactions: T-1528 (blocked, PMT/IPMT/PPMT/NPV bits) gets this as its reopen hypothesis; T-2243's 14 cells
  (PMT.xlsx F15, F33, G10, J15, J33, K10, L15, M10; IPMT_PPMT.xlsx G12, G14, H13, H31, H4; probes/fin-cumipmt-overflow-and-forms
  A13) are this change's regression list, and if the form makes them bit-exact say so in T-2243. RATE's own model
  (`rate_residual`, T-2527) is not to be touched.

## Visible-parity stage
Decision 0114, `visible`: the 113 RATE cells above are visibly off because Excel's PMT is not the exact payment, so
the work is reproducing Excel's formulation, not a more accurate kernel. Target: each of the 113 is within half a
unit of the 15th significant digit of Excel's value. Bit-level criteria still unmet then move to a follow-up with
`defer: phase-1b`.

## Acceptance criteria
- [ ] Visible parity: the 113 owned RATE cells of probes-numeric/m9-fin-irr-rate are within half a unit of the 15th
      significant digit of Excel's value in `make eval` (the case is ungated: show the before/after counts from the
      report or `make eval-show CASE=probes-numeric/m9-fin-irr-rate.case.json`), or the time-box is reached and the
      forms tried, with their m9-fin-pmt scores, are recorded in financial.md and in T-1528.
- [ ] m9-fin-pmt: PMT bit-exact count above the engine's current one, and no PMT cell visibly off.
- [ ] No new known-failure line in any baseline (PMT.xlsx, IPMT_PPMT.xlsx, CUMPRINC_CUMIPMT.xlsx, probes,
      gen-m9 included); `make eval-baseline` records the fixed ones; T-2243's 14 cells re-measured in the Outcome.
- [ ] financial.md's PMT entry updated with the rule, its evidence cells and status.
- [ ] Any fitting script reuses `docs/research/t2464/excelmath.py`'s kernels (Kahan expm1, atanh-series log,
      rounding); `docs/research/t3014_pmt_discount_score.py` is then switched to them or deleted (one definition).
- [ ] `make check` green; reviewers approve.

## Out of scope
- IPMT, PPMT, CUMIPMT, CUMPRINC and NPV (they keep their current forms unless a test proves they share the change).
- RATE's secant and stop rule (T-2005, T-2527: done and bit-exact on Excel's PMT bits).
- IRR's cells of the same case (T-2131).
