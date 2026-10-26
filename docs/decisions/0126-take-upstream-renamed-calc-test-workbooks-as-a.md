# 0126 — Take upstream renamed calc test workbooks as a rename, not a test deletion

- **Status:** accepted (2026-10-26, T-2634).
- **Context:** Upstream IronCalc f04cbbed/9cfbdfce replaced
  `xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/ROUND.xlsx` with an extended, Excel-saved
  `ROUND_ROUNDUP_ROUNDDOWN.xlsx` (docProps/app.xml `Microsoft Excel` 16.0300). Taking it deletes
  our `crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/ROUND.xlsx`, so
  `scripts/check_governance.py` fails CI `hygiene` ("test files deleted"). Comparing
  `rustsheet-eval snapshot` of both files, formula by formula with references resolved to their
  inputs: 126 of the old file's 127 formula cells reappear in the new one with the same cached
  value (the new file has 307). The one exception is the old `issues!A2`,
  `=ROUND(1.05*(0.0284+0.0046)-0.0284,4)` = 0.0063, whose sheet upstream dropped.
- **Decision:** An upstream rename or replacement of an Excel-authored calc test workbook is taken
  as a rename: the old file is deleted in the same PR that adds its successor, provided the PR
  shows that every old formula cell's evidence survives, in the successor or in an eval case added
  by that PR. Here the dropped `issues!A2` is kept as `probes/round-float-noise-carry` (excel-graph
  returns 0.0063, the same as the desktop cached value).
- **Consequences:** No evidence is lost, and the ironcalc-calc suite tracks upstream's file names,
  so later upstream re-saves port as plain modifications. The governance check is unchanged: each
  such PR still needs a decision record (this one or a later one citing it) and reviewer sign-off
  on the evidence comparison. A rename that drops cells without carrying them over is a test
  deletion and needs its own decision.
