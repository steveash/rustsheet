---
id: T-3391
title: "Probe criteria operands in VALUE's grammar under <, > and <=, with month-name date-times, $ with an exponent, a 15-digit percent tie and NBSP-padded text"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence (excel-graph) decides the criterion-operand cases that criteria.md "Criterion operands and text cells
read numbers with VALUE's grammar" still lists as UNVERIFIED. The section then states Excel's rule for each one,
and every engine cell that disagrees has an owning task.

## Context
- Gap triage 2026-10-30e. Every visibly-off M0-M2 cell has an open owner (`make exit-owners MILESTONE=M2`: 0),
  gen-m2 is at 100%, and the M2 argument-class pairs are covered by T-3277 to T-3279. So the remaining M2 risk is
  in rules the engine implements by analogy and has no Excel cell for. This is one of them.
- What is verified: probes/t3136-criteria-formatted-number-operands (excel-graph, 2026-10-29) covers 14 operands
  under four operators: plain, `=`, `<>` and `>=`. probes/t3151-number-criterion-formatted-text covers a number
  criterion from a cell against the same text cells. The rule: an operand is a number exactly when VALUE reads it as
  one. `>=` with a numeric operand never matches a text cell, and `<>` counts every text cell.
- UNVERIFIED in that section (criteria.md, the "Verified in text.md (T-3178)" paragraph), which this probe settles:
  - the `<`, `>` and `<=` operators with formatted operands (`"<$1,000"`, `">50%"`, `"<=12:00 PM"`,
    `"<1 1/2"`, `">(1)"`). The engine handles them like `>=` by analogy: numbers only, never a text cell;
  - other VALUE forms: month-name date-times (`"Jan 2, 2020 12:00"`, `"2-Jan-2020 12:00 PM"`,
    `">=January 2, 2020 12:00"`), and `$` with an exponent (`"$1e2"`, `"$1E+2"`, `"-$1e2"`, `"1e2$"`);
  - percent rounding at the 15-digit tie: an operand like `"0.1234567890123455%"` against a number cell holding
    the double of 0.001234567890123455, and its neighbours one ulp either side. This shows whether the operand is
    rounded to 15 digits before it is compared;
  - CHAR(160)-padded formatted text on either side: a text cell `"$1"&CHAR(160)` or `CHAR(160)&"50%"` against
    the numeric criteria 1 and 0.5, and the operand `"="&CHAR(160)&"$1"`. The section "Numeric criteria against text
    cells and non-ASCII spaces" says plain number text padded with NBSP stays text; formatted text is unprobed.
  Non-en-US locales are out of scope (exotic locales never get a probe here).
- Reuse t3136's layout: Data numbers with power-of-two weights so one SUMIF names the matched set, text cells with
  their own weights, a Crit sheet for cell criteria, literal criteria for COUNTIF/COUNTIFS, a DC sheet for DSUM, and
  column H = VALUE of each operand. Every *IF(S) path is scored this way: SUMIF, SUMIFS (indexed), COUNTIF, COUNTIFS
  and DSUM.
- Engine code: `criterion_text_number` (crates/rustsheet-engine/src/functions/criteria.rs) over
  `value_number_text` (cast.rs), `OPERAND_CURRENCIES` vs `CELL_CURRENCIES`.

## Acceptance criteria
- [ ] A new probes case `probes/t3296-criteria-operand-forms` with an excel-graph oracle sidecar. It covers each
      bullet above: every operand under `<`, `>`, `<=` (and `=` for the new forms), over number cells and text
      cells, through SUMIF, SUMIFS, COUNTIF, COUNTIFS and DSUM, with VALUE of each operand beside it. At least
      150 scored cells.
- [ ] criteria.md's section states Excel's rule for each bullet with cell citations, and drops each UNVERIFIED item
      the probe settles.
- [ ] Every cell the engine fails is a known failure in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions). Each one is owned by a task filed in this PR, in the `functions-criteria` lane, or by an
      existing open one; `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (the owning tasks).
- Non-en-US decimal and currency conventions.
- Wildcard and collation behaviour of text operands (criteria.md's other sections; T-3180, T-3203).
