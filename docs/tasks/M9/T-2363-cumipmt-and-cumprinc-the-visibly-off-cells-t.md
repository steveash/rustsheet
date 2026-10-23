---
id: T-2363
title: "CUMIPMT and CUMPRINC: the visibly-off cells T-2192 left (periods near 1E15, type 1 from period 1, past 2^53)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-2192]
---

## Goal
CUMIPMT and CUMPRINC are within half a unit of the 15th significant digit of Excel on the nine
cells T-2192 left visibly off.

## Context
T-2192 made type 0's short sums per-period forms (`pmt·exp((k-1-nper)·L)` for the principal, the
balance form for the interest) and gave CUMPRINC of type 1 a value past an overflowing `pv·r`.
These cells are still visibly off (docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over
many periods"):
- probes/fin-cumipmt-overflow-and-forms A9, A10, A11 and fin-remaining-later-argument-ranges K3
  (type 0, periods near 1E15, where `(1+r)^(k-1)` overflows; 32 to 128 ulps), and
  fin-cumipmt-period-bound A18 (the same of type 1, 16 ulps).
- probes/fin-cumipmt-overflow-and-forms A7 = CUMPRINC_CUMIPMT.xlsx G2 (`CUMIPMT(0.0083,36,8000,1,10,1)`,
  6 ulps; type 1 from period 1) and CUMPRINC_CUMIPMT.xlsx H26 (`CUMPRINC(7,36,8000,3,10,1)`, 21 ulps).
- probes/fin-cumipmt-overflow-and-forms A5, A6 (a 1E-15 rate past period 2^53).

Leads from T-2192's fit (float64 checks against the oracle values, no engine change yet):
- Excel's CUMIPMT is `-pmt·m - CUMPRINC` in doubles, `m = end-start+1`, given Excel's own CUMPRINC:
  bit-exact on K3 and A9 (with A19's -10367.62824821948), G2 (with H2), G4, G5, G6, G21, G22 and
  others, and type 0 with `pmt = (pv/(1-v^nper))·r` on all 13 such pairs. So the interest cells
  reduce to the principal: K3/A9 need A19 to the bit (ours is 1 ulp off), G2 needs H2 (1 ulp).
- A11 needs a one-period principal of 990.0990099009903 (`pmt·v` with `v` 2 ulps above
  `1/1.01`); no `exp`, `pow`, `expm1` or chained-product form tried gives it.
- The per-period `exp` principal of type 1 reaches H26 (2 ulps) but misses H22 and H25 by an ulp or
  two (they pass today), and `pv·(r/(1+r))/(1-v^nper)` as type 1's payment gets H3, H16 and H31 to
  the bit but misses H25 by an ulp.
- A6's per-period `exp` sum is -4.085993687045532E-19 against Excel's ...525 (just past half a unit).
- T-2192's type 0 principal form moved CUMPRINC_CUMIPMT.xlsx H29 from 1 ulp to 3 (still visibly
  right); a bit-exact fit (T-1374) should start there.
- `CUMULATIVE_WEIGHT` (functions/lift/kinds/financial.rs) was measured on the IPMT/PPMT sums; type 0
  now runs up to 32 `exp` steps first and can fall through to them. Re-measure it if a form here
  adds work.
Code: functions/financial/cumulative.rs.

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): CUMIPMT passed through STDEVA (visibly off).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 visibly off.
Examples:
- gen-m9/6e18550e9fc9c1b9 Calc!A329 `=STDEVA(Data!C1:C4,CUMIPMT(1E-10,77,380096,1,18,1))`: Excel 0.8164966307790201, ours 0.8164966307790212
Cells:
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A329

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): CUMIPMT and CUMPRINC values of the financial sweep: at huge rates ours is wildly off (`CUMIPMT(1E15,170,271151,8,16,0)` Excel -2.440359000000003E+21, ours -4.85E+229; `CUMIPMT(1E+10,235,99106,6,18,0)` -1.288378E+16, ours 1.39E+129; `CUMIPMT(100,167,7,6,16,0)` -7700, ours -1.1E15), at tiny rates ours misses Excel's rounding (`CUMIPMT(1E-300,243,80923,12,22,0)` -4.547473508864641E-13, ours -8.9E-295), and the type-1 and fractional-rate sums are off in the 12th to 15th digit; `DOLLARDE(CUMPRINC(1E-300,129,257125,1,17,0),16)` follows CUMPRINC.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 35 known-failure lines in eval/baseline/gen-m9.txt, 35 visibly off.
Examples:
- gen-m9/0a957119475760c5 Calc!A383 `=CUMPRINC(10,121,308352,2,16,1)`: Excel -1.263054652346343e-105, ours -1.2630546523463632e-105
- gen-m9/1cd3096e3e87c965 Calc!A316 `=CUMIPMT(10,89,267388,3,13,0)`: Excel -29412680.0, ours -29413399.999991894
- gen-m9/3c06f9298f65a3ba Calc!A189 `=CUMIPMT(0.5,96,265678,1,23,0)`: Excel -3055296.9999999627, ours -3055297.0
- gen-m9/5c6a23c4f6597f1c Calc!A392 `=CUMIPMT(1E-9,55,157761,1,18,0)`: Excel -0.002400835590378847, ours -0.0024008355911119897
- gen-m9/9a1fa5366bc50918 Calc!A361 `=CUMPRINC(10,261,484823,4,24,0)`: Excel -7.507871520276804e-242, ours -7.50787152027683e-242
- gen-m9/a91bad4e84f49739 Calc!A193 `=CUMPRINC(1,331,199079,3,14,1)`: Excel -3.727183608915611e-91, ours -3.727183608915638e-91
Cells:
- gen-m9/01523d247594952e.case.json (1): Calc!A322
- gen-m9/0a957119475760c5.case.json (1): Calc!A383
- gen-m9/1cd3096e3e87c965.case.json (2): Calc!A147, A316
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A81
- gen-m9/3c06f9298f65a3ba.case.json (3): Calc!A91, A140, A189
- gen-m9/5c6a23c4f6597f1c.case.json (3): Calc!A49, A343, A392
- gen-m9/5ded7594e597e31e.case.json (3): Calc!A22, A81, A267
- gen-m9/6376e27be7a05871.case.json (1): Calc!A365
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A110
- gen-m9/79d87fc527927626.case.json (1): Calc!A352
- gen-m9/81bcdc0bec0c8e6b.case.json (2): Calc!A274, A372
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A5
- gen-m9/9a1fa5366bc50918.case.json (2): Calc!A18, A361
- gen-m9/9d9ceba257f8001b.case.json (1): Calc!A372
- gen-m9/a91bad4e84f49739.case.json (2): Calc!A83, A193
- gen-m9/abb7d565af89b849.case.json (1): Calc!A84
- gen-m9/b6a02893a8d6b151.case.json (2): Calc!A305, A398
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A257
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A62
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A162
- gen-m9/d0854366830dcdee.case.json (1): Calc!A98
- gen-m9/d912671baa1364aa.case.json (1): Calc!A169
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A330, A379

## Notes (opus run, 2026-10-23; measurements, no engine change)
- A5/A6 (rate 1E-15, periods 1E16..1E16+8, past 2^53): Excel is within an ulp of the correctly
  rounded sums. A 60-digit `decimal` reference gives P = -4.0859936870455272E-19 (Excel ...525)
  and I = -8.9995914191816788E-15 (Excel ...679). The closed form's `v^(b-nper) - v^(a-1-nper)`
  cancels (-3.25E-19). A per-period `exp((k-1-nper)·L)` sum over the nine distinct integers
  `start + i` gives ...532 (7E-34 off, just past half a unit). Its error is the rounding of
  `n·L` with n near 1E16, plus `log1p`'s own rounding times n: both are relative 1E-15. So an
  accurate kernel needs the exponent in double-double: `n = (start - nper) + (i - 1)` split
  hi/lo; `L_lo = (r - expm1(L_hi))/exp(L_hi)`; `e = n·L_hi` with its `mul_add` residual plus
  `n·L_lo`; then `exp(e_hi)·(1 + e_lo)`. With that P, `I = -pmt·m - P` is already visibly
  right on A5 (...677 with the naive P).
- Type 1's interest as the closed form (`payment·m - principal`, the IPMT sum disabled): G2
  becomes 1 ulp, but CUMPRINC_CUMIPMT.xlsx G3, G9, G15 and probes/fin-cumipmt-overflow-and-forms
  A8 regress from bit-exact, and gen-m9 A329 does not change. Not that way.
- G2 confirmed: `-pmt1·10 - H2` with `pmt1 = pv·r/(-expm1(-nper·L))/(1+r)` and Excel's H2 gives
  G2 to the bit, while `pv·(r/(1+r))/...`, `pow` and the classic form do not. So G2 needs H2 to
  the bit, i.e. type 1's principal.

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 35 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2256): these 1 known-failure lines (the T-2256 cells in Context) disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the nine cells above are within half a unit of the 15th significant digit
  of Excel's value, with no new known-failure lines in any baseline (`make eval-baseline`).
- [ ] The financial.md entry's "Ours:" line names this task's change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-exact fits once the cells are visibly right (T-1374).

## Visible-parity stage
Decision 0083: the acceptance criteria are the visible-parity stage.
