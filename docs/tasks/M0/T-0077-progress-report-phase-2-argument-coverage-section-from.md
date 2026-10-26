---
id: T-0077
title: "Progress report phase 2: argument-coverage section from a per-function argument-type census in the eval report"
milestone: M0
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: [T-0081]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
The dashboard shows, per in-scope function, which input types each parameter has seen in
Excel-checked cells. The coverage that the 2026-10-25 assessment measured once becomes a daily
number.

## Context
- SPEC-0022 lists argument coverage as a later phase.
- The assessment parsed 7.3M calls with a one-off parser and classified each argument as number,
  zero, negative, fraction, numeric text, text, empty text, boolean, error, blank cell or
  range/array. It found 94% average single-argument coverage and a 44% pairwise median.
- In the repository, reuse the engine's parser through the harness: `formula_calls.rs` and the
  parsed formula the runner already has. Do not write a second formula parser.
- The catalog's parameter list (eval/catalog/functions.yaml through scope.json) needs the parameter
  names and modes in the view. Check `CatalogFunction` (deny_unknown_fields) before adding fields.

## Acceptance criteria
- [x] report.json gains `argument_coverage[]`: per in-scope function, per catalog parameter, the
      set of argument classes seen in Excel-evidence cells. Literal arguments and single-cell
      references are classified, the latter by the referenced cell's value. Unit tests cover
      each class.
- [x] A generated `arguments` block on docs/progress.md shows the functions per single-argument
      coverage band, with the method note "indicative, rule-based expected types".
- [x] The Outcome compares the average single-argument coverage with the assessment's 94%.
- [x] `make check` green; reviewers approve.

## Out of scope
- Pairwise coverage. File it as a follow-up if this lands within M.
- Generator changes to raise coverage.

## Outcome
- The engine exposes a hidden testing hook, `rustsheet_engine::call_arguments`
  (crates/rustsheet-engine/src/call_arguments.rs): each worksheet-function call of a formula with
  its arguments' shapes (number, text, boolean or error literal, a cell resolved against the
  formula's cell, an area, an omitted argument, other). It walks the engine parser's tree with an
  explicit stack; no second formula parser.
- The runner classifies the arguments of every Excel-evidence formula cell into eleven classes
  (crates/rustsheet-eval/src/argument_coverage.rs): literals as written, single-cell references by
  the evaluated cell's value, a `range` parameter's reference as a range. Cases carry the
  deduplicated classes in memory only (`CaseResult::arguments`, not serialized). report.json gains
  `argument_coverage[]`: per in-scope function, per catalog parameter, the classes seen, the
  classes its mode expects by rule (`range`: a reference; `lambda`/`name`: none; others: all
  eleven) and the function's single-argument share. Arguments map to parameters through the
  catalog's repeating groups (SUMIFS pairs, LET's and SWITCH's trailing parameter).
- eval/catalog/scope.json now carries each function's parameters (name, mode, repeating-group
  size); `CatalogFunction` reads them as `params`.
- docs/progress.md gets a generated `arguments` block (tools/src/rustsheet_tools/progress_report/
  arguments.py, SPEC-0022 row 8a): functions per coverage band, the mean and median, the ten
  lowest, with the note "indicative, rule-based expected types". The steward's next
  `make progress-report` writes it; this PR does not regenerate the page.
- `make eval` on this branch (calipers, corpus-bench and poi not fetched locally): average
  single-argument coverage **87.0%** (median 88.6%) over 499 functions with a counted parameter
  (7 have none), against the assessment's 94%. Bands: < 50% 7, 50–74% 19, 75–89% 226, 90–99% 145,
  100% 102. The gap is mostly the classifier: nested calls and operators are not classified,
  written LAMBDAs are not worksheet-function calls (LAMBDA 0%), and optional parameters count
  every class. Eval delta: none (no scoring change).
- Follow-up: T-0083 (pairwise coverage).
