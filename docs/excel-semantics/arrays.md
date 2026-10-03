
## ROW / COLUMN of a multi-cell reference (T-0911)
Evidence: Excel-authored `ROW_COLUM.xlsx` (ironcalc-calc suite, cached values).
`ROW(ref)` of a reference spanning several rows returns a column vector of row numbers; `COLUMN(ref)`
returns a row vector. A single-row (ROW) / single-column (COLUMN) reference returns a scalar. In a
scalar position the call is wrapped in implicit intersection (`@ROW(A1:A10)`), taking the first element.
