# Typed mixed-fraction number formats, excel-graph 2026-10-30 (T-3237)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes` for
`eval/suites/probes/cases/typed-mixed-fraction-number-format.case.json`. The sidecar scores values and the
displayed `text` of A1:A8; these formats are the evidence for docs/excel-semantics/text.md (typed mixed fractions).

```
ENTERED Sheet1!A1 '3 1/2': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A2 '-$3 1/2': values=[[-3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A3 '-3 1/2%': values=[[-0.035]] numberFormat=[['0.00%']]
ENTERED Sheet1!A4 '+ 3 1/2': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A5 '$3 1/2': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A6 '3 1/2%': values=[[0.035]] numberFormat=[['0.00%']]
ENTERED Sheet1!A7 '-3 1/2': values=[[-3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A8 '1 3/2': values=[[2.5]] numberFormat=[['# ?/?']]
```
