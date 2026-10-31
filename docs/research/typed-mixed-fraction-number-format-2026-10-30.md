# Typed mixed-fraction number formats, excel-graph 2026-10-30 (T-3237)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes` for
`eval/suites/probes/cases/typed-mixed-fraction-number-format.case.json`. The sidecar scores values and the
displayed `text` of A1:A31; these formats are the evidence for docs/excel-semantics/text.md (typed mixed fractions).

```
ENTERED Sheet1!A1 '3 1/2': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A2 '-$3 1/2': values=[[-3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A3 '-3 1/2%': values=[[-0.035]] numberFormat=[['0.00%']]
ENTERED Sheet1!A4 '+ 3 1/2': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A5 '$3 1/2': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A6 '3 1/2%': values=[[0.035]] numberFormat=[['0.00%']]
ENTERED Sheet1!A7 '-3 1/2': values=[[-3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A8 '1 3/2': values=[[2.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A9 '3 1/12': values=[[3.08333333333333]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A10 '3 10/100': values=[[3.1]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A11 '3 01/02': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A12 '1 1/16': values=[[1.0625]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A13 '-$1 7/16': values=[[-1.4375]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A14 '2 5/16%': values=[[0.023125]] numberFormat=[['0.00%']]
ENTERED Sheet1!A15 '-$3 1/2%': values=[['-$3 1/2%']] numberFormat=[['0.00%']]
ENTERED Sheet1!A16 '3 1/200': values=[[3.005]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A17 '3 1/120': values=[[3.00833333333333]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A18 '3 1/002': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A19 '3 1/99': values=[[3.01010101010101]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A20 '3 1/9': values=[[3.11111111111111]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A21 '3 1/10': values=[[3.1]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A22 '3 1/999': values=[[3.001001001001]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A23 '3 1/1000': values=[[3.001]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A24 '3 1/10000': values=[[3.0001]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A25 '3 001/2': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A26 '3 1/0002': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A27 '3 1/2000': values=[[3.0005]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A28 '3 5/10': values=[[3.5]] numberFormat=[['# ??/??']]
ENTERED Sheet1!A29 '3 2/4': values=[[3.5]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A30 '3 12/5': values=[[5.4]] numberFormat=[['# ?/?']]
ENTERED Sheet1!A31 '3 10/5': values=[[5]] numberFormat=[['# ?/?']]
```
