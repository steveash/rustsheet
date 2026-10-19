# Rejected typed date+clock text, number formats, excel-graph 2026-10-19 (T-1986)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes`, the run that
wrote `eval/suites/probes/cases/date-text-rejected-clock-formats.oracle.json` (N1-N16). The sidecar
scores values only; these formats are the evidence for docs/excel-semantics/dates.md (typed input).

```
ENTERED S!N1 '1/2/2020 13:00 AM': values=[['1/2/2020 13:00 AM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N2 '1/2/2020 24:00 PM': values=[['1/2/2020 24:00 PM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N3 '1/2/2020 13 PM': values=[['1/2/2020 13 PM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N4 '1/2/2020 0:00 PM': values=[[43832.5]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N5 '1/2/2020 10:00 XM': values=[['1/2/2020 10:00 XM']] numberFormat=[['General']]
ENTERED S!N6 '1/2/2020 abc': values=[['1/2/2020 abc']] numberFormat=[['General']]
ENTERED S!N7 '1/2/2020 10:00 A': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N8 '1/2/2020 25:00 PM': values=[['1/2/2020 25:00 PM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N9 'July 1, 2026 13:00 PM': values=[['July 1, 2026 13:00 PM']] numberFormat=[['d-mmm-yy']]
ENTERED S!N10 '2020-01-02 13:00 PM': values=[['2020-01-02 13:00 PM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N11 '1/2/2020 13:00:00 PM': values=[['1/2/2020 13:00:00 PM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N12 '1/2/2020 99:00 PM': values=[['1/2/2020 99:00 PM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N13 '1/2/2020 10:00 PM PM': values=[['1/2/2020 10:00 PM PM']] numberFormat=[['General']]
ENTERED S!N14 '1/2/2020 1300 PM': values=[['1/2/2020 1300 PM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N15 '13:00 PM': values=[['13:00 PM']] numberFormat=[['General']]
ENTERED S!N16 '1/2/2020 10:00.5 PM': values=[[43832.5069502315]] numberFormat=[['mm:ss.0']]
wrote 1 oracle sidecars (0 failures, 0 skipped, 0 web-vs-desktop conflicts left pending)
```
