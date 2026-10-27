---
id: T-0470
title: "Element-wise functions in whole-column aggregates take the tailed path (SPEC-0020)"
milestone: M11
lane: perf
status: dropped
priority: P2
size: S
tier: opus
depends_on: [T-0469]
---

## Goal
The element-wise functions that real workbooks use inside whole-column aggregates (ISNUMBER, ISBLANK,
ISERROR, NOT, LEN, IFERROR, ...) take SPEC-0020's tailed path too, with bit-identical results.

## Context
SPEC-0020 "Which expressions are tailed": each function is added only with an argument whose cells
past the used area read the same in every row. Make the list from a scan of the fetched suites
(corpus-bench, calipers) for aggregates over whole-column expressions.

## Acceptance criteria
- [ ] The scan and the resulting list in the Outcome; each listed function admitted by
      `tailed_leaves` (tailed_array.rs; the two evaluations already run it through the ordinary
      evaluator, so only the static check changes).
- [ ] Equivalence tests per function (as T-0469's).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Position- or call-dependent functions (ROW, COLUMN, SEQUENCE, RAND, INDEX, OFFSET).

## Outcome (dropped, 2026-10-27, perf)
The scan found nothing to admit.
- **What was scanned:** every `<f>` formula in the fetched corpus-bench, calipers and poi
  workbooks (2026-10-27). Formulas were kept when they call SUM, MAX, MIN, AVERAGE, COUNT,
  COUNTA, PRODUCT, SUMPRODUCT, AND, OR, MEDIAN, LARGE, SMALL, STDEV, TEXTJOIN or CONCAT with
  a whole-column operand (`X:X`) in an argument.
- **Functions applied to a whole column inside those arguments:**
  - IF, in `MAX(IF(...))` (2 files, 555 uses) and `MIN(IF(...))` (1 file, 150 uses). T-0469 already
    tails it.
  - UNIQUE and IF under TEXTJOIN (1 file, 27 uses). UNIQUE is not element-wise.
  - COUNTIF and COUNTIFS under SUMPRODUCT (1 file, 3 uses). They are criteria functions, which
    already clip.
- **Operator-only expressions:** the one other form was `SUMPRODUCT(A:A=x)*...` in corpus-bench
  55049, which T-0471 now tails.
- **Result:** no workbook uses ISNUMBER, ISBLANK, ISERROR, NOT, LEN, IFERROR or any other
  element-wise function inside a whole-column aggregate. Admitting them would add code and test
  surface that no evidence asks for.
- **If a case appears:** a future workbook with such a pattern (a corpus timeout, or a gap
  report) should get a task naming that function and that file.

