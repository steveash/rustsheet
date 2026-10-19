# Typed date and clock text number formats, excel-graph 2026-10-19 (T-1952)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes`, the run that
wrote `eval/suites/probes/cases/timevalue-non-glued-date-text.oracle.json` (values are scored; the
formats are the evidence for docs/excel-semantics/dates.md). Year-less values reflect the oracle's 2026 clock.

```
ENTERED Sheet1!E1 '1/2/2020': values=[[43832]] numberFormat=[['m/d/yyyy']]
ENTERED Sheet1!E2 'July 1, 2026': values=[[46204]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!E3 '1 July 2026': values=[[46204]] numberFormat=[['d-mmm-yy']]
ENTERED Sheet1!E4 '2020-01-02': values=[[43832]] numberFormat=[['m/d/yyyy']]
ENTERED Sheet1!E5 '1/2/2020 10:00': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED Sheet1!E6 '2020-01-02 10:00': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED Sheet1!E7 'July 1, 2026 10:00': values=[[46204.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED Sheet1!E8 '7/1 10:00': values=[[46204.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED Sheet1!E9 '1/2/2020 10:00:30': values=[[43832.4170138889]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED Sheet1!E10 '1/2/2020 10 AM': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
```
