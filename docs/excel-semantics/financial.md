# Financial annuity functions (PV/FV/PMT/NPER/IPMT/PPMT)

## Annuity evaluation order and `(1+r)^n`
Rule: Excel's ULP-level results for PV/FV/PMT are not reproduced by our formulas; the exact
expression order and power routine are unknown.
Evidence (Excel-cached values, `ironcalc-calc/FINANCIAL/PMT.xlsx`, `PV.xlsx`, `FV.xlsx`):
- `PMT.xlsx` Sheet1!F36 (rate 0.2, nper 5, pv -200): Excel `(1+r)^n` behaves as exactly `2.48832`
  (square-and-multiply, or exp/log), not libm `powf` (`2.4883199999999994`). With that value
  `(fv*r+pv*r*P)/(1-P)` reproduces `66.87594065792302`.
- But `PMT.xlsx` Sheet1!J32 (rate 0.2, nper 3) matches only with libm `powf` (`1.7279999999999998`).
  So no single power routine plus one formula shape fits all cells.
- Fit experiment (Python over 112 PV / 50 PMT / 53 FV cached cells): best candidates —
  PV `-(fv + pmt*(1+r*t)*((P-1)/r))/P`, FV `-pv*P - pmt*(1+r*t)*((P-1)/r)`, powi — gain ~39 cells
  but regress 13 others (PMT F15/F33/I15/J15/J32/J35/L26, PV F14/G14, IPMT_PPMT G13/H31/H4,
  CUMPRINC_CUMIPMT G15), which the gate forbids.
Status: UNVERIFIED (needs oracle probes, T-0004)
Implemented in: n/a (see T-0901)
