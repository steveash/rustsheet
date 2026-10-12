# Typed currency/percent formats, excel-graph 2026-10-11 (T-0283)

`ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes` for
`eval/suites/probes/cases/typed-currency-percent-input.oracle.json` (same format as
typed-input-formats-2026-10-07.md). The sidecar scores the values only; the formats are the evidence.

```
ENTERED Sheet1!A1 '$1.50': values=[[1.5]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A2 '1.5%': values=[[0.015]] numberFormat=[['0.00%']]
ENTERED Sheet1!A3 '(1.5%)': values=[[-0.015]] numberFormat=[['0.00%']]
ENTERED Sheet1!A4 '($1.50)': values=[[-1.5]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A5 '-$1': values=[[-1]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A6 '-$1.50': values=[[-1.5]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A7 '$-1': values=[[-1]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A8 '1.50%': values=[[0.015]] numberFormat=[['0.00%']]
ENTERED Sheet1!A9 '0.5%': values=[[0.005]] numberFormat=[['0.00%']]
ENTERED Sheet1!A10 '12.345%': values=[[0.12345]] numberFormat=[['0.00%']]
ENTERED Sheet1!A11 '$1,234.5': values=[[1234.5]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A12 '$1e3': values=[[1000]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A13 '1e3%': values=[[10]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A14 '1,000%': values=[[10]] numberFormat=[['0%']]
ENTERED Sheet1!A15 '(1,000.5)': values=[[-1000.5]] numberFormat=[['#,##0.00']]
ENTERED Sheet1!A16 '1,000.50': values=[[1000.5]] numberFormat=[['#,##0.00']]
ENTERED Sheet1!A17 '1.5e3': values=[[1500]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A18 '$0': values=[[0]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A19 '0%': values=[[0]] numberFormat=[['0%']]
ENTERED Sheet1!A20 '-1%': values=[[-0.01]] numberFormat=[['0%']]
ENTERED Sheet1!A21 '-1.5%': values=[[-0.015]] numberFormat=[['0.00%']]
ENTERED Sheet1!A22 '$1,000': values=[[1000]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A23 '1€': values=[['1€']] numberFormat=[['General']]
ENTERED Sheet1!A24 '1.5€': values=[['1.5€']] numberFormat=[['General']]
ENTERED Sheet1!A25 '-1€': values=[['-1€']] numberFormat=[['General']]
ENTERED Sheet1!A26 '€1': values=[['€1']] numberFormat=[['General']]
ENTERED Sheet1!A27 '1.0%': values=[[0.01]] numberFormat=[['0.00%']]
ENTERED Sheet1!A28 '$1.0': values=[[1]] numberFormat=[['$#,##0.00_);[Red]($#,##0.00)']]
ENTERED Sheet1!A29 '-$1e3': values=[[-1000]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A30 '($1e3)': values=[[-1000]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A31 '-1e3%': values=[[-10]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A32 '(1e3)': values=[[-1000]] numberFormat=[['0.00E+00']]
ENTERED Sheet1!A33 '$1,000.5e0': values=[[1000.5]] numberFormat=[['0.00E+00']]
```
