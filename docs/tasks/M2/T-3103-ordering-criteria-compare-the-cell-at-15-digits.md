---
id: T-3103
title: "Ordering criteria (<, <=, >, >=) compare the cell at 15 digits, as = and <> do"
milestone: M2
lane: functions-criteria
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
COUNTIF, SUMIF, COUNTIFS, SUMIFS (and every other consumer of the shared criteria grammar) match a number cell
against `"<n"`, `"<=n"`, `">n"` and `">=n"` by its 15-digit value, as Excel does and as `"=n"`/`"<>n"` already do.

## Context
- Evidence: probes/t3072-criteria-near-equal-order (T-3084); docs/excel-semantics/numbers.md "Criteria ordering
  operators compare at 15 digits too" and criteria.md "Numeric `<`, `<=`, `>`, `>=` criteria compare the 15-digit
  value of the cell".
- Failing cells (known failures in eval/baseline/probes.txt): rows 1-5 and 7-10 of that case, where the cell is
  1+k·2^-52 (k = 1, 2, 3, 8, 22) or 1-k·2^-53 (k = 1-4) and prints as 1. Above 1 the `"<="` and `">"` columns are off
  (F, G COUNTIF; L, M SUMIF; R, S COUNTIFS; X, Y SUMIFS); below 1 the `"<"` and `">="` columns (E, H; K, N; Q, T; W, Z).
- Code: crates/rustsheet-engine/src/functions/criteria.rs `result_is_less_than_number`,
  `result_is_less_or_equal_than_number`, `result_is_greater_than_number`,
  `result_is_greater_or_equal_than_number` compare raw doubles; use `cmp_excel_precision` (number_format.rs), as
  `numbers_equal` does, so the six matchers share one comparison.

## Acceptance criteria
- [ ] Every cell of probes/t3072-criteria-near-equal-order passes (`make eval-baseline` drops its known failures),
      with no regressions elsewhere (D-functions and AVERAGEIFS/MINIFS/MAXIFS share the matchers).
- [ ] numbers.md and criteria.md entries cite the implementation (Status line).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Text and boolean ordering criteria; approximate lookups.
