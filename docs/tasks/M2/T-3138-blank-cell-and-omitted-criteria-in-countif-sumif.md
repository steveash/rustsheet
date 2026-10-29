---
id: T-3138
title: "Blank-cell and omitted criteria in COUNTIF, SUMIF and the IFS family: probe what they match (0, tiny numbers, blanks, empty text, FALSE) and match it"
milestone: M2
lane: functions-criteria
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A criterion that is a blank cell (`COUNTIF(A1:A9,C1)` with C1 empty) or an omitted argument (`COUNTIF(A1:A9,)`)
matches the cells Excel's does, in COUNTIF, SUMIF, AVERAGEIF, COUNTIFS, SUMIFS, AVERAGEIFS, MINIFS and MAXIFS, on
the plain and the indexed (`ifs_cache`) paths. The evidence comes from a committed oracle case, and criteria.md
records the rule as VERIFIED.

## Context
- The engine compiles such a criterion to `result_is_equal_to_empty` (crates/rustsheet-engine/src/functions/criteria.rs,
  the `CalcResult::EmptyCell | CalcResult::EmptyArg` arm of the plain-value criterion). It matches only a number
  cell that is exactly `0.0`: never a blank cell, an empty-string cell, `FALSE` or `"0"` text. The indexed
  *IFS path keys it as `CriterionKey::Empty` (functions/statistical/ifs_cache.rs). No probe covers either form.
- The docs disagree with the code. numbers.md "Criteria equality" says: "The blank-cell criterion path
  (`result_is_equal_to_empty`) uses the same rule but is inferred, not probed". That rule is the 15-digit
  comparison under which `COUNTIF(r,0)` counts a cell holding 1E-17 (probes/tiny-vs-zero-criteria A1). The code
  instead compares exactly (`*f == 0.0`). So for a cell holding 1E-17, the code and the doc disagree, and Excel has
  never been asked.
- What is probed nearby: the text criteria `"="` (blank cells only) and `""` (blank and empty-string cells).
  See criteria.md "\"=\" matches blank cells, not an empty-string cell", probes/criteria-gaps,
  probes/criteria-grammar and probes/criteria-ifs-bare-equals (T-0011, T-0246). A blank criterion *cell* could
  behave like `""`, like `"="`, like the number 0, or like none of them. A blank cell criterion is common in real
  sheets: a filter cell left empty.
- Why opus: the probe must tell four candidate models apart, and two of them depend on the 15-digit rule:
  (a) number 0 compared exactly (today's code); (b) number 0 at 15 digits (the doc); (c) `""`; (d) `"="`.
  Pick data cells on which the models disagree: blank, `=""`, 0, `-0` (`=-0`), 1E-17, -1E-300, 2^-1074 as a
  subnormal (`=2^-1074`), 5E-16, `FALSE`, `"0"`, `" "`, `"x"`, `#N/A`. Use SUMIF/SUMIFS weights that are distinct
  powers of two, so that one sum tells exactly which cells matched. Read each form through every function in the
  title. Use several criterion sources: a blank cell, an omitted argument, `IF(FALSE,1,)` (empty, if Excel passes
  it), a blank cell on another sheet, and a blank cell picked by INDEX. AVERAGEIF needs a case with no match, to
  show `#DIV/0!` against a 0 average.
- Method: the `excel-probe` skill (excel-graph), one case `probes/t3138-blank-criterion`. The *IFS cells must go
  through the indexed cache as well. ifs_cache.rs caches only ranges of at least `MIN_CACHED_CELLS` (64) cells, so
  repeat the data block in a criteria range of 64 cells or more (pad it with distinct numbers no model matches)
  and read it from several *IFS cells. Keep a short-range copy for the plain path.

## Acceptance criteria
- [ ] `probes/t3138-blank-criterion` (milestone tag M2) is committed with its excel-graph sidecar. It holds the data
      cells and criterion sources above, read through COUNTIF, SUMIF (with and without sum_range), AVERAGEIF,
      COUNTIFS, SUMIFS, AVERAGEIFS, MINIFS and MAXIFS, with power-of-two weights.
- [ ] Every cell of the case passes. The engine's blank/omitted criterion goes through one matcher, shared by the
      plain path (`build_criteria_with`) and the indexed path (`CriterionKey::Empty` in ifs_cache.rs). Reuse
      `numbers_equal` or the empty-text matchers that already exist; do not add a parallel comparison.
- [ ] criteria.md has a VERIFIED section "A blank-cell or omitted criterion", citing cells. numbers.md "Criteria
      equality" no longer calls the blank path inferred, and states the comparison Excel uses there.
- [ ] Unit tests in criteria.rs cover a blank criterion against a blank cell, an empty-string cell and a 1E-17 cell.
      probes/tiny-vs-zero-criteria, criteria-gaps, criteria-grammar and criteria-ifs-bare-equals, and the gen-m2
      criteria cells, show 0 regressions (`make eval-check`). `make eval-baseline` is run.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The D-functions' blank criteria-range cells (database.rs), which follow their own criteria-range rules.
- Formatted-number criterion operands (T-3136) and ordering operators (T-3103).
- An array criterion holding an empty element (`{""}`) and lifted criteria (criteria.md "*IF(S) lift array
  criteria"). File a follow-up if the evidence suggests they differ.
