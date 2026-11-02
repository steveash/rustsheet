---
id: T-3807
title: "Criteria: probe and match a blank or omitted criterion over padded and formatted zero text, and the percent operand's 15-digit cut order"
milestone: M2
lane: functions-criteria
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Two rules that criteria.md leaves open for the M2 criteria functions are settled by Excel cells and matched: (1)
what a blank-cell or omitted criterion matches among padded zero texts (`" 0"`, `"0 "`, `" 0 "`, NBSP-padded `0`,
`"+0"`, `"0.0"`, `"-0"`, `"0E0"`, `"00"`), and (2) whether a percent operand is cut to 15 significant digits before
or after its division by 100.

## Context
- Gap triage 2026-11-02 (`make eval` on main 2290b8769). Every failing Excel cell of COUNTIF, COUNTIFS, SUMIF,
  SUMIFS, AVERAGEIF(S), MINIFS and MAXIFS already has an open owner (`make unowned-off ALL=1` lists none; the
  t3334-criteria-operand-forms cells belong to T-3404 and T-3637), and their argument-class coverage is 1.0. What is
  left are two rules criteria.md states with no Excel cell behind them. This task fills those evidence gaps
  (decision 0031). It owns no failing cell today.
- Rule (1): criteria.md "A blank-cell or omitted criterion is the criterion 0" says padded text such as `" 0"` is
  not probed and the engine treats it as `0` does. Evidence so far: probes/t3138-blank-criterion D1:H9 (blank, `=""`,
  0, `=-0`, tiny and subnormal numbers, FALSE, "0", " ", "x", #N/A) and probes/t3151-number-criterion-formatted-text
  C2, C3, C8 ("0%", "$0", "0:00", "12:00 AM" and "0" match). The number criterion 0 itself matches ASCII-space-padded
  number text and not NBSP-padded text (criteria.md "Numeric criteria against text cells and non-ASCII spaces",
  probes/criteria-padded-text C1:C14). No cell shows that the blank criterion does the same. Code:
  `indexed_criterion` in crates/rustsheet-engine/src/functions/criteria.rs turns `EmptyCell`/`EmptyArg` into
  `IndexedCriterion::Number(Eq, 0.0)` for the walk and the indexed *IFS path. If Excel differs on any padded form,
  the fix goes there.
- Rule (2): criteria.md "Criterion operands and text cells read numbers with VALUE's grammar", the percent-tie
  bullet (probes/t3334-criteria-operand-forms rows 65-84): the operand is truncated to 15 significant digits, but
  "whether the cut comes before or after the division by 100 is not separable with these operands ... it stays
  open". Settling it means finding operands where the two orders give different doubles. Search with a 50-digit
  reference (mpmath, as tools/ does for Gate A) for `x%` operands with 16-17 significant digits where
  `trunc15(x)/100` and `trunc15(x/100)` round to different doubles. Then probe at least four such operands under
  plain, `=`, `<` and `>` against number cells that tell the two apart, with `VALUE` of each operand beside it.
  Code: `criterion_text_number` / `parse_criterion_number` (criteria.rs) and VALUE's reader.
- Probe both with the excel-probe skill (excel-graph). Use power-of-two SUMIF weights as t3138 and t3151 do, cover
  COUNTIF, SUMIF, SUMIFS and COUNTIFS (the walk and the 80-cell indexed path, as t3138 does), and take criteria
  both from a cell and from a literal.
- Tier: opus. The Excel evidence does not exist yet, and rule (2) needs the operand search before a probe can tell
  the orders apart.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes (excel-graph sidecar) covers the nine padded zero texts above against a
      blank-cell and an omitted criterion, in COUNTIF, SUMIF, SUMIFS and COUNTIFS, on a short range and on an
      80-cell range.
- [ ] The same or a second probe case holds at least four percent operands that the two cut orders read as
      different doubles (the PR lists each operand with both doubles from the 50-digit search), with VALUE of each.
- [ ] Every probed cell passes bit-exact: no new line in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions).
- [ ] criteria.md drops "padded text such as `" 0"` is not probed" and "it stays open" and replaces them with the
      probed rules, citing the probe cells, in the same PR. Any engine change in criteria.rs has a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-en-US decimal and group separators (criteria.md: out of scope).
- D-function criteria (there a blank criterion cell means "match all", a different rule).
- VALUE's own `1e2$` and NBSP rules (T-3404).
