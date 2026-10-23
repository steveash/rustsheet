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

## Acceptance criteria
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
