# Typed mixed-separator dates and a leading-space clock, excel-graph 2026-11-01 (T-3751)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes`, the run that wrote
`eval/suites/probes/cases/date-text-mixed-separator-typed.oracle.json`. The sidecar scores values only; these
formats are the evidence for docs/excel-semantics/dates.md (typed input). (`CELL("format", ...)` is #VALUE! on
excel-graph, so formats come from this log only.)

```
ENTERED S!A1 '7/1-2025': values=[[45839]] numberFormat=[['m/d/yyyy']]
ENTERED S!A2 '7-1/2025': values=[[45839]] numberFormat=[['m/d/yyyy']]
ENTERED S!A3 '2025/07-01': values=[[45839]] numberFormat=[['m/d/yyyy']]
ENTERED S!A4 '2025-07/01': values=[[45839]] numberFormat=[['m/d/yyyy']]
ENTERED S!A5 '1-Jul/2025': values=[[45839]] numberFormat=[['d-mmm-yy']]
ENTERED S!A6 '1/Jul-2025': values=[[45839]] numberFormat=[['d-mmm-yy']]
ENTERED S!A7 '7/1-25': values=[[45839]] numberFormat=[['m/d/yyyy']]
ENTERED S!A8 ' 10:00 7/1/2025': values=[[' 10:00 7/1/2025']] numberFormat=[['General']]
ENTERED S!A9 ' 10:00 AM 7/1/2025': values=[[' 10:00 AM 7/1/2025']] numberFormat=[['General']]
ENTERED S!A10 '10:00 7/1/2025': values=[[45839.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
```
