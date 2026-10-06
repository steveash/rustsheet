# Financial functions

## Annuity evaluation order and `(1+r)^n` (PV/FV/PMT/NPER/IPMT/PPMT)
Rule: with `P = (1+r)^n` by right-to-left square-and-multiply (numbers.md, "Integer exponents"),
`k = 1+r*type` and `T = pmt*(k*((P-1)/r))`, Excel computes PV = `-(fv+T)/P` and FV = `-(pv*P+T)`
(integer nper, rate ≠ 0). PMT/IPMT/PPMT/NPER/NPV are still unexplained.
Evidence (Excel-cached values, `ironcalc-calc/FINANCIAL/PMT.xlsx`, `PV.xlsx`, `FV.xlsx`):
- `PMT.xlsx` Sheet1!F36 (rate 0.2, nper 5, pv -200): Excel's `(1+r)^n` behaves as exactly `2.48832`
  (square-and-multiply, or exp/log), not libm `powf` (`2.4883199999999994`). With that value
  `(fv*r+pv*r*P)/(1-P)` reproduces `66.87594065792302`.
- But `PMT.xlsx` Sheet1!J32 (rate 0.2, nper 3) matches only with libm `powf` (`1.7279999999999998`),
  so no single power routine plus one formula shape fits all cells.
- Fit experiment (2026-10-02, over 112 PV / 50 PMT / 53 FV cached cells): the best candidates
  (PV `-(fv + pmt*(1+r*t)*((P-1)/r))/P`, FV `-pv*P - pmt*(1+r*t)*((P-1)/r)`, powi) gain ~39 cells
  but regress 13 others (PMT F15/F33/I15/J15/J32/J35/L26, PV F14/G14, IPMT_PPMT G13/H31/H4,
  CUMPRINC_CUMIPMT G15), which the gate forbids.
- Related: gen-m2 shows `^` itself is not correctly rounded (numbers.md, T-0221).
- Oracle sweep (oracle: excel-graph, 2026-10-06, T-0942; probes-numeric/m9-fin-power, m9-fin-pv-fv, m9-fin-pmt,
  m9-fin-nper-npv): with `P = (1+r)^n` computed by right-to-left square-and-multiply (numbers.md,
  "Integer exponents"), `k = 1+r*type` and `T = pmt*(k*((P-1)/r))`, PV = `-(fv+T)/P` matches
  308 of 308 cells and FV = `-(pv*P+T)` matches 308 of 308 (with and without fv/pv and type=1).
  The parenthesization of T matters: `(pmt*k)*((P-1)/r)` matches 270 of 308 PV cells. The same
  P and T do not give PMT: 16 candidate shapes and 4 power routines match at most 63 of 154.
  Desktop agreement: the same PV formula matches all 23 cells of `ironcalc-calc/FINANCIAL/PV.xlsx`
  (Mac Excel 16 cached values) with integer nper > 0 and rate ≠ 0.
  The engine passes 315/616 power, 208/616 PV/FV, 258/924 PMT/IPMT/PPMT and 119/220 NPER/NPV cells.
Status: PARTLY VERIFIED (PV/FV formulation fitted on the sweep; PMT/IPMT/PPMT/NPER/NPV open; T-0901)
Implemented in: n/a (T-0901). Salvaged from draft PR #13 by T-0031.
