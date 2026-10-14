---
id: T-1524
title: "x87 hypothesis: does 80-bit evaluation of pow, exp, ln, erfc and gamma reproduce Excel's last digits"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1523]
---

## Goal
Settle whether Excel evaluates its elementary kernels in x87 80-bit extended precision (a 64-bit significand, then one rounding to double), as its division provably does (numbers.md "Division rounds twice"), and if so, model it for pow, exp, ln, erfc and gamma.

## Context
Owner request 2026-10-14 (decision 0056); the near-miss figures and their source are in T-1522. The 80-bit division model fixed `/` and the reciprocal in `^`. `^` is known not to be correctly rounded (numbers.md, T-0221); GEOMEAN's root models failed (T-1284), but x87 compositions were only tried for GEOMEAN.

## Acceptance criteria
- [x] Using the fit lab, each of pow, exp, ln, erfc and gamma is scored on its sweep with: correctly rounded, the engine's current code, and an x87 model (operations rounded to a 64-bit significand, final rounding to double). Plus any classic algorithm suggested by the evidence (for example Cephes, Cody's erfc).
- [x] Findings recorded in `docs/excel-semantics/numbers.md` / `statistics.md` with the sweep cells; engine changes stay with the kernels' own tasks: erfc and the normal CDF in T-0968, gamma in T-0973, `^` in T-0221's successors. This task only models and scores, and adds the winning model to those tasks' Context.
- [ ] `make check` green (done locally, 2026-10-14); reviewers approve (pending: no PR from this session).

## Out of scope
- Solvers (T-1525).

## Outcome
**Verdict: the x87 model is rejected as a kernel-wide model.** Excel's kernels do not keep 64-bit
intermediates and round once. Each operation inside a kernel acts like the Excel operator, rounded to
64 bits and then to a double (numbers.md "Division rounds twice"), and that is ordinary double
arithmetic except near a midpoint. Holding intermediates at 64 bits makes a kernel about correctly
rounded. That model never beats the double model, and it beats correctly rounded nowhere (PMT 132 vs
132). Recorded in numbers.md "x87 precision is per operator, not per kernel (T-1524)". No engine task for a
software extended type was filed: the confirmed piece (per-operator double rounding) already exists as
`arithmetic::extended_quotient`/`extended_product`/`extended_sum`.

Lab additions (tools/src/rustsheet_tools/fitlab/): sweep arguments may be literal arithmetic (Excel's
operator model) and patterns may have several groups; `exact.py` gains `square_and_multiply`, the
roundings `to_double_value`/`x87_value`/`extended_value`, an x87 instruction model `X87Insn`
(fyl2x, frndint, f2xm1, fscale; pow/exp/ln), `pi_cr` and an exponent-range guard; new kernels
`annuity.py`, `power.py`, `gamma.py`, and the `erfc-x87` fit in `erfc.py`. 18 new fits
(docs/architecture/fit-lab.md).

Scores, bit-exact / within 4 ulp, best member of each candidate family (`uv run rustsheet-fitlab <fit>`;
"ext" = every intermediate at 64 bits, one final rounding; "x87 op" = each operation rounded to 64 bits then
to a double):

| fit (cells) | results |
|---|---|
| annuity-pv (308) | double 308/308, x87 op 308/308, ext 174/300, ext with P a double 174/300, engine 107/204, correctly rounded 69/161 |
| annuity-fv (308) | double 308/308, x87 op 308/308, ext 230/308, engine 101/194, correctly rounded 53/149 |
| annuity-pmt (308) | correctly rounded 132/308, ext (exact or fyl2x/f2xm1/fscale P) 132/308, double with P-1 = expm1(n log1p r) 129/306 (120/308 for `-(pv*P+fv)/((P-1)/r*k)`), ext with P a double 96/247, double/x87 op 91/247, engine 54/176 |
| annuity-ipmt (308) | correctly rounded 219/308, ext 218/301, double 195/265, engine 189/257 |
| annuity-ppmt (308) | correctly rounded 94/297, ext 85/281, double 43/188, engine 15/108 |
| annuity-nper (95) | **double `ln(ratio)/ln(1+r)` 95/95**, x87 op 95/95, engine 61/88, log1p 11/24, correctly rounded 6/25, ext 6/25 |
| annuity-npv (88) | double (running factor or Horner) 27/77, ext 26/77, engine 21/79, correctly rounded 18/70 |
| power-1+r (462) | Excel model 462/462 (1+r a double or 64-bit), engine 462, x87 insn with 1/ for y<0 175/358, correctly rounded/ext/x87 pow 155-161/353-357 |
| exp-ln (154) | each function's result a double (libm, correctly rounded, or x87 at 64 bits then stored) 154/154, engine 154, ext chain 47/117 |
| power-nonint (183) | engine 183, Excel model 180/180 (3 out-of-range cells unmodelled), x87 insn 1/ 101/140, ext/x87 pow 94/142, correctly rounded 93/142 |
| power-recip (45) | engine 45, Excel model 45/45, x87 insn 1/ 43/45, ext 37/45, correctly rounded 20/45 |
| power-mantissa (76) | engine 76, Excel model 76/76 (both product roundings), ext 10-11/48, correctly rounded 10/48 |
| power-large-int (224) | engine 224; Excel model 150/197 either product rounding (the lab leaves out base 10 and the range rules; same cells), ext 85/147, correctly rounded 83/145 |
| power-sqrt5 (20) | **Excel model with twice-rounded products 20/20**; once-rounded products 1/5 = engine 1/5; correctly rounded, ext, x87 1/9 |
| gamma-int (171) | **downward product in doubles 92/137 (all n <= 88)**, correctly rounded 40/154, ext 40/154, tgamma 38/150, engine 1/3 |
| gammaln-int (171) | engine 108/171, ln of downward product 82/171, correctly rounded 80/171, ext 80/171, lgamma 72/171 |
| gamma-half-int (62) | correctly rounded 45/62, ext 45/62, double recurrence 34/62, tgamma 21/62, engine 0/1 |
| erfc-x87 (205 tail) | T-1187 `bu_even` in doubles 58/201, ext (exp64 or x87 insn) 44/203, 128-bit reference 44/203, engine 44/203 |

New rules (findings recorded; the engine changes stay with the owner tasks):
- `^`: square-and-multiply rounds each product twice. That resolves T-1495, 20/20 on SQRT(5)^n with no
  change elsewhere (numbers.md "Integer exponents beyond 360"; T-1495 Context: use `extended_product`).
- NPER = `ln((pmt*k - fv*r)/(pmt*k + pv*r)) / ln(1+r)` in doubles, 95/95 (financial.md; T-0901 Context).
- GAMMA(n), n <= 88, is the downward double product (statistics.md; T-0973 Context).
- PMT: an accurate double formulation with expm1/log1p would lift the engine from 54/176 to 120-129
  bit-exact and 306-308 within 4 ulp (T-0901 Context).

Time-boxed (decision 0056, two attempts each this session, no model above correctly rounded or above
the double grid): PMT, IPMT, PPMT, NPV, GAMMALN, GAMMA at half-integers, ERFC. The candidates tried are
in financial.md and statistics.md. The starting lead ("PMT 133/308 with a 64-bit q from the exact 1+r")
is the correctly rounded tie: correctly rounded gets 132, and Excel's PMT lies up to 3.5 ulps from the
exact value (cell A7 `PMT(0.2,2,10000)` = -6545.454545454547 at -1.45 ulps), which no half-ulp-accurate
model reproduces. Also fixed: numbers.md "Division rounds twice" gave …4171 as the correctly rounded
`1/(1+26955297717044*2^-52)`; it is 0.9940503319874169 (checked with `exact.to_double`).
