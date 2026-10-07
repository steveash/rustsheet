---
id: T-0223
title: "Port IronCalc MAX/MIN fixes (named ranges, LAMBDA/LET args) with MAX_MIN.xlsx evidence"
milestone: M2
lane: functions-math
status: in-progress
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
- [ ] `d7b09c33` and the MAX/MIN/LAMBDA parts of `c4641f0f` ported, each commit with `Ported-From: ironcalc@<sha>`.
- [ ] `MAX_MIN.xlsx` and `MIN_MAX_extras.xlsx` added to `crates/rustsheet-xlsx/tests/calc_tests`; their cells pass or are baselined as new coverage with reasons; 0 eval regressions.
- [ ] Any Excel quirk the workbooks show recorded in `docs/excel-semantics/`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ISREF (T-0940) and arithmetic error broadcasting (T-0129).
