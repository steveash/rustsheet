# Typed date+clock shapes, number formats, excel-graph 2026-10-19 (T-1979)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes`, the run that
wrote `eval/suites/probes/cases/date-text-typed-clock-shapes.oracle.json`. The sidecar scores
values only; these formats are the evidence for docs/excel-semantics/dates.md (typed input).

```
ENTERED S!N11 '1/2/2020 10:00:30 PM': values=[[43832.9170138889]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N12 '1/2/2020 10:00 pm': values=[[43832.9166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N13 '1/2/2020 10:00 Pm': values=[[43832.9166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N14 '1/2/2020 13:00 PM': values=[['1/2/2020 13:00 PM']] numberFormat=[['m/d/yyyy']]
ENTERED S!N15 '1/2/2020 10:00AM': values=[['1/2/2020 10:00AM']] numberFormat=[['General']]
ENTERED S!N16 '1/2/2020 10 AM': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N17 '1/2/2020 10 PM': values=[[43832.9166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N18 '1/2/2020 24:00': values=[[43833]] numberFormat=[['General']]
ENTERED S!N19 '1/2/2020 25:00': values=[[43833.0416666667]] numberFormat=[['General']]
ENTERED S!N20 '1/2/2020 10:00:30.5': values=[[43832.4170196759]] numberFormat=[['mm:ss.0']]
ENTERED S!N21 '1/2/2020 10:00.5': values=[[43832.0069502315]] numberFormat=[['mm:ss.0']]
ENTERED S!N22 '1/2/2020 10:00  AM': values=[[43832.4166666667]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N23 '1/2/2020 10:60': values=[[43832.4583333333]] numberFormat=[['General']]
ENTERED S!N24 '1/2/2020 10:00:60': values=[[43832.4173611111]] numberFormat=[['General']]
ENTERED S!N25 '1/2/2020 0:00 AM': values=[[43832]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N26 '1/2/2020 12:00 AM': values=[[43832]] numberFormat=[['m/d/yyyy h:mm']]
ENTERED S!N27 '1/2/2020 12:30 PM': values=[[43832.5208333333]] numberFormat=[['m/d/yyyy h:mm']]
```
