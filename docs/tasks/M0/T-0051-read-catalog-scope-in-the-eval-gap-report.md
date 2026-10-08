---
id: T-0051
title: "Read catalog scope in the eval gap report and evidence coverage"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-0040]
---

## Goal
`make eval`'s gap report and the evidence-coverage section (T-0020) count only in-scope
functions, so Phase 1 parity (decision 0027) is measured against the catalog's denominator.

## Context
Part (b) of T-0040. T-0040 made `eval/catalog/functions.yaml` list every Microsoft 365 Excel
function (526) with `scope: in | out` (an entry without `scope` is in), `reason` for `out`, and
`oracle: none` where the excel-graph oracle cannot return any value (INFO and SHEETS today, decision 0039). The eval crate
(crates/rustsheet-eval) has no YAML reader today, so the harness cannot see `scope`.
- Design choice left to this task: add a YAML dependency (serde_yaml or a maintained
  successor; check `cargo deny` and decision records on dependencies) to read the catalog
  directly, or have tools/ write a generated JSON view of the catalog that the harness reads
  (then a test must keep the view in step with the YAML). Record the choice in a decision.
- The report should show out-of-scope functions (and `oracle: none` ones) separately rather
  than drop them silently, and list catalog functions with no evidence at all as gaps.

## Acceptance criteria
- [x] The gap report (eval/runs/latest/report.md) excludes `scope: out` functions from parity
  counts and lists them in their own section with their reason.
- [x] The "Excel evidence per function" section (T-0020) reads scope the same way, flags in-scope
  catalog functions with zero evidence, and marks `oracle: none` functions as needing another
  evidence source.
- [x] A test fails if the harness's view of the catalog drifts from eval/catalog/functions.yaml.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing any catalog entry's scope (that needs probe evidence or a decision).

## Outcome
- tools/ writes `eval/catalog/scope.json` (`cd tools && uv run rustsheet-catalog-view`), a
  one-line-per-function view of the catalog's name, milestone, scope, reason and oracle. The
  harness reads it through the new `catalog` module (decision 0040: no YAML crate, since
  serde_yaml is unmaintained). `tools/tests/test_catalog_view.py` fails if the view drifts.
- report.md: `scope: out` functions leave "Functions with failing cells", the examples and the
  evidence table. They get an "Out of scope (21)" section with cells checked, failing and reason.
  The evidence section adds "In-scope catalog functions with no Excel evidence (gaps)" (3 on
  2026-10-07: RAND, RANDARRAY, RANDBETWEEN) and "Need an evidence source other than the oracle"
  (INFO, SHEETS). Suite cell counts and baselines are unchanged.
- Eval delta: none (report only; 0 regressions, no baseline change).
