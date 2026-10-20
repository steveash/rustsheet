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
| `1/2/2020 10:00 A.` | 43832.4166666667 | `m/d/yyyy h:mm` |
| `July 1, 2026 10:00 A` | 46204.4166666667 | `m/d/yyyy h:mm` |
| `1/2/2020 10:00 A A` | text | General |
| `10:00 A` | 0.416666666666667 | `h:mm AM/PM` |

T-2011 refill (same case, cells N13-N20):

| Entered | Value | numberFormat |
|---|---|---|
| `10:00:30 A` | 0.417013888888889 | `h:mm:ss AM/PM` |
| `10:00 p` | 0.916666666666667 | `h:mm AM/PM` |
| `10:00 A.` | 0.416666666666667 | `h:mm AM/PM` |
| `1/2/2020 13:00 A.` | text | `m/d/yyyy` |
| `1/2/2020 10 A.` | 43832.4166666667 | `m/d/yyyy h:mm` |
| `1/2/2020 10:00:30 P.` | 43832.9170138889 | `m/d/yyyy h:mm` |
| `10 A` | 0.416666666666667 | `h:mm AM/PM` |
| `10:00 AM` | 0.416666666666667 | `h:mm AM/PM` |

T-2053 refill (cells N21-N29):

| Entered | Value | numberFormat |
|---|---|---|
| `10 A.` | 0.416666666666667 | `h:mm AM/PM` |
| `0:00 A` / `12:00 A` | 0 | `h:mm AM/PM` |
| `13:00 A` | text | General |
| `10:00:30.5 A` | 0.417019675925926 | `mm:ss.0` |
| `10:00.5 A` | 0.00695023148148148 | `mm:ss.0` |
| `10 AM` / `10 PM` | 0.416666666666667 / 0.916666666666667 | `h:mm AM/PM` |
| `10:00:30 AM` | 0.417013888888889 | `h:mm:ss AM/PM` |
