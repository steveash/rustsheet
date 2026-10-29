---
id: T-3151
title: "Criteria operands and text cells read numbers with VALUE's grammar (percent, currency, grouping, time, date-time, mixed fraction, parentheses)"
milestone: M2
lane: functions-criteria
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A *IF(S) or D-function criterion operand reads a number exactly when VALUE does, and a numeric `=` criterion matches
the text cells that VALUE reads as that number, as Excel does (docs/excel-semantics/criteria.md "Criterion operands and text
cells read numbers with VALUE's grammar", T-3136).

## Context
- Evidence: probes/t3136-criteria-formatted-number-operands (excel-graph 2026-10-29). Failing cells: A1:G52 (SUMIF, SUMIFS,
  COUNTIF, COUNTIFS, DSUM over the operands `50%`, `$1`, `1,000`, `$1,000`, `12:00`, `12:00 PM`, `1 1/2`, `(1)`, `$-1`,
  `-$1`, `1/2/2020 12:00`, `1e2%`, `1,000.5%` under plain, `=`, `<>`, `>=`) and H25 (`VALUE("1 1/2")` = 1.5).
  (B, D and F rows 25, 26, 39, 41, 42, 49 and 50 already pass by chance; they and rows 53-56 (`1,00`, text for both VALUE
  and the criteria) must keep passing.)
- Rule: the operand side reads with VALUE's grammar (`1,00` is `#VALUE!` there, so it stays text). Text cells: `=` and the
  plain operand match a text cell whose VALUE is the number (`"12:00"`, `"12:00 PM"` and `"50%"` all match `50%`); `<>`
  counts every text cell; `<`/`>=` never match a text cell.
- Code: `parse_criterion_number` / `parse_date_criterion` and `result_is_equal_to_number` in
  crates/rustsheet-engine/src/functions/criteria.rs; the indexed *IFS path (`number_interval`,
  functions/statistical/ifs_cache.rs) and the D-functions share `build_criteria_with`. Reuse VALUE's reader
  (`value_text_number`, functions/text/common.rs, over `parse_formatted_number`); do not add another.
- VALUE does not read a mixed fraction yet (`VALUE("1 1/2")` is `#VALUE!` in the engine): T-2646 (M9) owns that for
  VALUE. Rows 25-28 and H25 need it here; land it with this task, or after T-2646 if that lands first.
- Check that typed or plain operands that pass today (probes/criteria-padded-text, criteria-operands, date-text-dotted-
  criteria) do not regress: CHAR(160)-padded number text is a text criterion (criteria.md "Numeric criteria against text
  cells and non-ASCII spaces"); whether Excel's VALUE reads it is unprobed, so keep that case as it is.

## Acceptance criteria
- [x] Every cell of probes/t3136-criteria-formatted-number-operands passes; `make eval-check` shows 0 regressions.
- [x] criteria.md's T-3136 section says the engine implements the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- Non-en-US locales.
- VALUE's other T-2646 forms (`- 5`, `1-1-1`) beyond what these cells need.

## Outcome
Criterion operands and text cells now read numbers with VALUE's grammar. `value_number_text` (cast.rs) is VALUE's reader
as a free function, and `Model::value_text_number` and the criteria (`criterion_text_number`, functions/criteria.rs) share it.
The criteria try the plain parser first, then VALUE's reader, then the date parser. Text padded with a non-ASCII space stays
text, and text without a digit is skipped. VALUE also reads a mixed fraction now (`mixed_fraction`), which takes the first
part of T-2646. A numeric `=` criterion reads the current year once per criterion, for year-less text cells.
A new probe, probes/t3151-number-criterion-formatted-text (11 formulas, excel-graph 2026-10-29), confirms the same reading
for a number-cell, blank-cell and omitted criterion. It also confirms that an operand followed by a tab is text.
VALUE does not read a signed mixed fraction (`-3 1/2`) yet; T-2646 probes it.
Eval: probes 346 fixed, 0 regressions: all 344 failing cells of t3136-criteria-formatted-number-operands, plus
eng-inumber-date-boundary S!A22 and B22 (T-2646's mixed fraction).
