# Typed date + clock with a single-letter A/P suffix (T-1994)

Formats seen in the ENTERED log of the excel-graph fill of probes/date-text-single-letter-ap
(2026-10-19); the committed sidecar holds only the ISNUMBER results.

| Entered | Value | numberFormat |
|---|---|---|
| `1/2/2020 10:00 P` | 43832.9166666667 | `m/d/yyyy h:mm` |
| `1/2/2020 10:00 a` | 43832.4166666667 | `m/d/yyyy h:mm` |
| `1/2/2020 10:00A` | text | General |
| `1/2/2020 10 A` / `10 p` | 43832.4166666667 / .9166666667 | `m/d/yyyy h:mm` |
| `1/2/2020 10:00 Am` | 43832.4166666667 | `m/d/yyyy h:mm` |
| `1/2/2020 10:00:30 A` | 43832.4170138889 | `m/d/yyyy h:mm` |
| `1/2/2020 13:00 A` | text | `m/d/yyyy` |
| `1/2/2020 10:00 A.` | 43832.4166666667 | `m/d/yyyy h:mm` (not yet in engine, T-2011) |
| `July 1, 2026 10:00 A` | 46204.4166666667 | `m/d/yyyy h:mm` |
| `1/2/2020 10:00 A A` | text | General |
| `10:00 A` | 0.416666666666667 | `h:mm AM/PM` (not yet in engine, T-2011) |
