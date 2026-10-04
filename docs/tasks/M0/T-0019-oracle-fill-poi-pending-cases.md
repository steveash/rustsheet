---
id: T-0019
title: "Oracle-fill the 237 pending POI cases (excel-graph, canary-verified)"
milestone: M0
lane: research
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
Excel answers for the POI cases that have no usable cached values, so that suite measures parity
instead of reporting 237 of 336 cases as pending.

## Context
`make eval` 2026-10-04: poi 336 cases, 237 pending. Since T-0013 the oracle verifies every
workbook with a canary and makes Excel save Excel-authored files, so these can now be filled
safely. Expect refusals (SHEETS/INFO, protected or corrupt packages); those stay pending.

## Acceptance criteria
- [x] `rustsheet-oracle fill -b excel-graph --suite poi` run; sidecars committed for every case the canary accepts; refusals listed with reasons in the PR.
- [x] `make eval-baseline` records the new known failures (new coverage, 0 regressions); status.md table updated.
- [x] New Excel quirks found are recorded in docs/excel-semantics/ (decision 0014); failing groups handed to the gap-triager.
- [x] `make check` green; reviewers approve.

## Out of scope
Engine fixes for what the new evidence shows.

## Outcome
- Fill: 40 excel-graph sidecars, committed under `eval/suites/poi/oracle/` (new suite key `oracle_dir`: fetched
  suites keep their sidecars in the repo, mirroring the case path). Skipped 292: 208 workbooks without formulas,
  79 Excel-authored ones whose cached values are already trusted, 5 the importer cannot read. Refused by Excel for the
  web (`unsupportedWorkbook` / `FileCorruptTryRepair`): 52575_main.xlsx, bug67784.xlsx, sample-beta.xlsx (and 47804.xlsx by the CELL("width") refusal); they stay
  pending. poi pending 237 -> 198 (the rest have no formulas to score).
- Harness bug found: sidecar JSON floats parsed up to 1 ulp off (serde_json without `float_roundtrip`); fixed with a
  regression test. ~294 would-be failures were artefacts; probes-numeric 60.19% -> 68.08%. Re-audit filed as T-0026.
- Baseline: poi known failures 647 -> 959 (new coverage only, 0 regressions).
- Web vs desktop (decision 0019): 257 cells where Excel for the web contradicts the values desktop Excel cached are
  left pending and printed as `CONFLICT` (NumberFormatApproxTests 238 lowercase-e formats, ElapsedFormatTests 10,
  47804 9); `CELL("width")` workbooks are now refused (47804). Fill discovery comes from `rustsheet-eval cases`.
- Quirks recorded: numbers.md (SIN/COS at multiples of 90°, ACOS(0.5)), new number-formats.md (TEXT: empty format,
  text section, elapsed tokens, conditions, scientific literals, General 11 chars), new references.md (3-D refs,
  backslash names, uncached external refs), criteria.md (COUNTIFS array criteria).
- Gap-triager tasks: T-0933..T-0938 (TEXT), T-0122, T-0123, T-0124 (engine), T-0207 (COUNTIFS), T-0939 (ACOS);
  T-0915 extended with the 12 poi/47813 cells. Excel-cached TEXT failures in NumberFormatTests/DateFormatTests/
  FormatConditionTests/DateFormatNumberTests are older and still need their own triage.
