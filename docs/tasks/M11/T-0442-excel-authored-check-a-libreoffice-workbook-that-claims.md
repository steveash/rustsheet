---
id: T-0442
title: "Excel-authored check: a LibreOffice workbook that claims Microsoft Excel is not Excel evidence (poi/clone_sheet.xlsx)"
milestone: M11
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The harness scores cached values only from workbooks Excel itself saved. A workbook that LibreOffice Calc
wrote, whatever its `docProps/app.xml` says, is not Excel evidence, so its cached values stop being
known failures.

## Context
- Gap triage 2026-10-20: poi/clone_sheet.xlsx holds 39 known failures in eval/baseline/poi.txt
  (Arkusz1!H6..H19, J6..J20, M6..M21) and no open task cites them. `rustsheet-eval show poi/clone_sheet.xlsx`
  (after `rustsheet-eval fetch --suite poi`) reports `source=excel-cached (Microsoft Excel)`, but the
  workbook is not Excel's: `xl/workbook.xml` has `<fileVersion appName="Calc"/>` (Excel writes
  `appName="xl"`), `docProps/app.xml` claims `<Application>Microsoft Excel</Application>` with no `AppVersion`,
  the cached numbers carry 15 significant digits (H6 `=G6/60` cached 1.66666666666667, ours 1.6666666666666667;
  J6 `=I6/6` 3.55 vs 3.5500000000000003), every formula has LibreOffice's `aca="false"` attribute, and M6
  stores `L6-(-4,3)`, a Polish decimal comma Excel could not have written (cached 0.2, ours `#ERROR!`).
  The same scan of the fetched POI tree finds one more such file, poi/56295.xlsx (it has no scored cells today).
- Code: `WorkbookSnapshot::is_excel_authored` in crates/rustsheet-eval/src/xlsx_snapshot.rs accepts any
  `Application` containing "Excel" unless `AppVersion` starts with "12." (XlsxWriter); it is used by
  `SuiteManifest::trusts_cached_values` (crates/rustsheet-eval/src/suite.rs, `require_excel_authored`), which the
  runner and `rustsheet-eval cases` (the oracle fill's case list) share, and by runner.rs.
  The snapshot does not read `fileVersion` yet.
- Excel is the oracle (CLAUDE.md directive 1): these 39 values are not Excel's, so they are not evidence for
  or against the engine. Once rejected, the case has no expectations and is pending (no oracle sidecar exists
  under eval/suites/poi/oracle/); the baseline drops a pending case, which is not a regression
  (crates/rustsheet-eval/src/baseline.rs).

## Acceptance criteria
- [x] The snapshot records `fileVersion`'s `appName`, and `is_excel_authored` is false when it is present
      and not `xl` (unit test with a `Calc` workbook claiming Microsoft Excel; the existing calibrate test keeps passing).
- [x] The 39 poi/clone_sheet.xlsx F lines and its C line leave eval/baseline/poi.txt (`make eval-baseline`),
      with no other change to poi.txt, gen-*, ironcalc-calc, fixtures or probes baselines.
- [x] docs/research/test-suites-and-oracle.md (or wherever the excel-authored rule is described; `rg
      is_excel_authored docs`) states the `fileVersion` rule and cites clone_sheet.xlsx and 56295.xlsx.
- [x] `make check` green; reviewers approve.

## Out of scope
- Oracle-filling clone_sheet.xlsx from excel-graph (after this change `rustsheet-eval cases` lists it as
  pending, so a later fill can try; Excel may refuse the `(-4,3)` formula).
- corpus-bench, whose expectations come from excel-graph sidecars (decision 0030), not cached values.

## Outcome
`WorkbookSnapshot` now records `xl/workbook.xml` `fileVersion appName` (`file_app_name`, serde default) and
`is_excel_authored` is false when it is present and not `xl`; unit test with a `Calc` workbook claiming
Microsoft Excel (`a_calc_workbook_claiming_excel_is_not_excel_authored`). The rule is in
docs/research/test-suites-and-oracle.md. Eval delta: poi known failures 181 -> 142 (39 F lines and the C line of
poi/clone_sheet.xlsx leave eval/baseline/poi.txt, which has no other change; the case is now pending, no cell
regressed). Updated with `rustsheet-eval update-baseline --suite poi` because `make eval-baseline` also bakes in a
probes drift that is already on main (probes 1632 -> 1633, 2 new-coverage failures, same with this change stashed);
follow-up filed for it if the steward does not already have one.

## Outcome
Implemented by #1369 (fileVersion appName check in `is_excel_authored`, unit test, docs rule, poi baseline drop). This PR only closes the task file; no code or eval change (poi/clone_sheet.xlsx has no baseline lines on main).
