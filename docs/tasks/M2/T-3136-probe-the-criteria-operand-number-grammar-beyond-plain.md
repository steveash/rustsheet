---
id: T-3136
title: "Probe the criteria operand number grammar beyond plain numbers and dates (percent, currency, grouping, time, fraction, parentheses) on the criterion and the text-cell side"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence settles which text forms a *IF(S) and D-function criterion reads as a number. That covers
`"50%"`, `">40%"`, `"$1"`, `"1,000"`, `">=$1,000"`, `"12:00"`, `"1 1/2"`, `"(1)"` and `"1/2/2020 12:00"`, and which
text cells such a numeric criterion matches. docs/excel-semantics/criteria.md states the rule as VERIFIED. The
engine follow-up that makes the cells pass is filed and owns every failing cell.

## Context
- Today a criterion operand compares numbers only when `parse_criterion_number` (`parse_excel_number` on the
  space-trimmed operand: plain digits, `.`, an exponent) or `parse_date_criterion` (date text, no time) reads it.
  See `indexed_criterion` and `build_criteria_with` in crates/rustsheet-engine/src/functions/criteria.rs. Every other
  operand is text, so `COUNTIF(A4:A6,">40%")`, `COUNTIF(A4:A6,"50%")`, `COUNTIF(A4:A6,"$1")` and
  `COUNTIF(A4:A6,"1,000")` over the numbers 0.5, 1 and 1000 are all 0 in the engine (checked on main 97318e68a).
  Ordered criteria on percentages (`">=90%"`) are among the most common COUNTIF/SUMIF uses, and M2 scope includes
  "Criteria and wildcard grammar shared by the *IF(S) family" (docs/milestones.md).
- The cell side uses the same plain parser: a text cell `"50%"` is not matched by the criterion `0.5`
  (`result_is_equal_to_number`). Only plain number text such as `"1"`, `"1 "` and `" 1"` has evidence
  (criteria.md "Numeric criteria against text cells and non-ASCII spaces", probes/criteria-padded-text).
- The engine has three richer number readers that could be Excel's model here. Each is evidenced for its own path:
  - VALUE and `0+text` read percent, currency, grouping, parentheses and time through `parse_formatted_number`
    (crates/rustsheet-engine/src/formatter/parse_formatted.rs; coercion.md "Parenthesised number text is
    negative", "Percent text is rounded once").
  - Typed cell input has its own forms (coercion.md "Typed currency and percent formats").
  - Month-name date text is already a date in criteria (criteria.md "Text `<=` / `>=` criteria", `">=July 1, 2026"`).
  Nothing tells which of these, or which mix, the criteria parser uses. Excel could also apply one grammar to the
  operand and a narrower one to text cells.
- Why opus: the probe must tell the candidate models apart, not just confirm one. Two ways to do that:
  - pick forms on which they disagree: `"(1)"`, `"1 1/2"`, `"$-1"`, `"1,00"`, `"1,000.5%"`, `"12:00 PM"`, a date
    with a time, `"1e2%"`, and percent rounding near a 15-digit tie;
  - cross every operand with every operator (`=`, `<>`, `<`, `>=`) and with number cells and text cells.
  The answer also has to cover the indexed *IFS path (`number_interval` in functions/statistical/ifs_cache.rs) and
  the D-functions, which share `build_criteria_with`.
- Method: the `excel-probe` skill, excel-graph backend, one case `probes/t3136-criteria-formatted-number-operands`
  (milestone M2). Hold the data in number cells (0.5, 1, -1, 1.5, 1000, 0.5 as a 12:00 time, a date serial with a
  time) and in text cells holding the same forms (`="50%"`, `="$1"`, ...). Put the criteria in literals and in cells.
  Read them with COUNTIF, SUMIF, COUNTIFS, SUMIFS, AVERAGEIF and DCOUNT, and use SUMIF weights that are powers of
  two, so that one sum tells exactly which cells matched.

## Acceptance criteria
- [x] `probes/t3136-criteria-formatted-number-operands` is committed with its excel-graph oracle sidecar. It covers
      at least these operand forms: percent, `$` currency (with and without grouping and sign), thousands
      grouping, time, date plus time, mixed fraction, parenthesised negative and exponent-with-percent. Each form
      is read under `=`, `<>`, an ordering operator and a plain operand, against number cells and against text
      cells holding each form. Every cell is read through COUNTIF, SUMIF (power-of-two weights), COUNTIFS or SUMIFS,
      and at least one D-function.
- [x] criteria.md has a new VERIFIED section with the rule for the criterion side and for the text-cell side,
      citing cells. It names which existing reader (if any) the rule matches: `parse_formatted_number`, typed input
      or the date grammar. Forms the probe did not cover are listed as UNVERIFIED.
- [x] Every failing cell of the case has an open owner: an engine follow-up task (lane functions-criteria,
      milestone M2) names the case and the cells, and `make exit-owners MILESTONE=M2` reports 0.
      `make eval-baseline` records the case as new coverage, with no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's criteria parser (the follow-up task).
- Wildcard patterns, text ordering and the collation (T-3114, T-3115).
- Non-en-US locales.

## Outcome
probes/t3136-criteria-formatted-number-operands (406 formula cells, excel-graph 2026-10-29): a criterion operand reads a
number exactly when VALUE does (percent, currency, grouping, time, date-time, mixed fraction, parentheses, `1e2%`; `1,00`
stays text, as `VALUE("1,00")` is `#VALUE!`), and `=`/plain numeric criteria match the text cells VALUE reads as that
number; `<>` counts every text cell and `>=` none. SUMIF, SUMIFS, COUNTIF, COUNTIFS and DSUM agree. criteria.md has the
VERIFIED section. The engine fails A1:G52 and H25 (VALUE mixed fraction), owned by follow-up T-3151 (functions-criteria,
opus); rows 53-56 pass. Eval delta: new case, recorded as new coverage, 0 regressions.
