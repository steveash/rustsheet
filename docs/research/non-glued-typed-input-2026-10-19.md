# Typed-input number formats for non-glued date and clock text, excel-graph 2026-10-19 (T-1952)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes`, the run that
wrote `eval/suites/probes/cases/date-system-1904-timevalue-non-glued.oracle.json` (first three
lines, a 1904 workbook) and `eval/suites/probes/cases/date-text-non-glued-timevalue.oracle.json`
(the rest). The sidecars score values only; these formats are the evidence for
docs/excel-semantics/dates.md ("Date text with a month name glued to digits", typed input). The
year-less value (46204 = 2026-07-01) reflects the oracle's 2026 clock and is not scored.

```
ENTERED S!N11 '12/31/1903 10:00': values=[['12/31/1903 10:00']] numberFormat=[['General']]
ENTERED S!N12 '1/2/1904 10:00': values=[[1.41666666666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N13 '1/2/2020 10:00': values=[[42370.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N11 '1/2/2020 10:00': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N12 '2020-01-02 10:00': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N13 'July 1, 2026 10:00': values=[[46204.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N14 '7/1 10:00': values=[[46204.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N15 '1/2/2020': values=[[43832]] numberFormat=[['m/d/yyyy']]
ENTERED S!N16 '1/2/2020 10:00:30': values=[[43832.4170138889]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N17 '1/2/2020 10:00 AM': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
```
