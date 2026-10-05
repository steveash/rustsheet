---
id: T-0227
title: "COUNTIF: a bare = criterion does not match an empty-string cell (1 probes known failure)"
milestone: M2
lane: functions-criteria
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
In the standard criteria flavor (COUNTIF/SUMIF/AVERAGEIF and the *IFS family), the criterion
`"="` with nothing after it no longer matches a cell that holds the empty string, as in Excel.
`""` (no operator) still matches it.

## Context
docs/excel-semantics/criteria.md "\"=\" does not match an empty-string cell" (VERIFIED) has no
implementation yet. Evidence: `probes/criteria-grammar` over A1:A6 = "10", 10, "abc", "" (an
empty-string cell), TRUE, "#N/A":
- Sheet1!C3 `=COUNTIF(A1:A6,"")` is 1 (passes today).
- Sheet1!C4 `=COUNTIF(A1:A6,"=")` is 0 in Excel; the engine gives 1 (known failure).

Code: `build_criteria_with` in `crates/rustsheet-engine/src/functions/criteria.rs`, the final
`else` branch. It strips `=` and then compares with `result_is_equal_to_string(x, "")`, so `"="`
and `""` behave the same. For the Standard flavor with `explicit_equals` and an empty operand,
match nothing that is an empty string.

## Acceptance criteria
- [ ] `F	probes/criteria-grammar.case.json	Sheet1!C4` disappears from eval/baseline/probes.txt
      (`make eval-baseline`), with no new F lines in any baseline.
- [ ] Unit tests in the criteria tests: `"="` does not match `CalcResult::String("")`; `""` still
      does.
- [ ] criteria.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Whether `"="` matches a truly blank cell: there is no evidence yet (T-0011 probes it). Keep
  today's blank-cell behavior unchanged.
- D-function (`CriteriaFlavor::Database`) criteria: no evidence for `"="` there. Leave it unchanged.
