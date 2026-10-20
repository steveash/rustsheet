# 0078 — sort-error-args and date-math-lift-driver-edges G43 count toward their subject's milestone

- **Status:** accepted (2026-10-19, T-1971). Amends decisions 0051 and 0058 (the list of
  `eval/milestones/probe-milestones.tsv` lines); the rule of 0051 is unchanged.
- **Context:** At origin/main ca5c988f, `make eval` ("Probe milestones (decision 0051)") put 37
  probe known-failure cells in M0-M2. Two of those cases are there only because of their case.json
  `M2` tag, although their subject is a later-milestone function:
  - `probes/sort-error-args` (23 failing cells: H57:J59, H61, H65, H69:J71, H97:H99) was written
    by the M7 task T-1850 and tests SORT and SORTBY only (catalog M7 in `eval/catalog/scope.json`).
    Its only other call is `NA()`, which builds an argument and so counts as an observer (0058).
    No M2 function is a subject, so 0051 item 2's exception does not apply. Its twin
    `sort-omitted-args` was mapped to M7 by 0058 on the same grounds. T-1902 (M7) owns the fix.
  - `probes/date-math-lift-driver-edges` Sheet1!G43 is `=INDEX(DATEDIF(46000,45000,{"d";#N/A}),2)`:
    Excel gives #N/A, we give #NUM!. Its subject is DATEDIF (catalog M6); INDEX only observes
    (one of 0051's wrappers). The rest of the case tests the SPEC-0011 lift driver over M2
    functions and keeps its tag under the exception. T-1430 (M6) owns the fix.
- **Decision:** add the case line `probes/sort-error-args.case.json	M7	SORT, SORTBY` and, as
  0058 does for cells ("cell lines, by subject"), the cell line
  `probes/date-math-lift-driver-edges.case.json	Sheet1!G43	M6	DATEDIF`. No other line changes.
  The other M0-M2 failing cases (legacy `@`, SUBTOTAL/TAKE static analysis, sheet ops) test M2
  engine features, and M2 engine tasks own them; they stay M2.
- **Consequences:** M0-M2 probe known failures drop by 24 (37 → 13); M6 rises by 1 and M7 by 23.
  Both failing items keep their open task in their own milestone; nothing leaves the reports and
  eval/baseline does not change.
