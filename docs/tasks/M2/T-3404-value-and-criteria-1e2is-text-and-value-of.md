---
id: T-3404
title: "VALUE and criteria: 1e2$ is text, and VALUE of CHAR(160)-padded number text is #VALUE! (T-3334 probe)"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3334]
---

## Goal
The engine reads `1e2$` (a trailing `$` after an E-notation number) as text, both in VALUE and as a
criteria operand, and VALUE returns #VALUE! for number text padded with CHAR(160), as Excel does.
All 33 failing cells of probes/t3334-criteria-operand-forms then pass.

## Context
Evidence: probes/t3334-criteria-operand-forms (excel-graph), written up in docs/excel-semantics/criteria.md
(the "Verified (probes/t3334-criteria-operand-forms" list). Failing cells today:
- `1e2$`: Calc!A39:G42 (28 cells) and H39. Excel: VALUE("1e2$") is #VALUE!; as an operand it is text, so the
  plain and `=` rows match nothing, `<1e2$` matches eight text cells (B41 510) and `>1e2$` two (B42 513).
  The engine reads 100 (H39 100, A39 32). Whether `1$` or `1.5$` (no exponent) are numbers is settled by
  earlier probes (criteria.md, T-3136): check that the fix does not change those cells.
- CHAR(160) padding: Calc!H47, H51, H55, H59 (`VALUE("$1"&CHAR(160))`, `CHAR(160)&"50%"`, `"1,000"&CHAR(160)`,
  `CHAR(160)&"$1"`). Excel #VALUE!; the engine returns the number. The criteria cells already pass (criteria.rs
  keeps NBSP-padded text as text), so the gap is in VALUE's reader (`value_number_text` in cast.rs) or its
  trimming. Search for the shared helper before writing a new one.

- Lane: `functions-text`, since both gaps sit in VALUE's reader (`value_number_text_with`, cast.rs), which the criteria
  operand reader (`criterion_text_number`, functions/criteria.rs) calls; check the criteria cells after the fix too.

## Acceptance criteria
- [ ] The 33 cells above pass; their known-failure lines disappear from eval/baseline with no new ones.
- [ ] No other probes or gen-m2 cell changes verdict (`make eval`).
- [ ] criteria.md (and text.md if VALUE's grammar is listed there) cite the cells as implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other VALUE forms; locales other than en-US.
