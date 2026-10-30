---
id: T-3277
title: "report.json argument_coverage lists each function's unseen argument-class pairs, so the generator can target them"
milestone: M2
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Each `argument_coverage[]` row of report.json also lists the argument-class pairs its `pairwise` share
counts as expected but no Excel-evidence call has passed, so a generator policy (T-3278) can target
exactly those pairs without re-implementing the classifier.

## Context
- `summarize` in crates/rustsheet-eval/src/argument_coverage.rs already builds, per function, the set of
  covered pairs (`pairs`: two (parameter index, `ArgClass`) slots of one call, parameters in order) and
  counts the expected ones (`pairs_expected`: every two parameters, every expected class of each, from
  `expected_classes`). It only reports the share (`FunctionArguments::pairwise`, T-0083).
- The missing pairs are the expected pairs not in `pairs`. Today only the share reaches report.json, so
  the dashboard (tools/src/rustsheet_tools/progress_report/arguments.py) shows e.g. MID 0.399, LEFT 0.43,
  INDEX 0.559 (eval 2026-10-30) but nothing says which pairs are missing.
- One definition per concept: compute the list inside `summarize` from the same sets; do not add a second
  classifier or a Python re-count.
- Shape (a suggestion; keep `serde(default)` so older reports still deserialize): a field
  `unseen_pairs: Vec<UnseenPair>` on `FunctionArguments`, with `UnseenPair { params: [String; 2],
  classes: [ArgClass; 2] }` (catalog parameter names, classes serialized as report.json already writes
  them in `seen`, e.g. `"empty_text"`), sorted by parameter index then class order of `ALL_CLASSES`.
  Empty when `pairwise` is `None` or 1.0.

## Acceptance criteria
- [ ] report.json `argument_coverage[]` rows carry the new list; for every row,
      `len(unseen_pairs)` equals `pairs_expected - pairs_covered` (a unit test in
      crates/rustsheet-eval/src/argument_coverage/tests.rs over a two-parameter catalog function and
      two calls asserts the exact list, and another asserts it is empty when every pair is seen).
- [ ] `docs/architecture/eval-harness.md` (or the report.json field list the harness doc keeps) names the
      field in one sentence.
- [ ] The progress report still parses report.json (`make py-check` green); `make check` green;
      reviewers approve.

## Out of scope
- Any generator change (T-3278) or dashboard view of the list.
- Changing what counts as a pair or a class.
