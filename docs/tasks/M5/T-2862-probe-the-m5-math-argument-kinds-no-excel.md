---
id: T-2862
title: "Probe the M5 math argument kinds no Excel cell covers yet: empty text and an error in 25 value parameters"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M5 math functions below has at least one Excel-backed cell for each argument kind the
report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 7bd9090e, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool
(tools/src/rustsheet_tools/gen/generator.py `NUMBER_POOL`) has no empty text or error literal (T-2866), so
generated suites never reach most of these. `array`-mode parameters are not listed: their coverage is not
classified by kind.
These M5 math parameters have no Excel cell of the listed kinds in any suite (highest catalog prior first):
- SUBTOTAL.function_num, AGGREGATE.function_num, AGGREGATE.options: empty text, error.
- MOD.number: empty text; MOD.divisor: empty text, error.
- PRODUCT.number, SUMSQ.number (direct arguments): empty text.
- TRUNC.number, TRUNC.num_digits: a multi-cell range (implicit intersection or `#VALUE!`).
- CEILING.MATH.mode, FLOOR.MATH.mode: empty text, error.
- FACT.number, FACTDOUBLE.number: empty text, error.
- LOG.base, ODD.number, SQRTPI.number, MUNIT.dimension, BASE.number, ROMAN.number: empty text.
- ATAN2.x_num, ATAN2.y_num: empty text.
- COMBIN.number, COMBINA.number: empty text; COMBIN.number_chosen, COMBINA.number_chosen: empty text, error.
- QUOTIENT.numerator: empty text; QUOTIENT.denominator: empty text, error.
- ECMA.CEILING.number: empty text, error; ECMA.CEILING.significance: empty text, error, range.
- ROMAN.form: empty text, error.
- SERIESSUM.x: empty text, error; SERIESSUM.n: empty text; SERIESSUM.m: numeric text, empty text, error.
- PERCENTOF.data_subset, PERCENTOF.data_all: empty text, error.
- RANDBETWEEN.bottom/top are left out: the function is volatile and the harness skips its cells.

Write one probe case, probes/m5-math-argument-kind-gaps (excel-probe skill, excel-graph, tag M5), with each kind in two forms where
they differ: a literal (`""`, `NA()`, `#DIV/0!`) and a cell holding it (a formula cell `=""`, `=NA()`). Give every
other argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued
cell in `ERROR.TYPE` too.

## Acceptance criteria
- [ ] probes/m5-math-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/coercion.md (or math.md) citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- M2 functions (T-2861), M5 text (T-2863), M6 (T-2864, T-2865); generator pools (T-2866).
- `array`-mode parameters (SUMPRODUCT, MMULT, MDETERM, MINVERSE, SUMX2MY2, ...).
