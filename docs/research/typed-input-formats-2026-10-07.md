# Typed-input number formats, excel-graph 2026-10-07 (T-0280)

Raw `ENTERED` stderr lines of `rustsheet-oracle fill -b excel-graph --suite probes`, the run that
wrote `eval/suites/probes/cases/typed-paren-number-input.oracle.json`
(`input_sha256` 64dfcdfbf727eacdd081baf17ccf158a32731632a109e6b2efa2b9b54492f4f9). Each line is
the Graph range PATCH response after writing the text with `values`: the value and the
`numberFormat` Excel for the web picked for that typed input. The sidecar scores only the values
(column B and C formulas); these formats are the evidence for docs/excel-semantics/coercion.md
("Parenthesised number text is negative") and T-0283.

```
ENTERED Sheet1!A1 '(1)': values=[[-1]] numberFormat=[['General']]
ENTERED Sheet1!A2 '(1,000)': values=[[-1000]] numberFormat=[['#,##0']]
ENTERED Sheet1!A3 '( 1 )': values=[[-1]] numberFormat=[['General']]
ENTERED Sheet1!A4 '(-1)': values=[['(-1)']] numberFormat=[['General']]
ENTERED Sheet1!A5 '($1)': values=[[-1]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A6 '(1%)': values=[[-0.01]] numberFormat=[['0%']]
ENTERED Sheet1!A7 '1': values=[[1]] numberFormat=[['General']]
ENTERED Sheet1!A8 '-1': values=[[-1]] numberFormat=[['General']]
ENTERED Sheet1!A9 '1,000': values=[[1000]] numberFormat=[['#,##0']]
ENTERED Sheet1!A10 '$1': values=[[1]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
ENTERED Sheet1!A11 '1%': values=[[0.01]] numberFormat=[['0%']]
ENTERED Sheet1!A12 '($1,000)': values=[[-1000]] numberFormat=[['$#,##0_);[Red]($#,##0)']]
```

An earlier `recalc` of the same cells (one docProps property instead of one per cell, so a
different input sha) logged identical lines.
