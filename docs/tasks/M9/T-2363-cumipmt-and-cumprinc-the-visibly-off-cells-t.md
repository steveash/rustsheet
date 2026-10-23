---
id: T-2363
title: "CUMIPMT and CUMPRINC: the visibly-off cells T-2192 left (periods near 1E15, type 1 from period 1, past 2^53)"
milestone: M9
lane: functions-financial
status: in-progress
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

## Acceptance criteria
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
