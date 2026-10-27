---
id: T-2935
title: "IPMT where the balance cancels on a large fv: Excel's discounted Y at rate 1"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`IPMT(1,6,50,272273,AVERAGE(..))` (gen-m9 0a957119475760c5 Calc!A202) has Excel's value to 15 significant digits,
with no other PMT, IPMT, PPMT, CUMIPMT or CUMPRINC cell moved.

## Context
- Split from T-2587. The balance cancels on fv (`|B| < |fv|`, fv = 1371742112.8222222), so the engine takes
  "IPMT cancels on fv" (doubles on the exact `Y`): -272272.99996232986. Excel -272272.99996209145 is the discounted
  form of financial.md "IPMT follows Excel's own discount where it moves the value" with expm1 (`1-EXP` misses it).
- Taking the discounted form for every cancelling balance makes `IPMT(0.1,8,291,1,638)` (gen-m9 6376e27be7a05871
  A207, which needs the exact `Y`) and probes/fin-ipmt-ppmt-huge-rate A99, A100, A107, A108 (`IPMT(2,2,10,0,315)`,
  pv 0) visibly off. Look for what separates A202 (rate 1, LN branch of L, pv nonzero) from those, e.g. a probe
  sweep of `IPMT(r,6,50,272273,fv)` over r and fv (excel-probe skill).
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A202

## Acceptance criteria
- [ ] Calc!A202 within half a unit of the 15th significant digit (its line leaves eval/baseline/gen-m9.txt or
      stays only as a NumericUlp line), no new known-failure lines in any baseline.
- [ ] financial.md records the rule and its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity of the cancelling cells (T-2667).
