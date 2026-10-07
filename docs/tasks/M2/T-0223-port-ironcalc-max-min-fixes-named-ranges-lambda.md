---
id: T-0223
title: "Port IronCalc MAX/MIN fixes (named ranges, LAMBDA/LET args) with MAX_MIN.xlsx evidence"
milestone: M2
lane: functions-math
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
MAX and MIN (and their LET/LAMBDA-bound and FILTER-fed arguments) give Excel's results on the
upstream Excel-authored workbooks `MAX_MIN.xlsx` and `MIN_MAX_extras.xlsx` (named ranges).

## Context
Upstream IronCalc fixed these after our last triage (steward 2026-10-05, `scripts/upstream-log.sh`):
`d7b09c33` (MAX/MIN argument handling in `math_and_trigonometry/mathematical.rs`; LET/LAMBDA
variables keep a bound reference via a new `variable_value` in `model.rs`; FILTER returning a single
value returns the scalar, not a 1x1 array), the MAX/MIN and LAMBDA parts of `c4641f0f`, and the
test workbook `16987342` (`MIN_MAX_extras.xlsx`). Lane `functions-math` by its main module; the
`model.rs`/`spill_functions.rs` hunks ride along. Use the T-0911 patch recipe (`format-patch`, rewrite
paths `base/`→`crates/rustsheet-engine/`, `xlsx/`→`crates/rustsheet-xlsx/`, `git apply --reject`).
The model no longer has a lifetime (T-0101): `impl<'a> Model<'a>` hunks need hand-editing.
Reuse the shared aggregate-argument iterator (T-0200) instead of upstream's local loops if they
duplicate it (directive 3). If T-0940 finds that the ISREF/FILTER fix `29ee0601` depends on the
FILTER hunk of `d7b09c33`, port `29ee0601` (and its `test_spill_functions.rs` case) here.

## Acceptance criteria
- [x] `d7b09c33` and the MAX/MIN/LAMBDA parts of `c4641f0f` ported, each commit with `Ported-From: ironcalc@<sha>`.
- [x] `MAX_MIN.xlsx` and `MIN_MAX_extras.xlsx` added to `crates/rustsheet-xlsx/tests/calc_tests`; their cells pass or are baselined as new coverage with reasons; 0 eval regressions.
- [x] Any Excel quirk the workbooks show recorded in `docs/excel-semantics/`.
- [x] `make check` green; reviewers approve.

## Out of scope
- ISREF (T-0940) and arithmetic error broadcasting (T-0129).

## Outcome
- Ported d7b09c33 and the MAX/MIN/LAMBDA/ISREF parts of c4641f0f onto our engine. Our MIN/MAX
  already had upstream's value coercion (T-0217, `fold_min_max`), so `mathematical.rs` is unchanged.
  Instead, `evaluate_aggregate_arg` now routes defined names, LET/LAMBDA variables and `@` through
  `evaluate_node_with_reference`. That function resolves a single-cell name to its reference and
  returns a variable's binding (`variable_value`). LET/LAMBDA bind references, ISREF sees a
  variable's reference, and FILTER returns a scalar `if_empty` as itself.
- Added MAX_MIN.xlsx (55 cells) and MIN_MAX_extras.xlsx (21 cells) to calc_tests; all pass.
  Upstream's unit tests are ported (test_fn_max_min.rs, plus additions to test_fn_lambda.rs and
  test_spill_functions.rs).
- New probe probes/min-max-reference-forms (25 cells, excel-graph) confirms `@`, LET/LAMBDA (also in
  SUBTOTAL, SUMIF, COUNTA, ROWS and `x:A3`) and FILTER's scalar or one-cell if_empty. 24 cells pass; D8 (`SUM(LET(x,K1,x))`) is new coverage, baselined and
  filed as T-1106. The finding is recorded in docs/excel-semantics/references.md.
- Eval delta: ironcalc-calc +76 cells passing (2 new cases), probes +24 passing / +1 new-coverage
  failure, 0 regressions. T-0963's code landed here; it now waits on T-0965 for its workbook.
  Follow-ups: T-1106 (LET result stays a reference), T-0410 (defined-name lookup by key, perf).
