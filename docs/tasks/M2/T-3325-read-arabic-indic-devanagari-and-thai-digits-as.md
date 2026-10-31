---
id: T-3325
title: "Read Arabic-Indic, Devanagari and Thai digits as numbers in VALUE, arithmetic and numeric criteria (T-3276 probe)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
VALUE, text-to-number coercion in arithmetic and numeric criteria read Arabic-Indic (U+0660-U+0669),
Devanagari (U+0966-U+096F) and Thai (U+0E50-U+0E59) digits as their values, alone, mixed with ASCII digits and with
a decimal point, as Excel does; full-width digits stay text.

## Context
- Evidence: docs/excel-semantics/coercion.md "Other scripts, their marks and digits, symbols and
  astral characters (T-3276)": `VALUE("٣")` = `VALUE("३")` = `VALUE("๓")` = 3, `VALUE("٢٣")` = 23,
  `"٣"+0` = 3, `VALUE("٣.٥")` = 3.5, `VALUE("1٣")` = 13, `VALUE("３")` `#VALUE!` (Calc!H1:H8), and
  COUNTIF `"=٣"` counts the text cells "3", "٣" and "३" (Calc!E).
- Find the shared number-text parser the criteria and VALUE use (criteria.md "Rule (text-cell side)",
  `parse_formatted_number`) and map these digits there once; do not add a second parser.
- Unprobed, decide by evidence (excel-probe) or leave out: other decimal-digit blocks (Bengali, Lao,
  Tibetan, Myanmar), digits in dates and times, typed input in a cell.
- Failing cells:
  - probes/t3276-collation-other-scripts.case.json Calc!E9:E11, Calc!E81:E82, Calc!E87:E88, Calc!H1:H5, Calc!H7:H8

## Acceptance criteria
- [ ] The cells above pass; no regression (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Collation of these digits (T-3324).

## Outcome
`script_digit_to_ascii` (parse_formatted.rs) is applied in `cast::number_text_in`, the one coercion entry used by VALUE, arithmetic coercion and criteria. Typed cell input, dates and times, and other digit blocks are left out (unprobed). Percent/currency/parenthesis/group/exponent forms inherit the mapping through coercion; a probe follow-up is filed. 14 known failures removed from eval/baseline/probes.txt, no regressions.
