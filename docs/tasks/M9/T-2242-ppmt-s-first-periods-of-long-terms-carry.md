---
id: T-2242
title: "PPMT's first periods of long terms carry Excel's error in (1+r)^-n"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-2130]
---

## Goal
The four PPMT cells of probes-numeric/m9-fin-pmt that are still visibly off after T-2130 are
within half a unit of the 15th significant digit of Excel's value, with no other PMT, IPMT or
PPMT cell made visibly off.

## Context
- T-2130 made PMT, IPMT and PPMT the exact payment on the double arguments (double-double
  `accurate_growth` and closed forms in `functions/financial/annuity.rs`). That puts every PMT and
  IPMT cell of m9-fin-pmt and every cell of IPMT_PPMT.xlsx within 15 digits.
- Four PPMT cells are not, because Excel itself is 7 to 41 ulp from the exact value there (period 1
  of a long term, pv 10000, fv 0, type 0):
  - A82 `PPMT(0.2,1,360,10000)`: Excel -6.248581209907205E-26, exact -6.248581209907172E-26;
  - A166 `PPMT(0.1,1,360,10000)`: Excel -1.2549699967849914E-12, exact -1.254969996784983E-12;
  - A412 `PPMT(0.125,1,120,10000)`: Excel -0.0009090915796273315, exact -0.0009090915796273322;
  - A418 `PPMT(0.125,1,360,10000)`: Excel -4.808414874592364E-16, exact -4.808414874592376E-16.
- A lead from T-2130 (financial.md "PMT, IPMT and PPMT are the exact payment"): for the 154 fv-0,
  type-0 PPMT cells, `PMT * exp((per-1-nper)*log1p(rate))` with the exact PMT rounded to a double
  and libm `exp`/`log1p` is within 15 digits on 153 (all but A166) and bit-exact on 63. Excel's
  `exp` model (x87, numbers.md) instead of libm gives the same 153; `ln` of the 64-bit `1+rate`
  gives 150. At A166 Excel's exponent looks one ulp below RN(360*log1p(0.1)). Nothing is known yet
  for fv ≠ 0 or type 1, where the exact form already passes every cell.
- T-2130's fit counts came from one-off scripts over m9-fin-pmt.oracle.json. Reproduce them as a
  `rustsheet-fitlab annuity-ppmt` candidate before relying on them.
- probes-numeric is not gated (`gate = false`), so nothing locks T-2130's m9-fin-pmt gain
  (visibly off 250 → 4). Re-check the case with `make eval-show` after any change here.

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): PPMT passed through MINA and MAXA (`MINA(PPMT((-0.5),12,158,..))` Excel -0.00046386718749999705, ours -0.0004638671875).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 1 visibly off.
Examples:
- gen-m9/38e4fc309fe199ea Calc!A73 `=MINA(PPMT((-0.5),12,158,(Data!$B4-0.1)))`: Excel -0.00046386718749999705, ours -0.0004638671875
- gen-m9/9e2225ead8a6a07b Calc!A4 `=MAXA((1.005*PPMT(0.1,100,250,309299,Data!E5,Data!C12)))`: Excel -0.017468244384972693, ours -0.017468244384972673
Cells:
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A73
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A4

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): PPMT and IPMT of the financial sweep off in the 13th to 15th digit: mostly PPMT's first periods of long terms at rates of 1 to 100 (`PPMT(10,7,158,147532,763)` Excel -7.575972059482481E-153, ours ...517E-153), plus IPMT at fractional periods (`IPMT(3.14159,1.999,167,0.375)` -1.1780962499999998, ours ...083).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 44 known-failure lines in eval/baseline/gen-m9.txt, 44 visibly off.
Examples:
- gen-m9/3aa168259335be54 Calc!A291 `=PPMT(1,10,225,218476)`: Excel -2.0745536588226078e-60, ours -2.0745536588226178e-60
- gen-m9/3c06f9298f65a3ba Calc!A256 `=PPMT(0.05,3,114,302806)`: Excel -64.35825018498787, ours -64.35825018498795
- gen-m9/6376e27be7a05871 Calc!A207 `=IPMT(0.1,8,291,1,638)`: Excel -0.09999999994538485, ours -0.09999999994537818
- gen-m9/67a4c03afd63f988 Calc!A131 `=PPMT(10,6,33,6079)`: Excel -4.215382215765404e-25, ours -4.21538221576539e-25
- gen-m9/a91bad4e84f49739 Calc!A305 `=PPMT(Data!B1,2,60,0.01)`: Excel -4.98460498419341e-26, ours -4.984604984193435e-26
- gen-m9/c0681a8b150ce301 Calc!A130 `=PPMT(1,2,190,337671)`: Excel -4.303527509792124e-52, ours -4.30352750979205e-52
Cells:
- gen-m9/0a957119475760c5.case.json (2): Calc!A154, A202
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A227
- gen-m9/1dce4774f5c0e736.case.json (2): Calc!A63, A112
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A382
- gen-m9/3aa168259335be54.case.json (1): Calc!A291
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A256
- gen-m9/5422bdbf173fc233.case.json (1): Calc!A383
- gen-m9/5c6a23c4f6597f1c.case.json (2): Calc!A29, A176
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A177
- gen-m9/6376e27be7a05871.case.json (2): Calc!A158, A207
- gen-m9/67a4c03afd63f988.case.json (3): Calc!A82, A131, A180
- gen-m9/79d87fc527927626.case.json (4): Calc!A46, A95, A242, A291
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A374
- gen-m9/9d9ceba257f8001b.case.json (1): Calc!A228
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A305
- gen-m9/abb7d565af89b849.case.json (2): Calc!A16, A310
- gen-m9/b6a02893a8d6b151.case.json (2): Calc!A35, A152
- gen-m9/b72d952c5f52cb4d.case.json (4): Calc!A41, A188, A286, A364
- gen-m9/b73c6cc96e093f03.case.json (3): Calc!A205, A268, A352
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A69
- gen-m9/c0681a8b150ce301.case.json (3): Calc!A32, A104, A130
- gen-m9/d0854366830dcdee.case.json (2): Calc!A41, A335
- gen-m9/d912671baa1364aa.case.json (2): Calc!A244, A293
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A385

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 44 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2256): these 2 known-failure lines (the T-2256 cells in Context) disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A82, A166, A412 and A418 are within half a unit of the 15th digit; every other PMT, IPMT
      and PPMT cell of m9-fin-pmt and IPMT_PPMT.xlsx stays within it.
- [ ] financial.md records the PPMT formulation and its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity of the other cells (T-2243, T-1528).

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083).
