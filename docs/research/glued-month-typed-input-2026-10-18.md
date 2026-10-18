# Typed-input number formats for glued month-name text, excel-graph 2026-10-18 (T-1879)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes`, the run that
wrote `eval/suites/probes/cases/date-text-glued-month-2.oracle.json`. Each line is the Graph range
PATCH response after writing the text with `values`: the value and the `numberFormat` Excel for
the web picked. The sidecar scores only values (columns O-R); these formats are the evidence for
docs/excel-semantics/dates.md ("Date text with a month name glued to digits"). The year-less
values (46204 = 2026-07-01) reflect the oracle's 2026 clock.

```
ENTERED Sheet1!N1 'July1': values=[[46204]] numberFormat=[['d-mmm']]
ENTERED Sheet1!N2 'July01': values=[[46204]] numberFormat=[['d-mmm']]
ENTERED Sheet1!N3 'Jul2026': values=[[46204]] numberFormat=[['mmm-yy']]
ENTERED Sheet1!N4 'Jun31': values=[[11475]] numberFormat=[['mmm-yy']]
ENTERED Sheet1!N5 '1Jul 2026': values=[[46204]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!N6 'Jul1 , 2026': values=[[46204]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!N7 '1JUN2021': values=[[44348]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!N8 'July1 10:00': values=[[46204.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED Sheet1!N9 'Jun1.5': values=[['Jun1.5']] numberFormat=[['General']]
```
