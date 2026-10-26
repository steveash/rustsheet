---
id: T-0077
title: "Progress report phase 2: argument-coverage section from a per-function argument-type census in the eval report"
milestone: M0
lane: harness
status: in-progress
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
- [ ] report.json gains `argument_coverage[]`: per in-scope function, per catalog parameter, the
      set of argument classes seen in Excel-evidence cells. Literal arguments and single-cell
      references are classified, the latter by the referenced cell's value. Unit tests cover
      each class.
- [ ] A generated `arguments` block on docs/progress.md shows the functions per single-argument
      coverage band, with the method note "indicative, rule-based expected types".
- [ ] The Outcome compares the average single-argument coverage with the assessment's 94%.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pairwise coverage. File it as a follow-up if this lands within M.
- Generator changes to raise coverage.
